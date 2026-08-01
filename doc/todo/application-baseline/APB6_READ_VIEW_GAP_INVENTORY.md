# APB-6 — Stable read views gap inventory

Status: **T1 scaffold + T2 embedded segment pin 2026-08-02** · package `APB-6` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §10 · [PRODUCT_DEFICIENCIES.md](../../reference/product/PRODUCT_DEFICIENCIES.md) PD-008 · `spec/app/baseline-v1/` (`apb.heap.read_view`, `ReadView` type)

## Goal

| Task | Deliverable | Status |
|---|---|---|
| T1 | Map normative fields; public types + `HeapClient::read_view` fail-closed on product observation | **done** (scaffold) |
| T2 | Embedded frontier pin via store segment fingerprint; drift re-check | **done** (this labor) |
| later | View-bound executor, retention pin, remote pin, package accept | residual |

## Normative binding (ReadView)

| Field | Required by | Current labor |
|---|---|---|
| `heap_id` | types-v1 / MUST_ADD | **yes** — bound at open |
| `authoritative_frontier` | types-v1 | **T2 embedded** — `FrontierKind::SegmentFingerprint` = `HeapStore::segment_fingerprint` hex; remote still `LiveUnpinned` |
| `coverage` | types-v1 | **declared** from options; not proven complete |
| `semantic_versions` | types-v1 | **yes** — frozen profile labels (plan/predicate/cursor/app-core) |
| `expiry` | MUST_ADD | **yes** — `max_age` → expires_at |
| `resource_budget` | MUST_ADD | **optional** — stored, not enforced as retention pin |
| mutation isolation | MUST_ADD | **partial** — `check_drift` detects segment-layout movement; **not** snapshot isolation |
| `view.collection(..).query` | PD-008 | **fail-closed** until view-bound executor |
| share across export/count/watch | MUST_ADD | residual |

## Existing primitives (reuse, do not reinvent)

| Primitive | Location | Role vs APB-6 |
|---|---|---|
| Store segment fingerprint | `HeapStore::segment_fingerprint` / `Heap::segment_fingerprint` | **T2 pin** for embedded ReadView |
| Index build frontier | `residiuum-sdk` indexes meta | Index semantic versioning; not a heap read view |
| APP-6 cursors | `cursor_v1` + `query_exec_v1` | Generation-fenced restart / page continuation; **not** a snapshot |
| Coverage / consistency modes | `app_v1` | Declared on queries; not bound into a durable view object |
| APP-6 page executor | `query_exec_v1` | Live scan under no view pin |

## API (current)

```text
HeapClient::read_view(ReadViewOptions) -> ReadView
  // Embedded → SegmentFingerprint pin, observation_pinned=true
  // Remote   → LiveUnpinned residual, observation_pinned=false

ReadView::info() -> ReadViewInfo
ReadView::close()
ReadView::ensure_usable()
ReadView::check_drift() -> FrontierDrift { Stable | Drifted | Unpinned }
ReadView::refresh_pin()  // re-sample segment fingerprint (pinned only)
ReadView::pinned_fingerprint() -> Option<[u8;32]>
ReadView::open_collection(name)  // fail-closed (view-bound executor residual)
```

## Dependency honesty

| Dep | Status | Note |
|---|---|---|
| APB-1 | **active** | `HeapClient` bound backends required |
| APB-3 | **not_started** | Lifecycle/capabilities not required for pin; retention/reclaim pin may need APB-3 later |
| APP-6 T2 | **in_review** | Live `rql` exists; should eventually take optional `ReadView` |
| HAR-4 | **not_started** | Remote view pin / product remote posture residual |

## Residuals (package accept — not this task)

- view-bound `query_exec_v1` / export under a pin
- mutation-between-pages isolation proof (beyond segment drift token)
- compaction / tier movement while pinned (retention)
- expiry / resource budget enforcement as reclamation pin
- dual-backend parity for pin (remote residual)
- reopen tests after process restart

## Explicit non-claims

- No product “snapshot isolation” claim.
- `observation_pinned = true` means segment-fingerprint frontier is bound and re-checkable — **not** that multi-page observation is frozen.
- Generation-fenced cursors remain the live multipage path (APP-6).
- No APB-6 package accept until exit residuals close.
