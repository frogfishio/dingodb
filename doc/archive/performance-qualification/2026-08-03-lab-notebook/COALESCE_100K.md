# Spike: 100 KiB / 250 ms segment-tail coalesce (remeasure)

**Date:** 2026-08-03  
**Card:** `afba9d3a`  
**Artifacts:** `artifacts/coalesce100k-apfs/`

## Spike

`DiagnosticIoSink::Coalesce100k` — real active-file `write_all`, buffered until **≥100 KiB** or **250 ms** max age, then one flush.  
CLI: `peer-pump --diag-io coalesce100k` (aliases: `coalesce`, `coalesce64k`, `100k`, `64k`). Requires `--concurrency > 1` (QD=1 hangs on the 250 ms floor).

Diagnostic only — not product default.

## TPS (only meter)

Bed: `/tmp` APFS (~26 Gi free). Mode A · c=8 · 8 KiB · 256 MiB · seal 512 MiB · seed 42 · `min-free 0`. Work dirs `rm -rf`’d every cell.

| Cell | TPS (`ops_per_sec`) | Notes |
|------|--------------------:|-------|
| Real · grow | **~8 885** | baseline |
| Coalesce100k · grow | **~10 936** | ~+23% vs Real this bed |
| Discard | **~128 583** | cook/index, no `write_all` |
| Real · wm512 same-fd | **~9 707** | paired watermark (diag must stay `real`) |

Prior 64 KiB/250 ms spike was ≈ Real (~10–11k both). This 100 KiB remeasure shows a modest lift on this bed — still the same order as Real, far below Discard.

## Verdict

1. Buffering to **100 KiB or 250 ms** does **not** unlock a new thr band vs grow Real.  
2. **Discard ≫ coalesce** — wall is still doing the OS write path, not cured by chunk size.  
3. Do **not** productize write coalesce / flip default from this spike.

## How to re-run

```sh
BIN=target/release/residiuum-testrig
ART=doc/todo/performance-qualification/artifacts/coalesce100k-apfs
for d in real coalesce100k discard; do
  w=/tmp/residiuum-peer-$$-$d; rm -rf "$w"; mkdir -p "$w"
  $BIN peer-pump -w "$w" --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out \
    > "$ART/$d.json"
  rm -rf "$w"
done
```

## Bytes/sec for ~10 936 TPS

Already in the JSON: **~85.4 logical MiB/s**, **~174 disk MiB/s** (~89.6 / ~183 MB/s SI). See [BYTES_PER_SEC_10936.md](BYTES_PER_SEC_10936.md).
