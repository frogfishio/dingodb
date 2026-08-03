# Same-fd full-capacity zero (principal prealloc model)

**Date:** 2026-08-03  
**Card:** `04aeabd5`  
**Artifacts:** `artifacts/samefd-fullzero-apfs/`

## What shipped

Principal model on the product watermark path:

1. **Zero N MiB on the writer fd** at create / policy-set / rotate (`prepare_active_file` zeros full `capacity_bytes`; `apply_product_growth` same-fd full zero).
2. **Write forward** at advancing `durable_len`.
3. **Seal → next segment** also full-zeroed on its writer fd before puts resume.
4. Puts still **fail closed** if runway empty; **no put-path bulk-zero**.
5. `warm_segment_runway` now same-fd (does not rely on the preparer’s separate open for first-touch).
6. Default remains **grow-on-append** (not flipped on).

## TPS (only meter)

Bed: `/tmp` APFS Data (~27 Gi free). Mode A · c=8 · 8 KiB · 256 MiB · seed 42 · `min-free 0`. Work dirs `rm -rf`’d every cell.

| Cell | TPS (`ops_per_sec`) | Notes |
|------|--------------------:|-------|
| Residiuum grow · seal 512 MiB | **~8 641** | paired baseline |
| Residiuum wm 512/64 same-fd full zero · seal 512 MiB | **~9 435** | small lift vs grow-512 |
| Residiuum grow · seal 1 GiB | **~10 701** | best Residiuum this bed |
| Residiuum wm 1024/64 same-fd · seal 1 GiB | **~8 151** | worse than grow-1g |
| SQLite A | **~29 409** | same recipe |

## Verdict

Shape matches the agreed prealloc model (zero → write forward → seal → next).  
**TPS is not clearly ≫ grow** on this peer recipe (one cell edged grow-512; larger capacity lost to grow-1g). Per principal lock: **do not flip default-on**. Offline-diag ~35–50k remains a separate bed/sink story — not claimed as product thr here.

## Residual

- End-of-run `seal_active` still inside peer wall time.  
- Watermark disk amp large (`bytes_on_disk` ≈ reserved + sealed).  
- Next thr lever is elsewhere (or quieter bed / disclosed put-only meter) — not another put-path zero.
