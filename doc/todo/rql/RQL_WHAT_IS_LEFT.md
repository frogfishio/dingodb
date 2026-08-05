# RQL — what is left to do

Status: **2026-08-05** · single page for principals · **Decision 0 + X1 freeze**  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md)  
Detail: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §0 · [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)

If this page disagrees with PATH §6 or chat summaries, **this page wins** until amended.

---

## Ordered programme (only this order)

```text
Decision 0  ──►  RQL-X1 bytecode+host freeze  ──►  RQL-X2 converge runtime
                                                      │
                                                      ├─ lower all syntaxes
                                                      ├─ emb + op 118 same runtime
                                                      ├─ port frozen executors
                                                      ├─ equivalence
                                                      ├─ delete Rust semantic executors
                                                      └─ CI anti-executor gate
```

Everything else (S1, D1, wire enrich, perf, C1 accept) waits on **X2** (or an
explicit principal waiver).

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X2** | Implement one bytecode runtime; route emb+op118; port; prove; delete `query_exec_v1` + `execute_rql_full`; CI gate |
| **2** | **Human** | Review X1 / 0D | Confirm `residiuum-query-bytecode-v1` boundary |
| **3** | **Human** | **RQL-C1** | Scoreboard APP-6/7/APB-7 accept **after** shared-runtime honesty |

---

## Hard freeze (in force)

| Item | Rule |
|---|---|
| `query_exec_v1` | No feature growth |
| `execute_rql_full` | No feature growth |
| Host | scan / index / get only — no query algebra |
| Semantic executors | **One** only (bytecode machine) |
| Test oracle | Allowed in tests; never product |

Profile: **`residiuum-query-bytecode-v1`** — [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md)

---

## Board legend (bring to order)

| Stage | Cards | How to read them |
|---|---|---|
| **todo** | RQL-X2 (when staged) | Only claimable labor for query spine |
| **doing** | (empty when X1 closed) | Active convergence only |
| **in_review** | 0D, X1, Phase3/F*/I1/RQL-0/clarity | Charter + **port inventory** — not “grow these façades” |
| **done** | (principal only) | Human accept |

Prior Phase 3 / F1 / F2 / I1 evidence is **inventory to port**, not a license
to extend those modules.

---

## Blocked until X2

| Package | Note |
|---|---|
| RQL-S1 SQL+ enrich emit | Frontend → shared plan/bytecode only |
| Op-118 enrich parity | Same runtime — not a third executor |
| RQL-D1 `at rank` | Needs DDA + shared bytecode |
| Within-nested index / re-page | Port into bytecode machine |
| RQL-Q1 perf | After shared runtime |

---

## One-line status

```text
ORDERED    = Decision 0 → X1 freeze (done) → X2 converge (next)
FROZEN     = query_exec_v1 + execute_rql_full
BYTECODE   = residiuum-query-bytecode-v1 (architecture freeze)
NOT next   = S1, D1, façade features, premature C1 “query qualified”
```
