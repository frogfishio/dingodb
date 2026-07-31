//! Platform campaign plans (macOS Apple Silicon + Linux controlled runner).

use serde::{Deserialize, Serialize};

/// Minimum accepted-cell repetitions (SPEC §10 / plan §13).
pub const MIN_REPETITIONS: u32 = 5;
/// Minimum fresh process starts per accepted cell.
pub const MIN_PROCESSES: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformClass {
    /// macOS on Apple Silicon controlled runner.
    MacosAppleSilicon,
    /// Linux controlled runner.
    LinuxControlled,
    /// Synthetic/unit-test host (not a product baseline).
    SyntheticHarness,
}

impl PlatformClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MacosAppleSilicon => "macos_apple_silicon",
            Self::LinuxControlled => "linux_controlled",
            Self::SyntheticHarness => "synthetic_harness",
        }
    }

    /// Product baselines may only be claimed on controlled platforms.
    pub fn allows_product_baseline(self) -> bool {
        matches!(self, Self::MacosAppleSilicon | Self::LinuxControlled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignPlan {
    pub schema: String,
    pub campaign_id: String,
    pub platform: PlatformClass,
    pub profile: String,
    pub seed: u64,
    pub repetitions: u32,
    pub processes: u32,
    /// Max matrix cells to execute (smoke vs full).
    pub max_cells: usize,
    /// Whether multiproc 4K/8K finding cells are included.
    pub include_multiproc_finding: bool,
    pub notes: Vec<String>,
}

impl CampaignPlan {
    pub fn validate(&self) -> Result<(), String> {
        if self.repetitions < MIN_REPETITIONS {
            return Err(format!(
                "repetitions {} < MIN_REPETITIONS {}",
                self.repetitions, MIN_REPETITIONS
            ));
        }
        if self.processes < MIN_PROCESSES {
            return Err(format!(
                "processes {} < MIN_PROCESSES {}",
                self.processes, MIN_PROCESSES
            ));
        }
        if self.max_cells == 0 {
            return Err("max_cells must be > 0".into());
        }
        Ok(())
    }
}

/// macOS Apple Silicon controlled-runner campaign plan.
pub fn campaign_plan_macos_apple_silicon(seed: u64) -> CampaignPlan {
    CampaignPlan {
        schema: "residiuum-performance-campaign-plan-v1".into(),
        campaign_id: format!("pqh9-macos-as-{seed:016x}"),
        platform: PlatformClass::MacosAppleSilicon,
        profile: crate::PROFILE_ID.into(),
        seed,
        repetitions: MIN_REPETITIONS,
        processes: MIN_PROCESSES,
        max_cells: 32,
        include_multiproc_finding: true,
        notes: vec![
            "Controlled macOS Apple Silicon runner only for product baselines".into(),
            "Cross-machine aggregation forbidden".into(),
            "No absolute MB/s gate in PR CI".into(),
        ],
    }
}

/// Linux controlled-runner campaign plan.
pub fn campaign_plan_linux(seed: u64) -> CampaignPlan {
    CampaignPlan {
        schema: "residiuum-performance-campaign-plan-v1".into(),
        campaign_id: format!("pqh9-linux-{seed:016x}"),
        platform: PlatformClass::LinuxControlled,
        profile: crate::PROFILE_ID.into(),
        seed,
        repetitions: MIN_REPETITIONS,
        processes: MIN_PROCESSES,
        max_cells: 32,
        include_multiproc_finding: true,
        notes: vec![
            "Controlled Linux runner only for product baselines".into(),
            "Cross-machine aggregation forbidden".into(),
            "No absolute MB/s gate in PR CI".into(),
        ],
    }
}

/// In-process synthetic plan for unit tests (not a product baseline).
pub fn campaign_plan_synthetic(seed: u64, max_cells: usize) -> CampaignPlan {
    CampaignPlan {
        schema: "residiuum-performance-campaign-plan-v1".into(),
        campaign_id: format!("pqh9-synthetic-{seed:016x}"),
        platform: PlatformClass::SyntheticHarness,
        profile: crate::PROFILE_ID.into(),
        seed,
        repetitions: MIN_REPETITIONS,
        processes: MIN_PROCESSES,
        max_cells,
        include_multiproc_finding: true,
        notes: vec![
            "Synthetic harness campaign — not a product performance claim".into(),
            "Absolute throughput numbers are proxies from matrix driver".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_meet_minima() {
        for p in [
            campaign_plan_macos_apple_silicon(1),
            campaign_plan_linux(2),
            campaign_plan_synthetic(3, 4),
        ] {
            p.validate().unwrap();
            assert!(p.repetitions >= MIN_REPETITIONS);
            assert!(p.processes >= MIN_PROCESSES);
            assert_eq!(p.profile, crate::PROFILE_ID);
        }
        assert!(campaign_plan_macos_apple_silicon(0)
            .platform
            .allows_product_baseline());
        assert!(!campaign_plan_synthetic(0, 1)
            .platform
            .allows_product_baseline());
    }

    #[test]
    fn rejects_under_min_reps() {
        let mut p = campaign_plan_synthetic(1, 2);
        p.repetitions = 2;
        assert!(p.validate().is_err());
    }
}
