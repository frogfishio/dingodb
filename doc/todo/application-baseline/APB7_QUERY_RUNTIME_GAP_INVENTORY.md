# APB-7 T0 — Query runtime gap inventory

Status: **T0 inventory + T1 façade builder 2026-08-02** · package `APB-7` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §11 · [PRODUCT_DEFICIENCIES.md](../../reference/product/PRODUCT_DEFICIENCIES.md) PD-009 ·
[`spec/app/baseline-v1/operations-v1.json`](../../../spec/app/baseline-v1/operations-v1.json) ·
scoreboard `NEXT_BUILD_STATUS.md`

This document maps **APB-7 deliverables to current labor**. It does **not** claim
product query, remote `rql_query` wire, or package accept.

---

## 1. Package goal (normative)

MUST_ADD §11 / PD-009:

```rust
collection.query()           // builder path
collection.rql(source, parameters, options)
collection.explain_rql(...)
```

**Required:**

| Requirement | Normative intent |
|---|---|
| Canonical predicate/plan | APP-4/5 profiles; builder ↔ RQL same plan hash |
| Projection, scalar order, limit, bounded page, continuation | Application Core execution |
| Complete-by-default coverage | Coverage evidence on pages; no silent holes |
| Budgets, deadline, cancellation | Resource governance |
| Index-versus-scan correctness | Same results via index pushdown vs full scan |
| Authenticated Heap/collection/view/plan/parameter-bound cursors | `residiuum-cursor-v1` product binding |
| Embedded/remote parity | Same façade; remote via product wire |

**Exit (package, later):** builder and RQL compile to the same plan; all pages
reconcile with an independent complete-scan oracle; both backends.

**This T0 inventory does not meet exit.**

---

## 2. Dependency honesty

| Dep | Scoreboard | Inventory note |
|---|---|---|
| APP-4 | **accept** | `predicate` + `plan_v1` + plan vectors locked |
| APP-5 | **accept** | `rql_app_core` / `compile_app_core` → `RqlPlanV1`; corpus + fuzz |
| APP-6 | **active** (T1/T2 in_review) | Cursor mint/verify + `query_exec_v1` page executor |
| APB-1 | **active** | `CollectionClient` dual backend; `DocScan` over list_keys+get |
| APB-6 | **active** (T1/T2 in_review) | ReadView pin (embedded segment FP); **not** view-bound query |
| APP-3 | **active** | Data plane for scan (put/get/list_keys) |
| HAR-4 | **not_started** | Qualified remote posture residual for product remote query |
| APP-7 | **not_started** | Wire op **118** `rql_query` remains **reserved** |

**Unblock vs product claim:** APP-4/5/6 + APB-1 labor is enough to *inventory*
and later implement package slices. Product query language still requires package
accept + wire activation + HAR path honesty.

---

## 3. Baseline registry (APB-7 app ops)

From `operations-v1.json` (`must_add_package: APB-7`):

| `app_id` | Façade methods (registry) | Wire | Labor status |
|---|---|---|---|
| `apb.collection.scan_json` | `CollectionClient::scan_json` | **115** `scan_json` **active** | **gap** — wire exists on `RemoteHeap` / legacy `Collection`; **not** on neutral `CollectionClient` |
| `apb.collection.find` | `CollectionClient::find`, `collection.query()` | **116** `find` **active** | **gap** — builder on legacy `Collection`; façade `query`/`find` missing |
| `apb.collection.rql` | `CollectionClient::rql` | **118** `rql_query` **reserved** | **partial** — embedded/remote *collection-plane* executor via APP-6; **not** product wire |
| `apb.collection.explain_rql` | `CollectionClient::explain_rql` | **118** (explain projection) **reserved** | **partial** — plan tree + hash, no row scan; not wire |
| `apb.collection.sda_query` | `CollectionClient::sda_query` | **119** `sda_query` **reserved** | **optional / out of Core first cut** — flat `sda_query` only |

Staged schemas exist under `spec/heap/rpc-v1/rql_query.*` + fixtures, but
registry still marks 118 reserved with null schema links in baseline ops —
honest residual until APP-7/APB-7 activate wire.

---

## 4. What exists today (precursor)

### 4.1 Compile plane (APP-4 / APP-5) — **ready for APB-7**

| Piece | Location | Notes |
|---|---|---|
| Predicate AST + eval | `residiuum_sdk::predicate` | Totality model; name-binding fail-closed |
| Plan encoding | `plan_v1` / `PlanBuilder` | `rql-plan-encoding-v1`; plan_hash vectors locked |
| RQL Application Core | `rql_app_core::compile_app_core` | Profile `rql-app-core-v1` (not full RQL v1) |
| Builder ↔ RQL hash parity | APP-5 tests / corpus | Golden: compile equals PlanBuilder for fixtures |
| Non-Core reject | `rql_feature_unavailable` | enrich / within / at rank / access; source `after` deferred |

### 4.2 Execute plane (APP-6) — **scaffold, not product**

| Piece | Location | Notes |
|---|---|---|
| Cursor mint/verify | `cursor_v1` | Vector-lock key material; product secrets residual |
| Page executor | `query_exec_v1` | list_keys + get + `Predicate::eval` |
| `CollectionClient::rql` | `app_v1` | Bound clients only; `DocScan` both backends |
| `CollectionClient::explain_rql` | `app_v1` | Plan tree + hash |
| Page size / after / limit | `QueryRunOptions` | Continuation via mint; remaining_limit |
| Budget | merge source + options | **max_documents** enforced; max_bytes / max_result_bytes residual |
| Coverage evidence | `QueryPage.coverage` | Stub-complete when no list/get holes; not DEF-100 grade |
| Consistency evidence | mode echo | Not bound to ReadView pin |

