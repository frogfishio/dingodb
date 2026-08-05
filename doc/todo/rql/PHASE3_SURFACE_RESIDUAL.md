# Phase 3 — `rql-full-v1` surface residual inventory

Status: **labor 2026-08-05** · T3.10 + **RQL-F1/F2 residual close**  
Profile: **`rql-full-v1`**  
Corpus: [`spec/app/v1/rql_full_v1_corpus_v1.json`](../../../spec/app/v1/rql_full_v1_corpus_v1.json)  
Kickoff: [PHASE3_FULL_RQL_KICKOFF.md](./PHASE3_FULL_RQL_KICKOFF.md)  
**What’s left:** [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md) · detail [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §5

## Delivered surface (honest)

| Area | State |
|---|---|
| Enrich `exactly_one` / `optional` / `many` | **yes** |
| Enrich candidate `where` | **yes** |
| Ordered root pipeline (`enrich` / `within` / post-attach `where`) | **yes** |
| Nested `within` (bounded) + nested enrich | **yes** |
| Nested `where` inside `within` | **yes** |
| Nested brace `project { … }` after pipeline | **yes** |
| Façade `execute_rql_full` (scan attach oracle) | **yes** |
| Application Core still rejects enrich/within | **yes** (refuse lock) |
| Structured `explain_rql_full` (pipeline + base plan) | **yes** (RQL-F1) |
| Op-118 Core wire refuse for full-language | **yes** (RQL-F2 — explicit refuse, not wire parity) |
| Pre-enrich root `where` stays in Core | **yes** (correct; not a defect) |

## Still open (not labor-closed this turn)

| Residual | Why open | Owner |
|---|---|---|
| `at rank` / `access` policies | DDA-dependent | **RQL-D1** |
| Index pushdown for enrich match keys | Scan oracle only; no index claim | **RQL-I1** |
| Remote op-118 enrich/within/project **parity** | F2 chose refuse; wire execute still absent | future after F2 decision |
| Post-attach `where` global re-page / re-limit | Page-then-attach filters within Core page | design / later package |
| Package accept / APB-7 accept | Principal gate | **RQL-C1** |

## Evidence commands

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --test rql_full_corpus -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_explain -- --test-threads=1
```

Logs: `doc/todo/rql/evidence/phase3_corpus.log`,
`doc/todo/rql/evidence/phase3_explain_f1f2.log`

## Non-claim

This inventory locks the **delivered attach-class surface + explain + wire
refuse**. It does **not** claim full RQL-v1 product readiness, query
qualification, index-backed enrich, or op-118 enrich parity.
