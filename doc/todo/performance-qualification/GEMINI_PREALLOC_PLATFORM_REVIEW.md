# Gemini platform prealloc — review vs evidence

**Date:** 2026-08-03  
**Context:** Gemini argued sparse `set_len` failed because APFS creates holes; production should use `F_PREALLOCATE` / `fallocate` / `SetFileValidData` and get ~37k “out of the box.”

## Scorecard

| Gemini claim | Verdict | Evidence |
|--------------|---------|----------|
| Sparse `set_len` / `ftruncate` does not allocate physical blocks on APFS | **Agree** | `realprealloc` ≈ Real (~9k) |
| Page-touch forces physical extents and lifts thr | **Agree** | `realpreallocfill` ~33–37k |
| ~37k “dethrones” SQLite ~30k on this concurrent Mode A cell | **Agree as diagnostic** | Same bed; not product / not Scratch PEER |
| Page-touch is not a product design | **Agree** | Already in [NEXT_STEPS_WRITE_GROWTH.md](NEXT_STEPS_WRITE_GROWTH.md) |
| macOS should use `fcntl(F_PREALLOCATE)` | **Right API to try** | Standard advice |
| Linux `fallocate` / Windows `SetFileValidData` are the analogues | **Plausible platform map** | Not measured here (this host is macOS/APFS) |
| **`F_PREALLOCATE` will give ~37k without page-touch** | **Falsified on this bed** | See spike below |
| Subsequent writes into preallocated space are “cheap in-place overwrites” | **Falsified here** | Fcntl ≈ Real, not ≈ fill |

## Spike: Gemini’s macOS prescription

Same recipe: Mode A · c=8 · 8 KiB · 256 MiB · APFS `/var/tmp` · seal 512 MiB.

| `diag_io` | Mechanism | ops/s |
|-----------|-----------|------:|
| **real** | grow-on-append | **9 591** |
| **realprealloc** | `set_len(512M)` only | **9 302** |
| **realpreallocfcntl** | `fcntl(F_PREALLOCATE)` + `set_len` | **9 548** |
| **realpreallocfill** | `set_len` + touch every 1 MiB | **33 633** |

Artifacts: [`artifacts/firm-numbers-fpreallocate-apfs/`](artifacts/firm-numbers-fpreallocate-apfs/).

```text
F_PREALLOCATE  ≈  sparse set_len  ≈  Real     (~9.5k)
page-touch     ≈  34k             (still the only ~4× win)
```

So: **swapping `set_len` for `F_PREALLOCATE` did not move the odometer** on APFS for this workload. Gemini’s bottom line is wrong as stated.

## Why the platform map can still be useful (with humility)

Gemini’s cross-platform table is a reasonable *design menu*, not a measured Residiuum result:

| OS | Call | Role in our story |
|----|------|-------------------|
| **macOS/APFS** | `F_PREALLOCATE` | Tried — **no thr win** vs Real on this cell |
| **Linux** | `fallocate` / `posix_fallocate` (+ optional `FALLOC_FL_KEEP_SIZE`) | **Unmeasured** — may differ from APFS |
| **Windows** | `SetEndOfFile` + `SetFileValidData` | Interesting because `SetFileValidData` skips zero-fill — hints the real tax may be **first-touch zeroing**, not only “extent reservation” |

A coherent reading of *our* numbers: page-touch both (1) allocates and (2) **writes zeros into pages**. `F_PREALLOCATE` may reserve space without making first application writes as cheap as overwriting already-zeroed pages. That would explain fill ≫ fcntl without contradicting the sparse-file story.

**Follow-up measured:** [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md) — `F_PREALLOCATE` + bulk zero → **~51k** pump ops/s (fcntl alone still ~9k). Zero/first-touch hypothesis **confirmed**.

## Corrected bottom line (ours)

1. Gemini is **right** about the APFS sparse trap and why touch worked.  
2. Gemini is **wrong** that `F_PREALLOCATE` alone reproduces the ~37k win on this APFS hammerblast.  
3. Production design must be **proven per FS**, not assumed from syscall names. Next measures: Linux `fallocate`; APFS variants (allocate + explicit zero? watermark ahead-of-write); honest timing of setup cost.  
4. Do **not** ship “we use F_PREALLOCATE therefore we beat SQLite.”

## How to re-run

```sh
BIN=target/release/residiuum-testrig
for d in real realprealloc realpreallocfcntl realpreallocfill; do
  $BIN peer-pump -w /var/tmp/fp-$d --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out
done
```

See [PREALLOC_SPIKE.md](PREALLOC_SPIKE.md), [NEXT_STEPS_WRITE_GROWTH.md](NEXT_STEPS_WRITE_GROWTH.md).
