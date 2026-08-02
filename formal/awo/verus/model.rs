//! AWO pure-kernel Verus stub (AWO-0).
//!
//! Deepens in AWO-6. This file records the intended pure request / lane /
//! credit / ACK kernel surface without claiming verified proofs.
//!
//! Normative: ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md §17;
//! executable goldens live under `spec/performance/awo/`.

#![allow(dead_code)]

/// Closed profile id.
pub const AWO_PROFILE: &str = "residiuum-adaptive-write-v1";

/// Plan class for pure selection (mirrors Rust `AwoPlan`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    Natural,
    Batch,
}

/// Pure decision result (reason id is a closed string from decision-reasons-v1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decision {
    /// Selected plan.
    pub plan: Plan,
    /// Closed reason id.
    pub reason: &'static str,
}

/// Placeholder: real Verus specs attach to `residiuum_store::adaptive_write::model::decide`.
///
/// I/O timing and predictor accuracy are **assumptions**, never theorem conclusions.
pub fn decide_stub_docs_only() -> Decision {
    Decision {
        plan: Plan::Natural,
        reason: "natural_insufficient_evidence",
    }
}
