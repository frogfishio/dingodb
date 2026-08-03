# Answer — did watermark make TPS worse?

**Date:** 2026-08-03  
**Card:** `fd677990`  
**Ask:** “in fact this made things worse didn’t it?”  
**Metric:** TPS only ([TPS_ONLY.md](TPS_ONLY.md)).

## Short answer

**For TPS: yes — it failed.** Opt-in watermark did **not** raise TPS vs default grow. On the try bed it was flat (~same TPS) while using **more disk**. So as a TPS move it made the product story worse (extra complexity / space for no speed).

**What it did *not* do:** change the **default**. Default is still grow. Turning nothing on does not lower your TPS.

## TPS evidence (try bed)

| Path | TPS | Disk |
|------|----:|-----:|
| Grow (default) | **~6 700** | ~522 MiB |
| Watermark 64 MiB | **~6 500–6 800** | ~586 MiB |
| Watermark 512 MiB | **~7 500** | ~1.0 GiB |
| SQLite | **~29 000** | ~273 MiB |

Source: [TRY_WM_64MIB.md](TRY_WM_64MIB.md).

```text
TPS win?     No (64 MiB ≈ grow; 512 MiB tiny lift, still ≪ SQLite)
Default hurt? No (still grow unless you opt in)
Worth it for TPS?  Not on this evidence
```

## Also worse (not TPS, but real)

The **explanation pile** made you lose the plot. That is on labor. Locked now: answer with TPS first only.

## Next (only if TPS rises)

Judge the next lever (background preparer) the same way: **did TPS go up on the same peer recipe?** If not, it failed too.
