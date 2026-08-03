# AWO labor execution plan (developer start pack)

Status: **labor-ready planning v1 — board pre-staged; product mutation still gated**  
Program: `AWO`  
Profile: `residiuum-adaptive-write-v1`  
Date: 2026-08-02  
AWO-0 T1: **labor complete** — see [AWO-0_T1_CONTRACT_RESIDUAL_CHECKLIST.md](AWO-0_T1_CONTRACT_RESIDUAL_CHECKLIST.md) (not package accept)  
AWO-0 T2: **labor complete** — `adaptive_write::{mod,types,model}` + 12/12 goldens  
AWO-0 T3: **labor complete** — `formal/awo/tla/*`, `verification/awo/golden/`, `scripts/verify-awo.sh` green  
**AWO-0 package accept:** still principal/process — labor floor T1–T3 delivered; no self-accept  
AWO-1: **labor deepened** — ActiveSegment checkpoint/restore; single-shard + parallel-cook + **multi-shard all-or-nothing** persist-before-publish; `awo_writer_poisoned` + `AdaptiveWriterPoisoned`; `awo_persist_before_publish` 4/4. Residuals: full AdaptiveWriteLease (AWO-3), complete failpoint matrix crash suite.

Normative sources (do not re-invent):

| Authority | Path |
|---|---|
| Semantics | [ADAPTIVE_WRITE_OPTIMISER_SPEC.md](ADAPTIVE_WRITE_OPTIMISER_SPEC.md) |
| Closed implementation choices | [ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md](ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md) |
| Executable contracts | [spec/performance/awo/](../../../spec/performance/awo/) |
| Measurement lab | [PERFORMANCE_QUALIFICATION_*](README.md) |
| Delivery admission | [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md) |
| Active-writer layout | `doc/wip/heap/HEAP_SPEC.md` §34 (`active/<heap-id>/<shard-id>`) |

This document is the **pullable labor map** for developers and Gremlin board
cards. It does not amend norms. If code cannot implement a closed formula,
type, or default safely, stop for a **spec amendment** — do not invent locally.

---

## 0. What “done planning” means for this pack

After this pack:

1. Entry gates and honesty residuals are explicit.
2. Package DAG AWO-0…AWO-7 is sliced into board tasks with files, tests, deps.
3. First developable work is **AWO-0** (contracts + pure model only).
4. Product write-path mutation is **forbidden** until AWO-0 accept + entry
   evidence in §1.
5. Hard non-goals (Tokio, weakened CAS/durability/format) are non-negotiable.

---

## 1. Entry conditions (must stay honest)

From implementation plan §1. AWO product work may begin only when:

| # | Gate | Current honesty (2026-08-02, AWO-0 T1 stamp) | Blocks |
|---|---|---|---|
| E1 | Master-plan admission of AWO | **OPEN principal residual.** Not a named package series in `MASTER_DELIVERY_PLAN.md` (PQ0/PQH is; AWO is post-PQH candidate). Awarded turn labor may work AWO-0 pure contracts; do not claim critical-path without admission. | Claiming AWO on critical path |
| E2 | Core-storage ack + recovery suites green | **PASS for entry.** Master plan: `CSQ-0`…`CSQ-12` scoreboard **accept**. Still re-run CSQ ack/recovery subset via future `verify-awo.sh` before AWO-1 persist claims. | AWO-1+ persist path (re-verify) |
| E3 | PQH can measure L3 cooking + L4 real-store with honest boundary events | **PARTIAL.** PQH-0 green; PQH-6/7 (L3/L4) not delivered. OK for AWO-0; required for AWO-G8 / AWO-6 campaign claims. | AWO-6/G8 product claims |
| E4 | AWO registries pass `scripts/verify-awo-contract.sh` | **PASS** (T1 re-run exit 0: 11 states, 12 transitions, 20 reasons, 9 outcomes, 12 golden vectors). | — |
| E5 | **AWO-0 accepted** before product-path mutation | **Labor floor T1–T3 complete** (`bash scripts/verify-awo.sh` OK). **Package accept still NOT claimed** (principal/process). Product mutation remains blocked until accept. | AWO-1…AWO-4 store mutation |
| E6 | Heap-qualified active-writer layout for AWO-3 product | HEAP_SPEC §34 path `active/<heap-id-hex>/<shard-id>.residiuum`. Legacy empty-envelope active segment is diagnostic-only; **cannot** support AWO-G7 or default-on. | AWO-3 product integration / G7 |

