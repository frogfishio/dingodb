# Three-cell lifecycle attribution (frozen)

Status: **deprioritized** — `product64` measured and yielded the decisive
product number (~12.4K sustainable TPS). Remaining `auth64` / `ceiling`
cells are **not** the priority; next package is Enrichment Throughput
Qualification. AWO paused.  
Date: 2026-08-04  
Depends on: Derived Catalog Checkpointing **package accept**
(`doc/archive/performance-qualification/2026-08-04-derived-catalog-checkpoint/`).  
Product cell evidence:
`doc/archive/performance-qualification/2026-08-04-enrichment-on-2g/`.  
See: [ENRICHMENT_THROUGHPUT_QUALIFICATION.md](./ENRICHMENT_THROUGHPUT_QUALIFICATION.md).

## Purpose

Isolate, on **one frozen binary**, the three residual costs after catalog
checkpointing removed the O(n²) persist defect:

| Cell | Purpose |
|---|---|
| 64 MiB, enrichment **off** | Current authoritative rotation cost |
| Threshold **above** workload, enrichment **off** | Sustained no-rotation ceiling |
| 64 MiB, enrichment **on** | Actual product cost of derived enrichment |

Compare medians to separate:

1. unavoidable authoritative rotation cost;
2. derived CPU / disk / cache interference;
3. any workload-size or thermal decline unrelated to sealing.

## Frozen recipe

```text
Binary:        same release residiuum-testrig (record sha256 once; do not rebuild mid-campaign)
Cell:          real-full
Logical:       2 GiB
Payload:       8 KiB
Concurrency:   8
Seed:          42
AWO:           disabled / not connected
Work root:     delete between every run (no reuse)
Ordering:      alternate cells each rep (not block-all-A then all-B)
Reps:          ≥5 per cell (prefer 6); report median ack TPS
```

### Cell knobs

| Cell id | `--seal-threshold` | Enrichment |
|---|---|---|
| `auth64` | `64M` | `--no-enrichment` |
| `ceiling` | `4G` (≥ workload) | `--no-enrichment` |
| `product64` | `64M` | enrichment **on** (omit `--no-enrichment`) |

## Commands (template)

```bash
BIN=./target/release/residiuum-testrig
EV=doc/archive/performance-qualification/2026-08-04-three-cell-lifecycle
WORK_ROOT=/tmp/residiuum-three-cell-lifecycle
shasum -a 256 "$BIN" | tee "$EV/binary.sha256"

# Example single run — rotate cell/rep; always rm -rf workdir first:
$BIN ack-finalize -w "$WORK" --cell real-full \
  --target-bytes 2G --payload-size 8192 --concurrency 8 --seed 42 \
  --seal-threshold 64M --min-free 512M --no-enrichment --json-out
```

Alternate order example for reps 1…N:

`auth64 → ceiling → product64 → ceiling → auth64 → product64 → …`
(any Latin-square / round-robin that avoids long same-cell blocks).

## Required outputs

Under a new archive dir (create when the campaign runs):

- `binary.sha256`, `uname.txt`, `started-at.txt`, `finished-at.txt`
- `runs/{cell}-r{n}.json` (+ `.err`)
- `summary.json` with per-cell median / min / max ack TPS, sealed counts,
  reopen_exact, enrichment_backlog_at_last_ack
- Short README interpreting the three medians (no optimisation claims)

## Hard freezes

- **Do not** change seal architecture, catalog checkpointing, or AWO while
  this campaign is open.
- **Do not** treat the high-threshold cell as a product SLO — it is a ceiling.
- AWO-Q3 / AWO-Q4 and other AWO optimisation remain **paused** until principal
  re-opens after these three numbers exist.
