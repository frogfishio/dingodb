# HAR-1 — Collection creation evidence reconcile

Status: **evidence note v1** (2026-08-01) · package `HAR-1` → scoreboard **active / not accept**  
Authority: [HEAP_APPLICATION_READY_PLAN.md](./HEAP_APPLICATION_READY_PLAN.md) § HAR-1 ·
scoreboard `NEXT_BUILD_STATUS.md` · sibling labor APP-1 / APB-1

This note **reconciles scoreboard truth** with landed implementation. It does
**not** claim HAR-1 package accept.

---

## 1. Stale claim (corrected)

| Prior scoreboard | Reality |
|---|---|
| op **106** `collection_create` **reserved**, schemas null | **False.** Registry + schemas + implementation are **active**. |
| product create missing | **False as absolute.** Embedded + remote create exist; residuals remain. |

Normative registry:

- `spec/heap/operations-v1.json` — op **106**, `status: "active"`, `rights_mask: 4096` (HeapAdmin)
- `spec/heap/rpc-v1/collection_create.request.json`
- `spec/heap/rpc-v1/collection_create.response.json`
- Fixtures: `spec/heap/fixtures/collection_create.accepted.json`, `.rejected.json`
  (locked via `app0_contract_lock` / `verify-app0-contract.sh`)

---

## 2. Landed surfaces

| Surface | Path / command | Notes |
|---|---|---|
| Store admin create | `create_collection_idempotent` (UUIDv4 object ids) | APP-1 |
| Embedded SDK | `Heap::create_collection[_with]` | APP-1 |
| Server dispatch | `heap_dispatch` op 106 | APP-1 |
| Remote SDK | `RemoteHeap::create_collection` | APP-1 |
| App façade | `HeapClient::create_collection` both backends | APB-1 G1/G1b |
| Dual façade pack | `apb1_facade_parity` + remote HeapAdmin mint | APB-1 G6/G6b |

---

## 3. Plan-required tests vs labor

HAR-1 plan required tests (HEAP_APPLICATION_READY_PLAN):

| Required | Evidence | Status |
|---|---|---|
| create/list/open/use | `app1_collection_create` embedded; façade dual pack | **landed** |
| idempotent retry | `operation_id_exact_retry_replays_and_conflict`; dispatch `op_106_create_replay_and_conflict` | **landed** |
| duplicate conflict | same (name/op fingerprint conflict) | **landed** |
| wrong right | dispatch READ-only → `heap_unavailable` | **landed** |
| same name in two Heaps | `two_heaps_same_name_distinct_ids` | **landed** |
| foreign Heap key | isolation suite residual (hp007 isolation) | **partial / adjacent** |
| failpoint before/after publication | no dedicated collection_create failpoint matrix | **open** |
| rebuild catalog after deletion | not claimed for HAR-1 create exit | **open** |
| RPC golden fixtures | fixtures present + APP-0 lock; not full RPC golden runner for 106 | **partial** |

Evidence classes (plan): Unit ✓ · Isolation partial · Crash **open** · Journey **open**

### Commands (smoke)

```bash
cargo test -p residiuum-sdk --test app1_collection_create          # 4/4
cargo test -p residiuum-server --test app1_collection_create_dispatch  # 1/1
cargo test -p residiuum-sdk --test apb1_heap_client_embedded       # 2/2
cargo test -p residiuum-server --features dangerous-key-export \
  --test hp007_connect_heap apb1_heap_client_from_remote           # 2/2
```

---

## 4. Scoreboard disposition (this labor)

| Field | Value |
|---|---|
| HAR-1 state | **active** (was `not_started` with stale “reserved”) |
| Accept? | **No** — crash/journey/failpoint residual; product bootstrap cert still lacks HeapAdmin |
| APP-1 residual note | Façade create now exists (APB-1); residual = crash cells + ceremony/cert story |

---

## 5. Explicit non-claims

- No HAR-1 **accept**.
- No “product heap is provisioned” marketing without HAR-2 ceremony.
- Bootstrap vector cert remains rights_mask **13** (no HeapAdmin); TLS create tests
  mint admin COSE locally (APB-1 G6b) without rewriting public vectors.
- No invent of failpoint/crash cells in this note — residual listed honestly.
