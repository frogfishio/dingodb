# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane = architectural accept (principal)**;  
**Derived Catalog Checkpointing = package accept (principal)** —
gates met; O(n²) catalog persist risk removed; sustained ack
**47.8K → 57.6K TPS** (~+20.5%); `catalog_apply` under 1% of ack wall.  
**Next (only):** three-cell lifecycle attribution — see
[THREE_CELL_LIFECYCLE_ATTRIBUTION.md](./THREE_CELL_LIFECYCLE_ATTRIBUTION.md).  
**AWO paused** until those three medians exist.  
Date: 2026-08-04

## Wording (hard)

Chimera / derived enrichment no longer causes **queue backpressure** on the
put path (`max_pending_seals` counts authoritative finalize only).

That does **not** prove enrichment is free of **CPU / disk / cache
interference** while writes continue. Those are separate residuals.

## Architectural locks

1. Whole-segment BLAKE3 is **derived** (`ContentHashState::{Pending,Known}` —
   never a `[0;32]` magic sentinel). Authoritative publish is summary footer +
   rename; frame CRC/body hashes detect corruption.
2. Remaining gap vs a **no-rotation** high-threshold control is the cost of
   **real rotations**, not missing hash work. That control is a ceiling, not a
   sustainable product workload; the 90% paired micro-gate is retired.
3. Derived tier/segment catalogs are **rebuildable accelerators**. `SealDone`
   updates memory only; durable checkpoints coalesce asynchronously and may
   lag or disappear. Open rebuilds from authoritative segments.

## Sustained-rotation evidence

### After derived-catalog checkpointing (2 GiB @ 64 MiB, enrichment off)

- Ack TPS **~57.6K**, **32** rotations, reopen exact.
- `catalog_apply` **0.005%** of ack wall (was ~13.9%).

Evidence: `doc/archive/performance-qualification/2026-08-04-derived-catalog-checkpoint/`.

### Prior (pre-checkpointing)

- Ack TPS **~47.8K**; `catalog_apply` **~13.9%** of ack wall.
- Evidence: `doc/archive/performance-qualification/2026-08-04-sustained-rotation/`.

## Settled facts (prior)

- Stream-hash / meta-publish paired medians vs control: ~0.87–0.88 (FAIL old
  90% micro-gate).
- Put-path / write-tail rolling BLAKE3: measured regressions; stay **off**.
- Reopen before enrichment: proven with typed `Pending`
  (`tests/reopen_before_enrichment.rs`).

## Next developer instruction (freeze)

Stop speculative seal-architecture changes. Seal Fast Lane accepted
architecturally; catalog O(n²) persist defect fixed and **package accepted**.

**Only allowed next work:** run the matched sustained **three-cell** campaign
on the **same binary** (full recipe:
[THREE_CELL_LIFECYCLE_ATTRIBUTION.md](./THREE_CELL_LIFECYCLE_ATTRIBUTION.md)):

| Cell | Purpose |
|---|---|
| 64 MiB, enrichment off | Current authoritative rotation cost |
| Threshold above workload, enrichment off | Sustained no-rotation ceiling |
| 64 MiB, enrichment on | Actual product cost of derived enrichment |

Same 2 GiB workload, delete between runs, alternate ordering, compare
**medians**. Do **not** optimize anything else until those three numbers
exist. Return to AWO / append-path work only after principal re-opens that
lane.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
- `doc/archive/performance-qualification/2026-08-04-zero-scan-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-zero-read-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-paired-median-gate/`
- `doc/archive/performance-qualification/2026-08-04-defer-segment-blake3/`
- `doc/archive/performance-qualification/2026-08-04-sustained-rotation/`
- `doc/archive/performance-qualification/2026-08-04-derived-catalog-checkpoint/`
