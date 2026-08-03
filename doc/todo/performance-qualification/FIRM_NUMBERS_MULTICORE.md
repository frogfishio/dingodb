# Firm numbers — Residiuum multicore on the fast-disk bed

Status: **labor measure (self_check) — not package accept / not Scratch PEER**  
Date: 2026-08-03  
Principal: same Scratch→APFS table, but Residiuum **multicore** (defer Static/Adaptive).

## Verdict

**Multicore does not close the Mode A gap to SQLite on APFS.**

| Shape | Result |
|-------|--------|
| Mode A + `COOK_PARALLELISM` 1/2/4 | **Flat ~13k** — parallel cook **never engages** (`put_many` needs `items.len() >= 2`) |
| Mode B batch=128 + cook1 vs cook4 | **≈ flat** (~13.6k → ~13.8k) — cook pool engages, but this long-peer wall is **not** the short-micro cook wall |

```text
SQLite Mode A     Scratch ~9.5k  →  APFS ~29.7k   (~3.1×)
Residiuum-off c1  Scratch ~9.9k  →  APFS ~13.2k   (~1.3×)
Residiuum-off c4  (Scratch n/a)  →  APFS ~13.3k   (≈ c1)
```

Fast disk → CPU wall stays; **extra cook cores are not the unlock** for PEER Mode A
QD=1, and barely move Mode B on this 256 MiB sealed peer.

## Numbers (APFS `/var/tmp`, 8 KiB, 256 MiB, seed 42)

### Mode A (QD=1, batch=1) — the requested “same table + multicore”

| Cell | ops/s | peak CPU | cook N | notes |
|------|------:|---------:|-------:|-------|
| SQLite A | **29 702** | 78.5% | — | control |
| Residiuum A cook1 | **13 192** | 97.7% | 1 | matches FN-2 off band |
| Residiuum A cook2 | **12 915** | 89.1% | 2 | **no lift** |
| Residiuum A cook4 | **13 258** | 87.5% | 4 | **no lift** |

Why flat: `Store::put_many` parallel path requires `workers > 1 && items.len() >= 2`.
Mode A presents one key per `put_many` → always serial cook. Setting
`RESIDIUUM_COOK_PARALLELISM=4` is a no-op for Mode A. PARKED already said
“idle single-put stays ~1-core.”

### Mode B (batch=128) — where multicore *can* run

| Cell | ops/s | peak CPU | cook N |
|------|------:|---------:|-------:|
| SQLite B (txn-128) | **50 001** | 68.4% | — |
| Residiuum B cook1 | **13 627** | 92.5% | 1 |
| Residiuum B cook4 | **13 781** | 93.0% | 4 |

cook4 / cook1 ≈ **1.01×** on this long peer. Contrast PARKED Scratch **short micro**
cook1→cook4 ~**1.8×** (~330k class) — different bed (short, cache-friendly). Here
seals + append/index still dominate; more Blake workers don’t buy SQLite’s ~50k.

Scratch history Mode B Residiuum (cook1 default) was also ~10k class — same
“batch≠SQLite thr” story.

## Reading

1. **CPU wall (off vs SQLite)** remains the Mode A story on fast disk.
2. **Multicore cook** is the wrong lever for Mode A QD=1 (no batch depth).
3. Even with Mode B depth, on this APFS long peer multicore ≈ noise — next wall
   is not “add cores to Mode A.”
4. Short-micro parallel-cook wins stay labeled micro (`PARKED-write-path-wall`).

## Recipe

```bash
cargo build -p residiuum-testrig --release
BIN=target/release/residiuum-testrig
WORK=/var/tmp/residiuum-peer-mc
for N in 1 2 4; do
  RESIDIUUM_COOK_PARALLELISM=$N $BIN peer-pump -w "$WORK/A-c$N" \
    --engine residiuum --mode A --awo-mode disabled \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 --json-out
done
# Mode B (optional clarity):
RESIDIUUM_COOK_PARALLELISM=4 $BIN peer-pump -w "$WORK/B-c4" \
  --engine residiuum --mode B --awo-mode disabled \
  --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 --json-out
```

Artifacts: `artifacts/firm-numbers-multicore-apfs/`.
Harness: peer-pump JSON now reports `cook_parallelism`.

## Non-claims

Not Scratch re-run. Not AWO. Not package accept. Not “cores never help” — they
help on short batch-rich micros; not on this PEER Mode A / long-peer B shape.
