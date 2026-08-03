# Paired median gate — Seal Fast Lane (2026-08-04)

Status: **gate not met** on this bed (median ratio &lt; 0.90).  
Measurement only. Enrichment disabled. Same release `residiuum-testrig` binary
for all cells.

## Why this package

The frozen **≥74.7K** absolute floor was `0.90 × 83K` from an older
high-threshold control. Contemporary control is ~77–78K, so the intended gate
is a **paired median ratio**, not a stale absolute:

\[
\frac{\operatorname{median}(TPS_{64MiB})}
{\operatorname{median}(TPS_{control})} \ge 0.90
\]

## Recipe

| Knob | Value |
|---|---|
| Cell | Real Full |
| Logical | 256 MiB |
| Payload | 8 KiB |
| Concurrency | 8 |
| Seed | 42 |
| AWO | Disabled |
| Enrichment | **off** (causal isolation) |
| Stream-hash cell | seal_threshold=64 MiB |
| Control cell | seal_threshold=512 MiB (no mid-run seal) |
| Reps | 6 each, alternating control → stream64 |
| Machine | see `uname.txt` |
| Binary | `target/release/residiuum-testrig` (`binary.sha256`) |

Driver: `run_paired_median.py` (120 s per-run timeout).

## Results (medians)

| Cell | Median ack TPS | Min | Max | Multi-rotate | Exact reopen |
|---|---:|---:|---:|---|---|
| Control (512 MiB) | **78 485** | 58 250 | 80 629 | n/a (0 sealed mid-ack) | yes (6/6) |
| Stream-hash (64 MiB) | **68 925** | 67 506 | 72 021 | yes (≥3 sealed, 6/6) | yes (6/6) |

| Gate | Value | Result |
|---|---:|---|
| median(stream64) / median(control) | **0.878** | **FAIL** (&lt; 0.90) |
| Multi-rotate (64 MiB) | ≥2 all reps | PASS |
| Exact reopen | all reps | PASS |

Raw: `summary.json`, `runs/*.json`.

## Interpretation

- Contemporary control median (**~78.5K**) matches the principal’s ~77.6K
  refresh; the stale 83K / 74.7K absolute floor should not be used alone.
- Stream-hash @ 64 MiB median (**~68.9K**) is ~**87.8%** of that control —
  below the 90% paired gate. Peak-ish prior ~71K readings are not the median
  on this campaign.
- Against `0.90 × 78.5K ≈ 70.6K`, the 64 MiB median is still short (~1.7K).
- Background stream-hash finalisation remains the fastest correct architecture
  measured; resident-prefix / write-tail hashing previously regressed and stay
  off the hot path. This package does **not** re-enable those.
- Enrichment stays off here. Resource interference with enrichment on is a
  separate follow-up once the paired auth-seal gate closes or is waived.

## Principal accept?

**Not recommended from this evidence.** Labor leaves the Seal Fast Lane card
in review; human/principal decides accept vs further work vs waiver.
