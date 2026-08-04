# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane = architectural accept (principal)**;  
**Derived Catalog Checkpointing = package accept (principal)** —
gates met; O(n²) catalog persist risk removed; sustained ack
**47.8K → 57.6K TPS** (~+20.5%); `catalog_apply` under 1% of ack wall.  

**Locked product truth:** complete sustainable throughput ≈ **12.4K**
8 KiB writes/sec. The **~47.4K acknowledgement TPS is burst throughput
financed by deferred enrichment debt**. Correctness **PASS**; full-product
performance **FAIL**.

**Next (only):** [Enrichment Throughput Qualification](./ENRICHMENT_THROUGHPUT_QUALIFICATION.md)
(service floor ≥5.8 seg/s, prefer ≥7). Three-cell attribution residual
**deprioritized**. **AWO paused**.  
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

**ETQ-1 compact Chimera landed (labor):** `.cmr` v2 `SegmentFrame` default;
on-disk Chimera amp **~0.74%** (≤5% PASS); Chimera stage **~63 seg/s**.
Lifecycle TPS **~37.9K** (was ~12.4K). Evidence
`doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`.

**Residual (not package accept):** enrichment jobs/s **~4.93** (need ≥7);
backlog slope **+0.64**. Dominant stage is now **decode** (~82 ms/seg), not
Chimera. Do **not** add workers first. AWO / three-cell remain paused.

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
- Plan: `doc/todo/performance-qualification/ENRICHMENT_THROUGHPUT_QUALIFICATION.md`
- ETQ-1 charter: `doc/todo/performance-qualification/ETQ1_COMPACT_CHIMERA.md`
- `doc/archive/performance-qualification/2026-08-04-etq0-enrichment-stage-breakdown/`
