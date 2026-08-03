# Defer segment BLAKE3 — paired median evidence

Enrichment **off**. Alternating 6× control (512 MiB) + 6× 64 MiB meta-publish.
Medians, not peaks. Binary: see `binary.sha256`.

| Rep | Control ack TPS | Stream64 ack TPS | Sealed @ ack | Exact |
|---:|---:|---:|---:|---|
| 1 | 65852 | 66033 | 4 | yes |
| 2 | 80504 | 65709 | 4 | yes |
| 3 | 80921 | 60871 | 4 | yes |
| 4 | 75910 | 65538 | 4 | yes |
| 5 | 66758 | 58473 | 4 | yes |
| 6 | 72313 | 63282 | 4 | yes |
| **median** | **74112** | **64410** | — | — |

| Gate | Value | Result |
|---|---:|---|
| median ratio | 0.869 | FAIL (≥ 0.90) |
| Multi-rotate | 6/6 ≥ 2 | PASS |
| Exact reopen | 6/6 | PASS |

Overall: **FAIL** (see `summary.json`). Prior stream-hash ratio was 0.878.
