# RQL Phase 3 — Full language kickoff (labor)

Status: **labor 2026-08-04** · card `89a80e77` · profile **`rql-full-v1`**  
Authority: [PATH_TO_FULL_RQL.md](./PATH_TO_FULL_RQL.md) · [RQL_SPEC.md](../../wip/query/RQL_SPEC.md)

Promoted after principal **done** on PATH T1/T2. Application Core (`rql-app-core-v1`)
is **unchanged** and still rejects `enrich`.

## Kickoff slice (this turn)

| Deliverable | State |
|---|---|
| Gap inventory (this doc) | **yes** |
| `compile_rql_full` — strip enrich → Core compile + `EnrichStepV1` | **yes** |
| `attach_enrich_rows` — `exactly_one` / `optional` scan oracle | **yes** |
| Embedded integration `rql_full_enrich_kickoff` | **yes** |
| `within` / `at rank` / `expect many` / candidate `where` | **residual** |
| Façade `CollectionClient::rql` wiring for enrich | **residual** |
| Index pushdown for match keys | **residual** |
| Nested / chained enrich | **residual** |

## Two surfaces (honesty)

```text
rql-app-core-v1  → RqlPlanV1 → APB-7 executor     (product Core; no enrich)
rql-full-v1      → CompiledRqlFull (base + enrich) → attach oracle (kickoff)
dialects/rql v0.1 → ENR1+SDA                      (parallel legacy enrich)
```

Do **not** confuse dialect ENR enrich with the plan-encoded full-language path.

## Commands

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --lib rql_full_v1 -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_enrich_kickoff -- --test-threads=1
```

## Non-claims

- Not full RQL-v1 product accept.
- Not APB-7 package accept.
- Not “as expressive as SQL joins” until `many` + nested `within` land.
- Kickoff attach uses complete foreign scan — not an index claim.

## Next slices

1. Wire façade helper that opens bound collections + runs attach after base page.
2. `expect many` bag attach + cardinality oracles.
3. Enrich candidate `where` predicate.
4. `within` nested carrier.
5. Only then `at rank` / access policies (DDA-dependent).
