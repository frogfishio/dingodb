# Help me understand the numbers

**Date:** 2026-08-03 · Diagnostic only · APFS `/var/tmp` unless noted

One page that stitches the firm-number arc so the coalesce spike does not
fight the “CPU wall” story.

## “HOW we write makes all the difference?”

**Not quite.** See [HOW_WE_WRITE_CORRECTION.md](HOW_WE_WRITE_CORRECTION.md).

- Write **chunk size** / 64 KiB coalesce → **no** (Coalesce ≈ Real).
- Paying for segment-tail **`write_all`** → **yes, huge on our Real cell** (Discard ~13×).
- Our real-write path vs SQLite’s → **we lose ~2×**; that is “how/what we pay for durable bytes,” not “adaptive disk paging.”

## What every cell is measuring

Same work package, almost always:

- **Mode A** = one key per commit / `put_many(1)` (autocommit peer)
- **8 KiB** payload · **256 MiB** logical target → **32 768** acked puts
- Odometer = **acked puts/s** (SQLite-comparable)
- Not a 1-second timed race — fixed work; faster engines just finish sooner

Two feeds matter:

| Feed | Flag | Meaning |
|------|------|---------|
| Embedded sync | `--concurrency 1` | Client waits for each ACK before sending the next (QD=1) |
| Server-async | `--concurrency 8` | Eight clients in flight — AWO can microbatch |

## The ladder (what to remember)

```text
SQLite A .............. ~30 000 puts/s   (real disk writes)
Residiuum Real ........ ~10–14 000       (real segment write_all)
Residiuum Coalesce64k . ~11 000          (same, bigger write chunks)
Residiuum Discard ..... ~129 000         (cook+index, NO write_all)
```

Rough picture:

```text
  129k ─ Discard ─────────────────── “CPU cook alone can go here”
   30k ─ SQLite ──────────────────── real writes, cheaper per op
   13k ─ Residiuum Real ──────────── real writes, our expensive path
    2.5k ─ AWO on QD=1 only ──────── collection delay tax (not CPU)
```

## Story in order (so nothing contradicts)

### 1. Scratch ~10k parity was two different walls wearing the same number

On the Samsung T3 (exFAT), SQLite and Residiuum both hovered ~10k.
That looked like “we’re as fast as SQLite.” It was a coincidence:

- SQLite was **disk-bound** on that bed
- Residiuum was already near its **own** ceiling

Move SQLite to APFS `/var/tmp` → SQLite jumps to **~30k**. Residiuum stays
**~12–14k**. Same Residiuum number on slow and fast media → Residiuum was
**not** waiting on media the way SQLite was. That is the original “fast disk
→ Residiuum CPU/path wall” finding.

### 2. Adaptive ~2.5k on FN-2 was feed sabotage, not “smart mode is slow”

FN-2 used embedded sync (QD=1). Static/Adaptive sat at **~2.5k** while off
was **~12.5k**. That was the AWO **collection delay tax** with nothing else
in the waiting window — not Blake, not disk.

With `--concurrency 8`, Adaptive ≈ Static ≈ off at **~13–14k**. The delay tax
vanishes. AWO still does **not** beat Residiuum-off on this Buffered Mode A
cell. Multicore cook (4/8) also does not lift thr here.

**Do not quote FN-2 Adaptive ~2.5k as server-async smart mode.**

### 3. The coalesce spike separates “write size” from “doing writes”

| Cell | Result | Means |
|------|--------|--------|
| Real ≈ Coalesce64k (~10–11k) | Write **shape** (many small vs ≥64 KiB / 250 ms) does **not** move thr | No evidence for an “adaptive disk pager” that waits to fill pages |
| Discard ~129k (~13× Real) | Skipping segment-tail **`write_all`** unlocks a huge band | A large share of Real wall **is** the OS write path (syscall / page-cache), not Blake alone |

So the refined statement:

- **Not media-seek / not write-granularity** (T3≈APFS for Residiuum; coalesce flat)
- **Yes, write_all cost** inside Residiuum’s Real path (Discard proves it)
- **Also yes, we still lose to SQLite’s real-write path** (~30k vs ~13k) — SQLite pays less per autocommit for this shape

“It is not the disk” was half-right: it is not *which disk* or *how big each
write is*. It **is** still “we spend a lot of time in writing,” just not in
the way a 64 KiB buffer would fix.

### 4. Why Real was ~10k in the spike vs ~13k earlier

Same bed, same Mode A c=8, different runs. Treat **±20–30%** as machine
noise unless a ratio repeats. The spike signal is **relative**:

- Coalesce / Real ≈ **1.1×** → flat
- Discard / Real ≈ **13×** → decisive

## What each number is *not*

| Number | Not this |
|--------|----------|
| Residiuum ~13k | “CPU is 100% Blake” — Discard shows write_all dominates Real |
| Discard ~129k | Product thr — no durable bytes; upper bound on cook+index only |
| SQLite ~30k | Proof Residiuum must hit 30k tomorrow — different engine cost model |
| AWO ~13.6k concurrent | “Adaptive wins” — ≈ off, not a thr win on this cell |
| AWO ~2.5k FN-2 | Smart-mode ceiling — QD=1 delay tax only |

## One-sentence takeaways

1. **SQLite ~30k vs Residiuum ~13k** = we lose on the real-write autocommit peer; fast disk exposed that, it did not create it.
2. **AWO ~2.5k → ~13k with concurrency** = feed shape, not disk.
3. **Coalesce ≈ Real** = buffering write size does not help.
4. **Discard ~129k** = our Real path is heavily write-bound; cook alone is far above SQLite.
5. **Write-path bisect:** the wall is **appending / growing** the active segment file — not seek, not `write_all`→`/dev/null`, not overwrite-in-place ([WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md)).

## Where the detail lives

| Doc | Role |
|-----|------|
| [FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md) | Four-cell embedded sync |
| [FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md) | c=8 removes AWO delay tax |
| [FIRM_NUMBERS_DIAG_COALESCE.md](FIRM_NUMBERS_DIAG_COALESCE.md) | 64 KiB spike + Discard control |
| [FAST_DISK_CPU_WALL.md](FAST_DISK_CPU_WALL.md) | Scratch parity coincidence |
| [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md) | Why ~2.5k happened |
| [EMBEDDED_SYNC_VS_SERVER_ASYNC.md](EMBEDDED_SYNC_VS_SERVER_ASYNC.md) | Feed-shape lock |
