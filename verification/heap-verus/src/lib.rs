//! Verus-oriented pure kernel proof targets for Gate H6 (HP-010).
//!
//! **Kani harnesses are connected** in `dingo_heap::pure_proofs` (`#[cfg(kani)]`)
//! and exercised by CI job `kani-heap` / `scripts/check_kani_heap.sh`.
//!
//! Machine-checked **Verus** proofs remain open — `VERUS_PROOFS_CONNECTED` stays
//! false until Verus is installed in CI and proves [`H6_PROOF_OBLIGATIONS`].
//! Executable stand-ins live in `dingo_heap::pure_proofs` and
//! `h6_decide_obligations` (always CI-connected via `cargo test`).

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

/// Whether Kani harnesses are wired in-tree and run by CI (`kani-heap` job).
///
/// Harnesses live in `dingo_heap` pure_proofs (`#[cfg(kani)]`); CI installs
/// `kani-verifier` and runs `scripts/check_kani_heap.sh`.
pub const KANI_HARNESSES_CONNECTED: bool = true;

/// Obligation checklist mirrored from HEAP_SPEC §39.
///
/// Each item has a CI-connected executable stand-in in `dingo-heap`
/// (`pure_proofs`, `h6_decide_obligations`, IsolationModel, AuthorityModel).
/// Kani re-checks the pure_proofs lemmas under `#[cfg(kani)]`.
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
    "Kani harnesses re-check pure_proofs lemmas (KANI_HARNESSES_CONNECTED)",
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

/// Names of Kani proof harnesses in `dingo_heap` pure_proofs.
pub const KANI_HARNESS_NAMES: &[&str] = &[
    "kani_binding_rejects_foreign_heap",
    "kani_generation_grace_window",
    "kani_blacklist_hits_certificate_hash",
    "kani_non_serving_refuses_admission",
    "kani_isolation_model_inv_walk",
    "kani_authority_model_inv_walk",
    "kani_connected_pure_proof_bundle",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_honest_about_verus_and_kani() {
        assert!(heap_verus_scaffold());
        assert!(!VERUS_PROOFS_CONNECTED);
        assert!(KANI_HARNESSES_CONNECTED);
        assert!(H6_PROOF_OBLIGATIONS.len() >= 20);
        assert!(VERUS_TARGET_PREDICATES.contains(&"authority_admission_ok"));
        assert!(KANI_HARNESS_NAMES
            .iter()
            .any(|n| *n == "kani_connected_pure_proof_bundle"));
    }
}
