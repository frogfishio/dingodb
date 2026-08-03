# Spike: F_PREALLOCATE + bulk zero

**Date:** 2026-08-03  
**Hypothesis (from Gemini review):** `F_PREALLOCATE` reserves extents but first application writes still pay zero/first-touch tax; **explicit zeroing** is what page-touch bought.

## What we ran

Mode A · c=8 · 8 KiB · 256 MiB · APFS `/var/tmp` · seal 512 MiB

| `diag_io` | Setup (before odometer) | Pump ops/s | Pump ms | Process wall |
|-----------|-------------------------|----------:|--------:|-------------:|
| **real** | none | **8 925** | 3671 | 4.28 s |
| **realpreallocfcntl** | `F_PREALLOCATE` + `set_len` | **8 961** | 3656 | 3.80 s |
| **realpreallocfill** | `set_len` + 1 byte / 1 MiB | **35 475** | 923 | 1.02 s |
| **realprealloczero** | `F_PREALLOCATE` + **1 MiB zero writes** ×512 | **51 282** | 638 | 1.16 s |

Artifacts: [`artifacts/firm-numbers-prealloc-zero-apfs/`](artifacts/firm-numbers-prealloc-zero-apfs/).

## Verdict

```text
F_PREALLOCATE alone     ≈ Real (~9k)          ← still no
1-byte/MiB poke         ≈ 35k                 ← known win
F_PREALLOCATE + bulk 0  ≈ 51k pump            ← stronger; confirms zero/first-touch story
```

1. **Zero-fill hypothesis confirmed.** Reserving blocks is not enough; **writing zeros into the reserved range** before the timed puts is what unlocks thr (and bulk zero beats the sparse 1-byte poke on the pump odometer).
2. **Setup is real.** Bulk zero of 512 MiB costs ~0.5 s before the pump (wall 1.16 s − pump 0.64 s). 1-byte poke setup is cheaper (~0.1 s) but leaves a slower pump.
3. **End-to-end wall** (puts ÷ process wall): Real ~7.7k · fill ~32k · zero ~28k — both pre-touch styles still crush Real; fill slightly wins on total wall because cheaper setup.
4. Still diagnostic — not product. A shippable design must amortize zeroing (seal-sized chunks ahead of the write head, background prepare, etc.).

## Implication for Gemini’s platform map

`F_PREALLOCATE` / `fallocate` alone ≠ the win. Something that makes pages **already written** (zeros or prior data) before hot-path appends is required on this APFS bed — closer in spirit to “pay zero-fill offline” than to “syscall reserve.”

## How to re-run

```sh
BIN=target/release/residiuum-testrig
for d in real realpreallocfcntl realpreallocfill realprealloczero; do
  /usr/bin/time -p $BIN peer-pump -w /var/tmp/z-$d --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out
done
```

See [GEMINI_PREALLOC_PLATFORM_REVIEW.md](GEMINI_PREALLOC_PLATFORM_REVIEW.md), [PREALLOC_SPIKE.md](PREALLOC_SPIKE.md).
