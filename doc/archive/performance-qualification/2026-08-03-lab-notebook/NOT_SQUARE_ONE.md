# Answer — are we back to square 1?

**Date:** 2026-08-03  
**Card:** `2f9e91f5`  
**Ask:** “wait, so we’re back to square 1?”

## Short answer

**No.** We lost a **fake win**, not the map.

The ~32k watermark thr was a **measurement cheat** (broken seal skipped in the odometer). Walking that back puts product watermark back near **Real grow** — it does **not** erase the write-path diagnosis that got us here.

## What we still know (kept)

| Finding | Status |
|---------|--------|
| Mode A wall on fast disk is **append/growth / first-touch**, not “mystery idle” | Kept |
| Sparse `set_len` alone is **placebo** | Kept |
| Physical page touch / bulk zero **can** move thr (ceiling probes still real) | Kept |
| AWO on Mode A QD=1 pays a **collection delay tax** (~2.5k); judge multi-client at **c>1** | Kept |
| Concurrent feed: Adaptive ≈ off ~10–14k quiet; still ≪ SQLite ~25–30k quiet | Kept |
| Opt-in `SegmentGrowthPolicy::Watermark` **API exists** (space amp; not default-on) | Kept |
| Seal must truncate prealloc tails; diag must not clobber the durable prefix | **New** (bugfix) |

## What we lost (withdrawn)

| Claim | Why gone |
|-------|----------|
| “Watermark ≈ SQLite / ~28–32k product thr” | Diag seal failed; peer ignored error → seal/chimera never timed |

Honest re-pair: watermark ≈ grow ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md)).

## Square-1 vs square-N

```text
Square 1 would be: “we have no idea why Mode A is slow.”
Where we are: we know the wall class; we know what does NOT win;
              we know one attractive lever was a bad meter reading.
```

Progress = **narrower search**, not a thr trophy.

## What “next” actually is

1. **Put-path-only odometer** (disclose): time acked puts **excluding** end-of-run seal/chimera if the question is growth/first-touch — then re-pair grow vs watermark vs SQLite on a quiet disk.  
2. Or keep full E2E seal in the meter (product-honest) and accept watermark ≈ Real until a different lever moves both put path **and** seal.  
3. Separate lanes stay separate: AWO batching under pile-up (T11), Mode B / txn story, Discard ceilings ≠ product.

## Non-claims

Not that we beat SQLite. Not that watermark is useless forever. Not package accept / default-on.
