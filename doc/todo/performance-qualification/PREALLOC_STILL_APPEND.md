# Prealloc mode — are we still appending?

**Date:** 2026-08-03  
**Card:** `b002d722`  
**Ask:** grow/append = ~12–14k TPS — but in prealloc mode we are not appending, are we?

## Short answer

**Correct: inside reserved capacity, we are not extending EOF.**  
Watermark `set_len`s the file first, then writes at `durable_len` into space that already exists. That is **not** the grow-on-append path.

So why TPS still ≈ grow? Because product watermark still pays **first-touch (zeroing) on the put path** as the head advances — and/or runs out of runway and grows again. We removed EOF growth, not the cost that was killing TPS.

## Two different “appends”

| Mode | File length | What each put does |
|------|-------------|--------------------|
| **Grow (default)** | Grows with data | `write_all` past EOF → OS allocates + first-touches **new** pages |
| **Watermark (opt-in)** | Fixed up to `capacity` | `write_all` into already-sized range — **no EOF grow** while under capacity |

Your instinct matches the bisect: overwrite-into-existing ≈ **~96k**; grow ≈ **~12–14k**. Prealloc *should* look like the first.

## What product watermark actually does

Before each real write it may call `ensure_zero_watermark` — bulk-zero the next chunk **in the same TPS window**. So:

```text
Ideal prealloc:   [pages already hot] → write record → high TPS
Product watermark: [zero next chunk NOW] → write record → TPS stays ~grow
```

Also: with **64 MiB** capacity on a **256 MiB** run, after the runway you **do** start growing again (or seal/rotate and pay setup again).

## TPS evidence

| | TPS |
|--|----:|
| Grow | ~12–14k quiet · try ~6.7k |
| Watermark (product) | **≈ grow** — no win |
| Diag bulk-zero **before** timer | ~35–50k |

Diag got the shape you expect. Product did not: zero still sits on the hot path (and try bed was dirty).

## Bottom line

```text
Prealloc mode ≠ grow-append. You are right.
½ GiB watermark WAS run for TPS (~7.5k) with no need to extend — [CLARIFY_SHIPPED_FOR_TPS.md](CLARIFY_SHIPPED_FOR_TPS.md).
It still ≠ “free overwrite TPS” — product first-touches (zeros) on the put path.
That is why TPS did not jump to the diag ~50k / overwrite ~96k band.
```
