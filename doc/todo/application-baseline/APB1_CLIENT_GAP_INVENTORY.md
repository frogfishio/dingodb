# APB-1 — HeapClient / CollectionClient gap inventory

Status: **labor inventory v1.6** (2026-08-01) · package `APB-1` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §5 · [CORE plan](./CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) §3–4 ·
[`spec/app/baseline-v1/operations-v1.json`](../../../spec/app/baseline-v1/operations-v1.json) ·
scoreboard `NEXT_BUILD_STATUS.md`

This document tracks **gaps vs APB-1 exit**. G1/G1b façade bind is labor-
landed; dual-backend suite, Index/History, and package accept remain open.
It does not claim product readiness and does not change wire schemas.

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
| `HAR-1` | **active** (2026-08-01) | Scoreboard reconciled: op **106 active** + schemas/fixtures + create paths. See [HAR1_COLLECTION_CREATE_EVIDENCE.md](../heap-application-ready/HAR1_COLLECTION_CREATE_EVIDENCE.md). **Not accept** (crash/journey residual). |
| `APP-1` | **active** | Create paths exist; façade G1+G1b now wires create/open/list (+ basic put/get/delete) |
| `APP-2` | not_started | Normative “implementation core of APB-1” (MUST_ADD map) |
| `APP-4` / `APP-5` | **accept** | Pure plan/compiler ready for later APB-7; **not** APB-1 exit |

Provisioning path for a *usable* client still needs honest HAR-1 / APP-1
accept before APB-1 product claim language.

---

## 3. What exists today (precursor)

### 3.1 Façade types (`residiuum_sdk::app_v1`)

| Type / method | Status | Notes |
|---|---|---|
| `HeapClient` + `From<Heap>` sealed `Embedded` | **G1 landed** | Unbound fixtures remain fail-closed |
| `HeapClient` + `From<RemoteHeap>` sealed `Remote(Arc<Mutex<_>>)` | **G1b landed** | Shared session with collection handles |
| `HeapClient::create_collection[_with]` | **embedded + remote wired** | Remote → op 106; receipt maps wire hex |
| `HeapClient::open_collection` / `list_collections` | **embedded + remote wired** | Remote open 105 / list 110; list zeros `descriptor_hash` until wire grows field |
| `CollectionClient` + embedded/remote handle | **G1/G1b landed** | Bound from create/open |
| `CollectionClient::{put,get,delete,…}` | **embedded + remote forward** | Remote put maps event/version; delete receipt ids zeroed (wire returns bool only) |
| `CollectionClient::history` | **G4 landed** | Embedded `HeapCollection::history` (SubjectV2); remote op 117 → `KeyHistory` |
| `IndexManager` via `CollectionClient::indexes` | **G3 landed** | list/create/drop/rebuild/get; embedded + remote 130–133 |
| `CollectionClient::{rql,explain_rql}` | **stub** | APP-5…7 activation messages (compiler is APP-5 accept; **execution** APB-7) |
| `RecoveryClient` | **missing** | recover reserved — no invent |
| `HistoryClient` type | **optional residual** | Method on `CollectionClient` covers `apb.history.get` for now |
| Named connect helpers (`open_embedded` / `connect_remote`) | **optional residual** | `From` constructors cover bind; baseline registry may still want helpers |

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
| `apb.heap.bind` | `HeapClient::open_embedded`, `connect_remote` | app_facade | **`From<Heap>` / `From<RemoteHeap>` landed**; named helpers optional residual |
| `apb.collection.open` | `HeapClient::collection` / open | **active** (105) | **Wired** on both backends |
| `apb.collection.create` | `HeapClient::create_collection` | **active** (106) | **Wired** on both backends (HAR-1 evidence residual) |
| `apb.collection.list` | `HeapClient::list_collections` | **active** (110) | **Wired**; remote list zeros descriptor_hash |
| `apb.index.list` | `IndexManager::list` | **active** (130) | **Wired** both backends |
| `apb.index.create` | `IndexManager::create` | **active** (131) | **Wired**; requires IndexAdmin |
| `apb.index.drop` | `IndexManager::drop` | **active** (132) | **Wired** |
| `apb.index.rebuild` | `IndexManager::rebuild` | **active** (133) | **Wired** |
| `apb.history.get` | `HistoryClient::get` / `CollectionClient::history` | **active** (117) | **Wired** on both backends (`CollectionClient::history`) |
| `apb.recover.examine` | `RecoveryClient::examine` | **reserved** | Type missing; wire not active — **no invent** |

