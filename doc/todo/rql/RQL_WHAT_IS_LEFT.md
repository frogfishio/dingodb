# RQL — what is left to do

Status: **2026-08-05** · single page for principals · **Decision 0 active**  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md) (RQL → Atomics → Cluster)  
Detail: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §0 · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [PHASE3_SURFACE_RESIDUAL.md](./PHASE3_SURFACE_RESIDUAL.md)

If this page disagrees with PATH §6 or chat summaries, **this page wins** until amended.

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X1** | Freeze **one query bytecode** + host-capability boundary (scan/index/get only). No new RQL features until this exists. |
| **2** | **Labor** | **RQL-X2** | Lower all syntaxes; one runtime for emb + op 118; port; equivalence; delete `query_exec_v1` / `execute_rql_full`; CI anti-executor gate |
| **3** | **Human** | **RQL-C1** | Accept APP-6 / APP-7 / APB-7 only after shared-runtime honesty (or explicit waiver) |

---

## Hard freeze (Decision 0)

Parallel semantic executors are an **architectural violation**.

- **Frozen (no feature growth):** `query_exec_v1`, `execute_rql_full`
- **Allowed:** bugfix / evidence honesty only; test-only oracle interpreter
- **Not next:** RQL-S1, wire enrich parity, D1, within-index, more Phase-3 surface on the façades

```text
Multiple syntaxes: yes
Multiple compiler stages: yes
Multiple physical access strategies: yes
Multiple semantic executors: absolutely not
```

---

## Waiting on principal (not labor)

| # | Package | Action |
|---|---|---|
| **P1** | Review Decision 0 / RQL-0D | Confirm freeze + convergence sequence |
| **P2** | Review queue | Accept/reject prior labor `in_review` (inventory / Phase 3 — now **port inventory**, not grow-path) |
| **P3** | **RQL-C1** | Scoreboard accept after convergence (or waiver) |

---

## Later (blocked on RQL-X1)

| # | Package | Why deferred |
|---|---|---|
| **L1** | Op-118 enrich/within **parity** | Must be same runtime, not a third executor |
| **L2** | **RQL-S1** SQL+ → enrich/`within` emit | Frontend only after bytecode freeze |
| **L3** | **RQL-D1** `at rank` / access | Needs DDA + shared bytecode |
| **L4** | Within-nested enrich index | Port into bytecode machine |
| **L5** | Post-attach global re-page / re-limit | Design on shared runtime |
| **L6** | **RQL-Q1** query perf | After shared runtime + C1 |

---

## Already done (do not re-do / do not grow)

| Item | State under Decision 0 |
|---|---|
| APP-5 Application Core compile | scoreboard **accept** (compile remains) |
| Phase 3 attach surface + corpus | labor **in_review** — **port inventory** |
| RQL-0 gap ledger | amended with **§0 Decision 0** |
| RQL-0D convergence charter | labor **in_review** |
| RQL-F1 / F2 / I1 | labor **in_review** — frozen façades; port later |

---

## One-line status

```text
NEXT labor  = RQL-X1 define query bytecode + host boundary
FROZEN      = query_exec_v1 + execute_rql_full feature growth
NOT next    = S1, D1, wire parity, more façade features
JUST LANDED = Decision 0 (RQL-0 §0 + QUERY_RUNTIME_CONVERGENCE.md)
```
