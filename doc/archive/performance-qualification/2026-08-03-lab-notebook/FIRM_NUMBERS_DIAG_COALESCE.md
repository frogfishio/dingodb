# Spike: 64 KiB / 250 ms segment-tail coalesce

**Date:** 2026-08-03  
**Bed:** APFS `/var/tmp` · Mode A · Residiuum-off · `--concurrency 8` · 8 KiB · 256 MiB · seed 42 · seal 512 MiB  
**Question:** Does buffering bottom-end disk writes (64 KiB fill or 250 ms max age) change hammerblast throughput?

## Spike

`DiagnosticIoSink::Coalesce64k` — real active-file `write_all`, but coalesced into ≥64 KiB chunks (or flush after 250 ms). Wired as `peer-pump --diag-io coalesce64k` (requires `--concurrency > 1`; QD=1 would hang on the 250 ms floor).

Control: `--diag-io discard` (full cook/index path, **no** `write_all`).

## Numbers

| `diag_io` | ops/s | elapsed | on-disk |
|-----------|------:|--------:|--------:|
| **real** | **9 936** | 3.30 s | ~522 MiB |
| **coalesce64k** | **10 873** | 3.01 s | ~522 MiB |
| **discard** | **128 738** | 0.25 s | ~0 |

Artifacts: [`artifacts/firm-numbers-diag-coalesce-apfs/`](artifacts/firm-numbers-diag-coalesce-apfs/).

## Verdict

1. **Coalesce ≈ Real** (~10% — noise band). Larger bottom-end write chunks do **not** move the odometer. This is **not** evidence for an “adaptive disk pager” that waits to fill 64 KiB pages.
2. **Discard ≫ Real (~13×)**. Detaching segment-tail `write_all` (even Buffered / page-cache) unlocks ~129k ops/s. So the wall **does** include the OS write path — but it is **not** cured by coalescing write *size*.
3. Principal hope (“cached writes prove it is not the disk”) is only half-right: write **shape** is not the story; **doing** `write_all` still dominates vs pure CPU cook. T3≈local similarity can still be both sides hitting the same write-syscall / page-cache cost, not media seek.

## How to re-run

```sh
BIN=target/release/residiuum-testrig
for d in real coalesce64k discard; do
  $BIN peer-pump -w /var/tmp/diag-$d --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out
done
```

Diagnostic only — not a product SLO.
