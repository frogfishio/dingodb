# Append-frame short-circuit (Scratch, 2026-08-01)

**Diagnostic only.** Proves `append_frame` (Blake + segment buffer) is the data-cooking hog by zeroing it out.

## Method

`Store::set_diagnostic_skip_append_frame(true)` — put still runs prep + CBOR env encode (+ optional index), but **skips** `segment.append` / `encode_frame_into` and tail write.

## Result (20k × 8 KiB)

| Phase | ops/s | vs full |
|-------|------:|--------:|
| full Buffered (real disk) | **~134k** | 1.0× |
| no-index (cook + write) | ~141k | ~1.05× |
| **no-append** (keep index) | **~522k** | **~3.9×** |
| **no-append + no-index** | **~614k** | **~4.6×** |
| Memory put | ~921k | — |
| pure Blake alone | ~273k | — |
| encode_frame_into alone | ~232k | — |

Probe on no-append: append=0, write=0; residual is prep + env encode + publish.

## Read

Short-circuiting append_frame lifts Mode A micro from **~134k → ~520k ops/s** (~4×). That is decisive: **append_frame is the cooking bottleneck** (Blake body hash + copy into the active segment), not index and not disk.

Residual ~520–610k is prep/env/index overhead; Memory (~900k) is still higher because Buffered prep is a bit heavier than pure Memory path.
