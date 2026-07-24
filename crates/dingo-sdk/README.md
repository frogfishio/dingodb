# dingo-sdk

Stage 4 embedded collection SDK for DingoDB: ordinary `open` + named
collections with JSON and raw-byte put/get/delete, JSON filters, and streaming
scan over the Stage 3 [`dingo-store`](../dingo-store) append store.

Normative sources: repository root [`DX_SPEC.md`](../../DX_SPEC.md) §§1–7
(journeys 1–3, 6 partial), [`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md) Stage 4.

## Status

**Stage 4a–4d** — open, JSON/bytes put/get/delete, scan + streaming iter,
SDK-native filters (object + fluent builder, limit/order), stable `ErrorCode`
taxonomy, write receipts with achieved durability.

Application developers do not need to know about frames or segments.

## Surface

| API | Role |
|-----|------|
| `Dingo::open` | Create-or-open store directory with safe defaults |
| `Dingo::collection` | Lazy named collection handle (no disk write) |
| `Collection::put` / `get` / `delete` | JSON values (serde) |
| `Collection::put_bytes` / `get_bytes` | Opaque byte payloads |
| `Collection::scan_keys` / `scan_json` | Bounded live scan |
| `Collection::scan_json_iter` | Streaming JSON rows |
| `Collection::find` / `find_json` / `query` | Filters (DX §7.1–7.2) |
| `Filter` / `QueryOptions` / `SortOrder` | Predicate AST + limit/order |
| `WriteReceipt` / `DeleteReceipt` | Event identity + achieved durability |
| `PutOptions` | Optional durability override (default: durable) |
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
    let alice = users.get("user-42")?;
    assert_eq!(alice.unwrap()["name"], "Alice");

    // Object-style filter (no SDA required)
    let rows = users.find_json(&json!({
        "status": "active",
        "age": { "$gte": 18 }
    }))?;

    // Fluent builder
    let rows = users
        .query()
        .where_eq("status", "active")
        .order_by("age", SortOrder::Desc)
        .limit(100)
        .collect()?;
    let _ = (rows, Filter::always());
}
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

Both layouts are Stage 4 draft conventions above the Stage 3 opaque-byte store.

## Non-goals (yet)

- Secondary indexes and query budgets (Stage 6)
- Network `Dingo::connect`, CLI doctor/salvage (Stage 7)
- SDA examination of holes / recovery units — see [`dingo-examine`](../dingo-examine) (Stage 5)
- Full DX §15 codes that only apply in cluster mode
