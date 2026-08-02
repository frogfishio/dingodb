//! Public constants and closed plan/reason identifiers for AWO-0.

/// Closed AWO profile identity (`spec/performance/awo/profile-v1.json`).
pub const AWO_PROFILE: &str = "residiuum-adaptive-write-v1";

/// Default decision margin in parts-per-million (10% = 100_000).
///
/// Matches `policy-v1.json` `decision_margin_ppm` and implementation plan §12.
pub const DECISION_MARGIN_PPM_DEFAULT: u32 = 100_000;

/// Selected physical execution plan class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AwoPlan {
    /// Ordinary non-coalesced release for the selected requests.
    Natural,
    /// Coalesced cook and/or persist for independent requests.
    Batch,
}

impl AwoPlan {
    /// Wire / registry id (`natural` | `batch`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Natural => "natural",
            Self::Batch => "batch",
        }
    }

    /// Parse a closed plan id.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "natural" => Some(Self::Natural),
            "batch" => Some(Self::Batch),
            _ => None,
        }
    }
}

/// Closed decision-reason ids from `decision-reasons-v1.json` (stable strings).
pub fn decision_reason_ids() -> &'static [&'static str] {
    &[
        "natural_single_request",
        "natural_no_positive_gain",
        "natural_insufficient_evidence",
        "natural_stale_model",
        "natural_deadline",
        "natural_deadline_mitigation",
        "natural_incompatible",
        "natural_memory_bound",
        "natural_arithmetic_overflow",
        "natural_tie",
        "batch_existing_backlog",
        "batch_predicted_arrival_gain",
        "batch_deadline_mitigation",
        "forced_deadline",
        "forced_max_entries",
        "forced_max_bytes",
        "forced_segment_boundary",
        "forced_fence",
        "forced_drain",
        "controller_fallback",
    ]
}
