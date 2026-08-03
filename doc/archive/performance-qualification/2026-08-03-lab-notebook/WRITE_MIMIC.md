# Write-mimic — raw disk ceiling for peer write sizes

**Date:** 2026-08-03  
**Card:** `d14fc3ee`  
**Tool:** `residiuum-testrig write-mimic`  
**Artifacts:** `artifacts/write-mimic-apfs/`

## Why

Peer coalesce ~10 936 TPS (~85 MiB/s logical) is not a great speed. Before more store surgery: **how fast can this bed write if we only do the OS I/O pattern** (no cook/Blake/indexes logic)?

## Pattern (calibrated to Mode A peer)

| Piece | Source | Mimic |
|-------|--------|-------|
| Ops | 256 MiB / 8 KiB | **32 768** |
| Data write | skip-index store ÷ ops | **8 440 B** `seek`+`write_all` grow per op |
| Hot-path index publish | boundary probe ~11 ms / 32k | **in-memory** — not a per-put disk write |
| Derived index checkpoint | `DERIVED_CHECKPOINT_EVERY_OPS=65536` | none mid-run for 32k ops |
| End seal / full−skip disk delta | 547 MiB − 277 MiB | optional **one atomic rewrite ~270 MiB** (`tmp`+`sync`+`rename`) |
| Hypothesis: index as append log | locator-sized | **64 B** append per op (no fsync) |

Modes: `data` | `append` | `atomic`.

## Rates (`/tmp` APFS, cleaned every cell)

| Mode | ops/s (put-shaped) | data MiB/s | total MiB/s | Notes |
|------|-------------------:|-----------:|------------:|-------|
| **data-only** | **~128 602** | **~1035** | ~1035 | N × 8440 B append |
| **data+index-append** | **~135 727** | ~1093 | ~1101 | +64 B index append/op |
| **data+index-atomic** (end 270 MiB) | **~76 919** | ~619 | ~1225 | end atomic dominates wall |
| **atomic-hot** (no end blob) | **~204 680** | ~1648 | ~1648 | data path only |

## vs peer

| Cell | ops/s | logical MiB/s |
|------|------:|--------------:|
| Peer Real | ~8.9–9.6k | ~70–75 |
| Peer Coalesce100k | ~10.9k | ~85 |
| Peer Discard | ~128k | (no real write) |
| **Write-mimic data-only** | **~129k** | **~1035 data MiB/s** |

## Read

1. **Same-sized data `write_all`s alone** run at **~129k put-shaped ops/s** / **~1.0 GiB/s** on this bed — roughly **Discard’s band**, ~12× peer Real.  
2. Adding tiny per-op index appends does **not** slow it.  
3. A single end-of-run **270 MiB atomic+fsync** rewrite hurts wall (~77k ops/s equivalent) but is still ≫ peer TPS.  
4. So peer ~10k / ~85 MiB/s is **not** “the disk cannot accept this write shape.” The gap is **inside Residiuum** (cook, encoding, growth/first-touch, sealing, lock, index side effects — already partially bisected).

**Locked read:** [SEE_THE_PROBLEM.md](SEE_THE_PROBLEM.md) — disk fine; put path ~12× slower than same-sized writes.

**Faithfulness:** [WRITE_MIMIC_FAITHFULNESS.md](WRITE_MIMIC_FAITHFULNESS.md) — not a true put/transaction model; disk ceiling only.

## How to re-run

```sh
BIN=target/release/residiuum-testrig
ART=doc/todo/performance-qualification/artifacts/write-mimic-apfs
for m in data append atomic; do
  w=/tmp/wmimic-$$-$m; rm -rf "$w"; mkdir -p "$w"
  $BIN write-mimic -w "$w" --mode $m --json-out > "$ART/$m.json"
  rm -rf "$w"
done
```

Diagnostic experiment only — not a product SLO.
