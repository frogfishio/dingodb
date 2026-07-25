# dingo-sdk

Collection SDK for DingoDB: ordinary `open`, remote `connect`, or multi-node
`open_cluster` / `create_cluster` + named collections with JSON and raw-byte
put/get/delete, JSON filters, secondary indexes, per-key history, and query
budgets over [`dingo-store`](../dingo-store) / [`dingo-cluster`](../dingo-cluster).

Normative sources: repository root [`DX_SPEC.md`](../../DX_SPEC.md) §§1–10, §14;
[`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md) Stages 4, 6, 7, and 8d;
[`CLUSTER_SPEC.md`](../../CLUSTER_SPEC.md) §13 (client routing).

## Status

**Stages 4a–4d + 6 + 7 + 8d** — open, JSON/bytes put/get/delete, scan + streaming
iter, SDK-native filters, stable `ErrorCode`, write receipts, secondary field
indexes, query budgets, per-key history, chunked large payloads, Stage 7
`Dingo::connect("dingo://host:port")` over line-delimited JSON TCP, and Stage 8d
**cluster routing** with a client partition directory cache. Remote parity
covers put/get/delete/scan, **history**, **secondary indexes**,
**`get_payload`**, **server-side find**, and **`directory`**.

Application developers do not need to know about frames or segments.

## Surface

| API | Role |
|-----|------|
| `Dingo::open` | Create-or-open store directory with safe defaults |
| `Dingo::connect` / `connect_with` | Remote `dingo://host:port[/label]` or multi-seed `h1:p1,h2:p2` + `ConnectOptions` |
| `Dingo::create_cluster` / `open_cluster` | In-process multi-node cluster (Stage 8d); same collection API + route cache |
| `ClientDirectoryCache` | Client partition → leader cache; refresh on stale placement |
| `serve_store` / `serve_store_with` | TCP server helpers (`ServeOptions` token; `directory` op) |
| `Dingo::collection` | Lazy named collection handle (no disk write) |
| `Dingo::list_collections` / `rebuild_catalogs` | Derived catalog (rebuild embedded only) |
| `Collection::put` / `get` / `delete` | JSON values (serde) |
| `Collection::put_bytes` / `get_bytes` | Opaque byte payloads |
| `Collection::get_payload` | Completeness-aware chunked read (embedded + remote) |
| `Collection::scan_keys` / `scan_json` | Bounded live scan |
| `Collection::scan_json_iter` | Streaming JSON rows (embedded) |
| `Collection::find` / `find_json` / `query` | Filters + index acceleration (embedded + remote server-side) |
| `Collection::indexes` | Create / drop / rebuild / list secondary indexes (embedded + remote) |
| `Collection::history` | Immutable event stream for one key (embedded + remote) |
| `handle_connection` / `handle_connection_with` | Per-connection server dispatch |
| `Filter` / `QueryOptions` / `QueryBudget` | Predicates + limit/order/budget |
| `WriteReceipt` / `DeleteReceipt` | Event identity + achieved durability |
| `Error::code` / `ErrorCode` | Stable machine codes (DX §15) |

## Quick example

```rust
use dingo_sdk::{json, Dingo, Filter, SortOrder};

# let dir = tempfile::tempdir().unwrap();
# let path = dir.path().join("app.dingo");
let mut db = Dingo::open(&path)?;
{
    let mut users = db.collection("users")?;
    users.put("user-42", &json!({ "name": "Alice", "status": "active", "age": 30 }))?;
    users.indexes()?.create("by-status", &["status"])?;

    let rows = users.find(&Filter::field("status").eq("active"))?;
    let hist = users.history("user-42")?;
    let _ = (rows, hist);
}
# Ok::<(), dingo_sdk::Error>(())
```

Remote (requires `dingo serve ./app.dingo` or `dingo serve --token SECRET`):

```rust
use dingo_sdk::{json, ConnectOptions, Dingo};
use std::time::Duration;

let mut db = Dingo::connect("dingo://127.0.0.1:7434/app")?;
// Same collection API; connection-only policy:
let mut db = Dingo::connect_with(
    "dingo://127.0.0.1:7434/app",
    ConnectOptions::new()
        .auth_token("SECRET")
        .request_timeout(Duration::from_secs(10))
        .max_connect_attempts(5),
)?;
db.collection("users")?.put("user-42", &json!({ "name": "Alice" }))?;
# Ok::<(), dingo_sdk::Error>(())
```

## Subject encoding (draft)

Logical `(collection, key)` pairs map to store subjects as:

```text
0x01 || coll_len:u16 LE || collection UTF-8 || key UTF-8
```

Payloads are typed:

```text
0x01 || JSON UTF-8 text
0x02 || raw bytes
```

Large bodies may be stored as chunked payloads (store threshold); ordinary get
returns complete data only when every chunk verifies.

## Non-goals (yet)

- Cluster routing (Stage 8)
- Mutual TLS / multi-tenant ACLs (shared token only for Stage 7e)
- SDA examination of holes / recovery units — see [`dingo-examine`](../dingo-examine) (Stage 5)
- Unique secondary indexes with partition consistency scopes
- Full DX §15 codes that only apply in cluster mode