CRUD put/get/delete are **APB-2** in the registry (and APP-3 in façade stubs).
APB-1 inventory still lists them as *adjacent* so the façade seam is planned once:

| Adjacent | Owning package | Façade today |
|---|---|---|
| put / get / delete | APB-2 / APP-3 | **basic forward both backends**; create/upsert/list_keys landed (APB-2 active); CAS residual |
| rql / explain_rql | APB-7 (+ APP-6) | stubs; pure compiler APP-5 **accept** |
| read_view | APB-6 | missing |

---

## 5. Gap matrix (implementer backlog)

Priority order for **APB-1 labor after this inventory** (not started here):

| # | Gap | Proposed package slice | Depends |
|---:|---|---|---|
| G1 | Sealed backend + `From<Heap>` / `From<RemoteHeap>` / connect helpers | APB-1 / APP-2 | **G1+G1b 2026-08-01:** Unbound\|Embedded\|Remote; both `From`s; create/open/list + put/get/delete both backends |
| G2 | Wire `create` / `open` / `list` through façade (parity suite start) | APB-1 | **Wired**; exercised by G6 pack (embedded create; remote list/open) |
| G3 | `IndexManager` façade over embedded indexes + remote index_* | APB-1 | **Landed 2026-08-01:** `CollectionClient::indexes` both backends |
| G4 | `CollectionClient::history` / HistoryClient | APB-1 | **Landed 2026-08-01:** both backends; optional named HistoryClient residual |
| G5 | RecoveryClient only when wire un-reserves or pure local examine defined | APB-1 later | reserved wire honesty |
| G6 | Shared behavior suite: same tests embedded vs remote | APB-1 exit | **Matrix green 2026-08-01:** shared pack; remote full create via HeapAdmin mint; product vectors still rights 13; **not package accept** |
| G7 | put/get/delete façade binding | **APP-3 / APB-2** | **put/get/delete + create/upsert/list_keys**; replace/if_version/CAS residual |
| G8 | rql execution + cursor | **APP-6 / APB-7** | APP-5 accept ✓; G1; HAR-4 for remote |

**Out of APB-1 scope (do not pull forward):** watches, import/export, bulk,
aggregates, document-path mutate — later APB packages.

---

## 6. Recommended labor sequence (spine-aware)

Principal §0.8 still prioritizes query path. After APP-5 accept:

```text
DONE  APB-1 G1+G1b  From<Heap|RemoteHeap> + create/open/list + basic put/get/delete
DONE  APB-1 G4      CollectionClient::history both backends
DONE  APB-1 G3      IndexManager list/create/drop/rebuild both backends
DONE  APB-1 G6      dual suite scaffold + remote full create (HeapAdmin mint)
DONE  HAR-1 T1     scoreboard op-106 evidence reconcile (active, not accept)
DONE  APB-2 slice  create/upsert/list_keys on CollectionClient (active, not accept)
NOW   APB-2 replace/if_version || APB-6 read views || HAR-1 crash residual
  ||  HAR-0 residual; optional CI dual harness
THEN  APP-6 / APB-7 RQL execution (compiler ready)
```

Do **not** claim APB-1 accept until dual-backend suite exits.

---

## 7. Evidence for this task (T1)

| Item | Path / command |
|---|---|
| Inventory (this file) | `doc/todo/application-baseline/APB1_CLIENT_GAP_INVENTORY.md` |
| Ops registry | `spec/app/baseline-v1/operations-v1.json` (10 APB-1 rows) |
| Façade | `crates/residiuum-sdk/src/app_v1.rs` (`From`, history, `IndexManager`) |
| Dual suite doc | [APB1_DUAL_BACKEND_SUITE.md](./APB1_DUAL_BACKEND_SUITE.md) |
| Shared scenarios | `crates/residiuum-sdk/tests/common/apb1_facade_parity.rs` |
| Embedded suite | `cargo test -p residiuum-sdk --test apb1_heap_client_embedded` **2/2** (full G6 pack) |
| Remote suite | `… apb1_heap_client_from_remote` **2/2** (collection plane + full HeapAdmin create) |
| Store create ids | UUIDv4 mint in `create_collection_idempotent` |
| Scoreboard | `APB-1` → **active** (not accept); G1–G6 matrix green; HAR-1 / package exit residual |

---

## 8. Explicit non-claims

- No product “unified client” marketing language.
- No inventing wire ops for reserved recover/rql.
- No APB-1 package accept from this inventory alone.
- HAR-1 “106 reserved” was stale; corrected to **active** with residual honesty
  (HAR1_COLLECTION_CREATE_EVIDENCE.md). Still not HAR-1 package accept.