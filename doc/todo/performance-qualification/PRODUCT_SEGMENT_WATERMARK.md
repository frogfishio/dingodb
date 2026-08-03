# Product segment growth — watermark (opt-in)

**Date:** 2026-08-03  
**Board:** `3858600c` — Ship product watermark segment growth (opt-in)  
**Status:** labor `in_review` — **not** package accept / **not** default-on  

## What shipped

Product API (not `DiagnosticIoSink`):

```rust
store.set_segment_growth_policy(SegmentGrowthPolicy::watermark_default())?;
// or explicit sizes (64 MiB … multi‑GiB / 10 GiB):
// SegmentGrowthPolicy::watermark(capacity_bytes, chunk_bytes)
// or GrowOnAppend (default)
```

Mechanism:

1. OS preallocate + `set_len` to **`capacity_bytes`** per active segment (**default 64 MiB**, was historically 512 MiB spike-matched)  
2. Bulk-zero the first **`chunk_bytes`** at setup (**default 64 MiB**)  
3. On each real segment-tail write: keep ≥`chunk_bytes` of zeroed runway ahead of the write head  

Both knobs are host-owned — small DBs stay modest; large DBs may pass 10 GiB capacity. See [PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md](PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md).

Peer-pump:

```sh
residiuum-testrig peer-pump ... --engine residiuum --segment-growth watermark
# optional:
#   --wm-capacity-mib 64   --wm-chunk-mib 64
#   --wm-capacity-mib 10240 --wm-chunk-mib 1024
```

JSON reports `segment_growth`, `wm_capacity_mib`, `wm_chunk_mib`. Requires `--diag-io real`.

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
| [PREALLOC_WATERMARK_SPIKE.md](PREALLOC_WATERMARK_SPIKE.md) | Historical diag spike (~32k) — **corrected**: seal-fail cheat |
| [FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md) | Honest paired product flag: watermark ≈ grow (~6–8k), not ~32k |
| [HOW_MANY_TPS_NOW.md](HOW_MANY_TPS_NOW.md) | Default Real band; watermark opt-in; do not quote cheat 32k |
| `tests/wm_seal_probe.rs` | Diag+product watermark seal OK; sealed size ≪ 512 MiB prealloc |
| Seal truncate + diag zero-ahead fix | `store.rs` (this card) |

## Non-claims

Not AWO default-on. Not PQH qualification accept. **Not** that 28–32k is a product watermark floor (that band was a seal-fail cheat). Not crash/CSQ campaign for preallocated holes. Space amplification is intentional, **host-tunable** (default ~64 MiB/active; not locked to ½ GiB), and disclosed.

## Next (separate cards)

- Persist policy in store config / heap create options if product wants sticky enable  
- Default-on only after principal + CSQ/space disclosure  
- Cleaner re-pair on a host with disk headroom (this run was ~93% full / noisy)  
- Scratch + Linux cells  
