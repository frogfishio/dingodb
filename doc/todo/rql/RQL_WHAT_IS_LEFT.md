# RQL — what is left to do

Status: **2026-08-05** · single page for principals  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md) (RQL → Atomics → Cluster)  
Detail ledgers: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) · [PHASE3_SURFACE_RESIDUAL.md](./PHASE3_SURFACE_RESIDUAL.md)

If this page disagrees with PATH §6 or chat summaries, **this page wins** until amended.

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | *(none staged)* | Last labor pull **RQL-I1** is in_review. Next code pull needs a new `todo` card (suggest **RQL-S1** or op-118 parity if principal wants wire). |
| **2** | **Human** | **RQL-C1** | Accept scoreboard **APP-6 / APP-7 / APB-7** when satisfied |

---

## Waiting on principal (not labor)

| # | Package | Action |
|---|---|---|
| **P1** | **RQL-C1** | Accept **APP-6 / APP-7 / APB-7** on the scoreboard |
| **P2** | Review queue | Accept/reject labor `in_review` (RQL-0, F1, F2, I1, Phase 3 T3.*, clarity) |

---

## Later (do not pull yet)

| # | Package | Why deferred |
|---|---|---|
| **L1** | Op-118 enrich/within **parity** | F2 closed with **refuse**; new package only if you reject refuse-path |
| **L2** | **RQL-S1** SQL+ → enrich/`within` emit | After I1 (done) — ready to pre-stage when you want |
| **L3** | **RQL-D1** `at rank` / access | Needs DDA / DIRECT_ACCESS freeze |
| **L4** | Within-nested enrich index pushdown | I1 covers **root** enrich only |
| **L5** | Post-attach global re-page / re-limit | Design honesty residual |
| **L6** | **RQL-Q1** query perf campaign | After Core accept (C1) |

---

## Already done (do not re-do)

| Item | State |
|---|---|
| APP-5 Application Core compile | scoreboard **accept** |
| Phase 3 attach surface + corpus | labor **in_review** |
| RQL-0 gap ledger | labor **in_review** |
| RQL-F1 `explain_rql_full` | labor **in_review** |
| RQL-F2 op-118 full-language **refuse** | labor **in_review** |
| **RQL-I1** root enrich equality-index pushdown | labor **in_review** (`rql_full_enrich_index` 2/2) |

---

## One-line status

```text
NEXT labor  = (none on todo) — pre-stage RQL-S1 or wire-parity if desired
NEXT human  = RQL-C1 accept APP-6/7/APB-7 + clear in_review
JUST SHIPPED = RQL-I1 root enrich index pushdown (scan differential)
NOT next    = at-rank, within-nested index, query perf
```
