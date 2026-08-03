# Measure AWO three-way — T7 sparse / saturated (definitive next)

Status: **plan freeze v2 (principal correction) — not run, not package accept**  
Feature: **Measure adaptive write batching (three-way fair run)** `ac713f4d-…`  
Date: 2026-08-03  
Depends on: T1–T6 in_review; T5 honesty; T6 Scratch residual  

Principal thesis (paraphrased): *If Disabled paid one expensive barrier per write
while Static combined multiple writes behind each barrier, the external SSD
result is explicable and validates core batching. Definitive next test: two
shapes on APFS/ext4 — sparse (Adaptive ≈ Disabled latency) and saturated
(Adaptive converges toward Static).*

**v2 correction (principal):** Saturated must **not** use harness `batch_size=N`.
That calls `put_many(N)` for **Disabled** too and measures Residiuum’s existing
batch API — **not AWO**. Requests must be **independent singles**; AWO is what
may *form* batches.

---

## 0. Erratum — what T7 v1 got wrong

| Wrong (v1 §2.3) | Why it fails as an AWO test |
|-----------------|------------------------------|
| Shape B: set harness `batch_size ≥ 8` | Driver flushes `put_many(8)` / `admit_put_batch(8)` for **all** modes |
| Metric: Disabled also gets `file_sync/ops ≈ 1/N` | That is **store `put_many_single_shard_batched`**, already product path without AWO |
| Framing: “Disabled with batch=N” | Misattributes batch API thr as AWO |

**Correct framing:** Compare modes under the **same request presentation**.
Presentation is either sparse singles or saturated **independent** singles.
AWO may coalesce; Disabled must **not** receive a pre-batched slice.

---

## 1. Hypothesis vs T4/T6 evidence (unchanged, still true)

### 1.1 Measured primary cell

Primary cell after seed=42 counterbalance: **`L4-durable-s16384-c1-o8-43`**

| Knob | Value |
|------|--------|
| `batch_size` | **1** |
| `concurrency` | 1 |
| `outstanding` | 8 |
| durability | Durable |
| ops (smoke) | 24 |

| Mode | `boundary_counters` (every primary run) |
|------|------------------------------------------|
| disabled / static / adaptive | `append=24 file_write=24 file_sync=24 publish=24` |

T6 thr gap had **equal barrier counts** — not multi-write amortization.

### 1.2 Two different “batching” layers (do not confuse)

| Layer | What it is | Who gets it |
|-------|------------|-------------|
| **L-API** | Caller passes N items to `put_many` / `admit_put_batch` | Any mode if harness `batch_size=N` |
| **L-AWO** | Optimiser **collects independent single admits** into one install (collection delay, queue, Static/Adaptive plan) | static / adaptive only |

T7 measures **L-AWO**, not L-API.

### 1.3 Code anchors

- Store L-API: `put_many` → `put_many_single_shard_batched` (N appends, one Durable sync).
- AWO single admit today: `admit_put` → **natural** one-shot (`put_subject_bytes_if_awo_owned`) — **no cross-request collection on this floor**.
- AWO multi-item: `admit_put_batch` + `plan_batch_take` (Static = all; Adaptive = Natural k=1 or Batch k≤N) — still **caller-presented** slice, not independent-single collection.
- Spec intent: `collection_delay_ns`, `maximum_collection_delay`, natural vs batch plans (`ADAPTIVE_WRITE_OPTIMISER_SPEC.md`).

**Labor honesty:** Full **independent-single → AWO-formed batch** may require more AWO collection wiring than the current harness path. T7 freezes the *experiment*; implement residual explicitly rather than faking it with `batch_size=8`.

---

## 2. Definitive shapes (v2) — request presentation

### 2.0 Presentation law (always)

```text
Every client request is one independent Durable put (logical batch size 1).
Disabled:  store put / put_many([one]) per request — one barrier per ack (ideal).
Static:    AWO may coalesce concurrent outstanding singles into multi-item installs.
Adaptive:  sparse → behave like natural/Disabled (low latency);
           saturated → form batches, thr/sync-ratio → Static.
```

