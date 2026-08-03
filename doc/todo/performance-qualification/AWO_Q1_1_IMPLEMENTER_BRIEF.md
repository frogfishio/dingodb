# AWO-Q1.1 — Implementer brief (concurrent harness + ledger)

Status: **ready for code pull** (Planner grounding; no labor yet)  
Card: `fab1a943-8def-4ded-9bfe-0499a567da56`  
Parent package: `b6e2a138` AWO-Q1  
Feature: `d0ae3c06` AWO-Q series  
Series SoT: `AWO_QUALIFICATION_SERIES.md`  

## 1. Problem (one sentence)

T10/T11 thr amortization under “conc>1” was measured with **serial** `admit_put` +
outstanding depth, not real multi-thread producers — because the harness **maps
away** concurrent execution for reopen integrity.

## 2. What already exists (do not re-invent)

| Piece | Location | State |
|-------|----------|--------|
| Independent workload entry | `crates/residiuum-perf/src/store_driver/real.rs` `execute_workload_puts_independent` | **Forces serial** |
| Serial path | same file `execute_serial_admit_put` (~L695) | Used for T10/T11 smoke |
| Concurrent path (stubbed off) | same file `execute_concurrent_admit_put` (~L844) | **Exists but unused**; ledger incomplete; `valid: false` |
| Product wait-outside-mutex | `crates/residiuum-store/src/heap/heap_store.rs` `put_if` (~L290–322) | Admit under lock; **wait outside** already |
| Collection bind | `adaptive_write/collection.rs` `IndependentCollector::bind_physical` | Present |
| Runtime contract | `adaptive_write/runtime.rs` docs: must not hold mutex across `WriteCompletion::wait` | Normative |

### Hard gate that Q1.1 must flip

```text
// real.rs execute_workload_puts_independent ~L640–657
// Always serial admit_put with outstanding depth…
let effective_out = outstanding.max(workers);
let mut stats = execute_serial_admit_put(...);
// note: conc=N mapped to serial admit_put outstanding=M for reopen integrity
```

**Q1.1 exit requires:** for AWO independent `batch_size=1` with `workers > 1`, call
`execute_concurrent_admit_put` (or successor) instead of that map — **or** document
why concurrent still fails and keep residual (not success).

## 3. Deliverables for Q1.1 only

1. **Wire concurrent producers** when `cfg.cell.concurrency > 1` on independent AWO path.
2. **Deterministic ledger** — each logical put has unique id (`seq`); record
   attempt / admit / ack / fail with that id (not synthetic `for i in 0..acked`).
3. **Messages** must state `concurrent_admit_put workers=…` without the serial-map note.
4. **Tests** — at least one store-driver or focused test: N worker threads, M ops,
   ledger counts balance (attempts = acks + fails; no double-ack of same seq).
5. **Evidence note** (short) under `doc/todo/performance-qualification/` e.g.
   `AWO_Q1_1_HARNESS.md` — claim table; link test command.

### Explicit non-goals for 1.1

- Thr ranking / three-way re-run (Q1.3 optional after correctness)
- Product mutex redesign (mostly done; Q1.2 only if concurrent still serializes on store)
- Adaptive quality (Q2), sustained PQH (Q3), sparse product (Q4)
- Package accept / default-on

## 4. Known defects in current concurrent helper (fix when enabling)

Inspect `execute_concurrent_admit_put` before shipping:

| Issue | Why it matters |
|-------|----------------|
| Ledger rebuilds acks as `for i in 0..acked` (~L1037–1042) | Not deterministic per-op; cannot prove no lost/dup |
| `valid: false` always on return (~L1072) | Driver may discard / mislabel run |
| `records` capped at 256 samples | Reopen sample may be incomplete for smoke budgets |
| `floors_met: false` hard-coded | Downstream validity may fail non-smoke |

Q1.1 should fix ledger + validity enough that a smoke concurrent run is honest.
Full reopen integrity campaign can land in **Q1.3** if needed.

## 5. Suggested verify commands

```bash
# focused (adjust names after tests land)
cargo test -p residiuum-perf --features store-driver --lib -- concurrent_admit

# broader store-driver smoke
cargo test -p residiuum-perf --features store-driver --lib real_store
```

## 6. Board sequencing

| After | Next |
|-------|------|
| Q1.1 `in_review` | Q1.2 wait-outside-mutex residual (only if still blocked) |
| Q1.2 done/skip | Q1.3 correctness + evidence freeze |
| 1.1–1.3 principal `done` | Close umbrella Q1 `b6e2a138` |

## 7. Pull instruction

Implementer: stage `fab1a943` → `doing` → implement → evidence → `in_review`.  
Planner does **not** implement. Use **@direct**.
