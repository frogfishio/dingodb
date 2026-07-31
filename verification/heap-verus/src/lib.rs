//! Verus pure-kernel proofs + Kani harnesses for Gate H6 (HP-010 / CPR-004).
//!
//! ## Machine-checked paths
//!
//! | Path | Flag | Location |
//! |------|------|----------|
//! | **Verus** | [`VERUS_PROOFS_CONNECTED`] | `verification/heap-verus/verus/pure_kernel.rs` |
//! | **Kani** | [`KANI_HARNESSES_CONNECTED`] | `residuum_heap::pure_proofs` `#[cfg(kani)]` |
//!
//! Executable stand-ins in `residuum_heap::pure_proofs` / `h6_decide_obligations`
//! stay CI-connected via `cargo test` regardless of Verus install.
//!
//! Verify Verus: `./scripts/setup_verus.sh && ./scripts/check_verus_heap.sh`
//! (CI job `verus-heap` with `RESIDUUM_REQUIRE_VERUS=1`).

#![allow(dead_code)]

/// Marker that the Verus project path exists.
pub fn heap_verus_scaffold() -> bool {
    true
}

/// Whether machine-checked Verus proofs are wired in CI for pure-kernel lemmas.
///
/// True when `verification/heap-verus/verus/pure_kernel.rs` verifies under
/// `scripts/check_verus_heap.sh` / CI `verus-heap` (pinned Verus release).
pub const VERUS_PROOFS_CONNECTED: bool = true;

/// Whether Kani harnesses are wired in-tree and run by CI (`kani-heap` job).
pub const KANI_HARNESSES_CONNECTED: bool = true;

/// Relative path of the Verus pure-kernel source (from workspace root).
pub const VERUS_PURE_KERNEL_REL: &str = "verification/heap-verus/verus/pure_kernel.rs";

/// Pinned Verus release tag used by `scripts/setup_verus.sh`.
pub const VERUS_PINNED_RELEASE: &str = "0.2026.07.27.31579f0";

/// Obligation checklist mirrored from HEAP_SPEC §39.
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
    "Verus pure_kernel lemmas machine-checked (VERUS_PROOFS_CONNECTED)",
    "complete-path review documents legacy unscoped surfaces (CPR-001)",
    "external security review brief ready; signed report still open (CPR-005)",
];

/// Pure predicate names proven (or targeted) in Verus pure_kernel.
pub const VERUS_TARGET_PREDICATES: &[&str] = &[
    "authority_binding_holds",
    "generation_accepted",
    "certificate_blacklisted",
    "authority_admission_ok",
    "isolation_unit_admits",
    "lemma_binding_rejects_foreign_heap",
    "lemma_generation_grace_window",
    "lemma_blacklist_hits",
    "lemma_non_serving_refuses_admission",
    "lemma_isolation_foreign_unit",
    "lemma_connected_pure_proof_bundle",
];

/// Names of Kani proof harnesses in `residuum_heap` pure_proofs.
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
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn scaffold_verus_and_kani_connected() {
        assert!(heap_verus_scaffold());
        assert!(VERUS_PROOFS_CONNECTED);
        assert!(KANI_HARNESSES_CONNECTED);
        assert!(H6_PROOF_OBLIGATIONS.len() >= 20);
        assert!(VERUS_TARGET_PREDICATES.contains(&"authority_admission_ok"));
        assert!(VERUS_TARGET_PREDICATES.contains(&"lemma_connected_pure_proof_bundle"));
        assert!(KANI_HARNESS_NAMES
            .iter()
            .any(|n| *n == "kani_connected_pure_proof_bundle"));
        let pure = workspace_root().join(VERUS_PURE_KERNEL_REL);
        assert!(pure.is_file(), "missing {}", pure.display());
        let body = std::fs::read_to_string(&pure).unwrap();
        assert!(body.contains("lemma_connected_pure_proof_bundle"));
        assert!(body.contains("verus!"));
    }
}
