# Hang on — the ~100k overwrite number

**Date:** 2026-08-03  
**Ask:** “so hang on you just said about 100k (overwrite no growth)”

## Yes — that number is real

Fresh peer-pump (same recipe as DevNull remeasure):

| Cell | TPS | What the sink does |
|------|----:|--------------------|
| Discard | ~130k | no `write_all` |
| DevNull | ~124k | `write_all` → `/dev/null` |
| **RealOverwrite** | **~107k** | `seek(0)` + `write_all` on a **real** file that **never grows** |
| Real (append/grow) | ~10k | append extends the active segment |

Sources: [REMEASURE_DEVNULL.md](REMEASURE_DEVNULL.md) (~106 647 overwrite); earlier bisect ~96k ([WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md)).

## What it means

Writing bytes through a real file descriptor is **cheap** on this bed — **if the file does not grow**.

The cliff to ~10k appears only when each put **appends / extends** the active segment (new pages / extents / first-touch). That is why “most expensive = growth,” not “disk write in general.”

## What it is not

- **Not product.** Overwrite smashes at offset 0 — durability/locator correctness is gone. Diagnostic bisect only.
- **Not** “we already have 100k product TPS.” Product Real still ~10k.
- **Not** a shippable mode. Do not turn overwrite on.

## One line

```text
~100k overwrite = proof writing is fine; growth is the tax. Not a product path.
```
