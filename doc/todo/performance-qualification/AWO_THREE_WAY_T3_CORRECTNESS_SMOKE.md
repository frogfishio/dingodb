# Measure AWO three-way — T3 correctness smoke evidence

Status: **labor complete (self_check) — not package accept; no throughput claims**  
Card: `1d4fa349-c0a1-408f-b868-110d04559c16`  
Date: 2026-08-02  
Feature: Measure adaptive write batching (three-way fair run)

## Goal

Prove **disabled / static / adaptive** real_store paths write and reopen cleanly
before any diagnostic rate work (T4).

## Evidence

### Unit

```text
cargo test -p residiuum-perf --features store-driver --lib real_store_smoke
# 4/4 ok including real_store_smoke_three_way_correctness_before_numbers
```

### CLI (`driver-smoke`, 2026-08-02)

| Mode | validity | acknowledged | reopen_live_count | notes highlight |
|------|----------|--------------|-------------------|-----------------|
| disabled | valid | 24 | 24 | `awo_flush=put_many` |
| static | valid | 24 | 24 | `admit_put_batch`, `awo_detached=true` |
| adaptive | valid | 24 | 24 | `admit_put_batch`, `awo_detached=true` |

`product_claim_eligible=false` on all three.

## Explicit non-claims

- No MB/s, no mode ranking, no qualification floors.  
- Not T4 matrix run.  
- Not AWO package accept.

## Next

T4 — first real diagnostic measurement using `AWO_THREE_WAY_MEASURE_RUNBOOK.md` §6.
