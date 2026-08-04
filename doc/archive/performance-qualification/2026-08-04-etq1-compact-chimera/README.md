# ETQ-1 — Compact Chimera Persistence (2026-08-04)

Status: **architectural package accept (principal)** — Compact Chimera
Persistence. ETQ remains **open** (enrichment floors still short).  
Evidence of breakthrough: complete-lifecycle TPS **~12.4K → ~37.9K** (~3×);
Chimera amp **~98% → ~0.74%**.

## What landed

Default Chimera is layout **version 2** with `SegmentFrame` locators
(segment id + frame offset/len). Payloads stay in authoritative segments.
Full-payload embedding (`build_materialized_layout`) is obsolete/non-default.
Legacy v1 files remain readable.

## Recipe

| Knob | Value |
|---|---|
| Cell | Real Full |
| Logical | **2 GiB** |
| Seal threshold | 64 MiB |
| Payload / concurrency / seed | 8 KiB / 8 / 42 |
| Enrichment | **on** |
| Binary | `binary.sha256` |

## Headline numbers

| Metric | ETQ-0 (before) | ETQ-1 (this run) |
|---|---:|---:|
| Chimera derived / auth (on-disk `.cmr`) | ~98% | **~0.74%** |
| Enrichment written / read (mean, Hydra+Chimera) | ~99% | **~1.15%** |
| Chimera stage capacity | 2.56 seg/s | **63.0 seg/s** |
| Acknowledgement TPS | ~47.4K | **43.8K** |
| Complete-lifecycle TPS | ~12.4K | **37.9K** |
| Enrichment jobs/s (ack+drain wall) | ~1.61 | **4.93** |
| Backlog slope (OLS) | +4.14 | **+0.64** |
| Reopen exact / index-query | yes / yes | **yes / yes** |

## Gate scorecard

| Gate | Bound | Result |
|---|---|---|
| Default Chimera derived bytes | ≤5% auth | **PASS** (~0.74%) |
| Chimera stage capacity | ≥7 seg/s | **PASS** (63) |
| Enrichment capacity | ≥7 seg/s | **FAIL** (4.93) |
| Backlog slope | ≤0 | **FAIL** (+0.64) |
| Lifecycle TPS approaches ack | near ack | **PASS** (37.9K / 43.8K ≈ 87%) |
| Reopen + query | exact | **PASS** |
| Chimera optional for correctness | wipe → get | **PASS** (unit) |

## Residual (ETQ still open → ETQ-2)

Enrichment produces work at ~**5.57** seg/s but completes ~**4.93**, so backlog
still grows slowly (slope **+0.64**). After Compact Chimera, **decode**
(~82 ms/seg) dominates; Hydra ≈ Chimera (~15 ms).

**Next only:** [ETQ-2 Single-Pass Enrichment Decode](../../todo/performance-qualification/ETQ2_SINGLE_PASS_DECODE.md)
— one read, one verify/decode, one `EnrichmentPlan`; no workers first.

## Evidence

- `sustained-2g-64m-enrichment-on.json`
- `summary.json`
- `EVIDENCE_TABLE.md`
- Unit: `cargo test -p residiuum-store --lib chimera`
