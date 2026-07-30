//! Store-layer errors.

use dingo_format::{FrameVerifyError, SegmentError};
use std::io;
use thiserror::Error;

/// Errors from store open, write, and recovery paths.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Underlying IO failure.
    #[error("io: {0}")]
    Io(#[from] io::Error),

    /// Frame encode/verify failed.
    #[error(transparent)]
    Frame(#[from] FrameVerifyError),

    /// In-memory segment writer failed.
    #[error(transparent)]
    Segment(#[from] SegmentError),

    /// Path exists but is not a DingoDB store (missing store-info).
    #[error("not a dingodb store: missing store-info at {0}")]
    NotAStore(std::path::PathBuf),

    /// Store already exists at path when create was exclusive.
    #[error("store already exists at {0}")]
    AlreadyExists(std::path::PathBuf),

    /// Draft item envelope could not be decoded.
    #[error("invalid item envelope: {0}")]
    BadEnvelope(&'static str),

    /// Subject key exceeds draft limits.
    #[error("subject too long (max {max} bytes)")]
    SubjectTooLong {
        /// Maximum allowed subject byte length.
        max: usize,
    },

    /// Body exceeds safety limits.
    #[error("payload too large for configured safety limits")]
    PayloadTooLarge,

    /// Corrupt or incomplete store metadata.
    #[error("corrupt store metadata: {0}")]
    CorruptMeta(&'static str),

    /// Control document failed validation; recovery action is documented.
    ///
    /// Used when a mutable control file (endpoints, dedup table, catalogs, …)
    /// is damaged and the previous generation is also unusable (DEF-021).
    #[error("corrupt control document {path}: {detail} (recovery: {recovery})")]
    CorruptControl {
        /// Absolute or store-relative path of the damaged document.
        path: String,
        /// Why the primary generation was rejected.
        detail: String,
        /// Operator / automatic recovery action (rebuild, use .prev, etc.).
        recovery: String,
    },

    /// Payload is only partially available (missing/corrupt chunks).
    #[error("payload only partially available")]
    PayloadPartial,

    /// Chunk reassembly found conflicting content at a manifest position.
    #[error("conflicting chunk content")]
    PayloadConflict,

    /// Requested historical `event_id` is not present in subject history (DEF-099).
    #[error("history event not found")]
    HistoryEventNotFound,

    /// Requested sealed segment is not registered or not on disk.
    #[error("segment not found")]
    SegmentNotFound,

    /// Required storage tier is offline or unmounted (Stage 9).
    #[error("storage tier offline: {0}")]
    TierOffline(&'static str),

    /// Segment bytes use a wire major this build cannot interpret.
    ///
    /// Authoritative bytes are preserved; interpretation is refused
    /// (`format-unsupported`, OVERVIEW §9.5).
    #[error("format unsupported: wire major {wire_major}")]
    FormatUnsupported {
        /// Unsupported wire major observed.
        wire_major: u8,
    },

    /// Media locator requires a backend this build does not ship (e.g. live S3/GCS).
    #[error("media backend unsupported: {0}")]
    MediaUnsupported(String),

    /// Another process or handle already holds the exclusive writer lock (DEF-020).
    #[error("store writer lock held: {0}")]
    WriterLockHeld(String),

    /// Scan/get coverage is incomplete; ordinary complete results are refused (DEF-012).
    #[error("coverage incomplete: {0}")]
    CoverageIncomplete(String),

    /// Client operation id reused with different content (DEF-010).
    #[error("consistency violation: {0}")]
    ConsistencyViolation(String),

    /// Injected failure from an armed failpoint (DEF-022 testing only).
    #[error("failpoint hit: {0}")]
    Failpoint(&'static str),

    /// OS CSPRNG was required and unavailable (DEF-025).
    ///
    /// Store/event/operation identity must not fall back to wall-clock or
    /// weak mixes when secure randomness is needed.
    #[error("secure randomness unavailable: {0}")]
    RandomUnavailable(String),

    /// Continuation token is malformed, tampered, expired shape, or wrong store (DEF-026).
    #[error("invalid scan cursor: {0}")]
    CursorInvalid(String),

    /// Continuation token generation no longer matches live store state (DEF-026).
    ///
    /// The scan generation fence changed (segment fingerprint and/or live count);
    /// restart the scan from the first page.
    #[error("stale scan cursor: {0}")]
    CursorStale(String),

    /// Heap capability check failed (HP-003 façades).
    #[error("heap capability: {0}")]
    HeapCapability(String),

    /// One-heap admission rejected the frame (HP-002).
    #[error("heap admit: {0}")]
    HeapAdmit(String),
}

impl StoreError {
    /// Whether this error is an ordinary IO failure.
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }
}