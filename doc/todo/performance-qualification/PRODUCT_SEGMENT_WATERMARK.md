# Product segment growth — watermark (opt-in)

**Date:** 2026-08-03  
**Board:** `3858600c` — Ship product watermark segment growth (opt-in)  
**Status:** labor `in_review` — **not** package accept / **not** default-on  

## What shipped

Product API (not `DiagnosticIoSink`):

```rust
store.set_segment_growth_policy(SegmentGrowthPolicy::watermark_default())?;
// or GrowOnAppend (default)
```

Mechanism (matches `realpreallocwm` spike):

1. OS preallocate + `set_len` to **512 MiB** capacity per active segment  
2. Bulk-zero the first **64 MiB** at setup  
3. On each real segment-tail write: keep ≥64 MiB of zeroed runway ahead of the write head  

Peer-pump:

```sh
residiuum-testrig peer-pump ... --engine residiuum --segment-growth watermark
```

JSON reports `segment_growth: "watermark"`. Requires `--diag-io real`.

## Why this is “ship” not default

| Claim | Status |
|-------|--------|
| Opt-in product path exists | **yes** |
| Default store behavior changed | **no** (still grow-on-append) |
| Durability labels / CSQ changed | **no** |
| Persisted across reopen | **no** (process-local policy; host must re-enable) |
| Thr floor / SQLite parity as product SLO | **no** — prior diag numbers only |
| Linux / Scratch measured on product flag | **no** |

## Evidence trail

| Source | Finding |
|--------|---------|
| [PREALLOC_WATERMARK_SPIKE.md](PREALLOC_WATERMARK_SPIKE.md) | Diag ~32k pump / ~28.5k E2E on APFS c=8 |
| [HOW_MANY_TPS_NOW.md](HOW_MANY_TPS_NOW.md) | Product Real ~10–14k; watermark now **opt-in product**, still not default |
| `cargo test -p residiuum-store … segment_growth_watermark_opt_in_put_reopen` | Correctness smoke |

## Non-claims

Not AWO default-on. Not PQH qualification accept. Not that 28–32k is guaranteed on every host. Not crash/CSQ campaign for preallocated holes. Space amplification (~512 MiB/active) is intentional and host-owned.

## Next (separate cards)

- Persist policy in store config / heap create options if product wants sticky enable  
- Default-on only after principal + CSQ/space disclosure  
- Paired peer-pump Real vs `--segment-growth watermark` vs SQLite on product flag  
- Scratch + Linux cells  
