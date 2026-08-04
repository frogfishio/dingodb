# RQL Phase 2 — Core expressiveness + refusal (labor)

Status: **labor 2026-08-04** · package `APB-7` still **active / not accept**  
Authority: [PATH_TO_FULL_RQL.md](./PATH_TO_FULL_RQL.md) · [SQL_TO_RQL_SPEC.md](./SQL_TO_RQL_SPEC.md)

Board card: RQL PATH **T2** `b4ebdaf9` (Query spine Feature `1a8a3e05`).

## Verdict (Phase 2 question)

**Application Core is SQL-filter-class expressive** for filter / project / order /
page / budget / coverage shapes, with an explicit **refuse** matrix for joins,
aggregates, offset, and SQL DML. **Joins / enrich remain pending** full RQL-v1
(Phase 3; board `89a80e77` stays backlog until Phase 1 package accept).

## Delivered this turn

| Artifact | Role |
|---|---|
| `crates/residiuum-sdk/src/sql_plus.rs` | Pure `compile_sql_to_rql` emit/refuse scaffold (`residiuum-sql-plus-to-rql-v1`) |
| `spec/app/v1/sql_plus_corpus_v1.json` | Emit + refuse corpus |
| `tests/sql_plus_corpus.rs` | Corpus host |
| `tests/app_core_expressiveness.rs` | Gotchas: **absent≠null**, coverage, budgets |
| Evidence | `doc/todo/rql/evidence/phase2_expressiveness.log` |

### SQL-ish+ honesty

- `IS NULL` emits `(missing(path) or path is null)` with a mapping note (SQL
  document-view collapse; Core retains the distinction).
- JOIN / GROUP BY / OFFSET / aggregates / LIKE / CTE / DML → **refuse**
  (`sql_rql_construct_unsupported` / `sql_rql_statement_unsupported`).

### Core gotchas locked

- `missing(field)` ≠ `field is null` ≠ `present(field)`.
- Budgets fail closed (`ResourceLimit`).
- `coverage incomplete` compiles and runs on a quiet collection.

## Non-claims

- Not package accept for APB-7 / APP-6 / APP-7.
- Not a complete SQL-ish+ product compiler (scaffold; decimals / ACCESS /
  CONTINUE / conditional RRE joins still out).
- Not full RQL-v1 enrich.

## Next

1. Principal Phase 1 accept (T1 residual) when ready.
2. Phase 3: promote `89a80e77` only after that accept.
3. Optional: deepen sql+ (BETWEEN refuse already; decimals; conditional emit).
