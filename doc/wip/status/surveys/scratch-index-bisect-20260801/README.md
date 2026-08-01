# Index vs data-cooking bisection (Scratch, 2026-08-01)

**Diagnostic only.** Real disk restored. Compare full Buffered put vs append+write with dual-index publish skipped.

## Method

| Phase | Disk | Index publish |
|-------|------|----------------|
| `store_put_buffered_batch1` | Real | **On** (product path) |
| `store_put_buffered_no_index` | Real | **Off** (`set_diagnostic_skip_index(true)`) |

Skip removes: `index.get` for item_id (uses subject hash), `apply_durable_event`, collection note, derived checkpoint. Keeps: encode envelope, `encode_frame_into`/append (Blake+copy), `write_segment_tail`.

## Result (20k × 8 KiB, 512 MiB seal)

| Phase | ops/s | wall_ms |
|-------|------:|--------:|
| full Buffered (real) | **~136k** | ~148 |
| **no-index** (real, data cooking) | **~141k** | ~143 |
| Discard (no disk, full index) | ~166k | ~120 |
| Memory (index only) | ~1.0M | ~19 |

**Ratio no-index / full ≈ 1.04×** — indexing is **~4%** of wall, not the killer.

Probe on no-index: append ≈ 77 ms, write ≈ 24 ms, encode ≈ 10 ms, publish ≈ 0.3 ms.

## Read

**It is data cooking**, not dual-index publish:

1. Skipping index barely moves the needle (~4%).
2. Wall remains dominated by **append_frame** (Blake body hash + copy into segment buffer) + real-file write (~16%).
3. Memory put at ~1M ops/s still shows the dual-index alone is fast when there is no frame cook.

## Artifacts

`phase-bench.txt` · `phase-bench.json`
