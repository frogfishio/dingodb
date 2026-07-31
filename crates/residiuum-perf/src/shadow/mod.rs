//! PQH-5: PhysicalWritePlan + L2 Residiuum-shaped shadow writer.
//!
//! The closed plan type is the seam shared by a future store-boundary emitter
//! and the shadow executor. Shadow writes **opaque bytes only** — no parsing,
//! encoding, checksums, manifests, or index logic.
//!
//! Store-native emission of plans is residual: until the store exposes this
//! seam without changing outcomes, plans are produced by
//! [`plan::ShapeConfig`] matching the declared Residiuum physical shape.

mod equivalence;
mod plan;
mod replay;
mod shadow_exec;

pub use equivalence::{compare_traces, EquivalenceReport, SyscallShape};
pub use plan::{
    DestinationClass, PhysicalOp, PhysicalWritePlan, PlanBuilder, ShapeConfig, SyncBoundary,
    PLAN_SCHEMA,
};
pub use replay::{validate_plan_replay, ReplayError, ReplayReport};
pub use shadow_exec::{execute_shadow, ShadowReport, ShadowStats};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShadowError {
    #[error("shadow: {0}")]
    Msg(String),
    #[error("replay: {0}")]
    Replay(#[from] ReplayError),
    #[error("io: {0}")]
    Io(#[from] crate::envelope::IoError),
}

/// Architecture residual until store emits plans at the authoritative boundary.
pub const STORE_SEAM_STATUS: &str = "pending_store_boundary_emitter";
