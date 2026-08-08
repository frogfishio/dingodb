//! Transport-independent contracts for the async driver spine (DRV-0).
//!
//! This module deliberately contains no executor, socket pool, or Tokio types.
//! It freezes the small state vocabulary that later driver packages share.

use std::num::NonZeroUsize;
use std::time::Duration;

/// Stable 128-bit identity minted once for a mutating operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(pub [u8; 16]);

/// Stable 128-bit identity for one logical request attempt chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub [u8; 16]);

/// Observable lifecycle stage for a driver request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestStage {
    /// Created locally but not admitted to the bounded queue.
    Created,
    /// Waiting in the bounded admission or checkout queue.
    Queued,
    /// Written to an embedded worker or remote connection.
    Dispatched,
    /// Cooperative cancellation has been requested after dispatch.
    CancelRequested,
    /// One terminal outcome has been recorded.
    Terminal,
}

/// Exactly one terminal class is recorded for every admitted request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalOutcome {
    /// Operation completed with a value, receipt, or authoritative absence.
    Completed,
    /// Operation was explicitly refused before an ambiguous commit point.
    Refused,
    /// Cancellation won before dispatch; the server/kernel never saw the work.
    CancelledBeforeDispatch,
    /// Dispatched non-mutating work was cooperatively cancelled.
    CancelledAfterDispatch,
    /// The single end-to-end deadline expired.
    DeadlineExceeded,
    /// A mutation may have committed and must be resolved by operation identity.
    CommitOutcomeUnknown,
}

/// Machine-actionable retry decision; callers never parse error text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDisposition {
    /// Never retry this logical operation.
    Never,
    /// Retry the same read/request identity while its deadline permits.
    SafeSameRequest,
    /// Retry only with the original mutation [`OperationId`].
    SafeSameOperationId,
    /// Retry after the supplied bounded delay, within the original deadline.
    After(Duration),
    /// Resolve the original mutation outcome before any further action.
    OutcomeLookupRequired,
}

/// Bounded resource defaults frozen by the driver v1 specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverLimits {
    /// Maximum connections per remote endpoint.
    pub max_connections: NonZeroUsize,
    /// Maximum simultaneous connection handshakes.
    pub max_connecting: NonZeroUsize,
    /// Maximum queued remote checkouts.
    pub max_waiters: NonZeroUsize,
    /// Maximum queued embedded operations.
    pub embedded_queue: NonZeroUsize,
    /// Maximum prefetched pages held by one cursor.
    pub cursor_prefetch_pages: NonZeroUsize,
}

impl Default for DriverLimits {
    fn default() -> Self {
        Self {
            max_connections: NonZeroUsize::new(10).expect("non-zero constant"),
            max_connecting: NonZeroUsize::new(2).expect("non-zero constant"),
            max_waiters: NonZeroUsize::new(1024).expect("non-zero constant"),
            embedded_queue: NonZeroUsize::new(1024).expect("non-zero constant"),
            cursor_prefetch_pages: NonZeroUsize::new(1).expect("non-zero constant"),
        }
    }
}

/// Required feature identifiers for safe retry/cancellation semantics.
pub const FEATURE_REQUEST_DEADLINE_V1: &str = "request-deadline-v1";
/// Cooperative request cancellation feature.
pub const FEATURE_CANCEL_REQUEST_V1: &str = "cancel-request-v1";
/// Mutation outcome lookup by stable operation identity.
pub const FEATURE_OPERATION_OUTCOME_V1: &str = "operation-outcome-v1";
/// Complete, non-placeholder application receipts.
pub const FEATURE_COMPLETE_RECEIPTS_V2: &str = "complete-receipts-v2";

/// Required negotiated features for the full async driver profile.
pub const REQUIRED_ASYNC_DRIVER_FEATURES: &[&str] = &[
    FEATURE_REQUEST_DEADLINE_V1,
    FEATURE_CANCEL_REQUEST_V1,
    FEATURE_OPERATION_OUTCOME_V1,
    FEATURE_COMPLETE_RECEIPTS_V2,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn contract_types_are_send_sync() {
        assert_send_sync::<OperationId>();
        assert_send_sync::<RequestId>();
        assert_send_sync::<RequestStage>();
        assert_send_sync::<TerminalOutcome>();
        assert_send_sync::<RetryDisposition>();
        assert_send_sync::<DriverLimits>();
    }

    #[test]
    fn v1_limits_match_specification() {
        let limits = DriverLimits::default();
        assert_eq!(limits.max_connections.get(), 10);
        assert_eq!(limits.max_connecting.get(), 2);
        assert_eq!(limits.max_waiters.get(), 1024);
        assert_eq!(limits.embedded_queue.get(), 1024);
        assert_eq!(limits.cursor_prefetch_pages.get(), 1);
    }

    #[test]
    fn terminal_and_feature_registries_are_closed_and_unique() {
        let terminal = [
            TerminalOutcome::Completed,
            TerminalOutcome::Refused,
            TerminalOutcome::CancelledBeforeDispatch,
            TerminalOutcome::CancelledAfterDispatch,
            TerminalOutcome::DeadlineExceeded,
            TerminalOutcome::CommitOutcomeUnknown,
        ];
        assert_eq!(terminal.len(), 6);
        let features: BTreeSet<_> = REQUIRED_ASYNC_DRIVER_FEATURES.iter().copied().collect();
        assert_eq!(features.len(), REQUIRED_ASYNC_DRIVER_FEATURES.len());
    }
}
