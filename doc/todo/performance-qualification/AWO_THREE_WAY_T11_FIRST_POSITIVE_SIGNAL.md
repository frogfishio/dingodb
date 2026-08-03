# Measure AWO three-way — T11 first positive performance signal (freeze)

Status: **principal accept (`done`) — evidence-freeze card only**  
Card: `b7986427-e96a-437d-a443-34c3647d8f1e`  
Feature: Measure adaptive write batching `ac713f4d-…`  
Date: 2026-08-03  
Depends on: T10 harness re-run + collection connect + T9 path honesty  

**Not:** AWO package accept, default-on, product floors, adaptive decision quality,
multithreaded admit correctness, sustained qualification, or sparse-latency product claim.

Evidence (numbers only):
`artifacts/awo-three-way-t10-apfs-smoke/summary.json`  
Harness path: `AWO_THREE_WAY_T10_HARNESS_RERUN.md`

### Both sides of the freeze (principal accept summary)

| Shape | What is frozen |
|-------|----------------|
| **Saturated** | ≈ **2 logical acknowledgements / sync** and ≈ **2×** thr vs Disabled |
| **Sparse** | **No** batching benefit; observed **11–20%** smoke thr penalty vs Disabled |

This **closes the evidence-freeze card only**.

---

## 0. Metric law (freeze vocabulary)

Do **not** report barrier amortization as `file_sync/append` or “writes/sync”.

**Canonical ratios for this freeze:**

```text
file_sync / logical_acknowledged_operations
logical acknowledgements / sync
```

**Unchunked cell fact (this smoke):** every logical put is one append, so probe
`append_median` equals logical acknowledgement count:

```text
append_count == logical_ack_count == 24
```

Therefore numeric ratios equal the old harness `file_sync_per_append_median` on
**this** cell only. Naming must still be **logical acknowledgement** so future
**chunked** work (multiple appends per logical put, or multi-frame installs)
cannot silently re-label physical appends as acks.

Harness JSON still exposes `append_median` / `file_sync_per_append_median`; the
freeze **interprets** those values as logical-ack denominators for this unchunked
workload.

---

## 1. Frozen claim (one paragraph)

**This is the first real positive AWO performance signal.**

Under **queued independent writes** (presentation `batch_size=1`, outstanding pile-up so collection can form multi-item Durable installs), Residiuum combined approximately **two logical acknowledgements behind each durability barrier**:

```text
Disabled: 1 logical acknowledgement / sync → ~4.2 MiB/s
Static:   2 logical acknowledgements / sync → ~9.0 MiB/s
Adaptive: 2 logical acknowledgements / sync → ~8.8 MiB/s
```

Source cell (APFS smoke, seed 42, max_cells=1, saturated pin b1 c4 o8 → serial admit outstanding=8):

| Mode | thr med MiB/s | file_sync med | file_sync / logical_ack | logical acks / sync |
|------|---------------|---------------|-------------------------|---------------------|
| disabled | ~4.18 | 24 | **1.00** | **1** |
| static | ~9.04 | 12 | **0.50** | **2** |
| adaptive | ~8.76 | 12 | **0.50** | **2** |

Unchunked identity for all modes in this cell:

```text
append_count == logical_ack_count == 24
```

**Causal reading (freeze):** throughput roughly **doubles** as sync frequency roughly **halves**. That match is strong smoke evidence that the thr gain is **barrier amortization from L-AWO collection of independent admits**, not thr noise and not L-API `put_many(N)` presentation cheating (Disabled stayed 1 logical ack per sync on the same independent-single shape).

---

## 2. Negative control — sparse (required honesty)

Same artifact, sparse pin **b1 c1 o1**. **No amortization** when there is nothing to collect.

### Smoke observations (not established floors)

These are **smoke observations** only — **not** established product latency floors,
diagnostic ranks, or “AWO always costs N% on sparse” claims:

```text
Sparse independent singles — b1 c1 o1

Disabled: ~4.4 MiB/s
Static: ~3.9 MiB/s (~11% below Disabled)
Adaptive: ~3.5 MiB/s (~20% below Disabled)

file_sync / logical_acknowledged_operations = 1.0 for all modes
```

Detail table (same cell; medians from `summary.json`):

| Mode | thr med MiB/s | file_sync med | file_sync / logical_ack | logical acks / sync |
|------|---------------|---------------|-------------------------|---------------------|
| disabled | ~4.42 | 24 | **1.00** | **1** |
| static | ~3.88 | 24 | **1.00** | **1** |
| adaptive | ~3.52 | 24 | **1.00** | **1** |

