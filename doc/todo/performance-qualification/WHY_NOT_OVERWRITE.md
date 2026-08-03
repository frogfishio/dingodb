# Why not overwrite? Isn’t that prealloc?

**Date:** 2026-08-03  
**Ask:** “so why don’t we do that? overwrite no growth? I don’t get it. Isn’t that prealloc?”

## Short answer

**No — overwrite ≠ prealloc.**  
Overwrite is a **bisect cheat** that destroys data. Prealloc/watermark is the **product-shaped** “don’t grow EOF on put” idea — and we already tried it; it did **not** give overwrite’s ~100k TPS.

## Two different tricks

| | **Overwrite (diag)** | **Prealloc / watermark (product try)** |
|--|----------------------|----------------------------------------|
| Where bytes go | Always `seek(0)` — smash the same tiny region | Advance `durable_len` — each put is a **new** frame at a new offset |
| File length | Stays tiny | Reserved big (`set_len` / capacity) so EOF need not extend |
| Correctness | **Broken** (locators / history lie) | Intact (append log into reserved space) |
| TPS we saw | **~100–107k** | **≈ grow (~7–14k)** on peer E2E |

```text
Overwrite:   [====]← rewrite here forever     ← fake “no growth”
Prealloc:    [====|====|====|....]            ← write forward into reserved runway
                  ^ head advances
```

## Why we don’t ship overwrite

Residiuum is an append log. Frames at absolute offsets must stay. Overwrite throws that away — fine for proving “write_all into an existing page is cheap,” useless as a store.

## Why prealloc didn’t become the ~100k win

Ideal: pages already hot → write into them → look like overwrite.

What product watermark did: reserve length, but still **first-touch (zero) on the put path** (or run out of runway). Removing EOF extend ≠ removing that cost. Offline full-zero before the timer hit ~35–50k (diag). Background runway preparer was tried; E2E TPS still did **not** beat grow ([BG_RUNWAY_PREPARER.md](BG_RUNWAY_PREPARER.md), [PREALLOC_STILL_APPEND.md](PREALLOC_STILL_APPEND.md)).

## One line

```text
Overwrite = cheat (smash offset 0). Prealloc = real idea (write forward into reserved space). We tried prealloc; TPS stayed ~grow.
```
