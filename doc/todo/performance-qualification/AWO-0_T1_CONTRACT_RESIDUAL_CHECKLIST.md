# AWO-0 T1 — Contract residual honesty + entry gate checklist

Status: **labor complete (self_check) — not package accept**  
Card: `a24e502f-7b93-4f96-a0c1-104fc2bfc5c7`  
Profile: `residiuum-adaptive-write-v1`  
Evidence date: 2026-08-02  
Program: `AWO`

Normative authorities (unchanged by this card):

- [ADAPTIVE_WRITE_OPTIMISER_SPEC.md](ADAPTIVE_WRITE_OPTIMISER_SPEC.md)
- [ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md](ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md)
- [AWO_LABOR_EXECUTION.md](AWO_LABOR_EXECUTION.md)
- [spec/performance/awo/](../../../spec/performance/awo/)

This card **does not** mutate product write paths, invent controller formulas, or
mark AWO-0 package accept.

---

## 1. Entry gates E1–E6 (honesty stamp)

| Gate | Verdict | Evidence / residual |
|---|---|---|
| **E1** Master-plan AWO admission | **OPEN residual (principal)** | `MASTER_DELIVERY_PLAN.md` lists PQ0/`PQH-*` as post-C0 measurement lane. AWO is documented under performance-qualification as a post-PQH candidate and is **not** a named package series in the master DAG. Principal must admit AWO (or continue awarding turn labor) before treating AWO as critical-path delivery. Labor on AWO-0 pure contracts is allowed under awarded turn packages. |
| **E2** Core-storage ack + recovery green | **PASS for entry** | Master plan delivery record: `CSQ-0`…`CSQ-12` scoreboard **accept** (C0/A2). AWO-1 still must re-run the CSQ acknowledgement/recovery subset named by future `scripts/verify-awo.sh` before claiming persist-path safety. |
| **E3** PQH L3 cooking + L4 real-store | **PARTIAL** | PQH-0 registries green: `bash scripts/verify-performance-registry.sh` OK; `cargo test -p residiuum-perf --lib` previously 181 passed. PQH-6 (L3) / PQH-7 (L4) **not** delivered. Blocks AWO-G8 claims and honest AWO-6 campaign evidence; **does not** block AWO-0 model labor. |
| **E4** `verify-awo-contract.sh` | **PASS** | Re-run 2026-08-02: exit 0 — `AWO contract OK: 11 states, 12 transitions, 20 reasons, 9 outcomes, 12 golden vectors`. |
| **E5** AWO-0 package accept before product mutation | **NOT MET** | JSON contracts frozen and verifying. Rust `adaptive_write::model`, formal skeletons, `verify-awo.sh` **absent**. Product write-path mutation remains **forbidden**. Next cards: AWO-0 T2, T3. |
| **E6** Heap-qualified active-writer layout | **NOT required for AWO-0** | HEAP_SPEC §34: `active/<heap-id-hex>/<shard-id>.residiuum`. Required at AWO-3 product integration / AWO-G7. Legacy empty-envelope active is diagnostic-only. |

**May proceed:** AWO-0 T2 (pure model + goldens) under awarded labor.  
**Must not proceed:** AWO-1+ store mutation until E5; AWO-3 product path until E6; AWO-6/G8 claims until E3 depth.

Hard rule (impl plan §1): no package may weaken format verification, heap
qualification, writer locking, `WriteCondition`/CAS, or durability to improve a
number.

---

## 2. Contract inventory (on disk, profile-bound)

Directory: `spec/performance/awo/`  
All JSON registries carry `"profile": "residiuum-adaptive-write-v1"`.

| File | Role | Closed size |
|---|---|---:|
| `profile-v1.json` | Profile identity, modes, eligible/natural classes | modes 3; eligible 2; natural 7 |
| `states-v1.json` | Request lifecycle states | 11 |
| `transitions-v1.json` | Permitted transitions | 12 |
| `decision-reasons-v1.json` | Natural/batch/forced/fallback reasons | 20 |
| `outcomes-v1.json` | Completion / overload outcomes | 9 |
| `policy-v1.json` | Safe defaults + validation + candidate entries | defaults closed |
| `golden-decisions-v1.json` | Selector arithmetic vectors | 12 |
| `schemas/golden-decisions-v1.schema.json` | Golden JSON Schema (`title` = `residiuum-awo-golden-decisions-v1`) | — |
| `README.md` | Contract map + verify command | — |

