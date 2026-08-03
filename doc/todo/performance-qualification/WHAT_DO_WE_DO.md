# What do we do? (TPS)

**Date:** 2026-08-03  
**Card:** `60bd9862`  
**Ask:** “so what do we do?”

## Situation (one line)

Grow ≈ **12–14k TPS**. Watermark at ½ GiB avoids EOF extend but still zeros on the **put** → TPS ≈ grow. Offline zero before the timer was ~**35–50k**. Gap to SQLite ~**25–30k**.

## Do this next

**Move first-touch off the put path.**

1. **Implement** the board card already in `todo`: *Background runway preparer — first-touch off put path (TPS)* (`b223f225`).  
   - Background thread keeps runway zeroed ahead of the write head.  
   - Puts only write into ready pages (fail closed if empty).  
   - Use capacity big enough for the run (e.g. 512 MiB for a 256 MiB peer) so EOF does not extend.  
2. **Measure TPS only** on a **clean disk** (rm work dirs every cell): grow vs background-watermark vs SQLite, same recipe.  
3. **Keep** only if TPS clearly beats grow. **Kill** if not.  
4. **Do not** flip default-on until you say so after a TPS win.

## Do not

- More meter essays / component timing as the answer  
- Another watermark that zeros during puts  
- Leave test stores on disk  
- Cite diag ~50k as product TPS

## Success

```text
TPS(background-ready runway) ≫ TPS(grow)   on clean disk
preferably toward SQLite band
```

## Result (2026-08-03)

Shape shipped; **TPS did not win** (~8.9k wm-bg vs ~9.7k grow on `/tmp`). See [BG_RUNWAY_PREPARER.md](BG_RUNWAY_PREPARER.md). Default stays grow.