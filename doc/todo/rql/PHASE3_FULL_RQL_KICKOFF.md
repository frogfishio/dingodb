# RQL Phase 3 — Full language kickoff (labor)

Status: **labor 2026-08-05** · kickoff `89a80e77` + T3.2–**T3.4** (`0c101b14` where, `8ac147b5` within)  
Profile **`rql-full-v1`**  
Authority: [PATH_TO_FULL_RQL.md](./PATH_TO_FULL_RQL.md) · [RQL_SPEC.md](../../wip/query/RQL_SPEC.md)

Promoted after principal **done** on PATH T1/T2. Application Core (`rql-app-core-v1`)
is **unchanged** and still rejects `enrich` / `within`.

## Delivered slices

| Deliverable | State |
|---|---|
| Gap inventory (this doc) | **yes** |
| `compile_rql_full` — strip enrich → Core compile + `EnrichStepV1` | **yes** |
| `attach_enrich_rows` — `exactly_one` / `optional` / **`many`** | **yes** |
| Embedded integration `rql_full_enrich_kickoff` | **yes** |
| Façade `execute_rql_full` | **yes** (T3.2) |
| Tests `rql_full_many_facade` | **yes** |
| Enrich candidate `where` | **yes** (T3.3) |
| `within` nested carrier (one depth, one nested enrich) | **yes** (T3.4) |
| Nested `within` / multi enrich inside within / chained root enrich | **residual** |
| `at rank` / access | **residual** |
| Index pushdown for match keys | **residual** |

## Two surfaces (honesty)

```text
rql-app-core-v1  → RqlPlanV1 → APB-7 executor     (product Core; no enrich/within)
rql-full-v1      → CompiledRqlFull → execute_rql_full (base + enrich + within)
dialects/rql v0.1 → ENR1+SDA                      (parallel legacy enrich)
```

`expect many` attaches a JSON array ordered by foreign document key.
Optional enrich `where …` filters foreign candidates with Core `Predicate` eval before match.
`within path [as alias] { enrich … }` enriches each array element; absent/non-array → `rql_within_type`.
Foreign load is complete `list_keys`+`get` — **not** an index claim.

## Commands

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --lib rql_full_v1 -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_enrich_kickoff -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_many_facade -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_candidate_where -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_within -- --test-threads=1
```

Evidence: `doc/todo/rql/evidence/phase3_within.log`

## Non-claims

- Not full RQL-v1 product accept.
- Not APB-7 package accept.
- Nested `within` and multi/chained enrich still residual.
- Façade is HeapClient-local scan attach — not remote op-118 enrich wire.

## Next slices

1. Chained / multi enrich (root or inside within).
2. Nested `within` depth.
3. Only then `at rank` / access policies (DDA-dependent).
