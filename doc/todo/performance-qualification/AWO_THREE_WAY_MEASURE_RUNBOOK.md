# AWO three-way measure runbook (fixed comparison plan)

Status: **diagnostic matrix freeze (T2) — no throughput claims**  
Feature: **Measure adaptive write batching (three-way fair run)**  
Card: `f26d9788-7b82-40b0-b616-59526f824944`  
Date: 2026-08-02  
Profile: `residiuum-performance-qualification-v1`  
Harness: `residiuum-perf` with `--features store-driver`

Depends on T1 labor (`--awo-mode disabled|static|adaptive` on real_store).  
Does **not** implement harness filters. Does **not** run the campaign (T4).  
Does **not** claim product floors or bottleneck verdicts.

---

## 1. Purpose

One **small fixed matrix** a human or agent can re-run **identically** to compare
three write paths under the same seed and cell set:

| Mode | CLI | Meaning |
|------|-----|---------|
| **disabled** | `--awo-mode disabled` | Natural `Store::put_many` (default; product-like off path) |
| **static** | `--awo-mode static` | AWO lease, static batch limits |
| **adaptive** | `--awo-mode adaptive` | AWO lease, adaptive controller |

Every cell is **diagnostic class first**. Qualification floors (`--class qualification`,
120s + 512MiB, controlled host) are **out of scope** for this freeze unless a later
card intentionally promotes a campaign.

---

## 2. Fixed knobs (do not freestyle)

| Knob | Value | Notes |
|------|-------|--------|
| Seed | **`42`** (`0x2a`) | Same seed for all three mode campaigns |
| Class | **`diagnostic`** | Not smoke (smoke forbids honest sustained windows); not qualification |
| Driver | **`real_store`** | Requires `--features store-driver` |
| Platform plan | **`synthetic`** (CLI default) | Lab plan layout only — **not** a product baseline platform |
| Build | `cargo build -p residiuum-perf --features store-driver --release` recommended for T4 |
| Binary | `target/release/residiuum-perf` or `target/debug/…` — **same build for all three modes** |
| `max_cells` | **`64`** | Full matrix today is ~60 cells; 64 avoids truncate-after-shuffle drop |
| Multiproc | default harness plan (do not toggle unless all three modes match) |
| Controlled | **off** | No `--controlled` on this diagnostic freeze |

Work-dir layout (three siblings, wipe between full campaigns if re-running):

```text
$WORK_ROOT/
  disabled/     # --work $WORK_ROOT/disabled
  static/
  adaptive/
```

Campaign artifacts land under `$WORK_ROOT/<mode>/campaign/<campaign_id>/`.

---

## 3. Comparison matrix (logical cells)

**Cartesian product (18 cells):**

- **Payload sizes:** `256`, `4096` (4 KiB), `8192` (8 KiB) bytes  
- **Durability:** `Buffered`, `Durable`  
- **AWO mode:** `disabled`, `static`, `adaptive` (campaign-global; three full runs)

Fixed across all comparison cells (as emitted by the size-sweep leg of
`build_matrix_cells` for those sizes):

| Field | Value |
|-------|--------|
| Layer | L4 |
| concurrency | 1 |
| outstanding | 1 |
| batch_size | 1 |
| shards | 1 |
| db_state | Empty |
| distribution | none (fixed size) |

### How this maps onto today's harness

`residiuum-perf run` does **not** take a payload/durability filter. It:

1. Builds the full PQH matrix (`build_matrix_cells(seed)`), including a **size
   sweep** over the first six fixed sizes (`256 … 64KiB`) ×
   `Memory|Buffered|Durable`.
2. Counterbalances cell order by **seed**.
3. Runs `take(max_cells)` cells with **one** campaign-global `--awo-mode`.

**Fair three-way procedure:** run **three campaigns** that differ **only** in
`--awo-mode` and `--work`. Same seed, class, driver, max_cells, binary.

**Post-hoc comparison set:** from each campaign manifest / cell reports, keep
rows where:

```text
payload_size ∈ {256, 4096, 8192}
durability   ∈ {buffered, durable}   # exclude memory for this freeze
```

That is the 6 physical cells × 3 modes = **18 comparison points**. Other matrix
cells (threshold probes, submission sweep, L5/L6) may run in the same campaign
for free but are **not** part of the T2 freeze set — do not cherry-pick them for
AWO three-way claims.

**Honesty residual:** Memory durability appears in the size sweep; it is
**excluded** from this comparison freeze. Full PQH qualification is a different
program (PQH / AWO-6), not this card.

