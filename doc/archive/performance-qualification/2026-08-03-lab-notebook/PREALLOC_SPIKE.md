# “So basically we prealloc?”

**Short answer:** **Sparse `set_len` — no. Physically allocating pages ahead of time — yes, large lift.**

## Spike (same bed as write-all bisect)

APFS `/var/tmp` · Mode A · c=8 · 8 KiB · 256 MiB · seed 42 · seal 512 MiB

| `diag_io` | What | ops/s |
|-----------|------|------:|
| **real** | grow-on-append (baseline) | **9 164** |
| **realprealloc** | `set_len(512 MiB)` only (often sparse on APFS) | **8 908** (no help) |
| **realpreallocfill** | `set_len` + touch every 1 MiB (force pages) | **36 636** (~4× Real) |
| **realoverwrite** | smash offset 0 (no unique pages) | **106 180** (upper bound) |

Artifacts: [`artifacts/firm-numbers-prealloc-apfs/`](artifacts/firm-numbers-prealloc-apfs/).

## Verdict

```text
Sparse pre-size (set_len)     ≈ Real           ← does NOT fix append wall
Physical page pre-touch       ≈ 37k (~4×)      ← beats SQLite ~30k on this cell
Overwrite / Discard           ≈ 100–120k       ← still headroom above prealloc-fill
```

1. **Not** “call `truncate`/`set_len` and we’re done” — on APFS that is often a sparse hole; append still faults/allocates as you write → same ~10k band.
2. **Yes**, the growth wall is largely **first-touch allocation of new file pages**. Pre-touching every MiB before the timed run lifts thr to **~37k**.
3. That is **not yet a product design** — setup cost was outside the odometer; real product must amortize allocation (seal-sized extents, background fill, `F_PREALLOCATE`, etc.) and handle multi-segment / space waste.
4. Prealloc-fill still loses to overwrite (~106k): remaining cost is **dirtying unique payload pages** as durable bytes land, not only extent metadata.

## What to say in one line

**Prealloc helps only if it actually allocates.** Sparse size is a placebo here; forced page allocation is a real ~4× thr win on this hammerblast cell — enough to pass SQLite’s ~30k on the same bed — but it is a diagnostic spike, not a shipped feature.

## How to re-run

```sh
BIN=target/release/residiuum-testrig
for d in real realprealloc realpreallocfill realoverwrite; do
  $BIN peer-pump -w /var/tmp/pre-$d --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out
done
```

See also [WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md). Diagnostic only — not a product SLO.
