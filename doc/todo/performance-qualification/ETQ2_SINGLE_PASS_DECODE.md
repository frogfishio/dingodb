# ETQ-2 — Single-Pass Enrichment Decode (frozen)

Status: **next implementer package** (principal freeze 2026-08-04).  
Depends on: **Compact Chimera Persistence architectural accept**
(`doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`).

## Why this package

Compact Chimera removed ~2× write amplification and raised complete-lifecycle
TPS from ~**12.4K → ~37.9K** (~3×). Enrichment still lags acknowledgement:

| Signal | Value |
|---|---:|
| Burst acknowledgement | ~43.8K TPS |
| Complete sustainable lifecycle | ~37.9K TPS |
| Logical payload | ~296 MiB/s |
| Enrichment produce rate | ~5.57 seg/s |
| Enrichment complete rate | ~4.93 seg/s |
| Backlog slope | still slightly positive |

Stage timings show **decode** (~82 ms/seg) dominates after Chimera/Hydra each
sit ~15 ms. Multiple full-segment scans (Hydra, Chimera, catalog paths) are the
next duplication to remove — **not** more workers.

## Product direction

For each sealed segment enrichment job:

1. **Read** the segment bytes once.
2. **Verify/decode** frames once.
3. Produce one immutable **`EnrichmentPlan`** (frame offsets, subjects, put/delete
   classifications, body lengths / digests as needed — no repeated body clones).
4. Feed **BLAKE3**, **Hydra**, **compact Chimera**, and **catalog metadata** from
   that shared plan.
5. Instrument **full-segment pass count** and **decoded frame count** (oracle:
   exactly one authoritative decode pass per enriched segment).

## Acceptance gates (all required)

| Gate | Bound |
|---|---|
| Enrichment capacity | **≥ 7** segments/sec |
| Backlog slope | **≤ 0** during sustained ingestion (after warm-up) |
| Complete-lifecycle TPS | Close to acknowledgement TPS |
| Reopen | Exact (`coverage_scan`) |
| Query | Verified |
| Decode passes | **Exactly one** authoritative decode pass per enriched segment |

## Explicit non-starts

- Parallel enrichment workers (only after duplicated work is gone).
- Reverting Compact Chimera / re-embedding payloads.
- AWO / three-cell attribution.

## Evidence home

`doc/archive/performance-qualification/YYYY-MM-DD-etq2-single-pass-decode/`.
