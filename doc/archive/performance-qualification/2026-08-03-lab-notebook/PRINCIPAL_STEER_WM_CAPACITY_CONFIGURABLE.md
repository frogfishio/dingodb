# Principal steer — watermark capacity is configurable

**Date:** 2026-08-03  
**Card:** `7a377478`  
**Steer:** Capacity need not be ½ GiB. 64 MiB may be enough. It must be a **configurable parameter** defaulting around 64–100 MiB — small DBs may never fill ½ GiB; some hosts may need 10 GiB extend chunks.

## Labor response

**Agreed and shipped (API + defaults + peer knobs).**

| Knob | Meaning | Default |
|------|---------|--------:|
| `capacity_bytes` | Reserved file size per active (`set_len` + OS prealloc) | **64 MiB** |
| `chunk_bytes` | Zero-runway step ahead of write head | **64 MiB** |

```rust
// Modest default (product)
store.set_segment_growth_policy(SegmentGrowthPolicy::watermark_default())?;

// Explicit (small or huge)
store.set_segment_growth_policy(SegmentGrowthPolicy::watermark(
    64 * 1024 * 1024,           // or 100 MiB, or 10 GiB
    64 * 1024 * 1024,
))?;
```

Peer-pump:

```sh
# default 64 / 64 MiB when watermark
residiuum-testrig peer-pump ... --segment-growth watermark

# large-DB host
residiuum-testrig peer-pump ... --segment-growth watermark \
  --wm-capacity-mib 10240 --wm-chunk-mib 1024
```

Constants: `WATERMARK_DEFAULT_CAPACITY_BYTES`, `WATERMARK_DEFAULT_CHUNK_BYTES`.

## Why not hardcode 512 MiB

- Space amp scales with **#actives × capacity**, not with “honesty.”  
- Tiny stores paying ½ GiB forever is wasteful even when thr wins.  
- Huge stores may want multi‑GiB / 10 GiB steps so the preparer (or put-path ensure) extends less often.

## Non-claims

Not default-on growth policy (still `GrowOnAppend`). Not sticky across reopen. Not that 64 MiB is the proven thr optimum — it is the **sensible product default** until measured otherwise. Background watcher still a separate card.
