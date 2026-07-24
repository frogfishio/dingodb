//! DingoDB SDA examination host (Stage 5).
//!
//! Projects Stage 2–3 salvage evidence into normative
//! [`ExaminationUnit`] values ([`SDA_PROFILE.md`](../../SDA_PROFILE.md)),
//! streams them in deterministic order, and evaluates pure SDA programs over
//! each unit or over a bounded page.
//!
//! Storage damage remains examination **data** (status tags, holes,
//! uncertainty). SDA language errors remain `Fail`.
//!
//! Normative: SDA_PROFILE; OVERVIEW §11; DELIVERY_PLAN Stage 5.

#![deny(missing_docs)]

mod error;
mod eval;
mod limits;
mod project;
mod stream;
mod unit;

pub use error::ExamineError;
pub use eval::{
    eval_page, eval_unit, filter_holes, filter_status, filter_units, filter_verified_complete,
    is_sda_fail, is_sda_true, map_units, unit_as_value,
};
pub use limits::{ExamineLimits, ExaminePage, PageCoverage};
pub use project::{project_bytes, project_region, ProjectOptions};
pub use stream::{examine_bytes, examine_sources, examine_store};
pub use unit::{
    cmp_units, EnvelopeEntry, EnvelopeValue, ExaminationUnit, Extent, IntegrityEvidence,
    PayloadInfo, PhysicalLocation, ProvenanceEntry,
};
