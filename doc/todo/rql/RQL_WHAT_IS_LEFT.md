# RQL — what is left to do

Status: **2026-08-05** · single page for principals  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md) (RQL → Atomics → Cluster)  
Detail ledgers: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) · [PHASE3_SURFACE_RESIDUAL.md](./PHASE3_SURFACE_RESIDUAL.md)

If this page disagrees with PATH §6 or chat summaries, **this page wins** until amended.

---

## Do next (exactly one labor pull)

| # | Who | Package | Board | What “done” means |
|---|---|---|---|---|
| **1** | **Labor** | **RQL-I1** — index pushdown for enrich match keys | `todo` `dc4ee028` | Equality match keys: scan vs index differential oracle; no false index claim |

That is the only code pull staged on the board for keep-going.

---

## Waiting on principal (not labor)

| # | Package | Action |
|---|---|---|
| **P1** | **RQL-C1** | Accept scoreboard rows **APP-6 / APP-7 / APB-7** when you are satisfied with evidence (labor must not self-accept) |
| **P2** | Review queue | Accept or reject labor already in `in_review` (RQL-0, F1, F2, Phase 3 T3.* cards) — workflow only; does not change scoreboard by itself |

---

## Later (do not pull yet)

| # | Package | Why blocked / deferred |
|---|---|---|
| **L1** | Op-118 enrich/within **parity** | F2 closed with **refuse**; parity is a *new* package only if you reject the refuse-path exit |
| **L2** | **RQL-S1** SQL+ → enrich/`within` emit | After I1 (and honest wire story) |
| **L3** | **RQL-D1** `at rank` / access | Needs DDA / DIRECT_ACCESS freeze — do not invent early |
| **L4** | Post-attach global re-page / re-limit | Design honesty residual; not Gate-1 blockers for I1 |
| **L5** | **RQL-Q1** query perf campaign | After Core accept (C1) minimum; enrich costs after I1 |

---

## Already done (do not re-do)

| Item | State |
|---|---|
| Application Core compile (APP-5) | scoreboard **accept** |
| Op 118 Core wire + dual-pack labor | **active**, evidence in_review — **not** package accept |
| Phase 3 attach surface (enrich/within/project + corpus) | labor **in_review** |
| RQL-0 gap ledger | labor **in_review** |
| RQL-F1 `explain_rql_full` | labor **in_review** |
| RQL-F2 op-118 full-language **refuse** | labor **in_review** |

---

## One-line status

```text
NEXT labor  = RQL-I1 (enrich index pushdown)          ← board todo
NEXT human  = RQL-C1 (accept APP-6/7/APB-7) + review in_review cards
NOT next    = at-rank, SQL JOIN emit, query perf, op118 enrich parity
```
