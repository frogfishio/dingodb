# Most expensive work? (TPS)

**Date:** 2026-08-03  
**Ask:** “so the most expensive work is what?”

## Answer

**Appending / growing the active segment file** — OS allocation and first-touch as the log extends under Real puts.

That is what drops TPS from the ~**120k** ready-for-write band (Discard / `/dev/null`) and the ~**100k** overwrite band down to ~**10k** product Real.

## Ranked by TPS impact (same peer recipe)

| Rank | Work | Evidence |
|-----:|------|----------|
| **1** | **Active-segment append growth** | Discard/DevNull ~120–124k · Overwrite ~100–107k · Real grow ~10k ([WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md), [REMEASURE_DEVNULL.md](REMEASURE_DEVNULL.md)) |
| 2 | End-of-run seal / unaccounted wall | Probe sums ≪ peer wall (~3.4 s); seal inside odometer ([DATA_VS_INDEX_TIMING.md](DATA_VS_INDEX_TIMING.md)) |
| 3 | Index/derived side effects | skip-index Real ~30k vs full ~9.5k; publish timer itself only ~11 ms |
| — | Blake / cook | Not #1 — Discard still ~120k |
| — | Index publish timer alone | ~11 ms vs ~173 ms FileWrite |
| — | Watermark / bg runway as shipped | No TPS win vs grow |

## One line

```text
Most expensive = growing the active segment on the put path.
```
