# Phase 3 — `rql-full-v1` surface residual inventory

Status: **labor 2026-08-05** · board `4994c4bd` (T3.10)  
Profile: **`rql-full-v1`**  
Corpus: [`spec/app/v1/rql_full_v1_corpus_v1.json`](../../../spec/app/v1/rql_full_v1_corpus_v1.json)  
Kickoff: [PHASE3_FULL_RQL_KICKOFF.md](./PHASE3_FULL_RQL_KICKOFF.md)  
Next packages: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §5

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

## Residuals (not this Phase 3 labor)

| Residual | Why open |
|---|---|
| `at rank` / `access` policies | DDA-dependent (PATH / RQL_SPEC order) |
| Index pushdown for enrich match keys | Scan oracle only today; no index claim |
| Remote op-118 enrich/within/project wire | Façade is HeapClient-local attach |
| Post-attach `where` global re-page / re-limit | Page-then-attach filters within Core page |
| Nested `where` at root before enrich as pipeline Filter | Pre-enrich `where` stays in Core (correct) |
| Full explain artifact for `rql-full-v1` | Spec § explain later |
| Package accept / APB-7 accept | Principal gate; corpus ≠ accept |

## Evidence commands

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --test rql_full_corpus -- --test-threads=1
```

Log: `doc/todo/rql/evidence/phase3_corpus.log`

## Non-claim

This inventory + corpus locks the **delivered attach-class surface**. It does **not**
claim full RQL-v1 product readiness, query qualification, or index-backed enrich.