### Profile classes (must match support matrix)

**Eligible V1:** `unconditional_inline_put`, `unconditional_delete`  
**Natural V1:** `conditional_put`, `conditional_delete`, `chunked_put`,
`memory_durability`, `atomics`, `cluster_commit`, `maintenance`  
**Modes:** `disabled` | `static` | `adaptive` (default before AWO-7: disabled)

### Cross-checks performed this card

1. `bash scripts/verify-awo-contract.sh` → exit 0.
2. Every golden `expected_reason` ∈ closed decision-reasons set.
3. Policy defaults match implementation plan §12 table (`decision_margin_ppm =
   100000`, collection cap 250µs, mode_before_awo7 = disabled, queue/batch
   limits, estimator floors).
4. Required states from verifier present (received…uncertain_pending_recovery).
5. Terminal states have no outgoing transitions (enforced by verifier).

### Golden coverage residual (honest, not a blocker for T1)

Closed reasons **without** a golden vector today (T2+ may add vectors; do not
silently drop reasons):

```text
batch_predicted_arrival_gain
controller_fallback
forced_deadline
forced_drain
forced_fence
forced_max_bytes
forced_max_entries
forced_segment_boundary
natural_arithmetic_overflow
```

These remain in the closed set and must be implementable by the pure model /
controller; absence of goldens is a **test residual**, not permission to delete
or invent alternate reason ids.

---

## 3. AWO-0 package residual vs implementation plan §15

Plan AWO-0 deliverables vs tree (2026-08-02):

| Deliverable | Status |
|---|---|
| All files under `spec/performance/awo/` | **Present** + verify green |
| Pure model in `adaptive_write/model.rs` | **Present** (T2 labor) |
| Registry verifier | **Present** (`scripts/verify-awo-contract.sh`) |
| Golden-vector runner (Rust) | **Present** (unit + `awo_contract.rs`) |
| TLA+ skeleton (`formal/awo/tla/`) | **Present** (T3 labor) |
| Verus pure stub (`formal/awo/verus/`) | **Present** thin stub (T3); deepen AWO-6 |
| `verification/awo/golden/` | **Present** (symlink to contract goldens) |
| `scripts/verify-awo.sh` orchestrator | **Present** (T3); AWO-0 steps green |
| Store write-path change | **Correctly absent** (forbidden until AWO-0 accept) |

**AWO-0 package accept** requires T2+T3 exit commands green and principal/process
accept rules — not this T1 card alone.

---

## 4. Code-truth residual (no mutation this card)

Confirmed by tree inspect (not changed):

- No `crates/residiuum-store/src/adaptive_write/` module.
- Existing `put_many` parallel cooker remains the pre-AWO diagnostic path
  (scoped threads / clone / serial install) — AWO-1/2 will supersede on
  qualified path only after gates.
- No product Tokio introduction required or performed.

---

## 5. Exit for AWO-0 T1

Met when:

- [x] `verify-awo-contract.sh` exit 0 re-run recorded
- [x] E1–E6 honesty table stamped with evidence and blockers
- [x] Contract inventory + AWO-0 residual vs plan §15 listed
- [x] Golden/reason residual named (not hidden)
- [x] No store write-path mutation
- [x] No AWO-0 package accept claimed
- [x] Next pull named: **AWO-0 T2** (`23168ae2-…`) pure model + 12/12 goldens

---

## 6. Commands re-run this card

```bash
bash scripts/verify-awo-contract.sh
# → AWO contract OK: 11 states, 12 transitions, 20 reasons, 9 outcomes, 12 golden vectors
```

Optional (E3 partial; not T1 gate):

```bash
bash scripts/verify-performance-registry.sh
```

---

## 7. Board / process

- Feature: AWO — Adaptive Write Optimiser (`0513ca67-3c9d-4aa8-85bd-7db3beb9fe1f`)
- This task → stage **in_review** after labor (labor does not self-accept package)
- Unblocks: AWO-0 T2 model labor
- Does **not** unblock: AWO-1 product mutation

*End AWO-0 T1 residual checklist.*