**Labor may start on AWO-0 immediately** under this plan (no product write path).
AWO-1+ require E5. AWO-3 product integration requires E6. AWO-6 claims require
E3 depth (L3/L4), not smoke.

No package may weaken format verification, heap qualification, writer locking,
`WriteCondition` / CAS semantics, or durability to improve a number.

---

## 2. Code truth (start from here)

| Fact | Implication |
|---|---|
| `residiuum-store` is sync + `std` threads | AWO uses `std::thread`, `Mutex`, `Condvar`, `VecDeque`, atomics, credit accounting only |
| Host owns `Arc<Mutex<PhysicalStore>>` | Single mutation authority via `AdaptiveWriteLease` when AWO active |
| Mutations today hold physical-store mutex synchronously | AWO re-owns mutation through coordinator; direct `put` returns `AdaptiveWriterActive` |
| Parallel cooker for `put_many` creates scoped OS threads, clones bodies, installs serially | AWO-2 replaces per-batch thread create; bodies stay `Arc<[u8]>` |
| Batch path may publish in-memory visibility before tail write | **AWO-1 prerequisite:** persist-before-publish + checkpoint restore |
| CAS evaluates under ordered store ownership | Conditional work is **natural command** on same ticket stream — never overtaken |
| No `crates/.../adaptive_write/` yet | AWO-0 creates pure model module tree first |
| No Tokio/Rayon/crossbeam/actors in AWO-0…4 | Out of scope; do not add |

---

## 3. Architecture sketch (V1)

```text
  put/delete (HeapStore / StoreHost)
            │
            ▼
     capability + SubjectV2 + op-id
            │
            ▼
     AdaptiveWriteHandle.admit ──► credit reserve + lane ticket + queue
            │
            ▼
     Coordinator (writer command loop)
       • fairness: earliest completion_deadline, then admitted_at, then LaneKey
       • selector: natural vs batch (AWO-5; static fixed limits in AWO-3)
       • natural ineligible: CAS, chunked, Memory, Atomics, maintenance (fence)
            │
     ┌──────┴──────┐
     ▼             ▼
  NaturalCmd    Batch path
     │          Reserve → Cook (persistent workers) → Ready ring
     │             │
     └──────┬──────┘
            ▼
     Persist (lease token) → tail write + barrier → Publish → Complete
            │
            ▼ (on I/O uncertain)
     poison writer; no further mutation; reopen recovery
```

**Lane key (V1):** `(heap_id, writer_shard, durability∈{Buffered,Durable}, layout∈{InlinePut,Delete})`  
Physical writer key: `(heap_id, writer_shard)` — never append heap B to heap A.

**Modes:** `Disabled` (default until AWO-7) | `Static` (AWO-3+) | `Adaptive` (AWO-5+).

---

## 4. Package DAG and board tasks

```text
AWO-0 → AWO-1 → AWO-2 → AWO-3 → AWO-4 → AWO-5 → AWO-6 → AWO-7
         │         │       │
         │         │       └── needs heap active-writer layout (E6)
         │         └── may overlap AWO-0 analysis only
         └── persist-before-publish is mandatory even if later AWO deferred
```

Acceptance gates (SPEC §20): G1↔AWO-0 … G12↔AWO-7 principal default-on.

### AWO-0 — Contracts and executable model  **← first pull**

**Goal:** Freeze registries + pure selector arithmetic; **zero** store write-path change.

**Already on disk (do not redo blindly):**

- `spec/performance/awo/{profile,states,transitions,decision-reasons,outcomes,policy,golden-decisions}-v1.json`
- `spec/performance/awo/schemas/golden-decisions-v1.schema.json`
- `scripts/verify-awo-contract.sh` (green)

**Deliver (remaining):**

