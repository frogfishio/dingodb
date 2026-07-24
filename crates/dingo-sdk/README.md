# dingo-sdk

Collection SDK for DingoDB: ordinary `open` or remote `connect` + named
collections with JSON and raw-byte put/get/delete, JSON filters, secondary
indexes, per-key history, and query budgets over the
[`dingo-store`](../dingo-store) append store.

Normative sources: repository root [`DX_SPEC.md`](../../DX_SPEC.md) §§1–10, §14;
[`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md) Stages 4, 6, and 7.

## Status

**Stages 4a–4d + 6 + 7** — open, JSON/bytes put/get/delete, scan + streaming
iter, SDK-native filters, stable `ErrorCode`, write receipts, secondary field
indexes, query budgets, per-key history, chunked large payloads, and Stage 7
`Dingo::connect("dingo://host:port")` over line-delimited JSON TCP (same
collection put/get/delete/scan surface as embedded).

Application developers do not need to know about frames or segments.

## Surface

| API | Role |
|-----|------|
| `Dingo::open` | Create-or-open store directory with safe defaults |
| `Dingo::connect` | Remote `dingo://host:port[/label]` (Stage 7) |
| `Dingo::collection` | Lazy named collection handle (no disk write) |
| `Dingo::list_collections` / `rebuild_catalogs` | Derived catalog (rebuild embedded only) |
| `Collection::put` / `get` / `delete` | JSON values (serde) |
| `Collection::put_bytes` / `get_bytes` | Opaque byte payloads |
| `Collection::get_payload` | Completeness-aware chunked read (embedded) |
| `Collection::scan_keys` / `scan_json` | Bounded live scan |
| `Collection::scan_json_iter` | Streaming JSON rows (embedded) |
| `Collection::find` / `find_json` / `query` | Filters (+ index when ready) |
| `Collection::indexes` | Create / drop / rebuild / list secondary indexes (embedded) |
| `Collection::history` | Immutable event stream for one key (embedded) |
| `serve_store` / `handle_connection` | TCP server helpers used by `dingo serve` |
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

Remote (requires `dingo serve ./app.dingo`):

```rust
use dingo_sdk::{json, Dingo};

let mut db = Dingo::connect("dingo://127.0.0.1:7434/app")?;
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

- Authn / deadline / retry connection options (reserved; Stage 7e)
- Cluster routing (Stage 8)
- Full remote parity for indexes/history/chunk partial maps (embedded only for now)
- SDA examination of holes / recovery units — see [`dingo-examine`](../dingo-examine) (Stage 5)
- Unique secondary indexes with partition consistency scopes
- Full DX §15 codes that only apply in cluster mode