Also unchunked: `append_count == logical_ack_count == 24`.

Sparse is the **negative result** frozen with the positive: Static/Adaptive do **not**
reduce syncs; thr is slightly **worse** than Disabled (collection delay without
multi-item batches). Do not quote the saturated thr×2 without this control.

---

## 3. Why this is “first real” (vs T6–T9)

| Prior | What it showed | Why not this freeze |
|-------|----------------|---------------------|
| T6 Scratch thr gaps | thr numbers under mixed presentation | Diagnostic residual; batch path confusion |
| T7 v2 plan | Independent singles = fair L-AWO shape | Plan only |
| T8 | All modes `file_sync / logical_ack = 1` | AWO collection **not** on independent path yet |
| T9 | Code path: `admit_put` → natural under global mutex | Honesty that AWO was disconnected |
| T10 | Collection connect + PQH `admit_put` re-run | Numbers exist; this card **freezes the causal reading** |

T9 said: do not claim Adaptive/Static product wins on independent writes until collection is connected.  
T10 connected the harness/product collection path for independent admits and measured amortization.  
**T11 freezes the signal** — still smoke, still not floors or product ranking.

---

## 4. Scope of the signal (honest bounds)

**In scope for this freeze**

- Saturated independent singles with outstanding depth (collection window can form k≈2 batches).
- Three-way Off / Static / Adaptive on the same presentation pin.
- Proxy thr (MiB/s) co-moving with `file_sync / logical_acknowledged_operations`.
- Causal story: thr×2 ↔ sync/2 under queued independent writes (logical-ack basis).
- Sparse negative control (no amortization; thr smoke obs ~11% / ~20% below Disabled).

**Out of scope (do not over-read)**

| Claim | Status |
|-------|--------|
| Diagnostic floors / qualification campaign | **No** (smoke max_cells=1) |
| Product ranking Static vs Adaptive | **No** (Adaptive ≈ Static on this cell only) |
| Sparse shape amortization | **No** — sparse still `file_sync / logical_ack = 1` |
| Default-on AWO / package accept | **No** |
| Multi-thread admit_put product path | Residual (T10 maps conc→serial admit) |
| Host/FS portability (exFAT Scratch residual) | **No** |
| Universal “always 2× thr” | **No** — only this smoke cell + shape |
| Chunked workloads (`append_count ≠ logical_ack_count`) | **Not measured** — keep names honest |

---

## 5. Claim table (T11)

| Claim | Status |
|-------|--------|
| First positive **L-AWO** thr signal under independent singles | **Yes** (smoke) |
| thr doubling tracks sync-frequency halving (causal) | **Yes** (saturated cell) |
| Ratios use **logical_ack**, not raw append-as-product-story | **Yes** (this freeze) |
| Unchunked identity `append == logical_ack == 24` recorded | **Yes** |
| Disabled fair control stays 1 logical ack / sync | **Yes** |
| Sparse negative: no amortization; Static/Adaptive ≤ Disabled thr | **Yes** |
| Sparse thr smoke obs: ~4.4 / ~3.9 (~11%) / ~3.5 (~20%); sync ratio 1.0 all | **Yes** (observation only) |
| Sparse thr deltas established as product floors | **No** |
| Static ≈ Adaptive under saturated smoke | **Yes** (this cell) |
| Collection connected for independent admits (labor) | **Yes** (T10 + residual card) |
| Product floors / default-on / package accept | **No** |
| Evidence-freeze **card** principal accept (`done`) | **Yes** (this card only) |

### Open after T11 (not closed by this accept)

- Adaptive decision quality (when Adaptive ≠ Static under load)
- Multithreaded `admit_put` correctness residual
- Sustained / diagnostic qualification campaign
- Sparse latency as product claim (smoke penalty only)

---

## 6. Related

| Path | Role |
|------|------|
| `AWO_THREE_WAY_T10_HARNESS_RERUN.md` | Measurement + harness path (probe field names) |
| `artifacts/awo-three-way-t10-apfs-smoke/summary.json` | Numeric SoT for freeze table |
| `AWO_THREE_WAY_T9_DECISIVE_FINDING.md` | Pre-connect honesty (natural-only) |
| `AWO_INDEPENDENT_COLLECTION_CONNECT.md` | Collection connect labor |
| `AWO_THREE_WAY_T7_SPARSE_SATURATED.md` | Independent-singles presentation law |

---

## 7. Exit

Principal accepted this **evidence-freeze** card (`done`). No new measure required for T11.

```text
# numbers frozen from T10
cat doc/todo/performance-qualification/artifacts/awo-three-way-t10-apfs-smoke/summary.json
```