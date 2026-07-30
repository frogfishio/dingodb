# APP-0 — Application contract lock (`dingo-app-v1`)

Status: **frozen for implementers (APP-0)**

Normative plan:
[`doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md`](../../../doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md).

This directory holds machine-readable contract artifacts that APP-1…APP-8
must not contradict without amending the CORE plan.

## Inventory

| Artifact | Path | Role |
|---|---|---|
| Error mapping | [`error_mapping_v1.json`](error_mapping_v1.json) | Condition → `ErrorCode` + diagnostic |
| Plan vectors | [`plan_vectors_v1.json`](plan_vectors_v1.json) | Canonical logical `dql-plan-v1` samples + hashes |
| Cursor vectors | [`cursor_vectors_v1.json`](cursor_vectors_v1.json) | `dingo-cursor-v1` field binding examples |
| Residuals | [`residuals_v1.json`](residuals_v1.json) | Named non-blocking placeholders / open APP-1 items |
| Compile surface | `crates/dingo-sdk/src/app_v1.rs` | Public Rust types that **compile** |
| Contract test | `crates/dingo-sdk/tests/app0_contract_lock.rs` | Loads vectors + checks codes |
| Wire schemas (staged) | `spec/heap/rpc-v1/collection_create.*.json`, `dql_query.*.json` | Op 106 / 118 shapes |
| Wire fixtures (staged) | `spec/heap/fixtures/collection_create.*`, `dql_query.*` | Accepted / rejected goldens |

## Operation activation policy

Ops **106** (`collection_create`) and **118** (`dql_query`) remain
`reserved` in [`spec/heap/operations-v1.json`](../../heap/operations-v1.json)
with **null** schema pointers until APP-1 / APP-7 activate them.

APP-0 still freezes the **on-disk** request/response schemas and goldens so
implementers share one contract. `scripts/check_heap_architecture.sh` forbids
non-null schema refs on reserved ops — that is intentional.

## Profiles

| Profile | Meaning |
|---|---|
| `dingo-rust-app-v1` | Public Rust application façade names |
| `dql-app-core-v1` | Accepted DQL Application Core source subset |
| `dql-plan-v1` | Logical plan shape |
| `dingo-predicate-v1` | Shared predicate semantics |
| `dingo-cursor-v1` | Authenticated continuation |
| `rpc-v1` | Heap wire envelope |

## Verify

```bash
bash scripts/verify-app0-contract.sh
cargo test -p dingo-sdk --test app0_contract_lock
```