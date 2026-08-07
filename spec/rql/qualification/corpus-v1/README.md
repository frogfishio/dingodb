# RQL-Q1 practical query corpus (v1)

Machine-readable Gate-1 **intention corpus** for package **RQL-Q1**.

Authority: [`doc/todo/rql/RQL_QUERY_QUALIFICATION_PROGRAM.md`](../../../../doc/todo/rql/RQL_QUERY_QUALIFICATION_PROGRAM.md) §4  
Human report / amendment process: [`doc/todo/rql/RQL_Q1_CORPUS.md`](../../../../doc/todo/rql/RQL_Q1_CORPUS.md)  
Equivalence laws: [`doc/todo/rql/RQL_Q0_RESULT_EQUIVALENCE.md`](../../../../doc/todo/rql/RQL_Q0_RESULT_EQUIVALENCE.md)

## Non-claims

- This is **not** the APP-5 `rql_app_core` or Phase-3 full surface corpus under `spec/app/v1/`.
- Draft cases (Q1.2+) ≠ package accept ≠ Gate-1 pass. Floors not enforced until Q1.4.
- Decision 0 remains OPEN; this corpus does not close RQL-C1.

## Layout

| Path | Role |
|---|---|
| `corpus-v1.json` | Live versioned corpus document (`cases` grow in Q1.2–Q1.4) |
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

`floor_policy.enforce_floors` is **`false`** until Q1.4 / package exit (tag counts
already meet floors after Q1.3 bulk; enforcement still off).

## Amendment

1. Edit cases only under a versioned `corpus_version` bump.
2. Never silently redefine a frozen `case_id` — archive + replace; log both ids.
3. Principal review required (`amendment_policy.requires_principal_review`).
4. Full process: `doc/todo/rql/RQL_Q1_CORPUS.md`.

## Validate

```sh
bash scripts/verify-rql-q1-corpus.sh
```

Exit 0 means structural schema + fixture self-tests pass. It does **not** mean
floors are met or Q1 is accepted.

## Generators (Q1.2–Q1.3)

See [`generators/README.md`](./generators/README.md). Materialise:

```sh
python3 tools/rql_q1/materialise_fixture.py --generator commerce.orders_v1 --seed 1
python3 tools/rql_q1/materialise_fixture.py --generator directory.entries_v1 --seed 20
python3 tools/rql_q1/materialise_fixture.py --generator telemetry.events_v1 --seed 30
python3 tools/rql_q1/materialise_fixture.py --generator project_management.tasks_v1 --seed 40
```