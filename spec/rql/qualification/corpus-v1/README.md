# RQL-Q1 practical query corpus (v1)

Machine-readable Gate-1 **intention corpus** for package **RQL-Q1**.

Authority: [`doc/todo/rql/RQL_QUERY_QUALIFICATION_PROGRAM.md`](../../../../doc/todo/rql/RQL_QUERY_QUALIFICATION_PROGRAM.md) §4  
Human report / amendment process: [`doc/todo/rql/RQL_Q1_CORPUS.md`](../../../../doc/todo/rql/RQL_Q1_CORPUS.md)  
Equivalence laws: [`doc/todo/rql/RQL_Q0_RESULT_EQUIVALENCE.md`](../../../../doc/todo/rql/RQL_Q0_RESULT_EQUIVALENCE.md)

## Non-claims

- This is **not** the APP-5 `rql_app_core` or Phase-3 full surface corpus under `spec/app/v1/`.
- Cases status `ready` (Q1.4) ≠ package accept ≠ Gate-1 pass. Floors **enforced** (`enforce_floors=true`).
- Decision 0 remains OPEN; this corpus does not close RQL-C1.

## Layout

| Path | Role |
|---|---|
| `corpus-v1.json` | Live versioned corpus document (`rql-q1-corpus-v0.4.0`, 153 cases) |
| `corpus-v1.schema.json` | Document wrapper schema (version, floors, amendment policy) |
| `corpus-case-v1.schema.json` | Per-case record contract (programme §4.2) |
| `fixtures/case.accepted.min.json` | Minimum complete case (validator positive control) |
| `fixtures/case.rejected.incomplete.json` | Incomplete case (validator negative control) |
| `generators/` | Seeded fixture generator specs (all five domains as of Q1.3) |

## Record contract (every case)

Required fields mirror programme §4.2:

`case_id`, `tier`, `domain`, `plain_english_intent`, `fixture` (generator + seed),
`expected` (literal / oracle / refusal), `ordering_and_multiplicity`,
`implementations` (RQL + Mongo + CBL), `indexes`, selectivity/cardinality classes,
`variants` (missing/null/type + cursor/page), `exclusion_or_refusal`.

Plus **`family_tags`** for §4.3 floor measurement (see floor policy in the corpus).

## Floors

Measured as: count of cases that list each family tag (overlap allowed).

| Family tag | Floor |
|---|---:|
| `selection_key_eq_range_compound` | 20 |
| `predicate_missing_null_type_nested_array` | 20 |
| `projection_computed_conditional` | 15 |
| `order_topk_cursor` | 15 |
| `enrichment_cardinality` | 15 |
| `group_aggregate` | 15 |
| `budget_coverage_damage_refusal` | 10 |

`floor_policy.enforce_floors` is **`true`** as of Q1.4 (`rql-q1-corpus-v0.4.0`).
Validator fails if any family tag falls below its floor.

## Amendment

1. Edit cases only under a versioned `corpus_version` bump.
2. Never silently redefine a frozen `case_id` — archive + replace; log both ids.
3. Principal review required (`amendment_policy.requires_principal_review`).
4. Full process: `doc/todo/rql/RQL_Q1_CORPUS.md`.

## Validate

```sh
bash scripts/verify-rql-q1-corpus.sh
```

Exit 0 means structural schema + fixture self-tests pass, floors are met under
`enforce_floors=true`, and predeclared_native_diff cases are not competitive on
Mongo/CBL. It does **not** mean Q1 package accept or Gate-1 pass.

## Q2.1 capability audit (compile + product execute)

Machine gap report (Tier A only): [`q2_1_capability_audit.json`](./q2_1_capability_audit.json)  
Human report: [`doc/todo/rql/RQL_Q2_1_CAPABILITY_AUDIT.md`](../../../../doc/todo/rql/RQL_Q2_1_CAPABILITY_AUDIT.md)

```sh
cargo test -p residiuum-sdk --test rql_q2_capability_audit -- --nocapture
```

Rewrites the gap report. Exit 0 = audit completed (every Tier-A case classified) —
**not** 100% expressible and **not** Gate-1.

## Tiers (Q1.4)

- **A** — Gate-1 mandatory intentions (may be `deferred_q2` until Q2 expressibility).
- **B** — important expansion; non-blocking unless promoted (2 cases in v0.4.0).
- **C** — explicitly deferred with stable refusal (4 cases in v0.4.0).

## Generators (Q1.2–Q1.3)

See [`generators/README.md`](./generators/README.md). Materialise:

```sh
python3 tools/rql_q1/materialise_fixture.py --generator commerce.orders_v1 --seed 1
python3 tools/rql_q1/materialise_fixture.py --generator directory.entries_v1 --seed 20
python3 tools/rql_q1/materialise_fixture.py --generator telemetry.events_v1 --seed 30
python3 tools/rql_q1/materialise_fixture.py --generator project_management.tasks_v1 --seed 40
```