//! Adaptive Write Optimiser (AWO) — pure model first (AWO-0).
//!
//! Normative: `doc/todo/performance-qualification/ADAPTIVE_WRITE_OPTIMISER_*`
//! and executable contracts under `spec/performance/awo/`.
//!
//! AWO-0 delivers selector arithmetic and golden-vector parity only.
//! Product write-path mutation is forbidden until AWO-0 accept.

pub mod model;
pub mod persist;
pub mod types;

pub use model::{decide, Decision, GoldenDecisionInput};
pub use persist::{BatchReservation, LaneTicket, AWO_FAILPOINTS};
pub use types::{
    decision_reason_ids, AwoPlan, AWO_PROFILE, DECISION_MARGIN_PPM_DEFAULT,
};