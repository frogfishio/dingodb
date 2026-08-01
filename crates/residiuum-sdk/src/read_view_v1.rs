//! Stable bounded read views — APB-6 surface (T1 scaffold + T2 frontier pin).
//!
//! Normative: MUST_ADD §10, PD-008, `spec/app/baseline-v1` `apb.heap.read_view`.
//!
//! **T2 (embedded):** binds `authoritative_frontier` to the store segment
//! fingerprint and sets `observation_pinned = true`. Drift is detectable via
//! [`ReadView::check_drift`]. This is **not** multi-query snapshot isolation.
//!
//! **APB-7 T5:** pinned views may open a view-bound collection for Core query
//! when [`ReadView::ensure_observation_stable`] succeeds (`observation_pinned`
//! + [`FrontierDrift::Stable`]). Live data is still read; the pin only gates
//! on segment-fingerprint drift — not a frozen snapshot.
//!
//! **Remote:** still opens live-unpinned (honest residual; HAR-4 pin later);
//! view-bound query fail-closes with [`FrontierDrift::Unpinned`].
//!
//! Inventory: `doc/todo/application-baseline/APB6_READ_VIEW_GAP_INVENTORY.md`.

use crate::app_v1::{ConsistencyMode, CoveragePolicy, RQL_APP_CORE_PROFILE, RQL_PLAN_PROFILE};
use crate::cursor_v1::PROFILE as CURSOR_PROFILE;
use crate::error::Error;
use crate::predicate::PREDICATE_PROFILE_V1;
use residiuum_heap::HeapId;
use residiuum_store::HeapStore;
use std::sync::Arc;
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
    /// Used for remote backends and contract tests until a remote pin lands.
    LiveUnpinned,
    /// Store segment fingerprint pin (embedded APB-6 T2).
    ///
    /// Identity is segment path+size fingerprint. Detects some store layout
    /// movement via [`ReadView::check_drift`]; does **not** prove snapshot
    /// isolation for multi-page observation.
    SegmentFingerprint,
}

/// Authoritative frontier binding on a read view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritativeFrontier {
    /// Frontier class.
    pub kind: FrontierKind,
    /// Opaque frontier identity (hex).
    pub identity_hex: String,
    /// Capture time (unix seconds).
    pub captured_at_unix: u64,
}

/// Result of comparing the pinned frontier to the live store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontierDrift {
    /// Live segment fingerprint still matches the pin.
    Stable,
    /// Live fingerprint differs from the pin (store layout moved).
    Drifted,
    /// View has no re-checkable store pin (live-unpinned / remote).
    Unpinned,
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
    /// Whether an embedded segment-fingerprint pin is active.
    ///
    /// `true` does **not** mean product snapshot isolation or view-bound query.
    pub observation_pinned: bool,
}

/// Stable bounded read view handle (APB-6 / APB-7 T5 gate).
///
/// Embedded opens pin the store segment fingerprint. Call
/// [`Self::ensure_observation_stable`] before view-bound query; open a bound
/// collection via [`Self::open_collection`] (impl in `app_v1`). Not a
/// snapshot-isolation claim.
pub struct ReadView {
    info: ReadViewInfo,
    retention_budget: Option<ReadViewRetentionBudget>,
    /// Embedded pin: store + captured fingerprint for drift checks.
    pin: Option<SegmentPin>,
}

struct SegmentPin {
    store: Arc<HeapStore>,
    fingerprint: [u8; 32],
}

impl std::fmt::Debug for ReadView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadView")
            .field("info", &self.info)
            .field("retention_budget", &self.retention_budget)
            .field("pin", &self.pin.as_ref().map(|p| hex32(&p.fingerprint)))
            .finish()
    }
}

