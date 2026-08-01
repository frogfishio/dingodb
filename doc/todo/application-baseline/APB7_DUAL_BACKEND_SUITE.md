# APB-7 T7 — Dual-backend Application Core query parity + accept checklist

Status: **labor 2026-08-02** · package `APB-7` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §11 · [APB7_QUERY_RUNTIME_GAP_INVENTORY.md](./APB7_QUERY_RUNTIME_GAP_INVENTORY.md) ·
[HAR4_QUERY_REMOTE_GAP_INVENTORY.md](../heap-application-ready/HAR4_QUERY_REMOTE_GAP_INVENTORY.md)

Board: `9e19bd5f` (Query spine Feature `1a8a3e05`)

This document is the **T7 dual-pack matrix + package accept checklist**.
It does **not** authorize scoreboard `APB-7 = accept`.

---

## 1. Goal

One shared scenario pack for Application Core query surfaces runs against:

| Backend | Constructor | Host test | Path honesty |
|---|---|---|---|
| Embedded | `HeapClient::from(Heap)` | `cargo test -p residiuum-sdk --test apb7_query_dual_pack` | Full APP-6 / APB-7 collection-plane executor |
| Remote product `rql_query` | `HeapClient::from(RemoteHeap)` + op **118** | `… --test hp007_connect_heap apb7_query_from_remote` | APP-7 T6: façade `rql` uses wire 118 |

Shared code:

`crates/residiuum-sdk/tests/common/apb7_query_parity.rs`

(included via `#[path = …]` — not a public product API).

---

## 2. Scenario matrix

| Id | Embedded | Remote collection-plane | Product wire 118 | Notes |
|---|---|---|---|---|
| `builder_rql_plan_hash` | **yes** | **yes** | residual | MUST_ADD exit: same plan |
| `equality_filter_keys` | yes | yes | residual | builder == RQL == list_keys+get oracle |
| `multipage_key_order_oracle` | yes | yes | residual | pages vs complete-scan oracle |
| `multipage_field_order_oracle` | yes | yes | residual | `last_sort_tuple` path |
| `explain_rql_surface` | yes | yes | residual | plan hash only; no cost model claim |
| `scan_json_page` | yes | yes | n/a (ops 115) | façade scan; not RQL product |
| `budget_fail_closed` | yes | yes | residual | ResourceLimit not silent page |
| `index_equality_pushdown` | yes | yes | residual | needs IndexAdmin; range residual |

Matrix evidence (2026-08-02): embedded `apb7_query_dual_pack` **1/1**; remote
`apb7_query_from_remote_collection_plane` **1/1**.

Additional coverage (not duplicated into dual pack; already in slice tests):

| Evidence | Tests | Feeds accept? |
|---|---|---|
| Deadline + cancel | `apb7_deadline_cancel` 4/4 | required (governance) |
| Coverage grade | `apb7_coverage_grade` 4/4 | required (complete-by-default) |
| Cursor secrets + parameter_hash | `apb7_cursor_secrets` 4/4 | required (auth cursor residual honesty) |
| Multipage oracle matrix | `apb7_multipage_oracle_matrix` 6/6 | required (exit oracle) |
| ReadView-bound gate | `apb7_read_view_query` 4/4 | residual (not SI) |

---

## 3. Commands

```bash
# Embedded dual pack (T7 host)
cargo test -p residiuum-sdk --test apb7_query_dual_pack

# Related slice evidence (not dual pack itself)
cargo test -p residiuum-sdk --test apb7_multipage_oracle_matrix
cargo test -p residiuum-sdk --test apb7_coverage_grade
cargo test -p residiuum-sdk --test apb7_deadline_cancel
cargo test -p residiuum-sdk --test apb7_cursor_secrets

# Remote collection-plane dual pack (when host present)
cargo test -p residiuum-server --features dangerous-key-export \
  --test hp007_connect_heap apb7_query_from_remote
```

---

## 4. MUST_ADD §11 accept checklist (package gate — principal only)

Scoreboard may mark **`APB-7 = accept` only when every box is true** and principal
gates. Labor must **never** self-mark accept from partial dual-pack green.

### 4.1 Deliverable surfaces

- [x] `CollectionClient::query()` builder path (T1)
- [x] `CollectionClient::rql` / `explain_rql` (APP-6 + T1)
- [x] Projection, scalar order, limit, page, continuation (T2, APP-6 T3, T10)
- [x] Complete-by-default coverage grade (T9)
- [x] Budgets + deadline + cancellation (T2, T8)
- [x] Index-versus-scan equality path (T4; range residual)
- [x] Authenticated cursor mint/verify + parameter_hash (T10; Heap-confined durable secret residual)
- [x] **Product** remote wire path via op **118** (T6 labor; package accept residual)

### 4.2 Exit criteria (normative MUST_ADD §11)

| Criterion | Status | Evidence |
|---|---|---|
| Builder and RQL compile to the same plan | **partial green** | dual pack `builder_rql_plan_hash`; APP-5 corpus |
| All pages reconcile with independent complete-scan oracle | **partial green** (embedded) | dual pack multipage + T11 matrix |
| Both backends (product remote) | **labor green** | op 118 wire + dual pack remote; package accept residual |
| Dual-pack matrix green on product remote | **labor green** | `apb7_query_from_remote_collection_plane` 1/1 via wire |

### 4.3 Hard blockers before accept

1. ~~**APP-7 / APB-7 T6** activate op 118~~ — **labor done** (wire active; not package accept).
2. **HAR-4** package accept / T3–T4 residual (config UX, tutorial journey).
3. **Principal scoreboard gate** — only principal marks `APB-7 = accept`.
4. Residual honesty: range index, SI / view multipage, Heap-confined cursor secrets, plan-only wire args.

### 4.4 Explicit non-claims (always until accept)

- Dual pack green (embedded or remote wire) ≠ APB-7 package accept.
- Op 118 active ≠ product “query baseline qualified”.
- No snapshot isolation claim (APB-6 / T5 residual).
- Full RQL v1 language is **not** APB-7 (separate card).

---

## 5. Accept decision table (for principal)

| If … | Then … |
|---|---|
| Only T7 dual pack + T0–T5/T8–T11 labor | Keep **active**; stage tasks `in_review` |
| Op 118 still reserved | **Forbidden** to accept APB-7 |
| Collection-plane remote green but no 118 | Document as **partial remote**; not exit |
| All §4.2 criteria + residual honesty + principal gate | **May** set scoreboard accept |

---

## 6. Related artifacts

| Artifact | Role |
|---|---|
| `tests/common/apb7_query_parity.rs` | Shared scenarios |
| `tests/apb7_query_dual_pack.rs` | Embedded host |
| `APB7_QUERY_RUNTIME_GAP_INVENTORY.md` | G1–G20; T7 board map |
| `HAR4_QUERY_REMOTE_GAP_INVENTORY.md` | Remote product blockers |
| `NEXT_BUILD_STATUS.md` | Scoreboard SoT (principal accept) |