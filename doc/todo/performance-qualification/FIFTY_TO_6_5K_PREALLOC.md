# Answer — how do we go from ~50k → ~6.5k “with prealloc”?

**Date:** 2026-08-03  
**Card:** `e8086e9b`  
**Ask:** “how do we go from 50K → 6.5 k with prealoc?”

## Short answer

Those are **not the same experiment**. ~50k was a **diagnostic** cell where pages were bulk-zeroed **before** the put timer. ~6.5k is **product watermark / grow on a full-disk bed** with **seal inside the E2E meter**. Prealloc did not “fail and become 6.5k” — we changed **what is timed** and **which bed**, then measured a product path that still pays work the 50k cell hid.

## Two different “prealloc” stories

| | ~50k cell | ~6.5k cell |
|---|-----------|------------|
| What | `realprealloczero` / fill (diag) | Product `--segment-growth watermark` @64 MiB *or* grow on same try bed |
| First-touch | Paid **before** odometer (untimed setup) | Paid **in** put path (chunk zeros) and/or grow after runway |
| Seal / chimera | Often **outside** pump ops/s (or skipped in broken diag) | **Inside** peer-pump wall (`seal_active` at end) |
| Bed | Earlier quieter APFS runs | Try bed ~**95% full** ([TRY_WM_64MIB.md](TRY_WM_64MIB.md)) |
| Source | [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md) ~51k pump | Grow ~6.7k · WM64 ~6.5–6.8k |

```text
50k:   [==== bulk-zero 512 MiB UNTIMED ====][==== puts into ready pages ====]
6.5k:  [==== puts + mid-run zero/grow + SEAL/chimera all TIMED ====]  on tight disk
```

## The drop is stacked (not one cliff)

```text
~50–51k   Diag: F_PREALLOCATE + bulk zero BEFORE timer (pump ops/s)
              ↓  put first-touch back on the hot path
~10–14k   Quiet Real grow (default product) — [FIFTY_TO_TEN.md](FIFTY_TO_TEN.md)
              ↓  same recipe, 93–95% full /var/tmp
 ~6.5–7.7k  Noisy Real / product watermark E2E — [WHY_7K_VS_12K.md](WHY_7K_VS_12K.md)
              (+ seal in meter; watermark ≈ grow on that bed)
```

So **50 → 6.5 ≈ (50 → ~10–12) + (~12 → ~7)**  
1. **Meter honesty / growth timing** — biggest step ([FIFTY_TO_TEN.md](FIFTY_TO_TEN.md)).  
2. **Bed noise** — another ~2× on today’s volume ([WHY_7K_VS_12K.md](WHY_7K_VS_12K.md)).

Product watermark tries to look like the 50k setup by zeroing ahead of the head — but:

- Zeroing is still often **on the put path** (not a background watcher yet).  
- E2E peer-pump still pays **seal + chimera**.  
- 64 MiB capacity on a 256 MiB run is not “whole file pre-zeroed offline like the diag.”  
- On the try bed, **grow ≈ watermark** (~6.5–6.8k) — so “prealloc flag on” did not recreate the 50k meter.

## What is *not* the explanation

- Not “prealloc is useless” — offline bulk-zero **did** unlock ~35–50k on the **put** odometer.  
- Not Blake/cook suddenly 8× worse (Discard still ~120k).  
- Not the withdrawn ~32k watermark cheat (that was seal-fail, not a real 50→32).  
- Not that 6.5k is the quiet-bed floor forever.

## How to get 50k-class again (honestly)

| Lever | Role |
|-------|------|
| Pay first-touch **off** the put timer | Diag 50k shape; product = **background runway preparer** (still `todo`) |
| Measure put-path vs E2E seal separately | Disclose which claim you are making |
| Quiet disk (or Scratch) | Separates bed tax from growth tax |
| Capacity covering the run | 64 MiB default is space-honest; for thr trials use enough runway or seal within capacity |

## Non-claims

Not that product watermark should print 50k after seal. Not default-on. Not that 6.5k disproves the first-touch diagnosis — it **uses** a harsher meter on a worse bed.
