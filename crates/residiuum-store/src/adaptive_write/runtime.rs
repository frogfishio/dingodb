//! AWO-3 Static Intake Arbiter surface (product attach, mode default disabled).
//!
//! - [`AdaptiveWriteMode::Disabled`]: no lease, no cooker; natural store paths.
//! - [`AdaptiveWriteMode::Static`]: lease fences direct `Store` mutation; single
//!   admits execute natural under the lease; [`AdaptiveWriteHandle::admit_put_batch`]
//!   uses lease-owned `put_many` (persist-before-publish + optional parallel cook).
//! - [`AdaptiveWriteMode::Adaptive`]: same floor as Static until AWO-5 controller.
//!
//! E6 heap active-writer layout residual: product heap routing still uses the
//! shared physical store lock; heap-specific active segments remain open.

use super::coordinator::{PipelineCoordinator, PipelineError, PipelineStatus};
use super::cooker::PersistentCookerPool;
use super::credits::{mutation_credit, CreditLedger};
use super::policy::{AdaptiveWriteMode, AdaptiveWritePolicy, PolicyError};
use crate::durability::DurabilityMode;
use crate::error::StoreError;
use crate::store::{Store, WriteCondition, WriteReceipt};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Closed AWO admission / runtime errors (plan §5).
#[derive(Debug, Clone)]
pub enum AdaptiveWriteError {
    /// Queue entry or byte credit exhausted.
    QueueFull {
        /// Caller may retry after this delay (policy default collection cap).
        retry_after: Duration,
    },
    /// Admission or completion deadline exceeded.
    AdmissionDeadlineExceeded,
    /// Runtime is draining; no new admits.
    Draining,
    /// Mode is disabled — use natural store paths / ordinary create-open.
    ModeDisabled,
    /// Policy failed validation.
    InvalidPolicy(PolicyError),
    /// Writer poisoned after uncertain I/O.
    WriterPoisoned {
        /// Ordinary close/reopen recovery required.
        recovery_required: bool,
    },
    /// Adaptive lease owns mutation; direct path refused.
    WriterActive,
    /// Underlying store error.
    Store(String),
    /// Pipeline depth / seal / shutdown refusal (AWO-4).
    Pipeline(PipelineError),
}

impl AdaptiveWriteError {
    /// Stable wire / metric id.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueueFull { .. } => "write_overloaded",
            Self::AdmissionDeadlineExceeded => "write_deadline",
            Self::Draining => "server_draining",
            Self::ModeDisabled => "mode_disabled",
            Self::InvalidPolicy(_) => "invalid_policy",
            Self::WriterPoisoned { .. } => "write_outcome_uncertain",
            Self::WriterActive => "writer_active",
            Self::Store(_) => "store_error",
            Self::Pipeline(PipelineError::DepthExceeded { .. }) => "pipeline_depth_exceeded",
            Self::Pipeline(PipelineError::Sealing) => "pipeline_sealing",
            Self::Pipeline(PipelineError::ShuttingDown) => "pipeline_shutdown",
            Self::Pipeline(_) => "pipeline_error",
        }
    }
}

impl From<StoreError> for AdaptiveWriteError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::AdaptiveWriterPoisoned => Self::WriterPoisoned {
                recovery_required: true,
            },
            StoreError::AdaptiveWriterActive => Self::WriterActive,
            other => Self::Store(other.to_string()),
        }
    }
}

/// Snapshot of adaptive-write runtime (telemetry / drain).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveWriteStatus {
    /// Configured mode.
    pub mode: AdaptiveWriteMode,
    /// Whether the lease fences direct store mutation.
    pub lease_active: bool,
    /// Whether the runtime is draining.
    pub draining: bool,
    /// Active cooker permits (0 when disabled).
    pub active_cookers: usize,
    /// Threads created at warm (0 when disabled).
    pub cooker_threads: usize,
    /// Reserved queue entries.
    pub entries_used: usize,
    /// Reserved queue bytes.
    pub bytes_used: usize,
    /// Pending cook tasks.
    pub pending_cook_tasks: usize,
    /// Pipeline depth snapshot (AWO-4); zeros when disabled.
    pub pipeline: PipelineStatus,
}

