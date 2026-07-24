//! SDK-level errors (DX_SPEC §15 everyday surface).

use dingo_store::StoreError;
use thiserror::Error;

/// Stable machine-readable error codes (DX_SPEC §15).
///
/// Everyday callers should match on [`Error::code`] rather than English text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Store or path already exists when exclusive create was requested.
    AlreadyExists,
    /// Requested key/entity is absent (when an API chooses error over `None`).
    NotFound,
    /// Optimistic concurrency / version precondition failed.
    VersionConflict,
    /// Collection name, key, or argument failed validation.
    ValidationFailed,
    /// Filter / query object is malformed or uses unknown operators.
    QueryInvalid,
    /// Query would need an explicit resource budget (Stage 6+).
    QueryBudgetRequired,
    /// Configured resource limit hit.
    ResourceLimit,
    /// Coverage is incomplete; absence cannot be proven.
    CoverageIncomplete,
    /// Required partition/tier is offline (cluster).
    PartitionUnavailable,
    /// Requested durability mode cannot be met.
    DurabilityUnavailable,
    /// Payload or store bytes failed integrity / are damaged.
    DataDamaged,
    /// Payload is only partially available (chunks; Stage 6+).
    PayloadPartial,
    /// Encoding or format is not supported by this build.
    FormatUnsupported,
    /// JSON vs bytes (or other) type mismatch on get.
    TypeMismatch,
    /// Underlying IO failure.
    Io,
    /// Internal / unexpected failure.
    Internal,
}

impl ErrorCode {
    /// Stable snake_case code string for logs and interop.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AlreadyExists => "already_exists",
            Self::NotFound => "not_found",
            Self::VersionConflict => "version_conflict",
            Self::ValidationFailed => "validation_failed",
            Self::QueryInvalid => "query_invalid",
            Self::QueryBudgetRequired => "query_budget_required",
            Self::ResourceLimit => "resource_limit",
            Self::CoverageIncomplete => "coverage_incomplete",
            Self::PartitionUnavailable => "partition_unavailable",
            Self::DurabilityUnavailable => "durability_unavailable",
            Self::DataDamaged => "data_damaged",
            Self::PayloadPartial => "payload_partial",
            Self::FormatUnsupported => "format_unsupported",
            Self::TypeMismatch => "type_mismatch",
            Self::Io => "io",
            Self::Internal => "internal",
        }
    }
}

/// Errors from open, collection access, put/get/delete, and query.
#[derive(Debug, Error)]
pub enum Error {
    /// Underlying store failure.
    #[error(transparent)]
    Store(#[from] StoreError),

    /// Collection name is empty, too long, or contains NUL.
    #[error("invalid collection name: {0}")]
    InvalidCollectionName(&'static str),

    /// Key is empty, too long, or contains NUL.
    #[error("invalid key: {0}")]
    InvalidKey(&'static str),

    /// Stored payload is not valid UTF-8 JSON for a JSON get.
    #[error("invalid JSON payload: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// Payload type tag does not match the requested API (JSON vs bytes).
    #[error("payload type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        /// Requested logical type.
        expected: &'static str,
        /// On-disk type tag name.
        found: &'static str,
    },

    /// Payload is missing a type tag, truncated, or fails integrity.
    #[error("corrupt or unsupported payload encoding")]
    BadPayload,

    /// Filter / query specification is invalid.
    #[error("invalid query: {0}")]
    QueryInvalid(String),

    /// Result materialization exceeded a configured or explicit limit.
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),
}

impl Error {
    /// Stable machine code (DX_SPEC §15). Prefer this over parsing [`Display`].
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Store(e) => map_store(e),
            Self::InvalidCollectionName(_) | Self::InvalidKey(_) => ErrorCode::ValidationFailed,
            Self::InvalidJson(_) => ErrorCode::DataDamaged,
            Self::TypeMismatch { .. } => ErrorCode::TypeMismatch,
            Self::BadPayload => ErrorCode::DataDamaged,
            Self::QueryInvalid(_) => ErrorCode::QueryInvalid,
            Self::ResourceLimit(_) => ErrorCode::ResourceLimit,
        }
    }

    /// Whether this is an ordinary store IO failure.
    pub fn is_io(&self) -> bool {
        matches!(self.code(), ErrorCode::Io)
    }

    /// Whether the payload appears damaged or unreadable as requested.
    pub fn is_data_damaged(&self) -> bool {
        matches!(self.code(), ErrorCode::DataDamaged)
    }

    /// Whether the error is a validation problem on names/keys.
    pub fn is_validation(&self) -> bool {
        matches!(self.code(), ErrorCode::ValidationFailed)
    }

    /// Whether the filter/query object was rejected.
    pub fn is_query_invalid(&self) -> bool {
        matches!(self.code(), ErrorCode::QueryInvalid)
    }
}

fn map_store(e: &StoreError) -> ErrorCode {
    match e {
        StoreError::Io(_) => ErrorCode::Io,
        StoreError::AlreadyExists(_) => ErrorCode::AlreadyExists,
        StoreError::NotAStore(_) => ErrorCode::ValidationFailed,
        StoreError::SubjectTooLong { .. } => ErrorCode::ValidationFailed,
        StoreError::PayloadTooLarge => ErrorCode::ResourceLimit,
        StoreError::BadEnvelope(_) | StoreError::CorruptMeta(_) => ErrorCode::DataDamaged,
        StoreError::Frame(_) | StoreError::Segment(_) => ErrorCode::DataDamaged,
    }
}
