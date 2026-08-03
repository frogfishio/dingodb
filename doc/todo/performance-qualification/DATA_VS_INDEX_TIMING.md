# Data write vs index — separate timing

**Date:** 2026-08-03  
**Card:** `f5a5ecb9`  
**Ask:** Time the exact time to write **data** and **indexes** separately.

**Bed:** `/tmp` APFS · Mode A · c=8 · 8 KiB · 256 MiB · seed 42 · seal 512 MiB  
**Artifacts:** [`artifacts/data-vs-index-apfs/`](artifacts/data-vs-index-apfs/)  
Work dirs wiped every cell.

## How we timed

1. **Boundary probe** (exclusive step timers on the store path):  
   - **Data write** = `FileWrite` (`write_all` of segment tail)  
   - **Index** = `PublishVisibility` (dual-index publish) + `PutPost` (collection/derived)  
2. **Subtractive TPS:** full Real vs `--diag-skip-index` (skips dual-index + derived checkpoints).

Peer flags added: `--boundary-probe`, `--diag-skip-index`.

## Exact probe sums (before end-of-run seal)

| Cell | TPS | FileWrite (data) | Publish (index) | Encode | Append |
|------|----:|-----------------:|----------------:|-------:|-------:|
| Real full | **~9 556** | **172.6 ms** | **11.5 ms** | 22.0 ms | 156.6 ms |
| Real skip-index | **~29 954** | 153.0 ms | ~0.6 ms | 18.8 ms | 148.6 ms |
| Discard full | **~115 447** | 0.6 ms | 10.3 ms | 19.8 ms | 154.7 ms |
| Discard skip-index | **~126 328** | 0.6 ms | 0.6 ms | 19.2 ms | 149.0 ms |

Wall for Real full was **3428 ms** (includes end `seal_active`). Probe step sums ≪ wall — seal / lock / scheduling dominate residual.

## Read

- **In-put-path index publish is small** (~11 ms of wall) vs **data `write_all`** (~173 ms) when Real.  
- **Skipping index/derived still lifts Real TPS ~9.5k → ~30k** — so derived/index side effects (checkpoints, seal coupling, contention) cost far more than the publish timer alone.  
- Discard stays ~115–126k either way — index is not the Discard ceiling.

## Not product TPS

Diagnostic split only. Default path still does index. Do not cite skip-index ~30k as shippable thr.
