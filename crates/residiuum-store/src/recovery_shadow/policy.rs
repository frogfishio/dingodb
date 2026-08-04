//! Shadow reclaim policy — dual-run vs post-flip (Stage 2 step 6 lock).
//!
//! Clarification (normative before product flip):
//!
//! > During dual-run, Materialized may satisfy recovery authority. After the
//! > flip, reclaim must **always** require durable replacement Shadow coverage;
//! > “when present” is no longer sufficient.
//!
//! Compaction must never retire the last valid recovery source without a
//! durable replacement under [`ShadowReclaimPolicy::RequireReplacementShadow`].

use std::sync::atomic::{AtomicU8, Ordering};

const DUAL_RUN: u8 = 0;
const REQUIRE_REPLACEMENT: u8 = 1;

static POLICY: AtomicU8 = AtomicU8::new(DUAL_RUN);

/// How compaction may retire Recovery Shadows relative to Materialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowReclaimPolicy {
    /// Dual-run (steps 1–7): Materialized may satisfy recovery authority.
    /// Missing replacement `.rsh` does **not** block reclaim; old Shadows are
    /// still erased so retention does not leave orphan payloads.
    DualRunMaterializedAuthority,
    /// Post-flip (step 8+): reclaim **must** have durable replacement Shadow.
    /// Soft “when present” fallback is forbidden.
    RequireReplacementShadow,
}

/// Current process policy (default: dual-run until product flip).
pub fn shadow_reclaim_policy() -> ShadowReclaimPolicy {
    match POLICY.load(Ordering::SeqCst) {
        REQUIRE_REPLACEMENT => ShadowReclaimPolicy::RequireReplacementShadow,
        _ => ShadowReclaimPolicy::DualRunMaterializedAuthority,
    }
}

/// Set reclaim policy (product flip or CSE tests).
pub fn set_shadow_reclaim_policy(policy: ShadowReclaimPolicy) {
    let v = match policy {
        ShadowReclaimPolicy::DualRunMaterializedAuthority => DUAL_RUN,
        ShadowReclaimPolicy::RequireReplacementShadow => REQUIRE_REPLACEMENT,
    };
    POLICY.store(v, Ordering::SeqCst);
}

/// Restore dual-run default (test cleanup).
pub fn reset_shadow_reclaim_policy_for_tests() {
    set_shadow_reclaim_policy(ShadowReclaimPolicy::DualRunMaterializedAuthority);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dual_run() {
        reset_shadow_reclaim_policy_for_tests();
        assert_eq!(
            shadow_reclaim_policy(),
            ShadowReclaimPolicy::DualRunMaterializedAuthority
        );
    }

    #[test]
    fn can_switch_to_require_replacement() {
        set_shadow_reclaim_policy(ShadowReclaimPolicy::RequireReplacementShadow);
        assert_eq!(
            shadow_reclaim_policy(),
            ShadowReclaimPolicy::RequireReplacementShadow
        );
        reset_shadow_reclaim_policy_for_tests();
    }
}
