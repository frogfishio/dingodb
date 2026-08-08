# RQL-Q2.2d — `pkg_budget_partial_coverage`

Status: **labor complete** (2026-08-08)  
Authority: Q2.1 residual; RQL_SPEC budgets; expert review of Q2.3 (Tier-A expressibility)  
Task: Q2.2d · Feature `019fda4c-1227-7c93-b7e6-292141ec7a78`

## Gap (pre-fix)

Five Tier-A cases compiled but **hard-failed** execute with `ResourceLimit` even when
the Q2 audit set `QueryRunOptions.coverage = IncompleteAllowed`:

| Case | Budget surface |
|---|---|
| `commerce.orders.budget_documents` | documents: 10 |
| `directory.entries.budget_documents` | documents: 12 |
| `project_management.tasks.budget_documents` | documents: 20 |
| `messaging.messages.budget_cancel_surface` | result_bytes: 4096 |
| `telemetry.events.budget_result_bytes` | result_bytes: 4096 |

Oracle: partial page + incomplete coverage when the bound is reached — not silent
truncation without evidence, and not hard abort under incomplete-allowed policy.

## Spec freeze (honest)

RQL_SPEC remains:

- under **`coverage complete`**: budget exhaust **fails closed** (`ResourceLimit` /
  budget-exceeded diagnostic);
- under **`coverage allow incomplete`** (plan) **or** run options
  `CoveragePolicy::IncompleteAllowed`: return **partial page** + hole code
  `budget_exhausted_{documents|bytes|result_bytes}` and incomplete coverage.

No budget authorizes silent truncation without hole evidence.

## Delivered

1. `CoreFrame` soft budget stop (`budget_truncated`) when partial allowed.
2. Pre-check document/bytes/result budgets; stop before including the over-budget unit.
3. Force incomplete coverage evidence when soft-stopped; no multipage cursor after
   budget soft-stop (budget is query-wide).
4. APB fail-closed dual-pack / multipage budget tests remain green (default Complete).
5. Re-audit: execute_ok **134**/147 (was 129); gap **11**; `pkg_budget_partial_coverage` **0**.

## Evidence

- `crates/residiuum-sdk/src/query_bytecode_v1/core_phases.rs`
- `cargo test -p residiuum-sdk --test rql_q2_capability_audit`
- `cargo test -p residiuum-sdk --test apb7_executor_harden --test apb7_query_dual_pack --test apb7_multipage_oracle_matrix --test app_core_execute_corpus`
- `spec/rql/qualification/corpus-v1/q2_1_capability_audit.json`

## Non-claims

- Not Q2 package accept / not 100% Tier A
- Not Decision 0 / RQL-C1
- Residual: computed conditional project (5), cursor `after` (5), enrich within (1)
