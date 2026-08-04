# CSE-3 Stage 2h — segment-ID P0 + lifecycle honesty

## Segment-ID never-reuse matrix

`cse3_stage2_segment_id_never_reuse` — 8/8 PASS (`--test-threads=1`).

## Step 9 re-run (2 GiB / 64 MiB, product CompactShadow)

| Metric | Value |
|---|---|
| Ack = Lifecycle TPS | **12372** (same wall; no seal exclusion) |
| life/ack | **1.00** |
| Seal stages (s) | drain=0 auth=1.55 shadow=2.33 catalog=1.46 reopen=0.39 total=5.72 |
| Compact amp | 0.75% locator-only |
| Shadow amp | 100.1% |
| Gates | frontier / rsh / P★ / reopen / continue / no Materialized path |

Sustainable product class remains ~**12K TPS** (Materialized-era). The prior
23.3K figure was burst (seals excluded). **Default stays Materialized.**
