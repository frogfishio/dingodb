//! Adaptive Write Optimiser (AWO).
//!
//! Normative: `doc/todo/performance-qualification/ADAPTIVE_WRITE_OPTIMISER_*`
//! and executable contracts under `spec/performance/awo/`.
//!
//! - **AWO-0:** pure selector model + golden vectors.
//! - **AWO-1:** persist-before-publish on store batch paths + poison.
//! - **AWO-2:** persistent cooker pool, credit ledger, ordered ready ring.
//! - **AWO-3+:** product StoreHost admission (mode still disabled by default).

pub mod cooker;
pub mod credits;
pub mod model;
pub mod ordered_ready;
pub mod persist;
pub mod policy;
pub mod queue;
pub mod types;

pub use cooker::{
    cook_item_frame, CookOutcome, CookTask, CookedMutation, FrameBufferPool, PersistentCookerPool,
};
pub use credits::{
    mutation_credit, CreditError, CreditLedger, COMPLETION_SLOT_OVERHEAD, ENVELOPE_FIXED_OVERHEAD,
    FRAME_FRAMING_OVERHEAD, REQUEST_META_OVERHEAD,
};
pub use model::{decide, Decision, GoldenDecisionInput};
pub use ordered_ready::{OrderedReadyRing, ReadyError};
pub use persist::{BatchReservation, LaneTicket, AWO_FAILPOINTS};
pub use policy::{AdaptiveWriteMode, AdaptiveWritePolicy, PolicyError};
pub use queue::{BoundedQueue, QueueError};
pub use types::{
    decision_reason_ids, AwoPlan, AWO_PROFILE, DECISION_MARGIN_PPM_DEFAULT,
};