# What happened — disk prealloc / watermark (TPS only)

**Date:** 2026-08-03  
**Card:** `3ab0fd49`  
**Ask:** “so what has happened. We added prealloc of the disk and it made things worse?”

## What happened

1. We saw grow-on-append was the TPS wall (~12–14k quiet vs SQLite ~25–30k).  
2. Diag: pre-touch / bulk-zero the file **before** puts could print high TPS.  
3. We shipped **opt-in** watermark (reserve + zero runway). **Default is still grow** — not flipped on.  
4. Honest TPS try: watermark **≈ grow** (~same TPS), **more disk**.  
5. Labor also left test stores on disk (you cleaned). That made some runs look worse.

## Did it make things worse?

| Question | Answer |
|----------|--------|
| Did default TPS get slower? | **No** — default still grow |
| Did opt-in prealloc raise TPS? | **No** — failed as a TPS play |
| Did it use more disk when on? | **Yes** |
| Was the initiative a win for TPS? | **No** |

So: we **added** a prealloc option; it **did not help TPS**; when used it costs space. That is “worse” as a speed play. We did **not** make the default product slower by turning it on for everyone.

## TPS now (still)

| | TPS |
|--|----:|
| Residiuum default (quiet, clean disk) | **~12–14k** |
| Residiuum + watermark (tried) | **≈ default** |
| SQLite | **~25–30k** |

See [TPS_ONLY.md](TPS_ONLY.md), [WATERMARK_MADE_TPS_WORSE.md](WATERMARK_MADE_TPS_WORSE.md), [OWN_DISK_FILL_CLEANUP.md](OWN_DISK_FILL_CLEANUP.md).