---

## 4. Build (once)

```bash
# From repo root
cargo build -p residiuum-perf --features store-driver --release
PERF=target/release/residiuum-perf
# Or debug:
# cargo build -p residiuum-perf --features store-driver
# PERF=target/debug/residiuum-perf
```

Confirm store-driver compiled:

```bash
$PERF --help | head -5
# expect: store-driver compiled: true
```

---

## 5. Mode smoke (optional preflight — T1 path)

Functional only; **not** the comparison matrix. Uses smoke class and first
matrix cell after seed=1 counterbalance.

```bash
WORK=/tmp/awo-3way-smoke
rm -rf "$WORK" && mkdir -p "$WORK"
for MODE in disabled static adaptive; do
  $PERF driver-smoke --work "$WORK/$MODE" --driver real_store --awo-mode "$MODE"
done
# Expect validity=valid (or documented smoke validity) and awo_mode= in JSON notes
```

---

## 6. Full matrix commands (copy-paste — T2 freeze)

```bash
# Fixed knobs
SEED=42
CLASS=diagnostic
MAX_CELLS=64
WORK_ROOT="${WORK_ROOT:-/tmp/awo-three-way-seed42}"
PERF="${PERF:-target/release/residiuum-perf}"

rm -rf "$WORK_ROOT"
mkdir -p "$WORK_ROOT"/{disabled,static,adaptive}

for MODE in disabled static adaptive; do
  echo "=== campaign awo_mode=$MODE seed=$SEED class=$CLASS ==="
  $PERF run \
    --work "$WORK_ROOT/$MODE" \
    --driver real_store \
    --seed "$SEED" \
    --class "$CLASS" \
    --max-cells "$MAX_CELLS" \
    --awo-mode "$MODE"
done
```

### One-liners (same knobs)

```bash
PERF=target/release/residiuum-perf
SEED=42
WR=/tmp/awo-three-way-seed42

$PERF run --work $WR/disabled --driver real_store --seed $SEED --class diagnostic --max-cells 64 --awo-mode disabled
$PERF run --work $WR/static   --driver real_store --seed $SEED --class diagnostic --max-cells 64 --awo-mode static
$PERF run --work $WR/adaptive --driver real_store --seed $SEED --class diagnostic --max-cells 64 --awo-mode adaptive
```

### Analyze / verify (after T4 run — not required for T2)

```bash
# campaign_dir is printed in run JSON as campaign_dir
$PERF analyze --campaign "$WR/disabled/campaign/<campaign_id>"
$PERF verify  --campaign "$WR/disabled/campaign/<campaign_id>"
# Repeat for static/adaptive. No cross-mode throughput ranking in T2.
```

---

## 7. Cell identity checklist (for T4 analysis)

When extracting comparison rows, require:

1. Same `seed` (42) and same `cell_id` across the three mode campaigns.  
2. `payload_size` ∈ {256, 4096, 8192}.  
3. `durability` ∈ {buffered, durable}.  
4. Notes / config show matching `awo_mode` for that campaign.  
5. Class remains **diagnostic** — label every number **diagnostic, non-product**.

Do **not** publish absolute MB/s, rank modes, or claim AWO-G8 / qualification.

---

## 8. Non-goals (this card)

| Out of scope | Owner |
|--------------|--------|
| Implementing harness mode attach | T1 (done labor) |
| Correctness smoke before numbers | T3 |
| Actually running the full three-way campaign | T4 |
| Honesty pass on results | T5 |
| Interactive re-run / stop condition | T6 |
| Qualification floors / controlled runner | later PQH / AWO-6 |
| AWO package accept / default-on | AWO-0 / AWO-7 principal |

---

## 9. Done criteria (T2)

- [x] Matrix defined: 256 / 4KiB / 8KiB × Buffered / Durable × disabled|static|adaptive  
- [x] Fixed seed **42**, class **diagnostic**, driver **real_store**, features **store-driver**, max_cells **64**  
- [x] Copy-paste commands for all three modes  
- [x] Every cell labeled diagnostic; no throughput claims  
- [x] Harness filter residual documented (post-hoc cell filter; campaign-global awo_mode)

---

## 10. Next cards

1. **T3** — Correctness smoke before numbers (all three modes ack survivors).  
2. **T4** — First real measurement run using §6 commands.  
3. **T5** — Honesty pass (no overclaim).  
4. **T6** — Interactive re-run gate / stop condition.
