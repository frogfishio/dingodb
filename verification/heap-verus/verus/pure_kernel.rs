//! Pure-kernel Verus proofs for Gate H6 / CPR-004 (HEAP_SPEC §39).
//!
//! These lemmas model the same obligations as `residuum_heap::pure_proofs` with
//! integer stand-ins for heap/deployment/epoch/generation identities so the
//! proofs stay free of I/O, crypto, and OS APIs.
//!
//! Verify (from repo root, after tools/verus is installed):
//!   ./scripts/check_verus_heap.sh
//!   # or: tools/verus/verus verification/heap-verus/verus/pure_kernel.rs
//!
//! Executable Rust stand-ins remain in `crates/residuum-heap` and are always
//! CI-connected via `cargo test` + Kani.

use vstd::prelude::*;

verus! {

// ---------------------------------------------------------------------------
// Models (spec-only): mirror pure decide predicates with u64 identities.
// ---------------------------------------------------------------------------

/// O-binding: certificate heap/deployment/epoch must match resident snapshot.
pub open spec fn authority_binding_holds(
    cert_heap: u64,
    snap_heap: u64,
    cert_dep: u64,
    snap_dep: u64,
    cert_epoch: u64,
    snap_epoch: u64,
) -> bool {
    cert_heap == snap_heap && cert_dep == snap_dep && cert_epoch == snap_epoch
}

/// O-gen: current generation, or previous while trusted time is within grace.
pub open spec fn generation_accepted(
    cert_gen: u64,
    snap_gen: u64,
    prev_gen: Option<u64>,
    grace_deadline: Option<u64>,
    now: u64,
) -> bool {
    cert_gen == snap_gen
        || (prev_gen == Some(cert_gen)
            && match grace_deadline {
                Some(d) => now <= d,
                None => false,
            })
}

/// O-black: same-generation certificate fingerprint blacklist hit.
pub open spec fn certificate_blacklisted(
    cert_gen: u64,
    cert_fp: u64,
    bl_gen: u64,
    bl_fp: u64,
    entry_present: bool,
) -> bool {
    entry_present && bl_gen == cert_gen && bl_fp == cert_fp
}

/// Serving states: Active / ReadOnly. Terminal: Purged (simplified).
pub open spec fn is_serving(admin: u64) -> bool {
    // 0 = Active, 1 = ReadOnly, 2 = Suspended, 3 = Retired, 4 = Purging, 5 = Purged
    admin == 0 || admin == 1
}

pub open spec fn is_terminal(admin: u64) -> bool {
    admin == 5
}

/// O-admit: binding + serving + generation + not blacklisted (issuer elided).
pub open spec fn authority_admission_ok(
    cert_heap: u64,
    snap_heap: u64,
    cert_dep: u64,
    snap_dep: u64,
    cert_epoch: u64,
    snap_epoch: u64,
    cert_gen: u64,
    snap_gen: u64,
    prev_gen: Option<u64>,
    grace_deadline: Option<u64>,
    now: u64,
    admin: u64,
    blacklisted: bool,
) -> bool {
    authority_binding_holds(cert_heap, snap_heap, cert_dep, snap_dep, cert_epoch, snap_epoch)
        && is_serving(admin)
        && !is_terminal(admin)
        && generation_accepted(cert_gen, snap_gen, prev_gen, grace_deadline, now)
        && !blacklisted
}

/// Isolation: unit owner equals bound heap (admit path).
pub open spec fn isolation_unit_admits(bound: u64, owner: u64) -> bool {
    bound == owner
}

// ---------------------------------------------------------------------------
// Proofs
// ---------------------------------------------------------------------------

/// Foreign heap identity fails authority binding.
pub proof fn lemma_binding_rejects_foreign_heap(a: u64, b: u64)
    requires
        a != b,
    ensures
        !authority_binding_holds(a, b, 1, 1, 1, 1),
{
}

/// Matching identities succeed.
pub proof fn lemma_binding_accepts_match(h: u64, d: u64, e: u64)
    ensures
        authority_binding_holds(h, h, d, d, e, e),
{
}

/// Previous generation accepted only inside the grace window.
pub proof fn lemma_generation_grace_window()
    ensures
        generation_accepted(1, 2, Some(1), Some(100), 50),
        !generation_accepted(1, 2, Some(1), Some(100), 101),
{
}

/// Blacklist same-gen fingerprint hits.
pub proof fn lemma_blacklist_hits()
    ensures
        certificate_blacklisted(1, 0xabc, 1, 0xabc, true),
        !certificate_blacklisted(1, 0xabc, 1, 0xdef, true),
        !certificate_blacklisted(1, 0xabc, 2, 0xabc, true),
{
}

/// Non-serving / terminal states refuse admission even when binding holds.
pub proof fn lemma_non_serving_refuses_admission()
    ensures
        !authority_admission_ok(
            1, 1, 1, 1, 1, 1, 1, 1, None, None, 0, 2, false,
        ), // Suspended
        !authority_admission_ok(
            1, 1, 1, 1, 1, 1, 1, 1, None, None, 0, 5, false,
        ), // Purged terminal
        authority_admission_ok(
            1, 1, 1, 1, 1, 1, 1, 1, None, None, 0, 0, false,
        ), // Active
{
}

/// Isolation: foreign owner cannot admit under bound heap.
pub proof fn lemma_isolation_foreign_unit(bound: u64, foreign: u64)
    requires
        bound != foreign,
    ensures
        !isolation_unit_admits(bound, foreign),
        isolation_unit_admits(bound, bound),
{
}

/// Bundle: all concrete pure lemmas used by the H6 connected evidence path.
pub proof fn lemma_connected_pure_proof_bundle()
    ensures
        true,
{
    lemma_binding_rejects_foreign_heap(10, 11);
    lemma_binding_accepts_match(10, 20, 1);
    lemma_generation_grace_window();
    lemma_blacklist_hits();
    lemma_non_serving_refuses_admission();
    lemma_isolation_foreign_unit(1, 2);
}

fn main() {
}

} // verus!
