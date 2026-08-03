# Remeasure: DevNull write-ready TPS

**Date:** 2026-08-03  
**Card:** `9c832c50`  
**Ask:** How many transactions get ready for write all the way to actual writing — i.e. through cook + `write_all` → `/dev/null`, before durable media?

**Bed:** `/tmp` APFS · Mode A · c=8 · 8 KiB · 256 MiB · seed 42 · seal 512 MiB · `min-free 0`  
**Artifacts:** [`artifacts/remeasure-devnull-apfs/`](artifacts/remeasure-devnull-apfs/)  
Work dirs wiped every cell.

## Ladder (TPS = peer-pump `ops_per_sec`)

| Cell | Meaning | TPS |
|------|---------|----:|
| **discard** | cook+index; **no** `write_all` | **~130 155** |
| **devnull** | cook+index + `write_all` → `/dev/null` | **~124 112** |
| **realoverwrite** | `write_all` into a real fd that does **not** grow | **~106 647** |
| **real** (grow) | durable append / file growth | **~10 014** |

## Answer

You can get about **~124k** transactions/sec ready through the path **up to and including `write_all`**, when the write goes to `/dev/null` (no durable media).

That is essentially the same band as Discard (~130k): the write syscall itself is cheap. Dropping to ~10k on Real is still the **append/growth** wall (same story as [WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md); prior DevNull was ~125k).

```text
~130k  discard     ← ready, no write
~124k  /dev/null   ← ready through write_all (this ask)
~107k  overwrite   ← real fd, no growth
 ~10k  real        ← durable append
```

Not product TPS. Diagnostic only.