/// One-shot completion for an admitted write (plan §5).
#[derive(Debug)]
pub struct WriteCompletion {
    receipt: Result<WriteReceipt, AdaptiveWriteError>,
}

impl WriteCompletion {
    /// Consume the completion (already resolved in the AWO-3 static floor).
    pub fn wait(self) -> Result<WriteReceipt, AdaptiveWriteError> {
        self.receipt
    }

    /// Whether the completion already holds a result (always true on this floor).
    pub fn is_ready(&self) -> bool {
        true
    }
}

/// Admission outcome.
#[derive(Debug)]
pub enum AdmissionResult {
    /// Accepted; completion may be waited.
    Admitted(WriteCompletion),
    /// Rejected before ownership transfer.
    Rejected(AdaptiveWriteError),
}

/// V1 eligibility class for a mutation (profile-v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EligibilityClass {
    /// Unconditional inline put — batch-eligible.
    UnconditionalInlinePut,
    /// Unconditional delete — batch-eligible.
    UnconditionalDelete,
    /// Conditional / natural-only path.
    Natural,
}

/// Classify a put/delete for Static routing.
pub fn classify_put(condition: WriteCondition, durability: DurabilityMode) -> EligibilityClass {
    if durability == DurabilityMode::Memory {
        return EligibilityClass::Natural;
    }
    match condition {
        WriteCondition::Unconditional => EligibilityClass::UnconditionalInlinePut,
        _ => EligibilityClass::Natural,
    }
}

/// Classify a delete for Static routing.
pub fn classify_delete(condition: WriteCondition, durability: DurabilityMode) -> EligibilityClass {
    if durability == DurabilityMode::Memory {
        return EligibilityClass::Natural;
    }
    match condition {
        WriteCondition::Unconditional => EligibilityClass::UnconditionalDelete,
        _ => EligibilityClass::Natural,
    }
}

struct RuntimeInner {
    policy: AdaptiveWritePolicy,
    lease_active: bool,
    draining: bool,
    credits: Option<CreditLedger>,
    cooker: Option<PersistentCookerPool>,
    pipeline: Option<Arc<PipelineCoordinator>>,
}

/// Process-local adaptive write runtime (threads + credit ledger).
pub struct AdaptiveWriteRuntime {
    inner: Mutex<RuntimeInner>,
}

/// Cloneable control/enqueue handle (plan §5).
#[derive(Clone)]
pub struct AdaptiveWriteHandle {
    runtime: Arc<AdaptiveWriteRuntime>,
}

impl AdaptiveWriteHandle {
    /// Build a disabled handle (no lease, no cooker).
    pub fn disabled(policy: AdaptiveWritePolicy) -> Result<Self, AdaptiveWriteError> {
        policy
            .validate()
            .map_err(AdaptiveWriteError::InvalidPolicy)?;
        let mut policy = policy;
        policy.mode = AdaptiveWriteMode::Disabled;
        Ok(Self {
            runtime: Arc::new(AdaptiveWriteRuntime {
                inner: Mutex::new(RuntimeInner {
                    policy,
                    lease_active: false,
                    draining: false,
                    credits: None,
                    cooker: None,
                    pipeline: None,
                }),
            }),
        })
    }

    /// Start a Static/Adaptive runtime and mark the store lease active.
    pub fn start_static(
        policy: AdaptiveWritePolicy,
        store: &mut Store,
    ) -> Result<Self, AdaptiveWriteError> {
        policy
            .validate()
            .map_err(AdaptiveWriteError::InvalidPolicy)?;
        if policy.mode == AdaptiveWriteMode::Disabled {
            return Self::disabled(policy);
        }
        if store.is_awo_writer_poisoned() {
            return Err(AdaptiveWriteError::WriterPoisoned {
                recovery_required: true,
            });
        }
        let credits = CreditLedger::new(policy.queue_entry_limit, policy.queue_byte_limit);
        let cooker = PersistentCookerPool::start(
            policy.maximum_cookers,
            policy.minimum_active_cookers,
            policy.queue_entry_limit.min(4096).max(16),
            policy.queue_byte_limit,
            0,
        );
        let pipeline = Arc::new(PipelineCoordinator::new(policy.pipeline_depth_limit));
        store.set_awo_lease_active(true);
        Ok(Self {
            runtime: Arc::new(AdaptiveWriteRuntime {
                inner: Mutex::new(RuntimeInner {
                    policy,
                    lease_active: true,
                    draining: false,
                    credits: Some(credits),
                    cooker: Some(cooker),
                    pipeline: Some(pipeline),
                }),
            }),
        })
    }