| Artifact | Notes |
|---|---|
| `crates/residiuum-store/src/adaptive_write/mod.rs` | Module root; feature-gated if needed (`legacy-raw-store` per plan tests) |
| `adaptive_write/model.rs` | Pure types + selector arithmetic matching golden vectors + `policy-v1.json` |
| `adaptive_write/types.rs` (minimal) | Shared public constants (`AWO_PROFILE`) if needed by model tests |
| `crates/residiuum-store/tests/awo_contract.rs` | Load JSON goldens; assert code == closed sets |
| `formal/awo/tla/AdaptiveWrite.tla` + `.cfg` | Skeleton with SPEC §17 variable/transition names |
| `formal/awo/verus/model.rs` | Pure kernel stub (optional thin in AWO-0; deepen AWO-6) |
| `verification/awo/golden/` | Copy or symlink golden decisions for formal/CI |
| `scripts/verify-awo.sh` | Orchestrator stub: contract → model tests (expand per package) |

**Exit:**

```bash
bash scripts/verify-awo-contract.sh
cargo test -p residiuum-store --features legacy-raw-store adaptive_write::model
# + awo_contract integration when present
```

All 12 golden vectors pass; code closed sets agree with JSON reasons/states.

**Non-goals:** StoreHost hooks, cooker threads, lease, server config.

**Board cards:** AWO-0 T1 contracts residual honesty · AWO-0 T2 pure model + goldens · AWO-0 T3 formal skeleton + verify-awo.sh

---

### AWO-1 — Persist-before-publish

**Depends:** AWO-0 accept.  
**Goal:** Reservation, active-segment checkpoint/restore, staged publication, poisoning, reopen recovery. Correct existing batch path that publishes before durable success.

**Files:**

- `residiuum-format` `ActiveSegment` checkpoint/restore (`bytes`, `base_offset`, `writer_sequence`, `frame_count`, sealed)
- `adaptive_write/{persist,types,credits}.rs` (as needed)
- `store.rs` batch install path correction
- Failpoints: `awo.reserve.after` … `awo.complete.before` (plan §15 list)

**Tests:**

- `awo_persist_before_publish.rs`
- `awo_partial_write_recovery.rs`
- `awo_publication_failure.rs`
- `awo_direct_writer_lease.rs` (lease denial of direct put)

**Invariants:** No publish without persist; partial I/O → uncertain + poison; no append behind uncertain tail.

**Gate:** AWO-G2.

---

### AWO-2 — Persistent cooker + credits

**Depends:** AWO-1.  
**Goal:** `maximum_cookers` parked threads at start; scale via permits not spawn/join; bounded queues; ordered ready (`BTreeMap` ticket → outcome in AWO-2); zero post-warm thread create; cookers never touch Store/indexes/FDs.

**Files:** `queue.rs`, `credits.rs`, `cooker.rs`, `ordered_ready.rs`, buffer pool in cooker.

**Tests:** `awo_credit_bounds.rs`; cooker isolation unit tests; frame-byte equivalence vs serial encode.

**Gate:** AWO-G3.

---

### AWO-3 — Static Intake Arbiter (product integration)

**Depends:** AWO-2 + E6 heap active-writer layout.  
**Goal:** `StoreHost::{create,open}_with_adaptive_write`, `HeapStore` routing, static batch limits, natural for ineligible, independent completions, fences, server error map, **mode default disabled**.

**Files:** store host/heap_store/host; `error.rs` codes; server config/metrics/heap_dispatch after store path green; put/delete integration tests.

**Routing:**

```text
unconditional eligible put/delete → Admit eligible → block WriteCompletion
conditional | chunked | Memory | Atomics | maintenance
  → NaturalCommand same coordinator → block completion
```

**Wire map:** plan §13 (`write_overloaded`, `write_deadline`, `server_draining`, `heap_unavailable`, `write_outcome_uncertain`).  
`operation_id` mandatory for qualified mutating RPC before remote enablement.

**Gate:** AWO-G4 (+ path toward G7).

---

### AWO-4 — Ordered overlap (depth ≤ 2)

**Depends:** AWO-3.  
**Goal:** Pipeline depth 2: write batch A while cooking reserved batch B; **no third** unresolved reservation; seal-safe rotation; bounded shutdown.

