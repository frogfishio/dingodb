# Enrichment Throughput Qualification (ETQ)

Status: **ETQ-0 accepted**; **ETQ-1 compact Chimera labor measured** (amp/stage
PASS; enrichment ≥7 / slope ≤0 still FAIL — decode residual).  
AWO: **paused**. Three-cell attribution residual: **deprioritized**.  
Date: 2026-08-04

## Honest product numbers (locked)

> Residiuum’s **complete sustainable throughput** is currently approximately
> **12.4K 8 KiB writes/sec**.

| Label | Value | Meaning |
|---|---:|---|
| Acknowledgement TPS | ~47.4K | Burst — financed by deferred enrichment debt |
| Enrichment service | ~1.61–2.7 seg/s | Limited by Chimera persist |
| Complete-lifecycle TPS | ~12.4K | Matches enrichment capacity |
| Chimera derived write | ~63 MiB / 64 MiB auth | ~2× write amplification |

### ETQ-0 verdict (accepted)

| Gate | Result |
|---|---|
| Correctness (reopen, digests, index/query) | **PASS** |
| Full-product performance | **FAIL** |
| Dominant stage | **Chimera persist** (~366 ms/seg) |
| Root cause | Eager full-payload Chimera materialization |

Evidence: `doc/archive/performance-qualification/2026-08-04-etq0-enrichment-stage-breakdown/`.

> The database is fast; eager full-payload Chimera materialization is not.

## ETQ-1 — Compact Chimera Persistence (next)

**Charter:** [ETQ1_COMPACT_CHIMERA.md](./ETQ1_COMPACT_CHIMERA.md)

Default Chimera persists locators + metadata only; payloads remain in
authoritative segments. Fully materialized layouts are lazy / opt-in.

**Do not** start with parallel enrichment workers.

### Accept gates

- Default Chimera derived bytes **≤ 5%** of authoritative bytes
- Enrichment capacity **≥ 7** segments/sec
- Backlog slope **≤ 0** during sustained ingestion
- Full lifecycle TPS approaches acknowledgement TPS
- Exact reopen + query verification (locator-based Chimera)
- Chimera not required for correctness

## Non-goals (until ETQ-1 exits)

- AWO / append-path optimisation
- Three-cell attribution medians
- Worker-count tuning as a substitute for removing write amplification

## Evidence homes

- ETQ-0: `doc/archive/performance-qualification/2026-08-04-etq0-enrichment-stage-breakdown/`
- Enrichment-on product: `doc/archive/performance-qualification/2026-08-04-enrichment-on-2g/`
- ETQ-1: `doc/archive/performance-qualification/YYYY-MM-DD-etq1-compact-chimera/`
