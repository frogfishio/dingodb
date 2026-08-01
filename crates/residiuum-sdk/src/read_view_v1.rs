//! Stable bounded read views — APB-6 T1 surface scaffold.
//!
//! Normative: MUST_ADD §10, PD-008, `spec/app/baseline-v1` `apb.heap.read_view`.
//!
//! This cut freezes **names/fields** and open/close/expiry checks. It does **not**
//! pin reclamation or prove multi-query snapshot observation. Product collection
//! access under a view fails closed until an authoritative frontier pin lands.
//!
//! Inventory: `doc/todo/application-baseline/APB6_READ_VIEW_GAP_INVENTORY.md`.

use crate::app_v1::{ConsistencyMode, CoveragePolicy, RQL_APP_CORE_PROFILE, RQL_PLAN_PROFILE};
use crate::cursor_v1::PROFILE as CURSOR_PROFILE;
use crate::error::Error;
use crate::predicate::PREDICATE_PROFILE_V1;
use residiuum_heap::HeapId;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Profile label for the first read-view façade cut (not package accept).
pub const READ_VIEW_PROFILE: &str = "residiuum-read-view-v1";

/// Options for [`crate::app_v1::HeapClient::read_view`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadViewOptions {
    /// Declared consistency mode for observation under the view.
    pub consistency: ConsistencyMode,
    /// Maximum age of the view after open (`None` = session-bounded default).
    pub max_age: Option<Duration>,
    /// Optional retention / resource budget (not yet enforced as a pin).
    pub retention_budget: Option<ReadViewRetentionBudget>,
}

impl Default for ReadViewOptions {
    fn default() -> Self {
        Self {
            consistency: ConsistencyMode::Available,
            max_age: Some(Duration::from_secs(900)),
            retention_budget: None,
        }
    }
}

/// Declared retention/resource budget (stored; pin enforcement residual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadViewRetentionBudget {
    /// Max documents the view may pin (optional).
    pub max_pinned_documents: Option<u64>,
    /// Max wall-clock hold after open (optional; combined with max_age).
    pub max_hold: Option<Duration>,
}

/// Kind of frontier currently bound (honest labels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontierKind {
    /// Live open generation — **not** a durable segment pin.
    ///
    /// First cut: opaque view generation id minted at open. Does not prevent
    /// mutations from changing later observations.
    LiveUnpinned,
    /// Future: store segment fingerprint pin (embedded residual).
    SegmentFingerprint,
}

/// Authoritative frontier binding on a read view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritativeFrontier {
    /// Frontier class.
    pub kind: FrontierKind,
    /// Opaque frontier identity (hex or label).
    pub identity_hex: String,
    /// Capture time (unix seconds).
    pub captured_at_unix: u64,
}

/// Semantic profile versions bound into the view (Class C labels).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticVersions {
    /// Application Core RQL profile.
    pub rql_app_core: String,
    /// Logical plan profile.
    pub plan: String,
    /// Predicate profile.
    pub predicate: String,
    /// Cursor profile.
    pub cursor: String,
    /// Read-view façade profile.
    pub read_view: String,
}

impl SemanticVersions {
    /// Frozen profile labels known to this build.
    pub fn current_build() -> Self {
        Self {
            rql_app_core: RQL_APP_CORE_PROFILE.into(),
            plan: RQL_PLAN_PROFILE.into(),
            predicate: PREDICATE_PROFILE_V1.into(),
            cursor: CURSOR_PROFILE.into(),
            read_view: READ_VIEW_PROFILE.into(),
        }
    }
}

/// Public description of an open (or closed) read view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadViewInfo {
    /// Owning Heap.
    pub heap_id: HeapId,
    /// Opaque view id (16 bytes as hex).
    pub view_id_hex: String,
    /// Bound frontier.
    pub frontier: AuthoritativeFrontier,
    /// Declared coverage policy at open.
    pub coverage: CoveragePolicy,
    /// Declared consistency mode.
    pub consistency: ConsistencyMode,
    /// Semantic profile versions.
    pub semantic_versions: SemanticVersions,
    /// Open time (unix seconds).
    pub opened_at_unix: u64,
    /// Expiry time (unix seconds); `None` if unbounded (discouraged).
    pub expires_at_unix: Option<u64>,
    /// Whether [`ReadView::close`] was called.
    pub closed: bool,
    /// Whether product observation is pinned (always false in T1).
    pub observation_pinned: bool,
}

