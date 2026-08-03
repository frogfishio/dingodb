# Next measurement — acknowledgement/finalisation split

Status: **superseded for branching by Seal Interference Control**  
Prior evidence archived:  
`doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`  
Seal interference evidence:  
`doc/archive/performance-qualification/2026-08-04-seal-interference-control/`  
Date: 2026-08-04

Harness: `residiuum-testrig ack-finalize` / `ack-finalize-matrix` /
`seal-interference-control` (`crates/residiuum-testrig/src/ack_finalize.rs`).

## Corrected conclusions (principal)

- Old “~8–10K write TPS” was **false as write TPS**. It was **campaign TPS**
  (ack + drain/seal + close + reopen + verification).
- Actual acknowledged Real Full throughput was **~22.6K writes/sec** (64 MiB seal).
- Disabling **live** dual-index publish raises ack only to **~27K** (~19%).
  That cell is **not** an index-free finalisation comparison: Hydra/Chimera still
  run during seal.
- Discard ~104K and mimic ~223K → product ack still has **~4.6×** / **~9.9×** gaps
  vs those diagnostics under the 64 MiB seal recipe.
- Seal was expensive, but the 64 MiB threshold across 256 MiB meant **background
  auto-sealing competed with writes**. Final `seal_active()` also combines
  `drain_lifecycle` with sealing the remaining active.

## Seal Interference Control (decisive)

Repeat Real Full with seal threshold **> workload** (512 MiB / 1 GiB) and record
pending seals at last acknowledgement.

| Seal threshold | Ack TPS | Pending seals @ ack | Campaign TPS |
|---|---:|---:|---:|
| 64 MiB | 24987 | 2 | 8097 |
| 512 MiB | 82732 | 0 | 6983 |
| 1 GiB | 82649 | 0 | 7128 |

**Verdict:** ack jumps to **~83K** when mid-run sealing is prevented →
**seal-pipeline interference suppresses live writes**. Attack sealing next.
Separately, final-seal stage breakdown shows Chimera dominating when one large
segment is sealed at end-of-run.

## Honesty

- Prefer **campaign TPS**, not “lifecycle TPS”, for ack+reopen+verify walls.
- Mimic `reopen_exact` is **not** full-ledger exact (length + endpoints).
- Store verify uses coverage-aware `scan_live_logical` ledger compare.
