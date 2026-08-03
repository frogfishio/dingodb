# Firm numbers FN-2 — Mode A four-cell odometer

Status: **labor measure (self_check) — not package accept / not Scratch PEER accept**  
Date: 2026-08-03  
Depends on: FN-0 goals (`FIRM_NUMBERS_GOALS.md`); FN-1 peer-pump `--awo-mode`

## 1. What we ran

PEER Mode A knobs, **acked puts/s** odometer, four cells:

| Cell | Engine | Shape |
|------|--------|--------|
| SQLite A | sqlite | autocommit / row |
| Residiuum-off | residiuum | Buffered, batch=1, `awo_mode=disabled` |
| Residiuum Static | residiuum | Buffered, batch=1, `awo_mode=static` |
| Residiuum Adaptive | residiuum | Buffered, batch=1, `awo_mode=adaptive` |

**Bed this run:** `/var/tmp` on host APFS (Scratch `/Volumes/Scratch` **not mounted**).
Same logical knobs as PEER-SQL v1: 8 KiB, QD=1, 256 MiB logical, seed 42,
seal 64 MiB. **Not** a Scratch-reportable PEER campaign — re-run on Scratch when
available for peer-ratio continuity with 2026-08-01 ~10k cells.

**Harness (FN-1):** `residiuum-testrig peer-pump --awo-mode disabled|static|adaptive`.
Mode A + lease uses `independent_admit_put+collection` (QD=1 wait per put),
matching PQH batch=1 AWO path.

Artifacts: `artifacts/firm-numbers-fn2-mode-a-apfs/`.

## 2. Numbers (acked puts/s)

| Cell | acked puts/s | elapsed | path |
|------|-------------:|--------:|------|
| **SQLite A** | **~29 200** | 1.1 s | sqlite insert |
| **Residiuum-off** | **~12 600** | 2.6 s | `put_many` |
| **Residiuum Static** | **~2 460** | 13.3 s | `independent_admit_put+collection` |
| **Residiuum Adaptive (X)** | **~2 470** | 13.3 s | `independent_admit_put+collection` |

```text
X_smart_ModeA  ≈  2470 acked puts/s   (this APFS bed)
```

## 3. One-sentence verdict

On this Mode A QD=1 bed, **Adaptive matches Static (~2.5k) and loses to
Residiuum-off (~12.5k) and SQLite (~29k)** — collection delay with no outstanding
pile-up taxes every put; Adaptive does not beat the off baseline here.

## 4. Compared to prior Scratch Mode A (~10k)

Scratch 2026-08-01: Residiuum-A ≈9924, SQLite-A ≈9458 (parity). This APFS
`/var/tmp` bed is **not** that bed (SQLite ~3× faster here; Residiuum-off in a
similar absolute band). Do **not** publish APFS/SQLite ratios as Scratch PEER
ratios. The Adaptive vs Residiuum-off **gap direction** (Adaptive slower under
QD=1 collection) is the firm Mode A smart-mode signal.

**Why SQLite ~10k → ~30k:** bed swap only — Scratch exFAT (Samsung T3) →
internal APFS `/var/tmp`. See [SQLITE_10K_TO_30K.md](SQLITE_10K_TO_30K.md).
SQLite peak CPU 16% → 78%; Residiuum only ~1.26× on the same move.

## 5. Optimize bound (feeds FN-3)

1. **Do not tune Adaptive for Mode A QD=1 wins** until the collection path stops
   charging full `maximum_collection_delay` when outstanding cannot pile up.
2. Named residual: QD=1 + bound collector → delay tax; T11 amortization needs
   outstanding pile-up (different bed).
3. Interim honest max on this shape: Residiuum-off ~12.5k (this host) / ~10k
   (Scratch history) — Adaptive is **worse**, not headroom.
4. **CPU wall (principal confirm):** Scratch 10k was not “our disk max” — SQLite
   was disk-waiting while we were already ~CPU-bound; APFS shows ~12.5k still
   CPU-hot vs SQLite ~29k. See [FAST_DISK_CPU_WALL.md](FAST_DISK_CPU_WALL.md).
   Next beat-SQLite work is per-put CPU (cook/frame), not hoping the disk was us.

## 6. Recipe

```bash
cargo build -p residiuum-testrig --release
BIN=target/release/residiuum-testrig
# Prefer Scratch when mounted: SCRATCH=/Volumes/Scratch/TEST
WORK=/var/tmp/residiuum-peer-fn2   # this labor used /var/tmp
for CELL in disabled static adaptive; do
  $BIN peer-pump -w "$WORK/ra-$CELL" --engine residiuum --mode A \
    --awo-mode $CELL --target-bytes 256M --payload-size 8192 --seed 42 \
    --min-free 0 --json-out
done
$BIN peer-pump -w "$WORK/sa" --engine sqlite --mode A \
  --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 --json-out
```

## 7. Non-claims

Not AWO package accept, default-on, PQH floors, or published SLO.
Not Scratch PEER-SQL T3 accept (volume was local APFS).
