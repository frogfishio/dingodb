# Answer — what happened between ~50k (zeroed) and ~10k (Real)?

**Date:** 2026-08-03  
**Card:** `bc6c95b1`  
**Ask:** We had ~130k on Discard/devnull, ~50k with a pre-zeroed segment (fair — disk costs). Why are we “suddenly” back at ~10k?

## Short answer

**Nothing new broke.** ~10k is ordinary **Real grow-on-append**.  
~50k was Real **after** you already paid “make the pages exist” **before** the put timer.  
The gap is **when** first-touch / file-growth is paid — not a third mystery tax that appeared later.

## The ladder (same Mode A · c=8 · APFS recipe)

```text
~120–130k   Discard / DevNull / SeekOnly
              cook+index; no real segment write_all

 ~96k        RealOverwrite
              write into a real fd, but file does NOT grow

 ~50k        Real + bulk-zero (or ~35k page-touch) BEFORE odometer
              append into pages that already exist

 ~10k        Real (default grow-on-append)
              each put extends the file → OS allocates + first-touches new pages
              INSIDE the timed window
```

Sources: [WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md), [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md).

## Between 50k and 10k — one sentence

**Real is still writing records to disk; it is also growing a sparse/new extent under those writes.**  
Pre-zero moved the growth/first-touch offline; Real left it on the hot path.

```text
50k cell:  [==== zero 512 MiB (untimed) ====][==== puts into ready pages ====]
10k cell:  [==== puts that ALSO allocate + first-touch as the file balloons ====]
```

That hot-path growth is ~**5×** on this bed (50k → 10k), on top of the earlier drop from cook-only (~130k) to “real fd but no growth” (~96k).

## Why it felt “sudden”

| Feeling | Reality |
|---------|---------|
| “We had 50k storing records” | Yes — but only with **setup zero/touch paid first** (diag), not default product Real |
| “Then we fell to 10k” | Default Real was always ~10k on this ladder; watermark ~32k was a **seal-fail cheat**, not a stable product band |
| “Did sealing / honesty erase the 50k?” | Honesty removed a **fake** mid band; it did **not** invent a new wall under Real |

Product `--segment-growth watermark` tries to amortize zeroing like the spike — after seal fixes, honest E2E ≈ Real ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md)), because full E2E still pays seal/chimera and/or mid-run chunk zeros inside the same meter. The **put-path** growth story from the bisect still stands.

## What is *not* in the 50→10 gap

- Not Blake/cook suddenly getting 5× worse (Discard still ~120k)  
- Not “seek” (SeekOnly ≈ Discard; RealNoSeek ≈ Real)  
- Not write *chunk size* (64 KiB coalesce ≈ Real)  
- Not “we stopped storing records” — Real still acks Buffered puts to a growing segment

## If you want 50k-class again (honestly)

You must either:

1. **Pay first-touch offline** (diag; not a silent product default), or  
2. **Amortize it** in a disclosed product policy (watermark chunks / background prepare) and measure with a meter that matches the claim (put-path vs full E2E seal), or  
3. Accept ~10k as default Real until a different lever moves growth cost without lying.

See also [FIFTY_TO_6_5K_PREALLOC.md](FIFTY_TO_6_5K_PREALLOC.md) for why recent E2E tries land ~6.5k (bed + seal), not a second mystery wall.

## Non-claims

Not that 50k is a product SLO. Not that SQLite’s growth is free (they also grow a file — cheaper *per Mode A ack* is a separate question). Not package accept / default-on.
