# Seal Fast Lane — 2026-08-04

Status: **implemented + measured** (ack gate not yet met — residual documented)  
Evidence: `artifacts/` (outside active `doc/todo/`).

## Architecture

Sealing is split into two systems:

1. **Authoritative seal** (put / rotate critical path + seal worker)
   - O(1) rename `active` → `pending`, start replacement active
   - Worker: summary append + publish into `segments/` → `SealDone`
   - Crash recovery via pending finalize (`recover_all_pending`)
2. **Derived enrichment** (separate `residiuum-seal-enrich` worker)
   - Hydra + Chimera (+ cheap catalog stub / BLAKE3 for tier apply)
   - **Never** counted in `max_pending_seals` / write backpressure
   - May lag; rebuildable from sealed segments

Critical rule enforced: derived backlog cannot stall `rotate_active_async`
beyond authoritative finalize lag.

Failpoints (crash matrix): `store.seal.before_authoritative_rename`,
`store.seal.after_authoritative_publish`, `store.seal.after_derived_enrichment`.

## Measurement (APFS · Real Full · 256 MiB · 8 KiB · c=8 · seed=42 · **64 MiB** seal)

| Metric | Value | Gate |
|---|---:|---|
| Ack TPS | 49834 | FAIL (≥ 74700 = 90% of ~83K) |
| Sealed @ last ack | 3 | PASS (≥ 2) |
| Pending seals @ ack | 1 | (info) |
| Campaign TPS | 8744 | (info) |
| Reopen exact (coverage scan) | yes | PASS |

High-threshold control same bed: **ack ≈ 82008** (confirms ~83K ceiling).

### Reading

- Pre–Seal Fast Lane @ 64 MiB: **~25K** ack (Chimera held the seal lane).
- Post–Seal Fast Lane @ 64 MiB: **~50K** ack (~2×), many rotations, exact reopen.
- Still **~39% below** the no-mid-run-seal control (~82K). Residual is
  **authoritative finalize** still `fs::read`ing ~64 MiB pending images on the
  seal worker while puts continue — disk/CPU contention, not derived enrichment.
- End-of-run drain dropped from ~1.1 s (Chimera-bound) to ~0.15 s.

### Next residual (not this package)

Append-summary finalize **without** full pending re-read (or hand off an
already-sealed RAM image without holding the put mutex) to close the remaining
gap to the ~83K control. Do **not** return to AWO / append tuning until that
lane is addressed.

## Harness

`residiuum-testrig seal-fast-lane`