### 4.3 Read views (APB-6) — **orthogonal residual**

| Piece | Status |
|---|---|
| Embedded segment-fingerprint pin | T2 landed; `check_drift` / `refresh_pin` |
| `ReadView::open_collection` / view-bound rql | **fail-closed** |
| Multipage under snapshot isolation | **not claimed** |

### 4.4 Legacy / parallel paths (do not confuse with façade product)

| Path | Role |
|---|---|
| `Collection::query` / `find` / `scan_json` | Legacy flat collection surface |
| `RemoteHeap::find` / `scan_json` | Wire ops 116/115 |
| `filter::QueryBuilder` | Store-era builder, not APB façade |
| Dialects / multi_query / SDA | Non-Core examination |

---

## 5. Gap matrix (MUST_ADD requirements → labor)

| # | Requirement | Status | Gap / next labor |
|---:|---|---|---|
| G1 | `CollectionClient::rql` façade | **partial** | Exists (APP-6); must not claim product until APB-7 exit + wire |
| G2 | `CollectionClient::explain_rql` | **partial** | Plan explain only; no cost model / index choice tree |
| G3 | `collection.query()` builder on façade | **T1 partial** | `CollectionClient::query()` → `CollectionQuery` wraps `PlanBuilder`; compile/run/explain; not product |
| G4 | Builder plan_hash == RQL plan_hash | **T1 partial** | Façade path tested vs `compile_app_core` for equality filter + project/order/limit |
| G5 | Projection | **partial** | Executor projects; deep field-order residual in APP-6 notes |
| G6 | Scalar order + key tie-break | **partial** | Key-stream + full-scan+sort paths; deep order residual |
| G7 | Limit + page + continuation | **partial** | Working under vector-lock keys; product cursor secrets residual |
| G8 | View / parameter bound in cursor | **gap** | Cursor binds heap/collection/plan; **not** ReadView id / full parameter MAC set |
| G9 | Complete-by-default coverage | **stub** | Hole evidence for list/get race only; DEF-100 / scan_json façade gap |
| G10 | Budgets max_bytes / max_result_bytes | **gap** | Documents budget only in executor |
| G11 | Deadline / cancellation | **missing** | No cooperative cancel token on façade |
| G12 | Index-versus-scan oracle | **missing** | No pushdown; IndexManager unused by executor |
| G13 | Independent complete-scan oracle suite | **partial** | APP-6 equality tests; not full dual-path differential |
| G14 | Remote op **118** product path | **blocked** | Wire reserved; APP-7 + HAR-4 |
| G15 | Remote parity without inventing wire | **partial** | Remote `rql` today = collection-plane list_keys+get (same as embedded) — honest, not product `rql_query` |
| G16 | `scan_json` / `find` on façade | **missing** | Wire active; façade methods absent |
| G17 | View-bound observation under ReadView | **blocked on APB-6 residual** | Pin exists; executor not view-bound |
| G18 | sda_query on façade | **deferred** | Optional; reserved 119 |
| G19 | Dual-backend product parity pack | **missing** | Need APB-7 scenario pack (like APB-1 G6) after wire |
| G20 | Package accept language | **forbidden** | Until exit tests + wire + scoreboard accept |

---

## 6. Suggested labor slices (not started this turn)

Ordered for honesty; board cards may refine:

| Task | Deliverable | Depends |
|---|---|---|
| **T1** | Façade `query()` / PlanBuilder path → same plan as RQL; no new wire | **done** (2026-08-02 labor; not package accept) |
| **T2** | Hardening APP-6 executor: bytes budgets, field-order matrix, scan oracle expansion | APP-6 |
| **T3** | Coverage evidence upgrade + façade `scan_json` (op 115) projection | DEF-100 / wire 115 |
| **T4** | Index pushdown + scan/index differential oracle | IndexManager ready |
| **T5** | Optional ReadView-bound page path (fail-open only when pin stable) | APB-6 pin |
| **T6** | APP-7: activate op 118 schemas in registry + server dispatch + RemoteHeap | HAR-4 path |
| **T7** | Dual-pack product parity + scoreboard accept checklist | T1–T6 |

T0 does **not** implement any of the above.

---

## 7. Explicit non-claims

- No product “query baseline” or APB-7 package accept.
- No claim that `CollectionClient::rql` is qualified product query.
- Op **118** remains **reserved** until APP-7/APB-7 activate wire.
- No snapshot isolation / view-bound multi-page product claim.
- No index-correctness or full remote product parity claim.
- Aggregates (APB-8), watches (APB-9) are out of scope.

---

## 8. Evidence pointers

| Artifact | Role |
|---|---|
| `crates/residiuum-sdk/src/{predicate,plan_v1,rql_app_core,cursor_v1,query_exec_v1,read_view_v1,app_v1}.rs` | Labor surfaces |
| `crates/residiuum-sdk/tests/app{4,5,6}_*` / `apb6_read_view_scaffold` | Package precursors |
| `spec/app/v1/{plan_vectors,rql_app_core_corpus,cursor_vectors}_v1.json` | Locked vectors |
| `spec/heap/rpc-v1/rql_query.*` + fixtures | Staged wire (not active product) |
| `spec/app/baseline-v1/operations-v1.json` | App ops + wire honesty |
| [APB6_READ_VIEW_GAP_INVENTORY.md](./APB6_READ_VIEW_GAP_INVENTORY.md) | View pin residual |

---

## 9. Scoreboard recommendation (T0)

| Field | Value |
|---|---|
| APB-7 State | **active** (inventory only) |
| Evidence | this file |
| Residual | G1–G20; primary product blockers: façade builder, executor hardening, op 118, HAR-4, dual oracle |
| Accept | **no** |