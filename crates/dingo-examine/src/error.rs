//! Examination host errors.

use dingo_store::StoreError;
use sda_core::SdaError;
use thiserror::Error;

/// Errors from projection, streaming, or SDA evaluation.
#[derive(Debug, Error)]
pub enum ExamineError {
    /// Underlying store IO or open failure.
    #[error(transparent)]
    Store(#[from] StoreError),

    /// Ordinary filesystem IO outside the store layer.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// SDA parse / eval failure (language error, not storage damage).
    #[error("sda: {0}")]
    Sda(#[from] SdaError),

    /// SDA program returned a non-boolean where a predicate was required.
    #[error("sda filter program must evaluate to Bool, got {0}")]
    FilterNotBool(String),

    /// Host resource limit would be required for a complete answer.
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(&'static str),
}
