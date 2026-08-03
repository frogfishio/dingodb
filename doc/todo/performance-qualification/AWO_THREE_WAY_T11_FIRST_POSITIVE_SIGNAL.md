# Measure AWO three-way — T11 first positive performance signal (freeze)

Status: **labor complete (honesty freeze) — not package accept**  
Card: `b7986427-e96a-437d-a443-34c3647d8f1e`  
Feature: Measure adaptive write batching `ac713f4d-…`  
Date: 2026-08-03  
Depends on: T10 harness re-run + collection connect + T9 path honesty  

Evidence (numbers only):
`artifacts/awo-three-way-t10-apfs-smoke/summary.json`  
Harness path: `AWO_THREE_WAY_T10_HARNESS_RERUN.md`

---

## 1. Frozen claim (one paragraph)

**This is the first real positive AWO performance signal.**

Under **queued independent writes** (presentation `batch_size=1`, outstanding pile-up so collection can form multi-item Durable installs), Residiuum combined approximately **two logical writes behind each durability barrier**:

```text
Disabled: 1 write/sync → ~4.2 MiB/s
Static:   2 writes/sync → ~9.0 MiB/s
Adaptive: 2 writes/sync → ~8.8 MiB/s
```

Source cell (APFS smoke, seed 42, max_cells=1, saturated pin b1 c4 o8 → serial admit outstanding=8):

| Mode | thr med MiB/s | file_sync med | file_sync/append | writes/sync |
|------|---------------|---------------|------------------|-------------|
| disabled | ~4.18 | 24 | **1.00** | **1** |
| static | ~9.04 | 12 | **0.50** | **2** |
| adaptive | ~8.76 | 12 | **0.50** | **2** |

**Causal reading (freeze):** throughput roughly **doubles** as sync frequency roughly **halves**. That match is strong smoke evidence that the thr gain is **barrier amortization from L-AWO collection of independent admits**, not thr noise and not L-API `put_many(N)` presentation cheating (Disabled stayed 1 sync/op on the same independent-single shape).

---

## 2. Why this is “first real” (vs T6–T9)

| Prior | What it showed | Why not this freeze |
|-------|----------------|---------------------|
| T6 Scratch thr gaps | thr numbers under mixed presentation | Diagnostic residual; batch path confusion |
| T7 v2 plan | Independent singles = fair L-AWO shape | Plan only |
| T8 | All modes `file_sync/ops = 1` | AWO collection **not** on independent path yet |
| T9 | Code path: `admit_put` → natural under global mutex | Honesty that AWO was disconnected |
| T10 | Collection connect + PQH `admit_put` re-run | Numbers exist; this card **freezes the causal reading** |

T9 said: do not claim Adaptive/Static product wins on independent writes until collection is connected.  
T10 connected the harness/product collection path for independent admits and measured amortization.  
**T11 freezes the signal** — still smoke, still not floors or product ranking.

---

## 3. Scope of the signal (honest bounds)

**In scope for this freeze**

- Saturated independent singles with outstanding depth (collection window can form k≈2 batches).
- Three-way Off / Static / Adaptive on the same presentation pin.
- Proxy thr (MiB/s) co-moving with `file_sync/ops`.
- Causal story: thr×2 ↔ sync/2 under queued independent writes.

**Out of scope (do not over-read)**

| Claim | Status |
|-------|--------|
| Diagnostic floors / qualification campaign | **No** (smoke max_cells=1) |
| Product ranking Static vs Adaptive | **No** (Adaptive ≈ Static on this cell only) |
| Sparse shape amortization | **No** — sparse still `file_sync/ops = 1` (no multi-item collect without pile-up) |
| Default-on AWO / package accept | **No** |
| Multi-thread admit_put product path | Residual (T10 maps conc→serial admit) |
| Host/FS portability (exFAT Scratch residual) | **No** |
| Universal “always 2× thr” | **No** — only this smoke cell + shape |

Sparse (same artifact, b1 c1 o1) remains the honest control: Static/Adaptive slightly **slower** than Disabled when there is nothing to batch — collection delay without amortization.

---

## 4. Claim table (T11)

| Claim | Status |
|-------|--------|
| First positive **L-AWO** thr signal under independent singles | **Yes** (smoke) |
| thr doubling tracks sync-frequency halving (causal) | **Yes** (this cell) |
| Disabled fair control stays 1 write/sync | **Yes** |
| Static ≈ Adaptive under saturated smoke | **Yes** (this cell) |
| Collection connected for independent admits (labor) | **Yes** (T10 + residual card) |
| Product floors / default-on / package accept | **No** |
| Sparse multi-write amortization | **No** (expected) |

---

## 5. Related

| Path | Role |
|------|------|
| `AWO_THREE_WAY_T10_HARNESS_RERUN.md` | Measurement + harness path |
| `artifacts/awo-three-way-t10-apfs-smoke/summary.json` | Numeric SoT for freeze table |
| `AWO_THREE_WAY_T9_DECISIVE_FINDING.md` | Pre-connect honesty (natural-only) |
| `AWO_INDEPENDENT_COLLECTION_CONNECT.md` | Collection connect labor |
| `AWO_THREE_WAY_T7_SPARSE_SATURATED.md` | Independent-singles presentation law |

---

## 6. Exit

Doc freeze only. No new measure required for this card.

```text
# numbers already frozen from T10
cat doc/todo/performance-qualification/artifacts/awo-three-way-t10-apfs-smoke/summary.json
```