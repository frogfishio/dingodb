# RQL-Q2.2 wave 1 — `pkg_group_aggregate`

Status: **labor complete** (2026-08-08) · parent task Q2.2 continues via residual packages  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §5; [RQL_SPEC.md](../../wip/query/RQL_SPEC.md) §9a  
Q2.1 order rank 1: 16 Tier-A cases

## Delivered

1. **Spec freeze** — RQL_SPEC §9a (group by + count/sum/min/max/avg semantics).
2. **Plan** — `GroupAggSpec` / `AggregateSpec` / `AggFn` on `rql-plan-v1` (optional `group_agg` in canonical JSON when active).
3. **Parse** — Application Core / Full base accepts `group by` and flat aggregate project forms used by the Q1 corpus.
4. **Execute** — group/agg phase in Core `ProjectPaths` (`residiuum-query-ir-group-agg-v1`); full filtered bag before aggregate; page/order on group rows.
5. **Tests** — `cargo test -p residiuum-sdk --test rql_group_aggregate`; unit tests in `group_agg.rs`.
6. **Re-audit** — `cargo test -p residiuum-sdk --test rql_q2_capability_audit` → execute_ok **107**, `pkg_group_aggregate` **0** residual.

## Non-claims

- Not Q2 package accept / not 100% Tier A
- Not Q3 result-correctness oracle
- Not Decision 0 / RQL-C1 (IR residual honesty)
- Numeric aggregates skip null/absent/non-numeric; decimal strings may contribute

## Residual packages (claim from todo children)

| Package | Cases (post wave-1) |
|---|---:|
| `pkg_enrich_corpus_dialect` | 15 |
| `pkg_array_predicate_surface` | 0 (closed Q2.2c) |
| `pkg_budget_partial_coverage` | 0 (closed Q2.2d) |
| `pkg_computed_conditional_project` | 5 |
| `pkg_cursor_after_clause` | 5 |
| `pkg_enrich_semantics` | 1 |

## Evidence

- `spec/rql/qualification/corpus-v1/q2_1_capability_audit.json` (refreshed)
- `crates/residiuum-sdk/src/query_bytecode_v1/group_agg.rs`
- `crates/residiuum-sdk/tests/rql_group_aggregate.rs`
