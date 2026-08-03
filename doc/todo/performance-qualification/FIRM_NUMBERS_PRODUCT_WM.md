# Firm numbers — product `--segment-growth watermark` (paired)

**Date:** 2026-08-03  
**Card:** `eaab6a4d` — Paired product-flag Real vs watermark vs SQLite (c=8)  
**Status:** labor `in_review` — **not** package accept / **not** default-on  

## Recipe

Mode A · `--concurrency 8` · 8 KiB · 256 MiB logical · `--seal-threshold 512M` · APFS `/var/tmp` · seed 42  

Artifacts: [`artifacts/firm-numbers-product-wm-apfs/`](artifacts/firm-numbers-product-wm-apfs/).

## Honest paired cells (post seal/prealloc fix)

| Cell | ops/s | elapsed_ms | bytes_on_disk | Notes |
|------|------:|-----------:|--------------:|-------|
| SQLite A | ~13 400 | 2438 | ~273 MiB | Noisy vs earlier ~26–30k on fuller disk |
| Residiuum grow (default) | ~7 700 | 4260 | ~522 MiB | Product Real |
| Residiuum `--segment-growth watermark` | ~6 200 | 5284 | ~1.0 GiB | Product flag |
| Residiuum `--diag-io realpreallocwm` | ~5 500 | 5960 | ~1.0 GiB | Diag sink (fixed) |

Sealed segment sizes after fix: **~275 MiB** for grow / watermark / diagwm (no longer renaming a 512 MiB prealloc tail).

## Correction — prior diag ~32k was a seal-fail cheat

Earlier [`PREALLOC_WATERMARK_SPIKE.md`](PREALLOC_WATERMARK_SPIKE.md) `realpreallocwm` ~32k pump ops/s came with:

1. **Diag setup zeroed from file offset 0**, clobbering the on-disk segment descriptor after create.  
2. End-of-run `seal_active()` then failed (`CorruptMeta("pending segment empty or unreadable")`).  
3. Peer-pump **ignores** seal errors (`let _ = store.seal_active()`), so the timer **skipped** seal + chimera cost.

Product `--segment-growth watermark` sealed correctly and looked “slow” by comparison. That gap was **not** proof the product path failed to apply first-touch amortization during puts.

## Fixes landed this card

| Fix | Where |
|-----|--------|
| Seal publish truncates active to durable prefix before writing summary (prealloc-safe) | `store.rs` `seal_active_shard` |
| Diag `RealPreallocWatermark` post-create setup zeros **ahead of** `durable_len` only | `store.rs` `prealloc_existing_actives` |
| Regression: diag + product watermark seal succeed; sealed size ≪ 512 MiB | `tests/wm_seal_probe.rs` |

## Verdict

```text
On this bed, honest end-to-end Mode A c=8:
  SQLite ≳ Residiuum grow ≳ product watermark ≈ fixed diag wm
Prior diag ~32k must not be cited as product watermark thr.
Watermark still reserves ~512 MiB/active (space amp); not default-on.
```

## Non-claims

Not that watermark beats SQLite. Not default-on. Not Scratch/Linux. Not that ~13k SQLite here replaces the prior ~30k band (disk was ~93% full / noisy). Not AWO / PQH accept.

## Next

- Host with headroom: re-pair grow vs watermark vs SQLite for a cleaner thr table  
- Sticky config / default-on only after principal + CSQ/space disclosure  
- Optional: exclude end-of-run seal from peer-pump odometer when measuring put-path only (disclose!)
