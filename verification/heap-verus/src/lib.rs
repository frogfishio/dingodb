//! Verus-oriented pure kernel proof targets for Gate H6 (HP-010).
//!
//! This crate is a **scaffold**: it records the obligation checklist and pure
//! predicate documentation that Verus/Kani should eventually check. Executable
//! proofs today live in `dingo_heap::pure_proofs` and `h6_decide_obligations`
//! (CI-connected). Machine-checked Verus proofs remain **open** — `qualified`
//! stays false (CPR-004).

#![allow(dead_code)]

/// Marker that the Verus project path exists.
pub fn heap_verus_scaffold() -> bool {
    true
}

/// Whether machine-checked Verus proofs are wired in CI.
///
/// Hard-coded false until Verus is installed in CI and proves
/// [`H6_PROOF_OBLIGATIONS`] over the pure kernel.
pub const VERUS_PROOFS_CONNECTED: bool = false;

/// Whether Kani harnesses are wired in CI.
pub const KANI_HARNESSES_CONNECTED: bool = false;

/// Obligation checklist mirrored from HEAP_SPEC §39.
///
/// Each item has a CI-connected executable stand-in in `dingo-heap`
/// (`pure_proofs`, `h6_decide_obligations`, IsolationModel, AuthorityModel).
pub const H6_PROOF_OBLIGATIONS: &[&str] = &[
    "allow implies certificate heap equals snapshot heap",
    "allow implies requested immutable object is inside every applicable constraint",
    "unknown rights or critical constraints cannot allow",
    "a terminal state cannot allow",
    "epoch/generation acceptance follows the frozen grace rule",
    "generation_accepted encodes GenOK (current or previous-within-grace)",
    "certificate_blacklisted encodes NotBlacklisted / BlacklistAdd",
    "authority_admission_ok encodes AdmissionOK over resident snapshot",
    "mint grace deadline binds only previous-generation certificates",
    "confine_query_observation never returns a foreign heap_id",
    "IsolationModel invariants match HeapIsolation.tla Inv",
    "AuthorityModel invariants match HeapAuthority.tla Inv",
    "confine_operational_observation drops foreign-heap metrics/logs",
    "confine_operational_observation_under metadata-hardened denies aggregate_load/fine_timing_ms",
    "confine_export_heaps refuses foreign heap ids",
    "confine_health_detail strips paths/global counts for public and auth views",
    "confine_support_bundle drops foreign heaps and secret-bearing entries",
    "authority_binding_holds is required for every non-public decide path",
    "IsolationProfileRegistry matches spec/heap/isolation-profiles-v1.json",
    "heap-metadata-hardened denies aggregate_load and fine_timing_ms",
    "HeapAuthority.tla covers generation/blacklist/grace/terminal",
    "connected_pure_proof_bundle covers binding/gen/blacklist/admission/models",
    "complete-path review documents legacy unscoped surfaces (CPR-001)",
    "external security review brief ready; signed report still open (CPR-005)",
];

/// Pure predicate names intended as Verus `spec fn` / proof targets.
pub const VERUS_TARGET_PREDICATES: &[&str] = &[
    "authority_binding_holds",
    "generation_accepted",
    "certificate_blacklisted",
    "authority_admission_ok",
    "IsolationModel::invariants_hold",
    "AuthorityModel::inv",
    "confine_query_observation postcondition: heap_id == cap.heap_id",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_honest_about_missing_machine_proofs() {
        assert!(heap_verus_scaffold());
        assert!(!VERUS_PROOFS_CONNECTED);
        assert!(!KANI_HARNESSES_CONNECTED);
        assert!(H6_PROOF_OBLIGATIONS.len() >= 20);
        assert!(VERUS_TARGET_PREDICATES.contains(&"authority_admission_ok"));
    }
}
