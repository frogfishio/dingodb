# APB-6 T1 — Stable read views gap inventory

Status: **scaffold 2026-08-01** · package `APB-6` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §10 · [PRODUCT_DEFICIENCIES.md](../../reference/product/PRODUCT_DEFICIENCIES.md) PD-008 · `spec/app/baseline-v1/` (`apb.heap.read_view`, `ReadView` type)

## Goal (T1)

1. Map normative fields to current code.
2. Land public façade types + `HeapClient::read_view` **fail-closed** on product observation claims.
3. Do **not** claim multi-page / multi-query snapshot consistency.

## Normative binding (ReadView)

| Field | Required by | Current labor |
|---|---|---|
| `heap_id` | types-v1 / MUST_ADD | **yes** — bound at open |
| `authoritative_frontier` | types-v1 | **stub** — live open generation id (not segment pin) |
| `coverage` | types-v1 | **declared** from options; not proven complete |
| `semantic_versions` | types-v1 | **yes** — frozen profile labels (plan/predicate/cursor/app-core) |
| `expiry` | MUST_ADD | **yes** — `max_age` → expires_at |
| `resource_budget` | MUST_ADD | **optional** — stored, not enforced as retention pin |
| mutation isolation | MUST_ADD | **not yet** — no reclamation pin / generation fence on scan |
| `view.collection(..).query` | PD-008 | **fail-closed** until pin lands |
| share across export/count/watch | MUST_ADD | residual |

## Existing primitives (reuse, do not reinvent)

| Primitive | Location | Role vs APB-6 |
|---|---|---|
| Store segment fingerprint | `residiuum_store::Store::segment_fingerprint` | Candidate authoritative frontier for embedded pin (not yet exposed on `HeapClient`) |
| Index build frontier | `residiuum-sdk` indexes meta | Index semantic versioning; not a heap read view |
| APP-6 cursors | `cursor_v1` + `query_exec_v1` | Generation-fenced restart / page continuation; **not** a snapshot |
| Coverage / consistency modes | `app_v1` | Declared on queries; not bound into a durable view object |
| APP-6 page executor | `query_exec_v1` | Live scan under no view pin |

## Dependency honesty

| Dep | Status | Note |
|---|---|---|
| APB-1 | **active** | `HeapClient` bound backends required |
| APB-3 | **not_started** | Lifecycle/capabilities not required for type scaffold; retention/reclaim pin may need APB-3 later |
| APP-6 T2 | **in_review** | Live `rql` exists; should eventually take optional `ReadView` |
| HAR-4 | **not_started** | Remote view pin / product remote posture residual |

## Scaffold API (this labor)

```text
HeapClient::read_view(ReadViewOptions) -> ReadView
ReadView::info() -> ReadViewInfo
ReadView::close()
ReadView::ensure_open()  // expired/closed → error
```

Product observation under a view (`collection` query/export) remains **fail-closed** with a stable residual message until:

1. embedded frontier pin (store segment fingerprint + reopen check),
2. retention budget enforcement,
3. optional view-bound executor path.

## Exit residual (package accept — not this task)

- mutation-between-pages under a view
- compaction / tier movement while pinned
- expiry / resource budget
- reopen tests
- dual-backend parity

## Explicit non-claims

- No product “snapshot isolation” claim.
- No inventable durable snapshot type beyond declared frontier binding.
- Generation-fenced cursors remain the live multipage path (APP-6).
