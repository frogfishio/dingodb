# HAR-4 dep — Qualified remote path for query product (gap inventory)

Status: **T0 inventory 2026-08-02** · package `HAR-4` **active / not accept**  
Board: `7872d5fa` (Query spine Feature)  
Authority: [HEAP_APPLICATION_READY_PLAN.md](./HEAP_APPLICATION_READY_PLAN.md) §HAR-4 ·
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md) ·
[APB7_QUERY_RUNTIME_GAP_INVENTORY.md](../application-baseline/APB7_QUERY_RUNTIME_GAP_INVENTORY.md)

This document maps **what blocks product remote Application Core query**
(op **118** `rql_query` / APB-7 dual remote claim). It is **not** HAR-4 package
accept and does **not** activate op 118.

---

## 1. Package goal (HAR-4 normative)

From HEAP_APPLICATION_READY_PLAN:

> Make the HeapKey path the **normal** Residiuum server.

Exit highlights:

| Requirement | Product meaning |
|---|---|
| HeapKey TLS listener is **default** | Tutorials use `connect_heap`; not shared token |
| Explicit legacy opt-in | `--legacy-token-server` (or equivalent) only |
| No co-host of qualified + legacy on one process/store | Config fail-closed |
| Absent/invalid HeapKey → indistinguishable reject | No existence leak |
| TLS exporter replay fails | Channel binding honesty |
| Remote parity for every active P1 op | Includes future product query |

---

## 2. What already exists (precursor — reuse)

| Surface | Location | Query relevance |
|---|---|---|
| TLS 1.3 server/client | `residiuum_sdk::tls` | Required for qualified path |
| HeapKey handshake (challenge/auth/welcome) | `residiuum_server::heap_session` / `heap_auth` | HP-008 |
| `validate_qualified_listener` | `heap_session` | TLS + no token + registry + deployment_id |
| `ServeOptions::qualified_heap_key` | `serve.rs` | **opt-in** (`default = false`) |
| `Residiuum::connect_heap` | `remote_heap.rs` | Qualified client entry |
| Façade remote CollectionClient | APB-1 | Data plane over RemoteHeap (not op 118) |
| APP-6/APB-7 embedded query | `query_exec_v1` | Product Core execute **embedded** |
| Op **118** schemas staged | `spec/heap/rpc-v1/rql_query.*` | Registry **reserved** |
| Op 118 registry honesty | `operations-v1.json` + `request_registry_allows(118)==false` | Locked in `hp008_heap_handshake` |

---

## 3. Gaps blocking product remote query

| ID | Gap | Blocks |
|---|---|---|
| H4-G1 | `ServeOptions::qualified_heap_key` not default | HAR-4 “normal path” exit |
| H4-G2 | CLI/tutorials still allow open/token postures without clear non-qualified labels | Product claim honesty |
| H4-G3 | Co-host prohibition not fully productized as config UX | HAR-4 exit |
| H4-G4 | Op **118** still **reserved** (no server dispatch / RemoteHeap rql) | APP-7 / APB-7 T6 |
| H4-G5 | Dual remote multipage oracle for **product** Core query (op 118) | APB-7 T7 partial (collection-plane dual green); product wire residual |
| H4-G6 | Remote ReadView pin (still `RemoteUnpinnedResidual`) | APB-6 T3 residual |
| H4-G7 | Heap-confined product cursor secrets on server | APB-7 T10 residual |
| H4-G8 | HAR-3 key lifecycle package not accept | Upstream of full HAR-4 exit |

**Unblock for APP-7 labor:** H4-G1…G3 may lag if APP-7 is labored only under **explicit** `qualified_heap_key=true` test harness (not product default). Product claim still requires HAR-4 exit + op 118 active.

**Unblock for APB-7 package accept:** needs H4-G4 + dual remote honesty (or explicit non-claim on remote product).

---

## 4. Query spine dependency map

```text
APP-6 / APB-7 embedded query labor     ── in_review (T0–T11 etc.)
         │
         ▼
HAR-4 qualified remote posture         ── this inventory (active / not accept)
         │
         ├──► APP-7 / APB-7 T6 activate op 118   (blocked until honest path)
         │
         └──► APB-7 T7 dual-pack accept checklist
```

Do **not** activate op 118 from this card alone.

---

## 5. Recommended labor slices (after this inventory)

| Slice | Deliverable | Notes |
|---|---|---|
| HAR-4 T1 | This inventory + scoreboard HAR-4→active | **this labor** |
| HAR-4 T2 | Default config / CLI qualified listener + legacy opt-in flag | Product default flip |
| HAR-4 T3 | Co-host config reject + help/error labels | HAR-4 exit bullets |
| HAR-4 T4 | Journey: tutorial uses `connect_heap` only | Evidence pack |
| APP-7 T1 | Op 118 registry active + dispatch + RemoteHeap | **after** qualified path honesty |
| APB-7 T6 | Wire façade remote `rql` to op 118 | same |

---

## 6. Explicit non-claims

- No HAR-4 package **accept**.
- No product remote query / op 118 active.
- No claim that `CollectionClient::rql` on remote is product wire.
- Embedded Application Core query labor remains valid evidence; it does not
  satisfy HAR-4 exit.

---

## 7. Evidence pointers

| Artifact | Role |
|---|---|
| `crates/residiuum-server/src/{serve,heap_session,heap_dispatch}.rs` | Qualified listener + registry |
| `crates/residiuum-sdk/src/{remote_heap,tls}.rs` | `connect_heap` + TLS |
| `crates/residiuum-server/tests/hp008_heap_handshake.rs` | Op 118 reserved lock; `validate_qualified_listener` matrix |
| `crates/residiuum-server/tests/har4_query_remote_gate.rs` | Gate locks for query dependency (this labor) |
| `spec/app/baseline-v1/operations-v1.json` | Op 118 reserved notes |