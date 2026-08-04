//! Operator lifecycle policies for tier retention (product follow-on).
//!
//! CSE-3: retention / secure-delete of user data MUST also erase Recovery
//! Shadow payloads (`.rsh`) via [`crate::recovery_shadow::secure_erase_shadow`] —
//! plaintext must not survive only on recovery media. Encryption / key rotation
//! treat `.rsh` as recovery-authoritative media (same key lifecycle as segment
//! payloads). Product recovery authority remains Materialized Chimera until
//! Stage 2 step 8.
//!
//! Automatic background enforcement is **not** required for Stage 9; this
//! module records declarative policy so operators and future schedulers share
//! one on-disk format. Evaluation is pure: given age/size signals, decide
//! target tier — transfer remains an explicit store API call.

use crate::error::StoreError;
use crate::tier::TierClass;
use std::fs;
use std::path::{Path, PathBuf};

/// Filename under `tiers/` for the lifecycle policy document.
pub const LIFECYCLE_POLICY_FILE: &str = "lifecycle.json";

/// One rule: when a segment is older than `min_age_secs` on `from_tier`,
/// prefer `to_tier` (copy or move left to the operator / scheduler).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LifecycleRule {
    /// Human name for logs.
    pub name: String,
    /// Source tier class name (`hot` / `warm` / `cold` / `archive`).
    pub from_tier: String,
    /// Destination tier class name.
    pub to_tier: String,
    /// Minimum age in seconds before the rule matches.
    pub min_age_secs: u64,
    /// When true, prefer move (delete source after verified copy); else copy.
    #[serde(default)]
    pub delete_source: bool,
}

impl LifecycleRule {
    /// Parse tier endpoints.
    pub fn from_tier_class(&self) -> Option<TierClass> {
        TierClass::parse(&self.from_tier)
    }

    /// Parse destination tier.
    pub fn to_tier_class(&self) -> Option<TierClass> {
        TierClass::parse(&self.to_tier)
    }

    /// Whether `age_secs` on `tier` matches this rule.
    pub fn matches(&self, tier: TierClass, age_secs: u64) -> bool {
        self.from_tier_class() == Some(tier) && age_secs >= self.min_age_secs
    }
}

/// On-disk lifecycle policy (JSON).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LifecyclePolicy {
    /// Format tag.
    #[serde(default = "default_format")]
    pub format: String,
    /// Ordered rules (first match wins).
    #[serde(default)]
    pub rules: Vec<LifecycleRule>,
}

fn default_format() -> String {
    "residiuum-lifecycle-1".into()
}

impl LifecyclePolicy {
    /// Empty policy (no automatic suggestions).
    pub fn new() -> Self {
        Self {
            format: default_format(),
            rules: Vec::new(),
        }
    }

    /// Load from `root/tiers/lifecycle.json`, or empty if missing.
    ///
    /// Falls back to the previous generation when the primary is corrupt (DEF-021).
    pub fn load(store_root: &Path) -> Result<Self, StoreError> {
        let path = policy_path(store_root);
        if path.is_file() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(pol) = serde_json::from_slice::<Self>(&bytes) {
                    return Ok(pol);
                }
            }
        }
        let prev = crate::atomic_file::previous_path(&path);
        if prev.is_file() {
            if let Ok(bytes) = fs::read(&prev) {
                if let Ok(pol) = serde_json::from_slice::<Self>(&bytes) {
                    return Ok(pol);
                }
            }
        }
        if path.is_file() || prev.is_file() {
            return Err(StoreError::CorruptControl {
                path: path.display().to_string(),
                detail: "parse tiers/lifecycle.json (and .prev) failed".into(),
                recovery: "restore a valid lifecycle.json or rewrite via LifecyclePolicy::save"
                    .into(),
            });
        }
        Ok(Self::new())
    }

    /// Persist under the store root (atomic durable; keeps previous generation).
    pub fn save(&self, store_root: &Path) -> Result<(), StoreError> {
        let path = policy_path(store_root);
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| StoreError::CorruptMeta("serialize tiers/lifecycle.json"))?;
        crate::atomic_file::write_atomic_keep_previous(&path, json.as_bytes())?;
        Ok(())
    }

    /// First matching rule for a segment age on `tier`, if any.
    pub fn evaluate(&self, tier: TierClass, age_secs: u64) -> Option<&LifecycleRule> {
        self.rules.iter().find(|r| r.matches(tier, age_secs))
    }
}

/// Path to the lifecycle policy file.
pub fn policy_path(store_root: &Path) -> PathBuf {
    store_root.join("tiers").join(LIFECYCLE_POLICY_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_first_match() {
        let mut pol = LifecyclePolicy::new();
        pol.rules.push(LifecycleRule {
            name: "warm-after-day".into(),
            from_tier: "hot".into(),
            to_tier: "warm".into(),
            min_age_secs: 86_400,
            delete_source: false,
        });
        pol.rules.push(LifecycleRule {
            name: "archive-after-year".into(),
            from_tier: "cold".into(),
            to_tier: "archive".into(),
            min_age_secs: 31_536_000,
            delete_source: true,
        });
        assert!(pol.evaluate(TierClass::Hot, 100).is_none());
        let r = pol.evaluate(TierClass::Hot, 86_400).unwrap();
        assert_eq!(r.name, "warm-after-day");
        assert!(!r.delete_source);
    }

    #[test]
    fn roundtrip_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut pol = LifecyclePolicy::new();
        pol.rules.push(LifecycleRule {
            name: "r".into(),
            from_tier: "warm".into(),
            to_tier: "cold".into(),
            min_age_secs: 60,
            delete_source: true,
        });
        pol.save(dir.path()).unwrap();
        let loaded = LifecyclePolicy::load(dir.path()).unwrap();
        assert_eq!(loaded, pol);
    }
}
