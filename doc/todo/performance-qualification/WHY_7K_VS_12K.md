# Answer — why ~7k now vs earlier ~12k?

**Date:** 2026-08-03  
**Card:** `fa9808a8`  
**Ask:** “wait we’ve got only 7K transactions now to our earlier 12k?”

## Short answer

**Not a silent product regression.** Same Mode A recipe family; different **host bed**. Quiet APFS runs land Residiuum Real around **~12–14k**. Today’s try / product-wm beds were **~95% full** and landed Real/grow around **~6.5–7.7k**. Quote a **band**, not one integer.

## Side-by-side (same recipe family)

Mode A · c=8 · 8 KiB · 256 MiB · APFS `/var/tmp` · Residiuum grow/Real (product default path):

| When / artifact | ops/s | Disk note |
|-----------------|------:|-----------|
| Concurrent feed ([firm-numbers-concurrent-apfs](artifacts/firm-numbers-concurrent-apfs/)) | **~13 200** | Quiet-era peer |
| Concurrent+cook ([firm-numbers-concurrent-multicore-apfs](artifacts/firm-numbers-concurrent-multicore-apfs/)) | **~13–14k** | Quiet-era |
| Write-all bisect Real | **~10 100** | Mid band |
| Product-wm paired grow ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md)) | **~7 700** | ~93% full / noisy |
| Try watermark@64 grow ([TRY_WM_64MIB.md](TRY_WM_64MIB.md)) | **~6 700** | ~95% full / ~12 GiB free |

So “earlier 12k” ≈ quiet concurrent Real. “Now 7k” ≈ **same grow path on a full/noisy volume**, not “watermark ate half our TPS.”

## Proof it isn’t the 64 MiB default change

On the try bed, **grow** and **watermark@64** were within noise of each other (~6.5–6.8k). If the default capacity change had broken the write path, grow would still be ~12k while watermark alone fell. It didn’t.

| Cell (try bed) | ops/s |
|----------------|------:|
| Grow | ~6 700 |
| WM 64/64 | ~6 500–6 800 |
| WM 512/64 | ~7 500 |
| SQLite (same bed!) | ~29 100 |

SQLite stayed in the quiet peer band on that volume — Residiuum’s grow-on-append / first-touch path is **more sensitive** to a tight APFS volume than SQLite’s journaled file on this recipe.

## What to remember

```text
Quiet bed Real     ~10–14k   ← “earlier 12k”
Noisy/full Real    ~7–9k     ← “now 7k” (documented in HOW_MANY_TPS_NOW)
Watermark E2E      ≈ Real on that bed   ← not the 12→7 cause
```

Already called out in [HOW_MANY_TPS_NOW.md](HOW_MANY_TPS_NOW.md): default Real is **~10–14k quiet · ~7–9k noisy/full disk**.

## Non-claims

Not that 7k is the new permanent floor. Not that full-disk is the only variance (APFS cache, thermal, concurrent I/O). Not that we should ignore 7k — for thr claims, prefer a quiet bed or disclose the disk %. Background preparer still the thr lever to try next; this question is **meter/bed**, not “we lost 5k in code overnight.”