| Shape | Requests presented | Purpose |
|-------|--------------------|---------|
| **Sparse singles** | One independent `put` at a time (conc=1, outstanding≈1; no pile-up) | Adaptive must **preserve low latency** (no forced collection wait) |
| **Saturated singles** | Many concurrent independent singles (high conc and/or outstanding; each request still size 1) | Adaptive must **converge toward Static** thr / barrier amortization; Disabled stays ~1 sync/op |

**Forbidden for three-way AWO ranking:** harness `batch_size > 1` (pre-batches Disabled).

Optional **control cell** (not AWO ranking): `batch_size=N` three-way — documents L-API baseline only; label `l_api_control`, never as Adaptive win.

### 2.1 Host constraints

| Requirement | Why |
|-------------|-----|
| **APFS or ext4** (not exFAT) | T6 diagnostic reopen failed on Scratch exFAT (all modes) |
| Prefer roomy POSIX volume | Diagnostic floors are heavy |
| Internal APFS ~31 GiB free | smoke / careful one-cell; reserve ≥15 GiB |
| Delete work dirs after each mode | Disk budget |

### 2.2 Shape A — Sparse singles

| Knob | Value |
|------|--------|
| FS | APFS/ext4 |
| Presentation | **independent singles only** (`batch_size=1`) |
| concurrency | **1** |
| outstanding | **1** (no pipeline pile-up) |
| durability | Durable |
| payload | 4 KiB or 16 KiB |
| modes | disabled · static · adaptive |
| class | smoke first; diagnostic if disk allows |
| seed | 42 |

**Pass signals (not product claims):**

| Signal | Expectation |
|--------|-------------|
| `file_sync ≈ ops` | All three (no L-API multi-item) |
| Adaptive latency / e2e | **Near Disabled** — must not sit on collection delay when queue depth is 1 |
| Static | May match Disabled on barrier count; thr may still differ by path cost |

### 2.3 Shape B — Saturated singles (AWO under test)

| Knob | Value |
|------|--------|
| FS | APFS/ext4 |
| Presentation | **independent singles only** (`batch_size=1` always) |
| concurrency | **≥ 4** (many concurrent independent puts) |
| outstanding | **≥ 8** (pile-up so a collector *can* see a queue) |
| durability | Durable |
| payload | 4 KiB or 16 KiB |
| modes | disabled · static · adaptive |
| class | **diagnostic** when disk allows |
| seed | 42 |

**Primary metrics (SoT for “AWO batching works”):**

| Metric | Disabled | Static (if collection works) | Adaptive saturated |
|--------|----------|------------------------------|--------------------|
| Request presentation | singles | singles | singles |
| `file_sync / ops` | **≈ 1** | **≪ 1** (amortized) | **→ Static** (not stuck at 1) |
| thr / sustained | baseline | higher if barriers drop | **converges toward Static** |
| validity + reopen | ok | ok | ok |

**Pass signals:**

1. Disabled under saturated singles still ≈ **1 sync per op** (proves we did not sneak `put_many(N)`).
2. Static forms multi-item installs from singles → `file_sync/ops` drops and thr rises vs Disabled.
3. Adaptive under saturation moves thr + sync-ratio toward Static; under sparse (Shape A) stays near Disabled.

If Static **cannot** drop `file_sync/ops` on independent singles, that is an **AWO collection residual** (product incomplete), not a reason to revive harness `batch_size=8` for ranking.

### 2.4 Harness / product gaps (honest)

| Gap | Impact |
|-----|--------|
| `residiuum-perf` groups work into `batch_size` then `put_many` / `admit_put_batch` | `batch_size>1` poisons Disabled (L-API) |
| Concurrent path still flushes prepared batches of size `batch_size` on main thread | With `batch_size=1`, concurrent workers produce **single-item** flushes — good presentation if AWO collects across flushes |
| `admit_put` is natural one-shot today | May **not** yet coalesce concurrent singles into one barrier |
| `admit_put_batch` + `select_plan` needs multi-item **slice** | Caller-presented batch ≠ independent-single collection |
| Spec `collection_delay` / queue | May need AWO labor before Shape B can pass |

