# Measure AWO three-way — T7 sparse / saturated (definitive next)

Status: **plan + hypothesis check (labor) — not run, not package accept**  
Feature: **Measure adaptive write batching (three-way fair run)** `ac713f4d-…`  
Date: 2026-08-03  
Depends on: T1–T6 in_review; T5 honesty; T6 Scratch residual  

Principal thesis (paraphrased): *If Disabled paid one expensive barrier per write
while Static combined multiple writes behind each barrier, the external SSD
result is explicable and validates core batching. Definitive next test: two
shapes on APFS/ext4 — sparse (Adaptive ≈ Disabled latency) and saturated
(Adaptive converges toward Static).*

This note **accepts the intent**, **corrects the T6 mechanism claim**, and freezes
the next experiment so labor does not chase smoke MiB/s.

---

## 1. Hypothesis vs T4/T6 evidence (must read first)

### 1.1 What the measured primary cell actually did

Primary cell after seed=42 counterbalance: **`L4-durable-s16384-c1-o8-43`**

From result `messages` (all modes, all 6 reps on Scratch and internal):

| Knob | Value |
|------|--------|
| `batch_size` | **1** |
| `concurrency` | 1 |
| `outstanding` | 8 |
| durability | Durable |
| ops (smoke) | 24 |

| Mode | `boundary_counters` (every primary run) |
|------|------------------------------------------|
| disabled | `append=24 file_write=24 file_sync=24 publish=24` |
| static | `append=24 file_write=24 file_sync=24 publish=24` |
| adaptive | `append=24 file_write=24 file_sync=24 publish=24` |

So on the cell we ranked:

- **Not** “Static combined multiple writes behind each barrier.”
- **Every mode paid one Durable `file_sync` per write** (`file_sync == ops`).
- Core multi-write batching was **not exercised** by this cell.

### 1.2 Where multi-write batching actually lives

In store code, `Store::put_many` and `put_many_awo_owned` share
`put_many_single_shard_batched`: **N in-memory appends → one `write_segment_tail`
(Durable → one `sync_all`)**.

| Path | When batch amortizes barriers |
|------|-------------------------------|
| **disabled** | Harness calls `put_many` with `batch_size = N` → **N:1** sync if N>1 |
| **static** | `admit_put_batch` takes **full** presented slice → same N:1 via `put_many_awo_owned` |
| **adaptive** | `plan_batch_take` → `Natural` takes **1**, `Batch` takes **k ≤ N** |

Implication:

1. **Disabled is not “one barrier per write” in general** — only when each flush
   is a single-item batch (as in T4/T6 primary).
2. **Static does not add a second batching layer** beyond presented `batch_size`;
   it admits the whole slice.
3. **Adaptive’s job** under sparse traffic is to prefer **Natural (k=1)** so it
   does not wait for / force large batches; under saturated traffic, to grow **k**
   toward Static.

### 1.3 What T6 *did* show (honest)

| Observation | Reading |
|-------------|---------|
| Scratch: static/adaptive ~2.4–2.7× thr vs disabled | Real wall-clock difference **with equal `file_sync` counts** |
| Internal T4: modes ~tied (~3.5–3.9) | Same barrier count; host noise dominates |
| Barrier amortization | **Not** the T6 mechanism — open residual (path attach, probe cost, FS latency distribution) |
| Adaptive slightly above static on Scratch | Not product ranking; batch=1 so `select_plan` has nothing to size |

**Do not** claim “core batching validated by T6.”  
**Do** claim “paths correct; thr proxy host-sensitive; barrier math needs batch>1 cells.”

---

## 2. Definitive next test (T7) — two shapes on POSIX

### 2.1 Host constraints

| Requirement | Why |
|-------------|-----|
| **APFS or ext4** (not exFAT) | T6 diagnostic reopen digest failed on Scratch exFAT for **all** modes |
| Prefer external POSIX volume **or** APFS with large free space | Diagnostic floors are heavy |
| Internal APFS today | ~**31 GiB free** — smoke / carefully deleted one-cell work only; reserve ≥15 GiB |
| Delete work dirs after each mode | Disk budget |

Scratch exFAT is **smoke-only** until reopen residual is closed.

### 2.2 Shape A — Sparse (latency / Natural)

**Intent:** Adaptive should **not** pay Static’s large-batch behavior when traffic
is shallow; ideally **Adaptive ≈ Disabled** on per-op latency when both do
single-item Durable flushes.

| Knob | Value |
|------|--------|
| FS | APFS/ext4 |
| Class | Prefer **diagnostic** one-cell if disk allows; else smoke for path-only |
| Modes | disabled · static · adaptive |
| `batch_size` | **1** (explicit sparse presentation) |
| concurrency | 1 |
| outstanding | 1 (stricter than T6’s o8) |
| payload | 16 KiB Durable (match T6 cell family) or 4 KiB Durable |
| Seed | 42 |

**Pass signals (not product claims):**

| Signal | Expectation |
|--------|-------------|
| `file_sync ≈ ops` | All three modes |
| Adaptive e2e / thr | **Near disabled**, not near a large-batch Static |
| Static | Also batch=1 → should also be near disabled on **barrier count**; thr may still differ by path |

**Honesty:** With batch=1, Static cannot “combine writes.” Sparse is mainly a
**regression guard** that Adaptive does not invent delay, and a clean baseline
for Shape B.

**Ideal Adaptive sparse (later, if harness loops partial admits):** present
`batch_size=N` with low arrival rate / cold estimator so Adaptive chooses
**Natural (k=1)** while Static takes N — then Adaptive latency → Disabled and
Static shows N:1 `file_sync`.

