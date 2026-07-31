//! Single-node qualification claim surface (`HEAP_SPEC` §27 / HP-010).
//!
//! The product MAY advertise the qualified `residiuum-heap-v1` profile only when
//! [`may_advertise_qualified`] returns `true`. Until every mandatory HP-010
//! matrix gate and drill is `accept`, this remains `false`.

/// Machine-readable matrix path relative to the workspace root.
pub const HP010_MATRIX_REL: &str = "spec/heap/qualification/hp010-matrix-v1.json";

/// Whether this build may advertise a qualified `residiuum-heap-v1` claim.
///
/// Hard-coded false until HP-010 records complete, reproducible evidence and
/// Gate H6 passes. Flipping this without matrix completion is a claim-level
/// defect (§41).
pub const QUALIFIED_CLAIM: bool = false;

/// Profile label that may be advertised only when [`QUALIFIED_CLAIM`] is true.
pub const QUALIFIED_PROFILE: &str = "residiuum-heap-v1";

/// Product language required before Gate H6 (§27).
pub const PRE_QUALIFICATION_LANGUAGE: &str = concat!(
    "Residiuum provides named heap namespaces; strong access-isolation ",
    "qualification is in progress."
);

/// Returns whether a release may advertise the qualified heap profile.
#[must_use]
pub fn may_advertise_qualified() -> bool {
    QUALIFIED_CLAIM
}

/// Honest product language for the current claim level.
#[must_use]
pub fn claim_language() -> &'static str {
    if QUALIFIED_CLAIM {
        "Cryptographically authorized systems are logically isolated between heaps."
    } else {
        PRE_QUALIFICATION_LANGUAGE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_claim_remains_false_until_hp010_complete() {
        assert!(!may_advertise_qualified());
        assert!(!QUALIFIED_CLAIM);
        assert_eq!(claim_language(), PRE_QUALIFICATION_LANGUAGE);
        assert_eq!(QUALIFIED_PROFILE, crate::HEAP_PROFILE);
    }
}