/// Stable bounded read view handle (APB-6 scaffold).
///
/// Observation under this view is **not** product-pinned in T1. Call
/// [`Self::ensure_usable`] before attaching future query/export APIs.
#[derive(Debug)]
pub struct ReadView {
    info: ReadViewInfo,
    retention_budget: Option<ReadViewRetentionBudget>,
}

impl ReadView {
    /// Construct a live-unpinned view for a bound heap (internal / tests).
    pub(crate) fn open_live_unpinned(
        heap_id: HeapId,
        options: ReadViewOptions,
    ) -> Result<Self, Error> {
        let now = unix_now()?;
        let max_age = options
            .max_age
            .or_else(|| {
                options
                    .retention_budget
                    .and_then(|b| b.max_hold)
            })
            .unwrap_or(Duration::from_secs(900));
        let expires = now.saturating_add(max_age.as_secs());
        let view_bytes = residiuum_store::random_id().map_err(Error::from)?;
        let frontier_bytes = {
            // Domain-separated open generation — not a store segment pin.
            let mut h = blake3::Hasher::new();
            h.update(b"residiuum:read-view-v1:live-unpinned");
            h.update(&[0u8]);
            h.update(heap_id.as_bytes());
            h.update(&view_bytes);
            h.update(&now.to_be_bytes());
            *h.finalize().as_bytes()
        };
        Ok(Self {
            info: ReadViewInfo {
                heap_id,
                view_id_hex: hex16(&view_bytes),
                frontier: AuthoritativeFrontier {
                    kind: FrontierKind::LiveUnpinned,
                    identity_hex: hex32(&frontier_bytes),
                    captured_at_unix: now,
                },
                coverage: CoveragePolicy::Complete,
                consistency: options.consistency,
                semantic_versions: SemanticVersions::current_build(),
                opened_at_unix: now,
                expires_at_unix: Some(expires),
                closed: false,
                observation_pinned: false,
            },
            retention_budget: options.retention_budget,
        })
    }

    /// Public description.
    pub fn info(&self) -> &ReadViewInfo {
        &self.info
    }

    /// Owning Heap id.
    pub fn heap_id(&self) -> HeapId {
        self.info.heap_id
    }

    /// Whether the view has been closed.
    pub fn is_closed(&self) -> bool {
        self.info.closed
    }

    /// Whether the view is past expiry at `now_unix`.
    pub fn is_expired_at(&self, now_unix: u64) -> bool {
        match self.info.expires_at_unix {
            Some(exp) => now_unix > exp,
            None => false,
        }
    }

    /// Close the view (idempotent).
    pub fn close(&mut self) {
        self.info.closed = true;
    }

    /// Fail closed if closed or expired.
    pub fn ensure_usable(&self) -> Result<(), Error> {
        if self.info.closed {
            return Err(Error::ConsistencyViolation(
                "read view is closed".into(),
            ));
        }
        let now = unix_now()?;
        if self.is_expired_at(now) {
            return Err(Error::ConsistencyViolation(
                "read view expired".into(),
            ));
        }
        Ok(())
    }

    /// Retention budget declared at open (if any).
    pub fn retention_budget(&self) -> Option<ReadViewRetentionBudget> {
        self.retention_budget
    }

    /// Product collection observation under this view (fail-closed in T1).
    ///
    /// Residual: pin authoritative frontier + wire `query_exec_v1` under the view.
    pub fn open_collection(&mut self, _name: &str) -> Result<(), Error> {
        self.ensure_usable()?;
        Err(Error::Internal(
            "APB-6 residual: read view does not pin observation yet; \
             use CollectionClient::rql on a live handle (generation-fenced) \
             or wait for frontier pin (see APB6_READ_VIEW_GAP_INVENTORY.md)"
                .into(),
        ))
    }
}

fn unix_now() -> Result<u64, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| Error::Internal(format!("clock: {e}")))
}

fn hex16(bytes: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use residiuum_heap::HeapId;

    #[test]
    fn open_close_and_expiry() {
        let heap = HeapId::from_bytes_unchecked_nonzero([1u8; 16]).unwrap();
        let mut v = ReadView::open_live_unpinned(
            heap,
            ReadViewOptions {
                consistency: ConsistencyMode::Available,
                max_age: Some(Duration::from_secs(60)),
                retention_budget: None,
            },
        )
        .unwrap();
        assert!(!v.info().observation_pinned);
        assert_eq!(v.info().frontier.kind, FrontierKind::LiveUnpinned);
        assert_eq!(v.info().semantic_versions.read_view, READ_VIEW_PROFILE);
        v.ensure_usable().unwrap();
        assert!(v.open_collection("orders").is_err());
        v.close();
        assert!(v.ensure_usable().is_err());
    }
}
