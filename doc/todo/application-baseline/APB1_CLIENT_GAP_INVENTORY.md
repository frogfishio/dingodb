# APB-1 — HeapClient / CollectionClient gap inventory

Status: **labor inventory v1.0** (2026-08-01) · package `APB-1` **not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §5 · [CORE plan](./CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) §3–4 ·
[`spec/app/baseline-v1/operations-v1.json`](../../../spec/app/baseline-v1/operations-v1.json) ·
scoreboard `NEXT_BUILD_STATUS.md`

This document is a **gap inventory only**. It does not implement the unified
client, does not claim product readiness, and does not change wire schemas.

---

## 1. Package goal (normative)

APB-1 delivers one Heap-bound backend-neutral application client:

| Surface | Role |
|---|---|
| `HeapClient` | Bound to immutable `HeapId`; create/open/list collections |
| `CollectionClient` | Bound to `HeapId` + immutable `CollectionId`; data path entry |
| `IndexManager` | Index list/create/drop/rebuild via collection |
| History / recovery clients | DEF-099 historical + recovery examination |
| Adapters | Embedded (`Heap`) and remote (`RemoteHeap`) behind one façade |

**Rules (MUST_ADD §5):** one app source differs only by constructor; no raw wire
JSON / caller-supplied Heap ID / FS paths on the façade; semantic outcomes
identical across backends; sync v1 OK.

**Exit (package, later):** shared compile + behavior suites pass **both**
backends. This inventory does **not** meet exit.

---

## 2. Dependency honesty

| Dep | Scoreboard | Inventory note |
|---|---|---|
| `APB-0` | **accept** | baseline-v1 frozen — ops/types OK to implement against |
| `HAR-1` | scoreboard still says op **106 reserved** | **Stale vs reality:** APP-1 labor has op **106 active** + schemas + `RemoteHeap::create_collection` / `Heap::create_collection`. HAR-1 T1 must reconcile Evidence/State before APB-1 claims *qualified* create. |
| `APP-1` | **active** | Embedded + remote create paths exist; façade not wired |
| `APP-2` | not_started | Normative “implementation core of APB-1” (MUST_ADD map) |
| `APP-4` / `APP-5` | **accept** | Pure plan/compiler ready for later APB-7; **not** APB-1 exit |

Provisioning path for a *usable* client still needs honest HAR-1 / APP-1
accept before APB-1 product claim language.

---

## 3. What exists today (precursor)

### 3.1 Façade types (`residiuum_sdk::app_v1`)

| Type / method | Status | Notes |
|---|---|---|
| `HeapClient` identity + `from_id_for_contract` | **compile-only** | No sealed backend |
| `HeapClient::create_collection[_with]` | **stub** | `Error::Internal` — redirects to `Heap::create_collection` |
| `HeapClient::open_collection` / `list_collections` | **stub** | Same |
| `CollectionClient` identity + `from_parts_for_contract` | **compile-only** | Ids held; no IO |
| `CollectionClient::{put,get,delete,…}` | **stub** | APP-3 activation messages |
| `CollectionClient::{rql,explain_rql}` | **stub** | APP-5…7 activation messages (compiler is APP-5 accept; **execution** APB-7) |
| `IndexManager` / `HistoryClient` / `RecoveryClient` | **missing types** | Not on `app_v1` façade |
| `From<Heap>` / `From<RemoteHeap>` constructors | **missing** | MUST_ADD “change only constructor” |

### 3.2 Working embedded path (`residiuum_sdk::Heap` / `HeapCollection`)

Product-shaped but **not** the neutral façade:

- `Heap::{create_collection, list_collections, collection, collection_by_id}`
- `HeapCollection::{put, put_with, put_bytes, get, get_bytes, delete, …}`
- Capability / same-heap checks

### 3.3 Working remote path (`RemoteHeap`)

Wire-oriented API (string collection ids, `Value` rows) — **not** façade:

- create/list/open, put/get/delete, list_keys, history, find, index_*, scan_json
- TLS + credential ceremony (HAR path)

### 3.4 APP-1 evidence

- Op **106** active; schemas + dispatch; tests `app1_collection_create` 4/4
- Scoreboard residual: HeapClient façade (APP1-R3 / APP-2)

---

## 4. Baseline-v1 ops owned by APB-1

From `operations-v1.json` (`must_add_package: APB-1`):

