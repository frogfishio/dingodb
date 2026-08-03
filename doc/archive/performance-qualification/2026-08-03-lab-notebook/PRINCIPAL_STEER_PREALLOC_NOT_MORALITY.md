# Principal steer — prealloc is not a morality play

**Date:** 2026-08-03  
**Card:** `ee0b5919`  
**Steer (principal):** People buy specialist hardware for single-digit % speed/resilience. Debating the “morality” of preallocating ~½ GiB is unserious. Extension need not tax transactions — a watcher can extend ahead. No serious database prefers grow-on-demand “honest empty” over preallocating modest capacity.

## Labor response

**Agreed.** The soft “honest storage / stay tiny” defense of default `GrowOnAppend` is withdrawn as product virtue ([GROW_ON_APPEND_BUYS_RETRACT.md](GROW_ON_APPEND_BUYS_RETRACT.md)). Space amp at movie-size scale is a **disclosure**, not a veto. Capacity itself is **tunable** (product default **64 MiB**, not locked to ½ GiB — see [PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md](PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md)).

## What this changes in the argument

| Old framing (wrong) | Steer (correct) |
|---------------------|-----------------|
| Is it ethical to reserve 512 MiB? | Industry spends real money for ≪5× thr; ½ GiB is cheap |
| Growth must happen on the put path | **No** — ahead-of-write / background prepare is the point |
| Grow-on-append is the virtuous default | It is **status quo**, not a DB-industry preference |
| Watermark only if thr proven E2E first | Ship the **shape** (runway ahead of head); prove thr with honest meters |

## Extension ≠ transaction cost

Principal’s design point (already listed as a candidate in [NEXT_STEPS_WRITE_GROWTH.md](NEXT_STEPS_WRITE_GROWTH.md)):

```text
put path:     write into already-zeroed runway
watcher:      keep N MiB (or seal-chunk) prepared ahead of durable_len
              optionally prepare segment N+1 in background
```

Current product watermark already zeros **on the put path** when runway runs low (`ensure_zero_watermark` in `write_segment_tail`). That still couples extension to transactions. **Next labor:** move that work to a **background preparer** so puts only consume runway.

## What “honest” still means (narrow)

Honesty = **disclose** space amp + when ENOSPC can fire + don’t cheat the odometer (seal-fail tricks).  
Honesty ≠ refuse to preallocate.

## Proposed next (not done this card)

1. **Background segment runway watcher** — extend/zero ahead of `durable_len` off the put critical path; put path fails closed if runway exhausted.  
2. **Quiet-disk re-pair** — grow vs watermark vs background-prep vs SQLite with put-path and E2E meters both disclosed.  
3. **Principal call on default-on** — after (1)–(2), not before.

## Non-claims

Not that background watcher is implemented. Not that default-on flipped tonight. Not raw-device ResidiuumFS.
