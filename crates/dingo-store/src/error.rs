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

    /// Payload is only partially available (missing/corrupt chunks).
    #[error("payload only partially available")]
    PayloadPartial,

    /// Chunk reassembly found conflicting content at a manifest position.
    #[error("conflicting chunk content")]
    PayloadConflict,
}

impl StoreError {
    /// Whether this error is an ordinary IO failure.
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }
}