    /// Current status snapshot.
    pub fn status(&self) -> AdaptiveWriteStatus {
        let g = self.runtime.inner.lock().expect("awo runtime");
        let (entries_used, bytes_used) = g
            .credits
            .as_ref()
            .map(|c| (c.entries_used(), c.bytes_used()))
            .unwrap_or((0, 0));
        let (active_cookers, cooker_threads, pending) = g
            .cooker
            .as_ref()
            .map(|c| (c.active_cookers(), c.threads_created(), c.pending_tasks()))
            .unwrap_or((0, 0, 0));
        let pipeline = g
            .pipeline
            .as_ref()
            .map(|p| p.status())
            .unwrap_or(PipelineStatus {
                depth_limit: g.policy.pipeline_depth_limit,
                in_flight: 0,
                cooking: 0,
                installing: 0,
                sealing: false,
                shutting_down: false,
            });
        AdaptiveWriteStatus {
            mode: g.policy.mode,
            lease_active: g.lease_active,
            draining: g.draining,
            active_cookers,
            cooker_threads,
            entries_used,
            bytes_used,
            pending_cook_tasks: pending,
            pipeline,
        }
    }

    /// Pipeline coordinator when Static/Adaptive (AWO-4).
    pub fn pipeline(&self) -> Option<Arc<PipelineCoordinator>> {
        self.runtime
            .inner
            .lock()
            .expect("awo runtime")
            .pipeline
            .clone()
    }

    /// Configured policy mode.
    pub fn mode(&self) -> AdaptiveWriteMode {
        self.runtime.inner.lock().expect("awo runtime").policy.mode
    }

    /// Whether direct store mutation is fenced.
    pub fn lease_active(&self) -> bool {
        self.runtime
            .inner
            .lock()
            .expect("awo runtime")
            .lease_active
    }

    /// Admit a put under the adaptive lease (natural execution on AWO-3 floor).
    pub fn admit_put(
        &self,
        store: &mut Store,
        subject: &[u8],
        value: &[u8],
        mode: DurabilityMode,
        condition: WriteCondition,
    ) -> AdmissionResult {
        let _class = classify_put(condition, mode);
        self.admit_natural(store, |s| {
            s.put_subject_bytes_if_awo_owned(subject, value, mode, condition)
        })
    }

    /// Admit a delete under the adaptive lease (natural execution on AWO-3 floor).
    pub fn admit_delete(
        &self,
        store: &mut Store,
        subject: &[u8],
        mode: DurabilityMode,
        condition: WriteCondition,
    ) -> AdmissionResult {
        let _class = classify_delete(condition, mode);
        self.admit_natural(store, |s| {
            s.delete_subject_bytes_if_awo_owned(subject, mode, condition)
        })
    }

    /// Admit a batch of unconditional puts under the lease (AWO-3 deepen).
    ///
    /// Reserves credit for each item, then installs via
    /// [`Store::put_many_awo_owned`] (persist-before-publish; uses store
    /// cook_parallelism when configured). Independent receipts returned in
    /// input order.
    pub fn admit_put_batch(
        &self,
        store: &mut Store,
        items: &[(&str, &[u8])],
        mode: DurabilityMode,
    ) -> Result<Vec<WriteReceipt>, AdaptiveWriteError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        // Memory / conditional work is natural one-by-one (profile natural class).
        if mode == DurabilityMode::Memory {
            let mut out = Vec::with_capacity(items.len());
            for (subject, value) in items {
                match self.admit_put(
                    store,
                    subject.as_bytes(),
                    value,
                    mode,
                    WriteCondition::Unconditional,
                ) {
                    AdmissionResult::Admitted(c) => out.push(c.wait()?),
                    AdmissionResult::Rejected(e) => return Err(e),
                }
            }
            return Ok(out);
        }

