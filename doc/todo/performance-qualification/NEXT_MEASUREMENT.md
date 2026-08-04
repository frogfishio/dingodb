# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane = architectural accept (principal)**;  
**Derived Catalog Checkpointing = package accept (principal)** —
gates met; O(n²) catalog persist risk removed; sustained ack
**47.8K → 57.6K TPS** (~+20.5%); `catalog_apply` under 1% of ack wall.  

**Locked product truth (performance only):** complete sustainable throughput ≈
**37.9K** 8 KiB writes/sec (~**296 MiB/s**) with Compact Chimera experiment.
Burst ack ≈ **43.8K**. Chimera amp ≈ **0.74%**. Enrichment still slightly
behind (~4.93 vs ~5.57 seg/s).

> Compact Chimera performance architecture accepted provisionally.
> Durability equivalence is unproven and blocks product/default acceptance.

**Next (only):** [CSE — Chimera Salvage Equivalence](./CHIMERA_SALVAGE_EQUIVALENCE.md)
(**CSE-0 labor done** → **CSE-1** → CSE-2 if needed). **ETQ-2 deferred** until Compact salvage
viability. **AWO paused**.  
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

### Full-product (2 GiB @ 64 MiB, enrichment on)

- Ack TPS **~47.4K**; complete-lifecycle TPS **~12.4K**; enrich drain **~14.9 s**.
- Enrichment **~1.61 seg/s** → capacity \(1.61 \times 8192 \approx 13.2K\) ops/s
  (matches lifecycle TPS). Writes create **~5.8 seg/s** → backlog slope **+4.1**.
- Reopen exact + index/query sample **PASS** (correctness).
- Full-product performance: **FAIL** — sustainable ≈ **12–13K TPS**.

Evidence: `doc/archive/performance-qualification/2026-08-04-enrichment-on-2g/`.

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

**Compact Chimera = provisional performance-architecture accept only**
(~12.4K→~37.9K TPS; amp ~0.74%). **Not** product default / durability-equivalent /
migration-eligible. Materialized format+reader stay intact.
Evidence `doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`.

**CSE-0 labor complete** (2026-08-04): Materialized recovery baseline frozen —
`doc/archive/performance-qualification/2026-08-04-cse0-materialized-recovery-baseline/`.

**Only allowed next work:** **CSE-1** (CSE-2 if needed) —
[CHIMERA_SALVAGE_EQUIVALENCE.md](./CHIMERA_SALVAGE_EQUIVALENCE.md).
Require \(\mathrm{Recoverable}_{compact}\supseteq\mathrm{Recoverable}_{materialized}\)
on the CSE-0 channels. **ETQ-2 deferred** behind CSE. AWO / three-cell remain paused.

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
- `doc/archive/performance-qualification/2026-08-04-enrichment-on-2g/`
- `doc/archive/performance-qualification/2026-08-04-etq0-enrichment-stage-breakdown/`
- `doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`
- `doc/archive/performance-qualification/2026-08-04-cse0-materialized-recovery-baseline/`
- Plan: `doc/todo/performance-qualification/ENRICHMENT_THROUGHPUT_QUALIFICATION.md`
- ETQ-1 charter: `doc/todo/performance-qualification/ETQ1_COMPACT_CHIMERA.md`
- CSE charter: `doc/todo/performance-qualification/CHIMERA_SALVAGE_EQUIVALENCE.md`
- ETQ-2 charter (deferred): `doc/todo/performance-qualification/ETQ2_SINGLE_PASS_DECODE.md`
