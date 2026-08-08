# RQL-Q3.3 — Adversarial + damage + one-command green suite

Status: **labor complete → board `in_review`** (2026-08-08) · package **not accepted**  
Package: RQL-Q3 · Feature `019fda4c-5994-77e2-a2c9-aaa0c3097b29` · Task Q3.3  
Depends: Q3.1 independent oracle · Q3.2 differential matrix  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §6.4 + exit  

## 1. Goal

Close the Q3 residual: adversarial dimensions, damage honesty, and a **single
command** that greens corpus + property + fuzz + damage evidence for principal
package review.

## 2. Implementation

| Artefact | Path |
|---|---|
| Suite | `crates/residiuum-sdk/tests/rql_q3_adversarial.rs` |
| One-command gate | `bash scripts/verify-rql-q3.sh` |
| Machine report | `spec/rql/qualification/corpus-v1/q3_3_adversarial_report.json` |
| Runtime copy | `target/rql-q3/q3_3_adversarial_report.json` |

`verify-rql-q3.sh` runs **Q3.1 + Q3.2 + Q3.3** (oracle, differential matrix,
adversarial) and asserts machine-report floors. Exit 0 = labor evidence only —
**not** package accept.

## 3. §6.4 dimensions covered

| Dimension | Test | Honesty law |
|---|---|---|
| Sparse / heterogeneous | `q33_sparse_heterogeneous_*` | oracle = force_scan = complete index |
| Absent / null / wrong-type | `q33_missing_not_null_and_wrong_type_range` | missing≠null; no string→number coerce |
| Empty / nested arrays | `q33_array_empty_and_contains` | `tags = []` / `contains` fail-closed on non-array |
| Order ties | `q33_order_ties_break_on_immutable_key` | score + `_key` stable break |
| Partial / stale index | `q33_stale_partial_index_*` | force_scan full truth; partial under-return **detected** (not silent equal) |
| Mutated QVM | `q33_mutated_qvm_refuses_validate` | `validate_qvm` refuses |
| Mutated continuation | `q33_mutated_continuation_token_fails_closed` | resume fails closed |
| Reopen | `q33_reopen_preserves_results` | pre-close = post-open multiset |
| Inter-page writes | `q33_inter_page_write_under_available_consistency` | Available: no crash; first page stable |
| Damage / holes | `q33_holey_complete_*` / `q33_holey_incomplete_*` | Complete fail-closed; IncompleteAllowed reports holes (no false complete) |
| Budget / cancel / deadline | `q33_budget_cancel_deadline_*` + soft-stop | ResourceLimit / DeadlineExceeded; soft-stop no false world |
| Seeded property (24) | `q33_seeded_property_sparse_*` | force_scan = index = status oracle |
| Enrich cardinality | `q33_enrich_exactly_one_violation_*` | exactly_one zero-match not fabricate |

## 4. Evidence (latest labor)

| Metric | Value |
|---:|---:|
| Adversarial unit + property tests | **16/16** |
| Property seeds | **24** |
| False absence / false completeness defects | **0** |
| Unresolved force_scan↔index diverge (complete index) | **0** |
| Q3.1 `oracle_ok` floor (via one-command) | ≥90 |
| Q3.2 `matrix_equal` floor (via one-command) | ≥90 |

Command:

```sh
bash scripts/verify-rql-q3.sh
# or focused:
cargo test -p residiuum-sdk --test rql_q3_adversarial
```

## 5. Non-claims

- Not Gate-1; **not RQL-Q3 package accept** (principal).
- Not Q4 cross-engine harness (Mongo/CBL).
- Decision 0 / RQL-C1 still open.
- Rotation/compaction physical media cells remain product residual beyond
  reopen equality (embedded re-open is the labor proxy here).
- Full-RQL multipage SI under concurrent writes not claimed (Available only).

## 6. Exit checklist (Q3.3)

- [x] Adversarial dimensions as hard tests (failures = defects)
- [x] No false absence / false completeness under holey + budget soft-stop
- [x] force_scan equals complete admitted index (property + sparse hand fixtures)
- [x] One-command green script unifies corpus+property+damage (Q3.1–Q3.3)
- [x] Machine report + human pack
- [ ] Principal package accept (not labor)

## 7. Principal accept gate (package RQL-Q3)

When principal reviews Q3.1–Q3.3 together:

1. `bash scripts/verify-rql-q3.sh` exit 0
2. Reports under `spec/rql/qualification/corpus-v1/q3_{1,2,3}_*.json`
3. Residual inventory accepted (page-concat `after`, Q4 comparator, Decision 0)
4. Scoreboard RQL-Q3 → `accept` only by principal