        let mut total_credit: usize = 0;
        let mut per_credits: Vec<usize> = Vec::with_capacity(items.len());
        for (subject, value) in items {
            let c = mutation_credit(subject.len(), value.len())
                .map_err(|_| AdaptiveWriteError::QueueFull {
                    retry_after: Duration::from_millis(1),
                })?;
            per_credits.push(c);
            total_credit = total_credit.checked_add(c).ok_or(
                AdaptiveWriteError::QueueFull {
                    retry_after: Duration::from_millis(1),
                },
            )?;
        }

        let retry_after;
        let pipeline = {
            let g = self.runtime.inner.lock().expect("awo runtime");
            retry_after = g.policy.maximum_collection_delay;
            if g.policy.mode == AdaptiveWriteMode::Disabled {
                return Err(AdaptiveWriteError::ModeDisabled);
            }
            if g.draining {
                return Err(AdaptiveWriteError::Draining);
            }
            if !g.lease_active {
                return Err(AdaptiveWriteError::ModeDisabled);
            }
            if store.is_awo_writer_poisoned() {
                return Err(AdaptiveWriteError::WriterPoisoned {
                    recovery_required: true,
                });
            }
            if let Some(credits) = g.credits.as_ref() {
                if credits
                    .try_reserve(items.len(), total_credit)
                    .is_err()
                {
                    return Err(AdaptiveWriteError::QueueFull { retry_after });
                }
            }
            g.pipeline.clone()
        };

        // AWO-4: one pipeline reservation for this batch (cook → install).
        let reservation = if let Some(ref pipe) = pipeline {
            match pipe.try_begin_reservation() {
                Ok(id) => Some(id),
                Err(e) => {
                    let g = self.runtime.inner.lock().expect("awo runtime");
                    if let Some(credits) = g.credits.as_ref() {
                        let _ = credits.release(items.len(), total_credit);
                    }
                    return Err(AdaptiveWriteError::Pipeline(e));
                }
            }
        } else {
            None
        };

        // Prefer parallel store cook when the runtime has a cooker pool.
        let cooker_n = self
            .runtime
            .inner
            .lock()
            .expect("awo runtime")
            .cooker
            .as_ref()
            .map(|c| c.maximum_cookers())
            .unwrap_or(1);
        if cooker_n > 1 {
            store.set_cook_parallelism(cooker_n);
        }

        if let (Some(ref pipe), Some(id)) = (pipeline.as_ref(), reservation) {
            // Synchronous batch path: cook+install are one store call; mark
            // cook complete then install complete around put_many_awo_owned.
            let _ = pipe.note_cook_complete(id);
        }

