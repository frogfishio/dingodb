# Spike: seal-sized ahead-of-write zero (watermark)

**Date:** 2026-08-03  
**Ask:** Amortize the confirmed zero/first-touch tax into the put path — 64 MiB chunks ahead of the write head, not 512 MiB upfront.

## Mechanism (`--diag-io realpreallocwm`)

1. `F_PREALLOCATE` + `set_len(512 MiB)`
2. Bulk-zero **only the first 64 MiB** at setup
3. On each segment-tail write: if the write head would enter un-zeroed space, zero the next **64 MiB** chunk(s) (cost counted in the pump odometer)

## Numbers

Mode A · c=8 · 8 KiB · 256 MiB · APFS `/var/tmp` · seal 512 MiB

| `diag_io` | Pump ops/s | Pump ms | Process wall | E2E ops/s (keys÷wall) |
|-----------|----------:|--------:|-------------:|----------------------:|
| **real** | **9 155** | 3579 | 4.23 s | **7 747** |
| **realprealloczero** (full upfront zero) | **48 141** | 680 | 1.31 s | **25 014** |
| **realpreallocwm** (64 MiB ahead) | **32 327** | 1013 | **1.15 s** | **28 494** |

Artifacts: [`artifacts/firm-numbers-prealloc-wm-apfs/`](artifacts/firm-numbers-prealloc-wm-apfs/).

## Verdict (historical spike — see correction)

```text
Watermark works: ~32k pump with zeroing inside the timed window
E2E wall winner: watermark (~28.5k) > full-zero (~25k) > Real (~7.7k)
Still ≫ SQLite ~30k on pump; E2E watermark ≈ SQLite band
```

1. **Product-shaped amortize is viable** on this bed — no need to zero the whole segment before the first put.
2. Pump ops/s is lower than full-zero (pays chunk zeros mid-run) but **end-to-end wall is better** (cheaper setup).
3. Still diagnostic. Next design knobs: chunk size vs seal threshold, background prepare of N+1, when to `F_PREALLOCATE`.

## Correction (2026-08-03)

The ~32k / ~28.5k figures above **must not** be reused as product watermark thr.
Post-create diag setup zeroed from offset 0 (clobbered the on-disk descriptor);
end-of-run `seal_active` failed and peer-pump ignored the error — seal/chimera
cost never entered the odometer. Honest re-pair after the seal/prealloc fix:
[`FIRM_NUMBERS_PRODUCT_WM.md`](FIRM_NUMBERS_PRODUCT_WM.md) (product watermark
≈ grow, both ≪ prior cheat band).

## How to re-run

```sh
BIN=target/release/residiuum-testrig
for d in real realprealloczero realpreallocwm; do
  /usr/bin/time -p $BIN peer-pump -w /var/tmp/wm-$d --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out
done
```

See [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md), [NEXT_STEPS_WRITE_GROWTH.md](NEXT_STEPS_WRITE_GROWTH.md).
