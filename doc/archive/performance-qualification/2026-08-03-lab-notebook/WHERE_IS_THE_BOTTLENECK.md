# Where is the bottleneck? (TPS)

**Date:** 2026-08-03  
**Card:** `6ca288c0`  
**Metric:** TPS only ([TPS_ONLY.md](TPS_ONLY.md)).

## Answer

**Growing the active segment file under append** — each put extends the log; the OS pays allocation / first-touch on that growth. That is what holds default Residiuum at ~**12–14k TPS** (quiet) instead of the ~**100k** band you get when the same code writes into a file that does **not** grow.

## Proof (same recipe, TPS)

| Path | TPS | Meaning |
|------|----:|---------|
| No real disk write (Discard) | ~120k | Cook/index not the wall |
| Write into real file, **no growth** (Overwrite) | ~96k | Writing bytes is fine |
| Default: **append / grow** the segment | ~10–14k | **This is the bottleneck** |
| SQLite peer | ~25–30k | Still faster than our grow path |

Source: [WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md).

## Not the bottleneck (already ruled out for TPS)

- Seek  
- “write_all as a syscall”  
- Blake / cook (Discard still ~120k)  
- Write chunk size (coalesce ≈ grow)  
- Opt-in watermark as shipped (no TPS win)

## Bottom line

```text
Bottleneck = append growth of the active segment (hot path).
Gap to SQLite ≈ that cost is still too high per acked put.
Prealloc opt-in did not remove it from TPS.
```