**Labor options (ranked for AWO fairness):**

| Option | What | AWO-fair? |
|--------|------|-----------|
| **S1** Pin cells: `batch_size=1`, vary conc/outstanding only | Matches v2 presentation | Yes for presentation; pass may fail until collection works |
| **S2** Harness: concurrent single `admit_put` / `put` with optional AWO collector hook | Best product shape | Needs collection implement or wire |
| **S3** Control only: `batch_size=N` three-way | L-API baseline | **No** for AWO ranking — label control |
| **S4** ~~Shape B via batch_size=8~~ | — | **Rejected (v2)** |

**Recommended path:** **S1** smoke on APFS (prove presentation + Disabled sync≈ops) → implement/wire **collection of independent singles** if Static does not amortize → re-run Shape B diagnostic.

---

## 3. Copy-paste skeleton (presentation-correct)

```bash
# POSIX only. Never set harness batch_size>1 for AWO ranking cells.
ROOT=/path/to/posix-volume/residiuum-awo-t7
PERF=target/release/residiuum-perf
SEED=42
CLASS=smoke   # diagnostic when disk allows

# Sparse:  conc=1 out=1 batch=1  (matrix pin when available)
# Saturated: conc>=4 out>=8 batch=1  — NOT batch=8

for SHAPE in sparse saturated; do
  for MODE in disabled static adaptive; do
    W=$ROOT/$SHAPE/$MODE
    mkdir -p "$W"
    # TODO: cell pin — batch_size must remain 1; only conc/outstanding differ
    $PERF run --work "$W" --driver real_store --seed $SEED \
      --class $CLASS --max-cells 1 --awo-mode $MODE --no-spawn-workers
    # Assert messages: batch=1; boundary file_sync vs ops
    rm -rf "$W/stores"
  done
done
```

Artifact root (when run):  
`doc/todo/performance-qualification/artifacts/awo-three-way-t7-sparse-saturated/`

---

## 4. Claim table after T7 (preview)

| Claim | After fair pass |
|-------|-----------------|
| Disabled under independent singles ≈ 1 Durable sync/op | Required baseline |
| Static amortizes barriers by **collecting singles** (not by receiving put_many(N)) | AWO Static validation |
| Adaptive: sparse latency ≈ Disabled; saturated → Static | AWO Adaptive validation |
| L-API put_many(N) thr for all modes | Control only — **not** AWO claim |
| T6 Scratch thr explained by barrier count | **Still false** on batch=1 evidence |
| Product default-on / qualification | **No** |

---

## 5. Stop condition

| Done when | Status |
|-----------|--------|
| T6 counters documented | Yes |
| v1 batch_size=N saturated **rejected** | **v2 this doc** |
| Sparse singles + saturated singles frozen | **v2 this doc** |
| Campaigns on APFS/ext4 | **Smoke done (T8)** — see `AWO_THREE_WAY_T8_SINGLES_RUN.md` |
| AWO single-collection if missing | **Open residual** — T8: Static/Adaptive still `file_sync/ops=1` under saturated singles |
| Package accept | Never from labor alone |

---

## 6. Related

| Path | Role |
|------|------|
| `AWO_THREE_WAY_T5_HONESTY.md` | Smoke non-claims |
| `AWO_THREE_WAY_T6_INTERACTIVE.md` | Scratch thr + exFAT |
| `artifacts/awo-three-way-t6-scratch-smoke/` | Equal `file_sync=24` |
| `ADAPTIVE_WRITE_OPTIMISER_SPEC.md` | natural vs batch, collection delay |
| `store.rs` `put_many_single_shard_batched` | L-API batching |
| `adaptive_write/runtime.rs` | `admit_put` natural; `admit_put_batch` / `plan_batch_take` |