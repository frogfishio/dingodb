# Measure AWO three-way — T6 interactive re-run gate

Status: **labor complete (self_check) — measurement slice stop met**  
Card: `aff93378-bf7d-44e6-a276-b6d6c7d8ff0d`  
Date: 2026-08-02  

Principal provided external volume: Samsung T3 mounted at **`/Volumes/Scratch`**  
(~**139 GiB free**). Path note: principal text said `Temp`; on-disk dir is **`TEST`**.

---

## 1. T1–T5 present?

| Item | Present |
|------|---------|
| T1 mode flag (`--awo-mode`) | Yes |
| T2 runbook | `AWO_THREE_WAY_MEASURE_RUNBOOK.md` |
| T3 correctness smoke | Yes |
| T4 disk-safe artifacts | `artifacts/awo-three-way-t4-disksafe/` |
| T5 honesty | `AWO_THREE_WAY_T5_HONESTY.md` |

---

## 2. Interactive re-run commands used

```bash
# Volume (exFAT). Do NOT use internal ~30 GiB free for diagnostic.
ROOT=/Volumes/Scratch/TEST/residiuum-awo-three-way
PERF=target/release/residiuum-perf   # cargo build -p residiuum-perf --features store-driver --release
SEED=42
CLASS=smoke          # see §4 residual for diagnostic
MAX_CELLS=1

for MODE in disabled static adaptive; do
  W=$ROOT/t6-smoke/$MODE
  mkdir -p "$W"
  $PERF run --work "$W" --driver real_store --seed $SEED \
    --class $CLASS --max-cells $MAX_CELLS --awo-mode $MODE --no-spawn-workers
  rm -rf "$W/stores"   # free space; keep campaign JSON if desired
done
```

---

## 3. Live readout (smoke on Scratch)

Primary cell after seed=42 counterbalance: **`L4-durable-s16384-c1-o8-43`**  
(16 KiB payload · Durable · smoke op-cap ~24 ack)

| Mode | valid+reopen | thr proxy MiB/s (med, n=6) | e2e proxy ms (med) |
|------|--------------|----------------------------|--------------------|
| disabled | yes | ~3.67 | ~102 |
| static | yes | ~8.69 | ~43 |
| adaptive | yes | ~10.01 | ~37 |

Machine summary: `artifacts/awo-three-way-t6-scratch-smoke/summary.json`

**Honest read:** On external SSD + smoke scale, lease modes look faster than disabled
in this micro-cell. Still **smoke proxy** — not diagnostic floors, not product ranking
(see T5). Static ≈ adaptive order of magnitude (adaptive slightly higher on this slice).

Internal-disk T4 smoke (earlier) was much closer (~3.5–3.9 all modes) — host/FS
noise is large at smoke scale.

---

## 4. Diagnostic attempt residual (important)

Attempted **`class=diagnostic` `max_cells=1`** on `/Volumes/Scratch/TEST` for all
three modes. **All failed:**

```text
error: campaign: matrix: invalid_correctness: reopen digest mismatch
```

| Mode | Result |
|------|--------|
| disabled | FAIL reopen digest |
| static | FAIL reopen digest |
| adaptive | FAIL reopen digest |

**FS:** Scratch is **exFAT**. Smoke reopens OK; diagnostic (2 GiB floors, durable
sync path) fails reopen digest verification on this volume. Treat as **host/FS
residual**, not “AWO broken” — even **disabled** fails the same way.

**Do not** claim diagnostic numbers from Scratch until reopen is fixed or a
POSIX volume is used (APFS/HFS+/ext4).

---

## 5. Stop condition

| Criterion | Met? |
|-----------|------|
| Can re-run Off/Static/Adaptive with documented commands | **Yes** (smoke on Scratch or internal) |
| Live numbers captured | **Yes** |
| No new code path invented | **Yes** |
| Feature measurement slice stop | **Yes** for interactive smoke re-run |

**Out of this Feature:** default-on, formal stamps, full T2 diagnostic matrix,
qualification host, fixing exFAT reopen digest (separate residual if desired).

---

## 6. Host notes / flaky

| Issue | Note |
|-------|------|
| Path name | `Temp` vs **`TEST`** |
| Internal free | ~30 GiB — too small for diagnostic |
| Scratch free | ~139 GiB — enough space; **exFAT reopen residual** |
| ifree 100% on exFAT df | common for exFAT; use Size/Avail not iused |

---

## Artifacts

| Path | Role |
|------|------|
| `artifacts/awo-three-way-t6-scratch-smoke/` | Successful interactive smoke re-run |
| `artifacts/awo-three-way-t6-scratch-diag/` | Failed diagnostic attempt logs |
| Scratch work root | `/Volumes/Scratch/TEST/residiuum-awo-three-way/` |
