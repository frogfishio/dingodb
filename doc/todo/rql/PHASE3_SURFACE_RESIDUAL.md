# Phase 3 — `rql-full-v1` surface residual inventory

Status: **labor 2026-08-05** · T3.10 + F1/F2/I1 · **Decision 0: PORT INVENTORY ONLY**  
Profile: **`rql-full-v1`** (frozen façade — do not grow)  
Corpus: [`spec/app/v1/rql_full_v1_corpus_v1.json`](../../../spec/app/v1/rql_full_v1_corpus_v1.json)  
Kickoff: [PHASE3_FULL_RQL_KICKOFF.md](./PHASE3_FULL_RQL_KICKOFF.md)  
**Architecture:** [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

Under Decision 0, this inventory records what the illegal `execute_rql_full`
façade already demonstrated so **RQL-X2 can port it** into the one bytecode
runtime. It is not a growth roadmap for a second executor.

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
| Root enrich equality-index pushdown | **yes** (RQL-I1; scan fallback; differential oracle) |
| Pre-enrich root `where` stays in Core | **yes** (correct; not a defect) |

## Still open (as port targets / later packages — not façade growth)

| Residual | Why open | Owner |
|---|---|---|
| Shared bytecode runtime for attach | Decision 0 | **RQL-X2** |
| `at rank` / `access` policies | DDA-dependent | **RQL-D1** (after X1) |
| Within-nested enrich index pushdown | I1 is root enrich only | port in X2 / later |
| Remote op-118 enrich/within/project | Must be same runtime | **RQL-X2** |
| Post-attach `where` global re-page / re-limit | Design honesty | later on shared runtime |
| Package accept / APB-7 accept | Principal gate | **RQL-C1** after X2 honesty |

## Evidence commands

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --test rql_full_corpus -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_explain -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_enrich_index -- --test-threads=1
```

Logs: `doc/todo/rql/evidence/phase3_corpus.log`,
`doc/todo/rql/evidence/phase3_explain_f1f2.log`,
`doc/todo/rql/evidence/rql_i1_enrich_index.log`

## Non-claim

This inventory locks the **delivered attach-class surface + explain + wire
refuse** as **port evidence**. It does **not** claim full RQL-v1 product
readiness, a legitimate second executor, query qualification, or op-118 enrich
parity. Decision 0 forbids growing `execute_rql_full`.
