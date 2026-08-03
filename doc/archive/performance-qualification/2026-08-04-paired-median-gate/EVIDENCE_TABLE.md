# Paired median gate — evidence table

Enrichment **off**. Alternating 6× control (512 MiB) + 6× stream64 (64 MiB).
Medians, not peaks.

| Rep | Control ack TPS | Stream64 ack TPS | Stream64 sealed @ ack | Stream64 exact |
|---:|---:|---:|---:|---|
| 1 | 58250 | 68793 | 3 | yes |
| 2 | 80629 | 72021 | 3 | yes |
| 3 | 78183 | 67506 | 3 | yes |
| 4 | 71192 | 67612 | 3 | yes |
| 5 | 78787 | 70472 | 3 | yes |
| 6 | 79545 | 69056 | 3 | yes |
| **median** | **78485** | **68925** | — | — |

| Gate | Value | Result |
|---|---:|---|
| median ratio | 0.878 | FAIL (≥ 0.90) |
| Multi-rotate | 6/6 ≥ 2 | PASS |
| Exact reopen | 6/6 | PASS |

Overall: **FAIL** (see `summary.json`).
