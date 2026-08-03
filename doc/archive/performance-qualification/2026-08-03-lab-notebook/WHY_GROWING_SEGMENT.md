# Why are we growing the segment?

**Date:** 2026-08-03  
**Ask:** “why are we growing the segment?”  
**Context:** [WHERE_IS_THE_12X.md](WHERE_IS_THE_12X.md) — ~4× of the 12× gap is Real grow / first-touch.

## Short answer

Because the **default policy is still `GrowOnAppend`**: each put appends frames at `durable_len`, and when that sits at EOF the OS **lengthens the file**. That is ordinary append-log mechanics — not a separate “grow API” we call for fun.

```text
put → cook → seek(durable_len) → write_all(frame)
                                  └─ past EOF ⇒ segment file grows
```

We are **not** required by the log model to grow under the put. A pre-sized / pre-zeroed runway (watermark) is the **same append log**, writing forward into space that already exists. Grow-on-append is **status quo default**, not physics.

## Why the default still does it

| Reason | Honest weight |
|--------|----------------|
| Historic default / inertia | **Strong** — this is what shipped |
| Avoid undiscussed space amp on every open | **Medium** — real cost to disclose; **not** a virtue that justifies ~10k TPS ([GROW_ON_APPEND_BUYS_RETRACT.md](GROW_ON_APPEND_BUYS_RETRACT.md), principal steer) |
| “Salvage needs grow-on-append” | **False** — scan works either way |
| “Empty stores must stay tiny” | **Withdrawn as product dogma** |

So: we grow because **we have not flipped default** to “reserve runway ahead of the head.” Watermark exists and is opt-in; E2E TPS has **not** clearly beaten grow, so default stayed put.

## Growing ≠ the log

| Shape | File length under puts | Same append log? |
|-------|------------------------|------------------|
| GrowOnAppend (default) | Extends as frames land | Yes |
| Watermark / prealloc | Fixed capacity; write at advancing head | Yes |
| Diag overwrite@0 | Does not grow; **wrong** product shape | No |

The expensive part is **first-touch of new pages while the put timer runs** — whether that comes from EOF growth or from zeroing runway on the put path. Principal model: zero ahead **off** the put; puts only consume ready bytes ([PREALLOC_IS_YOUR_MODEL.md](PREALLOC_IS_YOUR_MODEL.md)).

## Tie to the 12×

```text
~129k  same-sized writes without Residiuum grow semantics
 ~30k  Real + skip-index   ← ~4×  durable grow / first-touch
 ~10k  Real full           ← ×~3× index/derived
```

We grow (by default) **because that is the current product growth policy**, and that policy is exactly where the largest TPS slice sits. Not because append logs must extend under every transaction.

## One line

```text
We grow the segment because GrowOnAppend is still the default:
appends past EOF extend the file. That is inertia + space caution —
not a requirement of the log — and it is the ~4× thr tax in the 12× gap.
```

See also: [WHY_EXTEND_EACH_TIME.md](WHY_EXTEND_EACH_TIME.md) (older), [PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md](PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md).

**Tradition call-out:** [NOT_TRADITION.md](NOT_TRADITION.md) — default grow is inertia, not a vow.
