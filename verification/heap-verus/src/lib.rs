//! Verus placeholder — pure kernel proofs land with Gate H6.
//!
//! HP-001 delivers the executable Rust decision function; HP-010 records the
//! isolation observation helpers (`confine_query_observation`) as differential
//! evidence. Full Verus proofs connecting `decide` / confinement to
//! `HeapIsolation.tla` remain open — `qualified` stays false.

#![allow(dead_code)]

/// Marker that the Verus project path exists.
pub fn heap_verus_scaffold() -> bool {
    true
}

/// Obligation checklist mirrored from HEAP_SPEC §39 (documentation only until
/// Verus is wired in CI).
pub const H6_PROOF_OBLIGATIONS: &[&str] = &[
    "allow implies certificate heap equals snapshot heap",
    "allow implies requested immutable object is inside every applicable constraint",
    "unknown rights or critical constraints cannot allow",
    "a terminal state cannot allow",
    "epoch/generation acceptance follows the frozen grace rule",
    "confine_query_observation never returns a foreign heap_id",
];
