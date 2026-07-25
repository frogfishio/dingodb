# dingo-sdk

Collection SDK for DingoDB: ordinary `open`, remote `connect`, or multi-node
`open_cluster` / `create_cluster` + named collections with JSON and raw-byte
put/get/delete, JSON filters, secondary indexes, per-key history, query budgets,
and cluster find coverage over [`dingo-store`](../dingo-store) /
[`dingo-cluster`](../dingo-cluster).

Normative sources: repository root [`DX_SPEC.md`](../../DX_SPEC.md) §§1–10, §14;
[`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md) Stages 4, 6, 7, and 8d–8e;
[`CLUSTER_SPEC.md`](../../CLUSTER_SPEC.md) §13 (client routing), §17 (coverage).

## Status

**Shipped** (Stages 4a–4d + 6 + 7 + 8d–8e) — freeze label
`SDK_API_VERSION` = `1.0`.

| Area | What you get |
|------|----------------|
| Embedded | `Dingo::open`, JSON/bytes put/get/delete, scan + streaming iter |
| Filters | SDK-native `Filter` / `find` / `query`, secondary field indexes, budgets |
| History | Per-key immutable event stream |
| Chunks | Completeness-aware `get_payload` for large bodies |
| Remote | `Dingo::connect("dingo://host:port")` framed `dingo-rpc-v1` TCP (handshake + length-prefixed JSON); auth token, deadline, retry; optional diagnostic line mode |
| Parity | Remote put/get/delete/scan, history, indexes, `get_payload`, server-side find, `directory` |
| Cluster | `create_cluster` / `open_cluster`, client partition directory cache, `find_with_coverage` |
| Network multi-hop | `serve_cluster_node` + multi-seed connect; leader routing from directory |

Application developers do not need to know about frames or segments.

## Surface

| API | Role |
|-----|------|
| `Dingo::open` | Create-or-open store directory with safe defaults |
| `Dingo::connect` / `connect_with` | Remote `dingo://host:port[/label]` or multi-seed `h1:p1,h2:p2` + `ConnectOptions` |
| `Dingo::create_cluster` / `open_cluster` | In-process multi-node cluster; same collection API + route cache |
| `ClientDirectoryCache` | Client partition → leader cache; refresh on stale placement |
| `serve_store` / `serve_store_with` | Bounded TCP server (`ServeOptions` token, limits, shutdown; `directory` op) |
| `serve_cluster_node` | Serve one cluster node; advertise placement + endpoints |
| `ServerLimits` / `ServerRuntime` / `SERVER_PROFILE` | Connection admission, idle/drain timeouts, stats (DEF-030) |
| `PROTOCOL_PROFILE` / `RPC_WIRE_LABEL` / frame helpers | Framed RPC handshake + length-prefixed messages (DEF-031) |
| `Dingo::collection` | Lazy named collection handle (no disk write) |
| `Dingo::list_collections` / `rebuild_catalogs` | Derived catalog (rebuild embedded only) |
| `Collection::put` / `get` / `delete` | JSON values (serde) |
| `Collection::put_bytes` / `get_bytes` | Opaque byte payloads |
| `Collection::get_payload` | Completeness-aware chunked read (embedded + remote) |
| `Collection::scan_keys` / `scan_json` | Bounded live scan |
| `Collection::scan_json_iter` / `scan_json_page` | Streaming / paged JSON rows (embedded; DEF-026) |
| `Collection::find` / `find_json` / `query` | Filters + index acceleration (embedded + remote server-side) |
| `Collection::find_with_coverage` | Cluster find with explicit partition coverage |
| `Collection::indexes` | Create / drop / rebuild / list / continue_build secondary indexes (DEF-027 lifecycle; embedded + remote create/rebuild) |
| `Collection::history` | Immutable event stream for one key (embedded + remote) |
| `handle_connection` / `handle_connection_with` / `handle_connection_shared` | Per-connection server dispatch (shared store owner for workers) |
| `Filter` / `QueryOptions` / `QueryBudget` | Predicates + limit/order/budget (docs, bytes, result memory) / `allow_partial_coverage` |
| `Filter::to_sda` / `matches_sda` / `QueryPlan` | Filter→SDA alignment + versioned plans (`QUERY_PLAN_PROFILE`, DEF-028) |
| `ResourceLimits` / `CancelToken` / `RESOURCE_PROFILE` | Host depth/payload/RPC ceilings + cooperative cancel (DEF-029) |
| `WriteReceipt` / `DeleteReceipt` | Event identity + achieved durability |
| `Error::code` / `ErrorCode` | Stable machine codes (DX §15) |
| `SDK_API_VERSION` | Product freeze label for this collection surface |

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

In-process cluster:

```rust
use dingo_sdk::{json, ClusterConfig, Dingo, Filter, QueryOptions};

# let dir = tempfile::tempdir().unwrap();
let mut db = Dingo::create_cluster(
    ClusterConfig::development(dir.path().join("cluster")).with_virtual_partitions(16),
)?;
{
    let mut users = db.collection("users")?;
    users.put("user-42", &json!({ "status": "active" }))?;
    let covered = users.find_with_coverage(
        &Filter::field("status").eq("active"),
        QueryOptions::new().allow_partial_coverage(),
    )?;
    let _ = covered.coverage.is_complete();
}
# Ok::<(), dingo_sdk::Error>(())
```

## Subject encoding

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

## Out of scope (this crate)

These live elsewhere or remain product follow-ons:

- Mutual TLS / multi-tenant ACLs (shared token only for Stage 7e-style auth)
- SDA examination of holes / recovery units — see [`dingo-examine`](../dingo-examine)
- Unique secondary indexes with partition consistency scopes
- Full DX §15 codes that only apply beyond the current cluster profile
- Network Raft log shipping (multi-hop client routing is shipped; quorum write
  path remains in-process `open_cluster`)
