//! Cache / lifecycle classes (programme §7.3) with honest cold definitions.
//!
//! Law: “cold” must state how it was obtained. Reopen ≠ automatic device-cache cold.

use crate::metrics::LifecycleClass;
use serde::{Deserialize, Serialize};

/// How a “cold” measurement was obtained (required when claiming cold).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColdMethod {
    /// Fresh OS process; page cache may still be warm.
    FreshProcess,
    /// Drop process handles and re-open store root (product reopen).
    StoreReopen,
    /// Host attempted page-cache drop (platform-specific; may be unavailable).
    AttemptedPageCacheDrop,
    /// Explicitly not cold — warm steady-state measurement.
    NotColdWarmSteady,
    /// Declared damage path; surviving islands only.
    DeclaredDamageSurvivors,
}

impl ColdMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreshProcess => "fresh_process",
            Self::StoreReopen => "store_reopen",
            Self::AttemptedPageCacheDrop => "attempted_page_cache_drop",
            Self::NotColdWarmSteady => "not_cold_warm_steady",
            Self::DeclaredDamageSurvivors => "declared_damage_survivors",
        }
    }
}

/// Lifecycle class with honesty metadata for evidence bundles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleSpec {
    pub class: LifecycleClass,
    pub cold_method: ColdMethod,
    /// Human-readable constraint / residual.
    pub notes: String,
    /// True if this class may be scored as “device cold” (almost never for reopen alone).
    pub claims_device_cold: bool,
}

impl LifecycleSpec {
    pub fn for_class(class: LifecycleClass) -> Self {
        match class {
            LifecycleClass::WarmSteady => Self {
                class,
                cold_method: ColdMethod::NotColdWarmSteady,
                notes: "Warmed steady state; not cold.".into(),
                claims_device_cold: false,
            },
            LifecycleClass::FreshReopen => Self {
                class,
                cold_method: ColdMethod::StoreReopen,
                notes: "Process/store reopen only — page cache may remain warm; not device cold."
                    .into(),
                claims_device_cold: false,
            },
            LifecycleClass::LargerThanMemory => Self {
                class,
                cold_method: ColdMethod::NotColdWarmSteady,
                notes: "Working set > host memory ratio R400; cold not implied.".into(),
                claims_device_cold: false,
            },
            LifecycleClass::ReadOnly => Self {
                class,
                cold_method: ColdMethod::NotColdWarmSteady,
                notes: "Read-only load; no concurrent writers.".into(),
                claims_device_cold: false,
            },
            LifecycleClass::ConcurrentWrites => Self {
                class,
                cold_method: ColdMethod::NotColdWarmSteady,
                notes: "Readers concurrent with writers under declared consistency.".into(),
                claims_device_cold: false,
            },
            LifecycleClass::RotationCompaction => Self {
                class,
                cold_method: ColdMethod::StoreReopen,
                notes: "Segment rotation/compaction/rebuild cycle; measure drain honesty.".into(),
                claims_device_cold: false,
            },
            LifecycleClass::DeclaredDamage => Self {
                class,
                cold_method: ColdMethod::DeclaredDamageSurvivors,
                notes: "Declared damage with surviving readable islands; no false completeness."
                    .into(),
                claims_device_cold: false,
            },
        }
    }

    /// All programme §7.3 classes with frozen honesty notes.
    pub fn all_programme_classes() -> Vec<Self> {
        [
            LifecycleClass::WarmSteady,
            LifecycleClass::FreshReopen,
            LifecycleClass::LargerThanMemory,
            LifecycleClass::ReadOnly,
            LifecycleClass::ConcurrentWrites,
            LifecycleClass::RotationCompaction,
            LifecycleClass::DeclaredDamage,
        ]
        .into_iter()
        .map(Self::for_class)
        .collect()
    }
}

/// Validate that a cell run does not claim device cold without the right method.
pub fn validate_cold_claim(spec: &LifecycleSpec) -> Result<(), String> {
    if spec.claims_device_cold {
        match spec.cold_method {
            ColdMethod::AttemptedPageCacheDrop => Ok(()),
            other => Err(format!(
                "device cold claim requires attempted_page_cache_drop, got {}",
                other.as_str()
            )),
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_lifecycle_classes() {
        let all = LifecycleSpec::all_programme_classes();
        assert_eq!(all.len(), 7);
        // Reopen must not claim device cold.
        let reopen = LifecycleSpec::for_class(LifecycleClass::FreshReopen);
        assert!(!reopen.claims_device_cold);
        assert_eq!(reopen.cold_method, ColdMethod::StoreReopen);
        validate_cold_claim(&reopen).unwrap();
    }

    #[test]
    fn device_cold_requires_page_cache_drop() {
        let mut bad = LifecycleSpec::for_class(LifecycleClass::FreshReopen);
        bad.claims_device_cold = true;
        assert!(validate_cold_claim(&bad).is_err());
        bad.cold_method = ColdMethod::AttemptedPageCacheDrop;
        assert!(validate_cold_claim(&bad).is_ok());
    }
}