### 2.3 Shape B — Saturated (batching / converge)

**Intent:** Validate **core barrier amortization** and Adaptive convergence.

| Knob | Value |
|------|--------|
| FS | APFS/ext4 |
| Class | **diagnostic** (30 s + 2 GiB floors) when disk allows |
| Modes | disabled · static · adaptive |
| `batch_size` | **≥ 8** (prefer 32–128 once harness can pin it) |
| concurrency | ≥ 1 (optional 4 later) |
| outstanding | ≥ 4 |
| payload | 4 KiB or 16 KiB Durable |
| Seed | 42 |

**Primary metrics (SoT for “batching works”):**

| Metric | Disabled (batch=N) | Static | Adaptive (warm/saturated) |
|--------|--------------------|--------|---------------------------|
| `file_sync / ops` | **≈ 1/N** | **≈ 1/N** | **→ 1/N** (not stuck at 1) |
| thr proxy / sustained | high | high | **converges toward Static** |
| validity + reopen | ok | ok | ok |

**Pass signals:**

1. **Core batching:** For Static (and Disabled with same N),  
   `file_sync * N ≈ ops` (within small remainder).  
2. **Adaptive saturated:** thr and `file_sync/ops` move toward Static, not stay
   at Natural (`file_sync ≈ ops`).  
3. **Adaptive sparse (Shape A or Natural branch):** does **not** force large k
   when queue is shallow (if measurable).

**Non-claims:** default-on AWO, product floors, G8 bottleneck, package accept.

### 2.4 Today’s harness gap (do not freestyle)

`residiuum-perf` matrix (scheduler):

- Size / submission L4 legs mostly **`batch_size = 1`**
- Some L5 distribution cells use **`batch_size = 8`**
- No CLI `--batch-size` filter today; campaigns take first `max_cells` after seed shuffle

**Labor options (pick one, document which):**

| Option | Pros | Cons |
|--------|------|------|
| **B1** Post-hoc L5 `batch=8` cells from a larger campaign | No code change | Wrong durability/dist mix; hard to isolate; still not sparse o=1 |
| **B2** Minimal harness: pin/filter cell knobs or `--batch-size` override for real_store | Clean Shape A/B | Small code change; needs card |
| **B3** Unit/integration microbench calling `put_many` / `admit_put_batch` with N | Fast barrier proof | Not PQH campaign-shaped |

**Recommended:** **B2** for definitive three-way; optional **B3** smoke for
`file_sync` ratio before long diagnostic.

**Adaptive partial-take residual:** `admit_put_batch` may return fewer receipts
than presented when Adaptive selects Natural. Harness must **loop remaining
items** or only present what Adaptive should take — otherwise saturated
Adaptive under-acks. Check/fix before trusting Shape B Adaptive numbers.

---

## 3. Copy-paste skeleton (after knobs exist)

```bash
# POSIX only. Re-check: df -h "$ROOT"; diskutil info or findmnt for APFS/ext4.
ROOT=/path/to/posix-volume/residiuum-awo-t7
PERF=target/release/residiuum-perf
SEED=42
# CLASS=diagnostic   # when free space allows; else smoke for path only
CLASS=smoke

# Shape A — sparse (today: batch=1 cells only unless B2 lands)
# Shape B — saturated (requires batch>=8 pin — see §2.4)

for SHAPE in sparse saturated; do
  for MODE in disabled static adaptive; do
    W=$ROOT/$SHAPE/$MODE
    mkdir -p "$W"
    # TODO: pass shape-specific cell pin once B2 exists
    $PERF run --work "$W" --driver real_store --seed $SEED \
      --class $CLASS --max-cells 1 --awo-mode $MODE --no-spawn-workers
    # Extract: boundary_counters file_sync vs ops; thr; reopen
    rm -rf "$W/stores"
  done
done
```

Artifact root (when run):  
`doc/todo/performance-qualification/artifacts/awo-three-way-t7-sparse-saturated/`

---

## 4. Claim table after T7 (preview)

| Claim | After T7 pass |
|-------|----------------|
| N:1 Durable barrier amortization under batch=N (Static + Disabled put_many) | Allowed if `file_sync` math holds |
| Adaptive converges toward Static under saturated presentation | Allowed if thr + sync ratio move together |
| Adaptive preserves sparse/Natural behavior | Allowed only with measurable Natural branch |
| T6 Scratch thr gap explained by barrier count | **Still false** unless new data shows otherwise |
| Product default-on / qualification | **No** |

---

## 5. Stop condition for this card

| Done when | Status |
|-----------|--------|
| Hypothesis check written against T4/T6 counters | **This doc** |
| Sparse + saturated experiment frozen (knobs + metrics) | **This doc** |
| Board residual task staged | See Kanban T7 |
| Campaigns executed on APFS/ext4 | **Open** (disk + harness pin) |
| Package accept | **Never** from labor alone |

---

## 6. Related artifacts

| Path | Role |
|------|------|
| `AWO_THREE_WAY_T5_HONESTY.md` | Smoke non-claims |
| `AWO_THREE_WAY_T6_INTERACTIVE.md` | Scratch thr + exFAT residual |
| `artifacts/awo-three-way-t6-scratch-smoke/` | Equal `file_sync=24` evidence |
| `crates/residiuum-store/src/store.rs` | `put_many_single_shard_batched` |
| `crates/residiuum-store/src/adaptive_write/runtime.rs` | `plan_batch_take` Natural vs Batch |
