# Answer — why are we extending the file each time?

**Date:** 2026-08-03  
**Card:** `010f3db5`  
**Ask:** “there you go… so why are we extending the file each time?”

## Short answer

Because the **default product policy is append-only grow-on-demand**: the active segment is a log. New frames are written at the current durable end. That **extends EOF** as data arrives. We do **not** pre-size + pre-touch by default — that costs **space** (~512 MiB/active) and setup I/O, so it stays **opt-in** (`SegmentGrowthPolicy::Watermark`).

## What “each time” actually means

Not a special `set_len(+N)` ceremony per put.

```text
put → cook frame → seek(durable_len) → write_all(frame)
                                         │
                                         └─ if durable_len was at EOF, the file grows
```

So “extend each time” = **every successful Buffered append past EOF lengthens the file** under `GrowOnAppend`. That is how an append-only segment works.

## Why that design (not stupidity)

| Reason | Detail |
|--------|--------|
| **Log model** | Active segment = ordered frame stream; authority is the byte suffix, not a fixed-size page arena |
| **Pay for what you store** | Empty / small DBs stay small; no forced 512 MiB hole on every open |
| **Salvage / crash story** | Prefix of a growing file is still a scannable island; no dependency on “we promised capacity X” |
| **Default ≠ max thr** | First-touch on growth is expensive on APFS ([FIFTY_TO_TEN.md](FIFTY_TO_TEN.md)) — known; traded for space honesty |

## Why we don’t just preallocate always

The bisect said: **overwrite into an existing file ~96k**, **grow-as-you-go ~10k**. So growth hurts thr.

But pre-zero / watermark means:

- Reserve ~**512 MiB per active** (seal-shaped) even if you only write 1 MiB  
- Zero or first-touch that capacity (setup or mid-run chunks)  
- Host must **opt in** and accept space amp ([PRODUCT_SEGMENT_WATERMARK.md](PRODUCT_SEGMENT_WATERMARK.md))

Default stays `GrowOnAppend` until principal accepts default-on + disclosure.

## Two knobs (today)

```text
GrowOnAppend (default)     Watermark (opt-in)
─────────────────────      ──────────────────
file grows with puts       capacity reserved + runway zeroed ahead
small stores stay small    ~512 MiB/active amp
~10k Real class on APFS    put-path can look like pre-touch band;
                           honest E2E still ≈ Real so far after seal fix
```

## So are we “choosing” the 10k wall?

**Yes, as the default trade:** cheap empty stores + simple append log **over** paying growth/first-touch up front.  
The wall is not mysterious; it is the **price of that default**. Escaping it means enabling (and eventually proving) a growth policy that moves first-touch off the hot path **without** lying on the meter — not inventing a third write path.

## Non-claims

Not that watermark is proven faster E2E. Not that SQLite does the same thing. Not default-on. Not that per-put `write_all` size is the main story (coalesce ≈ Real).
