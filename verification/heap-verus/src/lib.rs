//! Verus placeholder — pure kernel proofs land with Gate H6.
//!
//! HP-001 delivers the executable Rust decision function; HP-010 records:
//! - isolation observation helpers (`confine_query_observation`, health/support);
//! - connected Rust models (`IsolationModel`, `AuthorityModel`) mirroring TLA Inv;
//! - executable §39 obligations (`dingo_heap::h6_decide_obligations`) including
//!   `generation_accepted` and `certificate_blacklisted`.
//! Full Verus proofs remain open — `qualified` stays false.

#![allow(dead_code)]

/// Marker that the Verus project path exists.
pub fn heap_verus_scaffold() -> bool {
    true
}

/// Obligation checklist mirrored from HEAP_SPEC §39 (documentation only until
/// Verus is wired in CI). Each item has a CI-connected executable stand-in.
pub const H6_PROOF_OBLIGATIONS: &[&str] = &[
    "allow implies certificate heap equals snapshot heap",
    "allow implies requested immutable object is inside every applicable constraint",
    "unknown rights or critical constraints cannot allow",
    "a terminal state cannot allow",
    "epoch/generation acceptance follows the frozen grace rule",
    "generation_accepted encodes GenOK (current or previous-within-grace)",
    "certificate_blacklisted encodes NotBlacklisted / BlacklistAdd",
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
];
