# Zero-read authoritative sealing — experiment (2026-08-04)

Status: **gate not met** — Seal Fast Lane remains open (`in_review`, not accept).

Recipe: APFS · Real Full · 8 KiB · 256 MiB · c=8 · Buffered · AWO=Disabled · seed=42 · seal=64 MiB.

## Interpretation of enrichment-off control

Baseline enrichment-off @ 64 MiB: **~45.9K** ack TPS (`enrichment-off-baseline/`).

Vs high-thr ~77–83K without mid-run seals → residual is
**`authoritative_finalisation_dominant`**, not enrichment queue backpressure alone.

## Zero-read variants tried

| Variant | Ack TPS (enrichment off / on / SFL) | Notes |
|---|---:|---|
| Stream-hash meta (prior peak) | ~/64K / **~71K** / ~50–66K | No frame scan; reads pending (page-cache). Best so far; still **&lt;74.7K**. |
| Resident prefix move (`FinalizeSealResident`) | ~47.9K / ~57.9K / ~65.6K | True zero-read at seal; write-through discard off → write-path RSS regression. |
| Write-tail rolling BLAKE3 (`FinalizeSealPlan`) | ~44.0K / ~45.9K / ~42.5K | True zero-read at seal; hash on put path. Matches prior ~50K reject class. |

## Acceptance

Seal Fast Lane accept requires **≥74.7K** ack TPS, ≥2 rotations, exact reopen.

**Not accepted.** Hot path remains stream-hash `FinalizeSealMeta` (zero-scan).
AWO stays paused until this lifecycle bottleneck is resolved or the principal waives.

## Evidence dirs

- `enrichment-off-baseline/` — control before zero-read attempts
- `enrichment-off-post/`, `enrichment-on-post/`, `seal-fast-lane-post/` — resident move
- `enrichment-off-incr/`, `enrichment-on-incr/`, `seal-fast-lane-incr/` — write-tail rolling hash
