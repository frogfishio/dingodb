# APP-0 — Application contract lock (`residiuum-app-v1`)

Status: **frozen for implementers (APP-0)**

Normative plan:
[`doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md`](../../../doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md).

This directory holds machine-readable contract artifacts that APP-1…APP-8
must not contradict without amending the CORE plan.

## Inventory

| Artifact | Path | Role |
|---|---|---|
| Error mapping | [`error_mapping_v1.json`](error_mapping_v1.json) | Condition → `ErrorCode` + diagnostic |
| Plan vectors | [`plan_vectors_v1.json`](plan_vectors_v1.json) | Canonical logical `rql-plan-v1` samples + BLAKE3 hashes (`rql-plan-encoding-v1`) |
| RQL Application Core corpus | [`rql_app_core_corpus_v1.json`](rql_app_core_corpus_v1.json) | APP-5 accept + reject matrix (`rql-app-core-v1`) |
| Cursor vectors | [`cursor_vectors_v1.json`](cursor_vectors_v1.json) | `residiuum-cursor-v1` field binding examples |
| Residuals | [`residuals_v1.json`](residuals_v1.json) | Named non-blocking placeholders / open APP-1 items |
| Compile surface | `crates/residiuum-sdk/src/app_v1.rs` | Public Rust types that **compile** |
| Contract test | `crates/residiuum-sdk/tests/app0_contract_lock.rs` | Loads vectors + checks codes |
| Wire schemas (staged) | `spec/heap/rpc-v1/collection_create.*.json`, `rql_query.*.json` | Op 106 / 118 shapes |
| Wire fixtures (staged) | `spec/heap/fixtures/collection_create.*`, `rql_query.*` | Accepted / rejected goldens |

## Operation activation policy

Op **106** (`collection_create`) is **active** (APP-1) with schema pointers in
[`spec/heap/operations-v1.json`](../../heap/operations-v1.json).
Op **118** (`rql_query`) remains `reserved` with **null** schema pointers until
APP-7.

APP-0 froze the **on-disk** request/response schemas and goldens; APP-1 wired
106. `scripts/check_heap_architecture.sh` still forbids non-null schema refs on
reserved ops (118).

## Profiles

| Profile | Meaning |
|---|---|
| `residiuum-rust-app-v1` | Public Rust application façade names |
| `rql-app-core-v1` | Accepted RQL Application Core source subset |
| `rql-plan-v1` | Logical plan shape |
| `residiuum-predicate-v1` | Shared predicate semantics (`residiuum_sdk::predicate`) |
| `rql-plan-encoding-v1` | Canonical plan JSON + domain-separated BLAKE3 (`residiuum_sdk::plan_v1`) |
| `residiuum-cursor-v1` | Authenticated continuation |
| `rpc-v1` | Heap wire envelope |

## Verify

```bash
bash scripts/verify-app0-contract.sh
cargo test -p residiuum-sdk --test app0_contract_lock
cargo test -p residiuum-sdk --test app4_predicate_plan
cargo test -p residiuum-sdk --test app5_rql_app_core
```