| app_id | Rust methods (registry) | Wire | Gap vs façade |
|---|---|---|---|
| `apb.heap.bind` | `HeapClient::open_embedded`, `connect_remote` | app_facade | **Missing** constructors / sealed backend enum |
| `apb.collection.open` | `HeapClient::collection` / open | **active** (105) | Stub; embedded+remote exist outside façade |
| `apb.collection.create` | `HeapClient::create_collection` | **active** (106) | Stub façade; real on Heap/RemoteHeap |
| `apb.collection.list` | `HeapClient::list_collections` | **active** (110) | Stub façade |
| `apb.index.list` | `IndexManager::list` | **active** (130) | Type missing; remote `index_list` exists |
| `apb.index.create` | `IndexManager::create` | **active** (131) | Type missing |
| `apb.index.drop` | `IndexManager::drop` | **active** | Type missing; remote has drop |
| `apb.index.rebuild` | `IndexManager::rebuild` | **active** | Type missing; remote has rebuild |
| `apb.history.get` | `HistoryClient::get` / `CollectionClient::history` | **active** (117) | Method missing on façade; remote `history` exists |
| `apb.recover.examine` | `RecoveryClient::examine` | **reserved** | Type missing; wire not active — **no invent** |

CRUD put/get/delete are **APB-2** in the registry (and APP-3 in façade stubs).
APB-1 inventory still lists them as *adjacent* so the façade seam is planned once:

| Adjacent | Owning package | Façade today |
|---|---|---|
| put / get / delete | APB-2 / APP-3 | stubs |
| rql / explain_rql | APB-7 (+ APP-6) | stubs; pure compiler APP-5 **accept** |
| read_view | APB-6 | missing |

---

## 5. Gap matrix (implementer backlog)

Priority order for **APB-1 labor after this inventory** (not started here):

| # | Gap | Proposed package slice | Depends |
|---:|---|---|---|
| G1 | Sealed backend + `From<Heap>` / `From<RemoteHeap>` / connect helpers | APB-1 / APP-2 | APP-1 evidence; remote connect ceremony (HAR) |
| G2 | Wire `create` / `open` / `list` through façade (parity suite start) | APB-1 | G1; HAR-1 scoreboard reconcile |
| G3 | `IndexManager` façade over embedded indexes + remote index_* | APB-1 | G1 |
| G4 | `CollectionClient::history` / HistoryClient | APB-1 | G1; DEF-099 |
| G5 | RecoveryClient only when wire un-reserves or pure local examine defined | APB-1 later | reserved wire honesty |
| G6 | Shared behavior suite: same tests embedded vs remote | APB-1 exit | G2–G4 |
| G7 | put/get/delete façade binding | **APP-3 / APB-2** | G1 |
| G8 | rql execution + cursor | **APP-6 / APB-7** | APP-5 accept ✓; G1; HAR-4 for remote |

**Out of APB-1 scope (do not pull forward):** watches, import/export, bulk,
aggregates, document-path mutate — later APB packages.

---

## 6. Recommended labor sequence (spine-aware)

Principal §0.8 still prioritizes query path. After APP-5 accept:

```text
NOW   APB-1 G1–G2  min client bind + create/open/list (enables later APB-6/7)
  ||  HAR-0 residual + HAR-1 scoreboard reconcile (provisioning honesty)
THEN  APP-3 / APB-2 CRUD on façade (or keep HeapCollection until suite demands)
THEN  APB-6 read views
THEN  APP-6 / APB-7 RQL execution (compiler ready)
```

Do **not** claim APB-1 accept until dual-backend suite exits.

---

## 7. Evidence for this task (T1)

| Item | Path / command |
|---|---|
| Inventory (this file) | `doc/todo/application-baseline/APB1_CLIENT_GAP_INVENTORY.md` |
| Ops registry | `spec/app/baseline-v1/operations-v1.json` (10 APB-1 rows) |
| Façade stubs | `crates/residiuum-sdk/src/app_v1.rs` |
| Embedded | `crates/residiuum-sdk/src/heap.rs` |
| Remote | `crates/residiuum-sdk/src/remote_heap.rs` |
| Scoreboard | `APB-1` → **active** with inventory evidence (not accept) |

---

## 8. Explicit non-claims

- No product “unified client” marketing language.
- No inventing wire ops for reserved recover/rql.
- No APB-1 package accept from this inventory alone.
- HAR-1 scoreboard “106 reserved” is **not** re-asserted as truth — listed for
  reconcile labor (HAR-1 T1).