        let result = store.put_many_awo_owned(items, mode);
        if let (Some(ref pipe), Some(id)) = (pipeline.as_ref(), reservation) {
            match &result {
                Ok(_) => {
                    let _ = pipe.note_install_complete(id);
                }
                Err(_) => {
                    let _ = pipe.abort_reservation(id);
                }
            }
        }
        {
            let g = self.runtime.inner.lock().expect("awo runtime");
            if let Some(credits) = g.credits.as_ref() {
                let _ = credits.release(items.len(), total_credit);
            }
        }
        result.map_err(AdaptiveWriteError::from)
    }

    fn admit_natural<F>(&self, store: &mut Store, op: F) -> AdmissionResult
    where
        F: FnOnce(&mut Store) -> Result<WriteReceipt, StoreError>,
    {
        let credit = match mutation_credit(0, 0) {
            Ok(c) => c,
            Err(_) => {
                return AdmissionResult::Rejected(AdaptiveWriteError::QueueFull {
                    retry_after: Duration::from_millis(1),
                });
            }
        };

        {
            let g = self.runtime.inner.lock().expect("awo runtime");
            if g.policy.mode == AdaptiveWriteMode::Disabled {
                return AdmissionResult::Rejected(AdaptiveWriteError::ModeDisabled);
            }
            if g.draining {
                return AdmissionResult::Rejected(AdaptiveWriteError::Draining);
            }
            if !g.lease_active {
                return AdmissionResult::Rejected(AdaptiveWriteError::ModeDisabled);
            }
            if store.is_awo_writer_poisoned() {
                return AdmissionResult::Rejected(AdaptiveWriteError::WriterPoisoned {
                    recovery_required: true,
                });
            }
            if let Some(credits) = g.credits.as_ref() {
                if let Err(e) = credits.try_reserve(1, credit) {
                    let _ = e;
                    return AdmissionResult::Rejected(AdaptiveWriteError::QueueFull {
                        retry_after: g.policy.maximum_collection_delay,
                    });
                }
            }
        }

        let receipt = op(store);
        {
            let g = self.runtime.inner.lock().expect("awo runtime");
            if let Some(credits) = g.credits.as_ref() {
                let _ = credits.release(1, credit);
            }
        }

        match receipt {
            Ok(r) => AdmissionResult::Admitted(WriteCompletion { receipt: Ok(r) }),
            Err(e) => AdmissionResult::Admitted(WriteCompletion {
                receipt: Err(AdaptiveWriteError::from(e)),
            }),
        }
    }

    /// Map AWO errors onto store errors for façade boundaries.
    pub fn to_store_error(err: AdaptiveWriteError) -> StoreError {
        match err {
            AdaptiveWriteError::WriterPoisoned { .. } => StoreError::AdaptiveWriterPoisoned,
            AdaptiveWriteError::WriterActive => StoreError::AdaptiveWriterActive,
            AdaptiveWriteError::QueueFull { .. } => StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "awo queue full",
            )),
            AdaptiveWriteError::AdmissionDeadlineExceeded => StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "awo admission deadline",
            )),
            AdaptiveWriteError::Draining => StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "awo draining",
            )),
            AdaptiveWriteError::ModeDisabled => StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "awo mode disabled",
            )),
            AdaptiveWriteError::InvalidPolicy(p) => StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("awo policy: {p:?}"),
            )),
            AdaptiveWriteError::Store(s) => StoreError::Io(std::io::Error::other(s)),
            AdaptiveWriteError::Pipeline(PipelineError::DepthExceeded { .. }) => {
                StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "awo pipeline depth exceeded",
                ))
            }
            AdaptiveWriteError::Pipeline(e) => StoreError::Io(std::io::Error::other(format!(
                "awo pipeline: {e:?}"
            ))),
        }
    }

    /// Drain admits until idle or deadline (cooker pending + credits + pipeline).
    pub fn drain_writes(&self, deadline: Instant) -> Result<(), AdaptiveWriteError> {
        {
            let mut g = self.runtime.inner.lock().expect("awo runtime");
            g.draining = true;
            if let Some(ref p) = g.pipeline {
                p.begin_shutdown();
            }
        }
        while Instant::now() < deadline {
            let pending = {
                let g = self.runtime.inner.lock().expect("awo runtime");
                let cook = g.cooker.as_ref().map(|c| c.pending_tasks()).unwrap_or(0);
                let cred = g.credits.as_ref().map(|c| c.entries_used()).unwrap_or(0);
                let pipe = g
                    .pipeline
                    .as_ref()
                    .map(|p| p.status().in_flight)
                    .unwrap_or(0);
                cook + cred + pipe
            };
            if pending == 0 {
                return Ok(());
            }
            std::thread::yield_now();
        }
        Err(AdaptiveWriteError::AdmissionDeadlineExceeded)
    }

    /// Release lease on the store and shut down cookers (drop path).
    pub fn detach(&self, store: &mut Store) {
        let mut g = self.runtime.inner.lock().expect("awo runtime");
        g.draining = true;
        g.lease_active = false;
        if let Some(ref p) = g.pipeline {
            p.begin_shutdown();
        }
        if let Some(cooker) = g.cooker.take() {
            cooker.shutdown();
        }
        g.credits = None;
        g.pipeline = None;
        store.set_awo_lease_active(false);
    }
}