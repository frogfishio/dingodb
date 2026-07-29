//! Authority ceremony / store errors.

use thiserror::Error;

/// Fail-closed authority errors.
#[derive(Debug, Error)]
pub enum AuthorityError {
    /// Filesystem / IO failure.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Authority store integrity failure.
    #[error(transparent)]
    Store(#[from] AuthorityStoreError),
    /// Heap kernel validation failure.
    #[error("heap: {0}")]
    Heap(String),
    /// Store provisioning / catalog failure.
    #[error("provisioning: {0}")]
    Provisioning(String),
    /// Cryptographic or encoding failure.
    #[error("crypto: {0}")]
    Crypto(String),
    /// Caller argument invalid.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// Operation refused (reload-only path, wrong state, etc.).
    #[error("refused: {0}")]
    Refused(String),
}

/// Two-slot / chain store failures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthorityStoreError {
    /// Head or time-floor slot corrupt.
    #[error("authority corrupt: {0}")]
    Corrupt(&'static str),
    /// Two integrity-valid successors / unequal equal-sequence payloads.
    #[error("authority fork")]
    Fork,
    /// Selector / anchor mismatch with no recoverable head.
    #[error("authority anchor mismatch")]
    AnchorMismatch,
    /// Time floor attempted to move backward.
    #[error("time floor rollback refused")]
    TimeFloorRollback,
    /// Event chain gap or kind mismatch.
    #[error("authority event chain invalid: {0}")]
    EventChain(&'static str),
    /// Staged genesis bytes missing or disagree after authority commit.
    #[error("staged genesis unavailable or conflict")]
    StagedGenesisConflict,
}
