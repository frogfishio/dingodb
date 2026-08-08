# RQL-Q3.2 — Differential matrix + metamorphic laws

Status: **labor complete → board `in_review`** (2026-08-08) · package **not accepted**  
Package: RQL-Q3 · Feature `019fda4c-5994-77e2-a2c9-aaa0c3097b29` · Task Q3.2  
Depends: Q3.1 independent oracle  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §6.2–6.3  
Equivalence: [RQL_Q0_RESULT_EQUIVALENCE.md](./RQL_Q0_RESULT_EQUIVALENCE.md)

## 1. Goal

Prove result correctness **independently of the product optimiser** by wiring:

```text
reference_oracle(Q)
  == forced_scan_QVM(Q)
  == admitted_index_plan(Q)
  == reopened_store(Q)
  == comparator_result(Q) where semantics overlap   // deferred Q4 (Mongo/CBL)
```

Compare **values, keys, multiplicity, order (when declared), coverage** — never
row count alone. Failures are **defects**, not warnings.

## 2. Implementation

| Arm | How |
|---|---|
| `reference_oracle` | Same model as Q3.1: full logical-fixture scan + `Predicate::eval` |
| `forced_scan_QVM` | `QueryBytecodeV1::from_core_plan_force_scan(..., true)` + `execute_bytecode` on `DiffHost` |
| `admitted_index_plan` | Same host with `lookup_index_keys` equality indexes; `force_scan=false` |
| `reopened_store` | Seed embedded Heap, `CollectionClient::rql`, drop writer, reopen deployment, re-query (budget 12 cases) |
| `comparator` | Explicitly `deferred_q4` (cross-engine harness owns Mongo/CBL) |

Suite: `crates/residiuum-sdk/tests/rql_q3_differential_matrix.rs`  
Command: `cargo test -p residiuum-sdk --test rql_q3_differential_matrix`  
Report: `spec/rql/qualification/corpus-v1/q3_2_differential_report.json`

### Product fix (defect found by matrix)

Core filter now injects logical `_key` when the store body omits it
(`with_logical_key` in `query_bytecode_v1/core_phases.rs`). Corpus `where _key = …`
and `project _key` now match the independent oracle. This is not an optimiser
change — it restores key-path semantics on the product path.

## 3. Metamorphic laws (unit)

| Law | Test |
|---|---|
| `filter(A and B) = filter(B and A)` | `q32_law_filter_and_commutes` |
| project identity (full doc when no project) | `q32_law_project_identity_on_full_doc` |
| equivalent frontends → identical QVM (builder/RQL) | `q32_law_frontend_qvm_identity_builder_rql` |
| SQL↔RQL QVM identity when SQL emits | `q32_law_sql_and_rql_qvm_identity_when_sql_compiles` |
| `indexed(Q) = forced_scan(Q)` | `q32_law_force_scan_equals_index_on_eq_predicate` |
| complete coverage ⇒ zero holes | `q32_law_complete_coverage_implies_zero_holes` |

Page-concat under `after $cursor` remains APP-6 residual (same five corpus
cases as Q3.1 unsupported).

## 4. Evidence (latest labor)

| Metric | Value |
|---:|---:|
| Tier-A `oracle_rule` + source considered | **106** |
| `matrix_equal` (oracle=scan=index + coverage) | **101** |
| `unsupported` (`after` / APP-6) | **5** |
| `matrix_diverge` / errors / reopen_fail | **0** |
| reopen_store checked | **12** (budget; all equal) |
| Metamorphic unit tests | **6/6** |

## 5. Non-claims

- Not Gate-1; not RQL-Q3 package accept.
- Q3.3 adversarial/damage/one-command: see [RQL_Q3_3_ADVERSARIAL_SUITE.md](./RQL_Q3_3_ADVERSARIAL_SUITE.md) (`bash scripts/verify-rql-q3.sh`).
- Mongo/CBL comparator deferred to Q4 harness.
- Decision 0 / RQL-C1 still open.

## 6. Exit checklist (Q3.2)

- [x] Differential arms wired; full shape compare
- [x] Failures fail the suite
- [x] Metamorphic laws as hard unit tests
- [x] Machine report + scoreboard note
- [ ] Principal package accept (not labor)