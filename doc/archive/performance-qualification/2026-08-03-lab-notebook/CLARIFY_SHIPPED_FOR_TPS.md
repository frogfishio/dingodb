# Clarify — “never shipped for TPS” was bad wording

**Date:** 2026-08-03  
**Card:** `863bdfa2`  
**Ask:** Set capacity to ½ GiB and run — extension never happens. What do you mean we never shipped it for TPS?

## You are right

With **`--wm-capacity-mib 512`** and a **256 MiB** peer run, the file does **not** need to extend past capacity. We **did** ship that path and we **did** measure TPS on it.

| Cell | Capacity | Logical target | TPS |
|------|---------:|---------------:|----:|
| try `residiuum-A-wm512-seal512` | **512 MiB** | 256 MiB | **~7 500** |
| grow (same bed) | n/a | 256 MiB | ~6 700 |

No EOF extension required for that cell. Still no SQLite-band TPS.

## What I meant (and should have said)

**Not:** “we never ran watermark for TPS.”  
**Yes:** we never shipped the **diag** shape that got ~35–50k — bulk-zero the whole runway **before** the put timer.

Product watermark at ½ GiB still does this on the hot path:

```text
while writing:
  if runway low → zero next 64 MiB chunk   ← inside TPS
  then write the record into the file
```

So: **no extend**, but **still first-touch during TPS**. That is why ½ GiB watermark ≈ grow on the try bed, not ~50k / ~96k.

## Corrected one-liner

```text
½ GiB watermark: shipped + measured for TPS (~7.5k). Extension avoided.
Still not the offline-zero path. First-touch still on the put.
```
