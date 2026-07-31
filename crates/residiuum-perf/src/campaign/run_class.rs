//! Run classes (SPEC §6.4): smoke vs diagnostic vs qualification vs soak.
//!
//! Smoke is harness verification only — never a product bottleneck claim.
//! Qualification must meet duration and byte floors and demonstrate a
//! steady-state window; it must not silently cap operation counts.

use serde::{Deserialize, Serialize};

/// SPEC §6.4 measured-interval floors (both time and byte conditions apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunClass {
    /// Functional harness verification only (3 s and 64 MiB).
    Smoke,
    /// Local bottleneck search (30 s and 2 GiB where safe).
    Diagnostic,
    /// Repeatable accepted evidence (120 s + enough bytes to leave burst).
    Qualification,
    /// Long-tail / thermal (explicit, normally ≥1 h).
    Soak,
}

impl RunClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Diagnostic => "diagnostic",
            Self::Qualification => "qualification",
            Self::Soak => "soak",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "smoke" => Some(Self::Smoke),
            "diagnostic" | "diag" => Some(Self::Diagnostic),
            "qualification" | "qual" => Some(Self::Qualification),
            "soak" => Some(Self::Soak),
            _ => None,
        }
    }

    /// Minimum measured wall time (seconds).
    pub fn min_duration_secs(self) -> u64 {
        match self {
            Self::Smoke => 3,
            Self::Diagnostic => 30,
            Self::Qualification => 120,
            Self::Soak => 3600,
        }
    }

    /// Minimum logical acknowledged bytes.
    pub fn min_logical_bytes(self) -> u64 {
        match self {
            Self::Smoke => 64 * 1024 * 1024,
            Self::Diagnostic => 2 * 1024 * 1024 * 1024,
            // "enough bytes to leave initial burst" — 512 MiB floor for V1 harness.
            Self::Qualification => 512 * 1024 * 1024,
            Self::Soak => 8 * 1024 * 1024 * 1024,
        }
    }

    /// Smoke may use explicit small op budgets for unit/CI speed.
    /// Qualification/diagnostic/soak MUST NOT cap cells to tiny op counts.
    pub fn allows_smoke_op_cap(self) -> bool {
        matches!(self, Self::Smoke)
    }

    /// Only qualification/soak may emit primary bottleneck verdicts, and only
    /// after a sustained/stable window (enforced by campaign attach logic).
    pub fn may_emit_bottleneck_verdict(self) -> bool {
        matches!(self, Self::Qualification | Self::Soak)
    }

    /// Maximum ops for smoke unit/CI only (never applied in qualification).
    pub const SMOKE_MAX_OPS: u64 = 32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualification_floors_exceed_smoke() {
        assert!(RunClass::Qualification.min_duration_secs() >= 120);
        assert!(RunClass::Qualification.min_logical_bytes() >= 512 * 1024 * 1024);
        assert!(!RunClass::Qualification.allows_smoke_op_cap());
        assert!(RunClass::Smoke.allows_smoke_op_cap());
        assert!(!RunClass::Smoke.may_emit_bottleneck_verdict());
        assert!(RunClass::Qualification.may_emit_bottleneck_verdict());
    }
}
