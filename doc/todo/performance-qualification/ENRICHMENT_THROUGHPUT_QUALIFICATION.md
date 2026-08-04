# Enrichment Throughput Qualification (ETQ)

Status: **ETQ open**.  
**ETQ-0 accepted** (root cause: full-payload Chimera).  
**ETQ-1 Compact Chimera Persistence = architectural package accept (principal).**  
**ETQ-2 Single-Pass Enrichment Decode = next** (frozen).  
AWO: **paused**. Three-cell attribution residual: **deprioritized**.  
Date: 2026-08-04

## Honest product numbers (locked after Compact Chimera)

> Residiuum’s **complete sustainable throughput** is currently approximately
> **37.9K 8 KiB writes/sec** (~**296 MiB/s** logical), up from ~12.4K with
> materialized Chimera (~**3×**).

| Label | Value | Meaning |
|---|---:|---|
| Acknowledgement TPS | ~43.8K | Burst — still slightly ahead of complete |
| Complete-lifecycle TPS | ~37.9K | Sustainable full product (this campaign) |
| Logical payload | ~296 MiB/s | 37.9K × 8 KiB |
| Chimera derived / auth | ~0.74% | Was ~98% |
| Enrichment produce | ~5.57 seg/s | Seals entering enrich |
| Enrichment complete | ~4.93 seg/s | Jobs finished (ack+drain wall) |
| Backlog slope | +0.64 | Still slowly growing |

Correct reopen and index/query verification: **PASS**.

### ETQ-1 verdict (architectural accept)

| Gate | Result |
|---|---|
| Chimera derived ≤5% auth | **PASS** (~0.74%) |
| Chimera stage ≥7 seg/s | **PASS** (~63) |
| Lifecycle approaches ack | **PASS** (~37.9K / ~43.8K) |
| Reopen + query | **PASS** |
| Enrichment ≥7 seg/s | **FAIL** (~4.93) — ETQ remains open |
| Backlog slope ≤0 | **FAIL** (+0.64) — ETQ remains open |

Evidence: `doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`.

> Compact Chimera is the right architecture; duplicated decode passes are not.

## ETQ-2 — Single-Pass Enrichment Decode (next)

**Charter:** [ETQ2_SINGLE_PASS_DECODE.md](./ETQ2_SINGLE_PASS_DECODE.md)

One segment read, one verify/decode, one immutable `EnrichmentPlan` feeding
BLAKE3 / Hydra / compact Chimera / catalog. Instrument pass + frame counts.
**Do not** add enrichment workers first.

### Accept gates

- Enrichment capacity **≥ 7** segments/sec
- Backlog slope **≤ 0**
- Complete-lifecycle TPS close to acknowledgement TPS
- Exact reopen + query verification
- Exactly **one** authoritative decode pass per enriched segment

## Non-goals (until ETQ exits)

- AWO / append-path optimisation
- Three-cell attribution medians
- Worker-count tuning before single-pass decode

## Evidence homes

- ETQ-0: `doc/archive/performance-qualification/2026-08-04-etq0-enrichment-stage-breakdown/`
- ETQ-1: `doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`
- Enrichment-on (pre-compact baseline): `doc/archive/performance-qualification/2026-08-04-enrichment-on-2g/`
- ETQ-2: `doc/archive/performance-qualification/YYYY-MM-DD-etq2-single-pass-decode/`
