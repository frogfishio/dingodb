//! PQH-6: L3 CPU transformation pipeline and stage probes.
//!
//! Runs selectable CPU stages against a bounded null/memory sink — **no
//! filesystem I/O** in the timed interval. Stage probes feed residual
//! accounting (SPEC residual ε ≤ 5% for attribution closure).

mod residual;
mod sink;
mod stages;
mod timeline;

pub use residual::{residual_from_stage_ns, ResidualReport, RESIDUAL_MAX_FRACTION};
pub use sink::{BoundedSink, SinkMode};
pub use stages::{
    run_l3_pipeline, InjectedDelays, L3Config, L3Report, StageId, StageSet, STAGE_ORDER,
};
pub use timeline::{check_timeline, TimelineEvent, TimelineReport, TimelineViolation};

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PipelineError {
    #[error("pipeline: {0}")]
    Msg(String),
    #[error("invalid stage order: {0}")]
    StageOrder(String),
    #[error("timeline: {0}")]
    Timeline(String),
}
