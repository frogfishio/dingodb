//! Fail-closed heap errors (`HEAP_SPEC` §22).

use thiserror::Error;

/// Public-facing rejection. Callers must not learn heap existence from this.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("heap unavailable")]
pub struct HeapUnavailable;

/// Internal diagnostic cause (never returned on the qualified wire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapUnavailableCause {
    /// Malformed encoding or failed signature.
    MalformedOrBadSignature,
    /// Unknown or reserved operation.
    UnknownOperation,
    /// Missing right.
    InsufficientRights,
    /// Constraint intersection empty or violated.
    ConstraintDenied,
    /// Administrative state does not admit the operation.
    InvalidState,
    /// Epoch/generation/revision/chain mismatch.
    StaleAuthority,
    /// Clock outside validity window.
    NotYetValidOrExpired,
    /// Certificate blacklisted.
    Blacklisted,
    /// Identity validation failed.
    InvalidIdentity,
    /// Other fail-closed path.
    Denied,
}

/// Closed error type for kernel APIs.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HeapError {
    /// Public-safe rejection.
    #[error(transparent)]
    Unavailable(#[from] HeapUnavailable),
    /// Internal diagnostic paired with [`HeapUnavailable`].
    #[error("heap unavailable ({cause:?})")]
    UnavailableDetailed {
        /// Stable public projection.
        public: HeapUnavailable,
        /// Internal cause.
        cause: HeapUnavailableCause,
    },
    /// Invalid argument to a constructor or decoder.
    #[error("invalid heap argument: {0}")]
    InvalidArgument(&'static str),
}

impl HeapError {
    /// Construct a fail-closed unavailable error with internal cause.
    pub fn unavailable(cause: HeapUnavailableCause) -> Self {
        Self::UnavailableDetailed {
            public: HeapUnavailable,
            cause,
        }
    }

    /// Public projection used on the wire.
    pub fn public_code(&self) -> &'static str {
        "heap_unavailable"
    }
}