**Files:** `coordinator.rs`, `persist.rs` pipeline; PQH stage-overlap evidence hooks later.

**Tests:** `awo_shutdown.rs`, overlap evidence test (deterministic clock).

**Gate:** AWO-G5.

---

### AWO-5 — Adaptive controller

**Depends:** AWO-4.  
**Goal:** EWMA estimator (`m',d'` with /8), candidate set `{1,2,4,...,1024,queued}`, margin `decision_margin_ppm=100_000`, collection cap 250µs, autoscaling hysteresis (plan §11.6), cold-start natural, model invalidation, `AwoClock` (no wall sleeps in correctness tests).

**Files:** `estimator.rs`, `selector.rs`, `controller.rs`, `policy.rs`, `telemetry.rs`.

**Tests:** `awo_adaptive_oracle.rs`, `awo_controller_stability.rs`, falsification vs lying/stale estimators.

**Gate:** AWO-G6.

---

### AWO-6 — Qualification + formal connection

**Depends:** AWO-5 + PQH L3/L4 ability (E3).  
**Goal:** Crash matrix, loom/model campaigns, controlled PQH (not smoke), Verus/TLA deepen, FAS registry, hash-bound evidence bundle.

**Files:** `residiuum-perf/src/awo/*`, `awo_crash_matrix.rs`, `awo_qualification.rs`.

**Gate:** AWO-G7…G10. Smoke **never** marks G8 green.

---

### AWO-7 — Productisation

**Depends:** AWO-6 + principal.  
**Goal:** Default posture decision, drain/reset/inspection ops, SDK docs, upgrade/rollback, support matrix, benchmark disclosure. **Default-on = principal only (G12).**

---

## 5. Module tree (normative names)

```text
crates/residiuum-store/src/adaptive_write/
  mod.rs
  types.rs          request, lane, ticket, completion, public status
  policy.rs         validated policy + machine defaults from policy-v1.json
  credits.rs        byte/entry ledger
  queue.rs          Mutex<VecDeque<T>> + Condvar
  estimator.rs
  selector.rs
  controller.rs
  cooker.rs
  ordered_ready.rs
  coordinator.rs
  persist.rs
  telemetry.rs
  model.rs          pure model for vectors (AWO-0 first)
```

Public surface (plan §5): `AdaptiveWriteMode`, `AdaptiveWritePolicy`,
`AdaptiveWriteRuntime`, `AdaptiveWriteHandle`, `WriteCompletion`,
`AdmissionResult`, `AdaptiveWriteError`, host create/open_with_adaptive_write,
`drain_writes`, `adaptive_write_status`.

---

## 6. Default policy (closed — do not freestyle)

From `policy-v1.json` / plan §12:

| Field | Default |
|---|---:|
| mode before AWO-7 | disabled |
| queue entries / bytes | 8192 / 64 MiB |
| batch entries / bytes | 1024 / 16 MiB |
| collection cap | 250 µs |
| completion deadline | 30 s |
| min/max cookers | 1 / min(max(parallelism-1,1),16) |
| pipeline depth | 2 |
| decision margin | 10% (100_000 ppm) |
| estimator warm / stale | 32 samples / 30 s |
| controller interval | 100 ms |
| scale-up / down dwell | 500 ms / 2 s |

Validation rejects: zero limits, queue < one max batch, pipeline ∉ 1..=4,
max cookers > 64, collection > 10 ms, deadline < collection cap.

---

## 7. Test inventory (create as packages land)

```text
crates/residiuum-store/tests/
  awo_contract.rs
  awo_persist_before_publish.rs
  awo_partial_write_recovery.rs
  awo_credit_bounds.rs
  awo_ordering.rs
  awo_lane_isolation.rs
  awo_cancellation.rs
  awo_shutdown.rs
  awo_static_equivalence.rs
  awo_adaptive_oracle.rs
  awo_controller_stability.rs
  awo_crash_matrix.rs
crates/residiuum-server/tests/awo_heap_rpc.rs
crates/residiuum-perf/tests/awo_qualification.rs
```

Concurrency tests need a bounded deterministic variant. Controller correctness
uses injected `AwoClock` only.