impl ReadView {
    /// Construct a live-unpinned view (remote residual / tests).
    pub(crate) fn open_live_unpinned(
        heap_id: HeapId,
        options: ReadViewOptions,
    ) -> Result<Self, Error> {
        let (now, expires, view_bytes) = open_common(&options)?;
        let frontier_bytes = {
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
            pin: None,
        })
    }

    /// Construct an embedded view pinned to a store segment fingerprint (APB-6 T2).
    pub(crate) fn open_segment_fingerprint_pinned(
        heap_id: HeapId,
        options: ReadViewOptions,
        store: Arc<HeapStore>,
        fingerprint: [u8; 32],
    ) -> Result<Self, Error> {
        let (now, expires, view_bytes) = open_common(&options)?;
        Ok(Self {
            info: ReadViewInfo {
                heap_id,
                view_id_hex: hex16(&view_bytes),
                frontier: AuthoritativeFrontier {
                    kind: FrontierKind::SegmentFingerprint,
                    identity_hex: hex32(&fingerprint),
                    captured_at_unix: now,
                },
                coverage: CoveragePolicy::Complete,
                consistency: options.consistency,
                semantic_versions: SemanticVersions::current_build(),
                opened_at_unix: now,
                expires_at_unix: Some(expires),
                closed: false,
                observation_pinned: true,
            },
            retention_budget: options.retention_budget,
            pin: Some(SegmentPin {
                store,
                fingerprint,
            }),
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

    /// Whether an embedded segment-fingerprint pin is active.
    pub fn is_observation_pinned(&self) -> bool {
        self.info.observation_pinned
    }

    /// Pinned segment fingerprint bytes when `FrontierKind::SegmentFingerprint`.
    pub fn pinned_fingerprint(&self) -> Option<[u8; 32]> {
        self.pin.as_ref().map(|p| p.fingerprint)
    }

    /// Compare the pin against the live store segment fingerprint.
    ///
    /// - [`FrontierDrift::Stable`] / [`FrontierDrift::Drifted`] for pinned views
    /// - [`FrontierDrift::Unpinned`] when no re-checkable store pin exists
    ///
    /// Does **not** enforce isolation; callers use this for residual honesty.
    pub fn check_drift(&self) -> Result<FrontierDrift, Error> {
        self.ensure_usable()?;
        let Some(pin) = self.pin.as_ref() else {
            return Ok(FrontierDrift::Unpinned);
        };
        let live = pin.store.segment_fingerprint()?;
        if live == pin.fingerprint {
            Ok(FrontierDrift::Stable)
        } else {
            Ok(FrontierDrift::Drifted)
        }
    }

    /// Gate for view-bound query / collection open (APB-7 T5).
    ///
    /// Requires usable view, `observation_pinned`, and
    /// [`FrontierDrift::Stable`]. Fail-closes on Unpinned/Drifted.
    ///
    /// **Not** snapshot isolation: Stable only means the segment-fingerprint
    /// pin still matches the live store layout.
    pub fn ensure_observation_stable(&self) -> Result<(), Error> {
        self.ensure_usable()?;
        if !self.info.observation_pinned {
            return Err(Error::ConsistencyViolation(
                "read view observation is not pinned (live-unpinned / remote residual); \
                 view-bound query requires segment-fingerprint pin (APB-7 T5)"
                    .into(),
            ));
        }
        match self.check_drift()? {
            FrontierDrift::Stable => Ok(()),
            FrontierDrift::Drifted => Err(Error::ConsistencyViolation(
                "read view frontier drifted (segment fingerprint moved); \
                 refresh_pin or open a new view before view-bound query (APB-7 T5)"
                    .into(),
            )),
            FrontierDrift::Unpinned => Err(Error::ConsistencyViolation(
                "read view has no re-checkable pin; view-bound query fail-closed (APB-7 T5)"
                    .into(),
            )),
        }
    }

    /// Re-sample the store segment fingerprint and update the pin in place.
    ///
    /// Residual: does not reclaim old segments or freeze document pages.
    /// Fail-closed on live-unpinned views.
    pub fn refresh_pin(&mut self) -> Result<(), Error> {
        self.ensure_usable()?;
        let Some(pin) = self.pin.as_mut() else {
            return Err(Error::ConsistencyViolation(
                "read view has no segment-fingerprint pin to refresh (live-unpinned)".into(),
            ));
        };
        let live = pin.store.segment_fingerprint()?;
        let now = unix_now()?;
        pin.fingerprint = live;
        self.info.frontier = AuthoritativeFrontier {
            kind: FrontierKind::SegmentFingerprint,
            identity_hex: hex32(&live),
            captured_at_unix: now,
        };
        self.info.observation_pinned = true;
        Ok(())
    }
}

fn open_common(options: &ReadViewOptions) -> Result<(u64, u64, [u8; 16]), Error> {
    let now = unix_now()?;
    let max_age = options
        .max_age
        .or_else(|| options.retention_budget.and_then(|b| b.max_hold))
        .unwrap_or(Duration::from_secs(900));
    let expires = now.saturating_add(max_age.as_secs());
    let view_bytes = residiuum_store::random_id().map_err(Error::from)?;
    Ok((now, expires, view_bytes))
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
    fn open_close_and_expiry_live_unpinned() {
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
        assert_eq!(v.check_drift().unwrap(), FrontierDrift::Unpinned);
        v.ensure_usable().unwrap();
        assert!(v.ensure_observation_stable().is_err());
        assert!(v.refresh_pin().is_err());
        v.close();
        assert!(v.ensure_usable().is_err());
    }
}