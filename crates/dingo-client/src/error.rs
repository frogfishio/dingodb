//! Client/protocol errors (MIT surface; no store types).

use thiserror::Error;

/// Stable machine-readable codes for protocol and transport failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Wire/protocol fields missing, malformed, or inconsistently optimistic.
    ProtocolViolation,
    /// Configured resource limit hit (e.g. frame too large).
    ResourceLimit,
    /// Shared token / authentication failed.
    AuthenticationFailed,
    /// Connect or request deadline exceeded.
    DeadlineExceeded,
    /// Underlying IO failure.
    Io,
    /// Internal / unexpected failure.
    Internal,
}

impl ErrorCode {
    /// Stable snake_case code string for logs and interop.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProtocolViolation => "protocol_violation",
            Self::ResourceLimit => "resource_limit",
            Self::AuthenticationFailed => "authentication_failed",
            Self::DeadlineExceeded => "deadline_exceeded",
            Self::Io => "io",
            Self::Internal => "internal",
        }
    }
}

/// Errors from framing, handshake, and client transport helpers.
#[derive(Debug, Error)]
pub enum Error {
    /// Wire response omitted or weakened required guarantee fields.
    #[error("protocol violation: {0}")]
    ProtocolViolation(String),

    /// Result materialization or frame size exceeded a limit.
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),

    /// Shared token missing or wrong.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Connect or request deadline exceeded.
    #[error("deadline exceeded: {0}")]
    DeadlineExceeded(String),

    /// Remote peer returned an error.
    #[error("remote error ({code}): {message}")]
    Remote {
        /// Stable code from the server when available.
        code: String,
        /// Human-readable message.
        message: String,
    },

    /// Internal client failure.
    #[error("internal: {0}")]
    Internal(String),

    /// Direct IO failure.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Wrap an IO error. Timeouts become [`Error::DeadlineExceeded`].
    pub fn from_io(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                Self::DeadlineExceeded(e.to_string())
            }
            _ => Self::Io(e),
        }
    }

    /// Stable machine code.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::ProtocolViolation(_) => ErrorCode::ProtocolViolation,
            Self::ResourceLimit(_) => ErrorCode::ResourceLimit,
            Self::AuthenticationFailed(_) => ErrorCode::AuthenticationFailed,
            Self::DeadlineExceeded(_) => ErrorCode::DeadlineExceeded,
            Self::Remote { code, .. } => match code.as_str() {
                "resource_limit" => ErrorCode::ResourceLimit,
                "authentication_failed" => ErrorCode::AuthenticationFailed,
                "protocol_violation" | "protocol_version_unsupported" => {
                    ErrorCode::ProtocolViolation
                }
                "deadline_exceeded" => ErrorCode::DeadlineExceeded,
                "io" => ErrorCode::Io,
                _ => ErrorCode::Internal,
            },
            Self::Internal(_) => ErrorCode::Internal,
            Self::Io(_) => ErrorCode::Io,
        }
    }
}
