# Background runway preparer — TPS try

**Date:** 2026-08-03  
**Card:** `b223f225`  
**Artifacts:** `artifacts/bg-runway-apfs/`

## What shipped (shape)

- Product watermark **no longer zeros on the put path**.
- Background thread (`wm-runway`) keeps runway ahead of `durable_len`.
- Puts **fail closed** if runway is empty or past capacity.
- Peer-pump calls `warm_segment_runway()` **before** the put timer (first-touch off odometer).
- Default remains **grow-on-append** (not flipped on).

## TPS (only meter)

Bed: `/tmp` on APFS Data volume (~27 Gi free; `/var/tmp` absent on this host).  
Recipe: Mode A · c=8 · 8 KiB · 256 MiB · seed 42 · `min-free 0`. Work dirs `rm -rf`’d every cell.

| Cell | TPS (`ops_per_sec`) | Notes |
|------|--------------------:|-------|
| Residiuum grow · seal 512 MiB | **~9 733** | baseline |
| Residiuum wm 512/64 + bg warm · seal 512 MiB | **~8 898** | no win |
| Residiuum grow · seal 1 GiB | **~9 358** | paired |
| Residiuum wm 1024/64 + bg warm · seal 1 GiB | **~7 696** | worse |
| SQLite A | **~28 590** | same recipe |

## Verdict

**TPS did not rise vs grow** on this peer recipe. Per principal lock: do **not** keep as a thr win; do **not** flip default-on.

Shape stays in-tree (correct product direction: extension off put path). Next thr lever is elsewhere, or a quieter bed + disclosed put-path-only meter — not another put-path zero.

## Residual

- Disk amp under watermark is large (`bytes_on_disk` ≈ reserved capacity + sealed).  
- End-of-run `seal_active` remains inside peer-pump wall time.  
- This bed is not the prior quiet ~12–14k grow floor.
