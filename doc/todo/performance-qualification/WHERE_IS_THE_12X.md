# Where is the 12× gap?

**Date:** 2026-08-03  
**Ask:** “if we have 2 systems writing similar things at the same time — where’s the 12× gap?”  
**Scoreboard:** TPS. Sources: [WRITE_MIMIC.md](WRITE_MIMIC.md), [REMEASURE_DEVNULL.md](REMEASURE_DEVNULL.md), [DATA_VS_INDEX_TIMING.md](DATA_VS_INDEX_TIMING.md), [SEE_THE_PROBLEM.md](SEE_THE_PROBLEM.md).

## The two systems

| System | What it does | TPS band |
|--------|----------------|---------:|
| **A — write-mimic / Discard / DevNull** | Same-sized appends *or* Residiuum cook with no durable grow | **~124–130k** |
| **B — Residiuum peer Real** | Cook + Real segment append (grow) + index/derived | **~9–11k** |

They are not fighting for the disk in these measures (separate runs, clean `/tmp`). The 12× is **not** “two writers starve each other.” It is **B’s put path vs A’s same-shaped I/O.**

## Where the 12× lives (multiplicative ladder)

Same recipe (Mode A · c=8 · 8 KiB · 256 MiB). Factors are approximate TPS ratios.

```text
~129k   write-mimic data-only     ← same byte sizes, seek+write_all grow
~128k   Discard / DevNull         ← Residiuum cook; write skipped or → /dev/null
         │
         │  ≈ 1×   cook ≈ bare writes; disk/syscall not the story yet
         ▼
~107k   RealOverwrite             ← real fd, smash existing pages (no grow)
         │
         │  ≈ 1.2×  still fine — writing into hot pages is cheap
         ▼
 ~30k   Real + skip-index         ← Real grow append + cook; no dual-index/derived
         │
         │  ≈ 4×   ★ GAP 1: durable append / file growth / first-touch
         ▼
 ~10k   Real full peer            ← + index publish + derived/seal coupling
         │
         │  ≈ 3×   ★ GAP 2: index/derived/seal side effects (not the 11 ms publish timer alone)
         ▼
 ~10k   ← total vs mimic ≈ 12–13×
```

| Slice | Factor | What it is |
|-------|-------:|------------|
| Mimic ≈ Discard ≈ DevNull | **~1×** | Cook and same-sized bare writes are both ~129k — **neither alone is the 12×** |
| → RealOverwrite | **~1.2×** | Real fd without growth still ~100k+ |
| → Real skip-index | **~4×** | **Durable grow / first-touch append** — largest single step |
| → Real full | **~3×** | Index/derived/seal coupling beyond the tiny Publish timer |
| **Mimic → Real full** | **~12×** | Product of the slices |

## What it is *not*

- Not “two systems writing at once” contention (we did not co-schedule them).  
- Not chunk size (coalesce ~11k).  
- Not “Blake/cook is 12×” (Discard ≈ mimic).  
- Not “the 11 ms index publish sum” (that is ≪ wall; skip-index still only ~30k).

## Probe check (same story, different meter)

On Real full, FileWrite sum ~173 ms and Publish ~11 ms of ~3428 ms wall — step timers ≪ wall; residual is seal/lock/scheduling. TPS ratios above are the honest localization.

## One line

```text
12× = (~4× Real grow/first-touch) × (~3× index/derived/seal coupling).
Bare same-sized writes and cook-without-grow are both fine (~129k).
The gap is how Residiuum grows and finishes Real puts — not “the disk” and not “two writers.”
```