---

## 8. Support matrix (V1) — quick reference

| Class | Path |
|---|---|
| Unconditional inline put | AWO eligible |
| Unconditional delete | Eligible after AWO-3 delete vectors |
| Conditional/CAS put/delete | Natural, same writer authority |
| Chunked/large put | Natural until chunk profile |
| Memory durability | Natural |
| Buffered / Durable | Dedicated AWO lanes |
| Atomics group | Never decomposed by AWO |
| Cluster/Raft | Separate profile later |
| Maintenance | Fence + natural |

---

## 9. Hard non-goals (labor stop conditions)

- Tokio, Rayon, crossbeam, actor frameworks, alternate executors (AWO-0…4)
- Weakening BLAKE3/CRC/frame verify, heap binding, CAS, durability
- Implicit transactions / all-or-nothing batch membership
- Reconstructing AWO decisions from receipts in PQH
- Caller deadline smuggled in qualified `args` (unknown fields rejected)
- Third unresolved reservation (AWO-4+)
- Publish before persist; append after uncertain I/O
- Default-on without principal (G12)
- Absolute ops/s as CI gate
- Multi-device striping in V1

If a closed constant cannot ship safely → **spec amendment**, not local invention.

---

## 10. Recommended pull order (first two weeks of labor)

| Day focus | Card | Outcome |
|---|---|---|
| 1 | AWO-0 T1 | Document contract residual vs plan; keep verify-awo-contract green |
| 1–2 | AWO-0 T2 | `model.rs` + golden runner; 12/12 vectors |
| 2 | AWO-0 T3 | TLA skeleton + `verify-awo.sh` stub; scoreboard AWO-0 labor evidence (not package accept unless principal) |
| next | **Stop** for principal admit if required; else AWO-1 design against current `put_many` | Persist-before-publish failpoint map |
| then | AWO-1 implementation | G2 evidence |
| … | AWO-2… | Follow DAG |

**Do not invent-at-pull** further tasks without pre-staging (GOV). Packages AWO-0…7 are pre-staged on the Kanban board under Feature **AWO — Adaptive Write Optimiser**.

---

## 11. Acceptance command (target)

```bash
bash scripts/verify-awo.sh
```

Order (plan §18): contract → model/unit → store integration → CSQ ack/recovery subset → heap isolation → server qualified mutation → formal checks available → PQH AWO smoke (not qualification claim).

Controlled qualification: `residiuum-perf --class qualification` only.

---

## 12. Board / process notes

- Labor never self-marks package **accept**.
- One package at a time; card includes deps, files, registry version, tests, residuals.
- Scoreboard: evidence only; host Kanban is labor SoT.
- Parallel M1/query work remains independent; AWO is performance lane, not APP query claim.

### Residuals for principal

1. **Master-plan AWO admission** (E1) — add AWO package series or explicit award.
2. **PQH-6/7** schedule relative to AWO-6 evidence (E3).
3. **AWO-0 accept** criteria when T1–T3 land (E5).
4. Default-on only at AWO-7 (G12).

---

## 13. Traceability: plan sections → labor

| Impl plan § | Labor use |
|---|---|
| §1 Entry | This §1 |
| §2 Code truth | This §2 |
| §3 Support matrix | This §8 |
| §4 Modules | This §5 |
| §5 Rust contracts | AWO-0 types + AWO-3 host |
| §6 Lease | AWO-1/3 |
| §7 Lanes/order | AWO-2/3 |
| §8 Admission | AWO-3 |
| §9 Persist-before-publish | AWO-1 |
| §10 Cooker | AWO-2 |
| §11 Controller | AWO-5 |
| §12 Defaults | policy-v1 + this §6 |
| §13 Errors/wire | AWO-3 |
| §14 Telemetry | AWO-3/5 |
| §15 Packages | This §4 |
| §16 Tests | This §7 |
| §17 Formal | AWO-0 skeleton, AWO-6 depth |
| §18 verify-awo.sh | AWO-0 T3 onward |
| §19 Hand-off | Board card template |

---

*End of AWO labor execution plan v1. Implementation choices remain owned by ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md.*