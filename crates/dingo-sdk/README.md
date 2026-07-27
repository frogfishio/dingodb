# dingo-sdk

**Collection SDK** for DingoDB: the ordinary application surface.

Open a local store, connect to a remote server, or (optionally) open an
in-process multi-node cluster. Name a collection; put/get/delete JSON or bytes;
filter JSON documents; manage secondary indexes; inspect per-key history —
without learning frames, segments, or SDA.

Freeze label: `SDK_API_VERSION` = `1.0`.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Application put/get/find on a local file | **`dingo-sdk`** (this crate) |
| Same API over TCP (`dingo serve`) | **`dingo-sdk`** (`Dingo::connect`) |
| CLI | [`dingo-cli`](https://crates.io/crates/dingo-cli) |
| Raw subject store / salvage | [`dingo-store`](https://crates.io/crates/dingo-store) |
| Embed a TCP server | [`dingo-server`](https://crates.io/crates/dingo-server) |

## Install

```toml
[dependencies]
dingo-sdk = "0.1"   # MPL-2.0: embedded + remote
```

Optional in-process multi-node cluster (pulls AGPL `dingo-cluster`):

```toml
dingo-sdk = { version = "0.1", features = ["cluster"] }
```

Or: `cargo add dingo-sdk`

### License

| Feature set | Effective license of your dependency graph |
|-------------|--------------------------------------------|
| Default (embedded + remote client) | **MPL-2.0** (+ MIT `dingo-client` / `dingo-format` / `sda-lib`) |
| `features = ["cluster"]` | Adds **AGPL-3.0-or-later** `dingo-cluster` |

Network **serve** is a separate AGPL crate (`dingo-server`), not a default
dependency of this SDK.

## Quick examples

### Embedded

```rust
use dingo_sdk::{json, Dingo, Filter};

# let dir = tempfile::tempdir().unwrap();
# let path = dir.path().join("app.dingo");
let mut db = Dingo::open(&path)?;
{
    let mut users = db.collection("users")?;
    users.put(
        "user-42",
        &json!({ "name": "Alice", "status": "active", "age": 30 }),
    )?;
    users.indexes()?.create("by-status", &["status"])?;

    let rows = users.find(&Filter::field("status").eq("active"))?;
    let hist = users.history("user-42")?;
    let _ = (rows, hist);
}
# Ok::<(), dingo_sdk::Error>(())
```

### Remote

Requires a running server (`dingo serve ./app.dingo` or `dingo serve --token SECRET`):

```rust
use dingo_sdk::{json, ConnectOptions, Dingo};
use std::time::Duration;

let mut db = Dingo::connect("dingo://127.0.0.1:7434/app")?;
// With auth and timeouts:
let mut db = Dingo::connect_with(
    "dingo://127.0.0.1:7434/app",
    ConnectOptions::new()
        .auth_token("SECRET")
        .request_timeout(Duration::from_secs(10))
        .max_connect_attempts(5),
)?;
db.collection("users")?
    .put("user-42", &json!({ "name": "Alice" }))?;
# Ok::<(), dingo_sdk::Error>(())
```

### In-process cluster

Requires `features = ["cluster"]`:

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

## What you get

| Area | Capability |
|------|------------|
| Embedded | `Dingo::open`, JSON/bytes put/get/delete, scan + streaming iter |
| Filters | SDK-native `Filter` / `find` / `query`, secondary field indexes, budgets |
| Multi-collection join | `Dingo::query().from(..).join(..).on(X,Y).collect()`; `.map_sda(..)` normalises |
| SDA/ENR text queries | `Collection::sda` / `filter_sda` (DX §7.6); multi-collection `Dingo::enr_query().bind(..).run` (`Match`/`enrich`) or `Dingo::sda(&[…], program)` |
| History | Per-key immutable event stream |
| Chunks | Completeness-aware `get_payload` for large bodies |
| Remote | `Dingo::connect("dingo://host:port")` framed `dingo-rpc-v1` TCP; auth token, deadline, retry |
| Parity | Remote put/get/delete/scan, history, indexes, `get_payload`, server-side find, `directory` |
| Cluster | Feature `cluster`: `create_cluster` / `open_cluster`, directory cache, `find_with_coverage` |

Application developers do not need to know about frames or segments.

## API surface

| API | Role |
|-----|------|
| `Dingo::open` | Create-or-open store directory with safe defaults |
| `Dingo::connect` / `connect_with` | Remote `dingo://host:port[/label]` or multi-seed |
| `Dingo::create_cluster` / `open_cluster` | In-process multi-node (`cluster` feature) |
| `Dingo::collection` | Lazy named collection handle |
| `Collection::put` / `get` / `delete` | JSON values (serde) |
| `Collection::put_bytes` / `get_bytes` | Opaque byte payloads |
| `Collection::get_payload` | Completeness-aware chunked read |
| `Collection::scan_keys` / `scan_json` / `scan_json_iter` / `scan_json_page` | Live scan |
| `Collection::find` / `find_json` / `query` | Filters + index acceleration |
| `Dingo::query` | Multi-collection equijoin (`from` / `join` / `on`) + optional SDA map |
| `Collection::sda` / `filter_sda` | Raw SDA/ENR1 text over one collection (DX §7.6) |
| `Dingo::enr_query` / `sda_query` / `sda` | Bind collections → free names + pure SDA/ENR1 text (`Match`/`enrich`) |
| `Collection::find_with_coverage` | Cluster find with explicit partition coverage |
| `Collection::indexes` | Create / drop / rebuild / list secondary indexes |
| `Collection::history` | Immutable event stream for one key |
| `Filter` / `QueryOptions` / `QueryBudget` | Predicates + limit/order/budget |
| `MultiQuery` / `map_joined_sda` | Join bag then pure SDA normalisation |
| `SdaTextQuery` / `eval_sda_program` | Text-program axis (ENR1 match bags + cardinality) |
| `WriteReceipt` / `DeleteReceipt` | Event identity + achieved durability |
| `Error::code` / `ErrorCode` | Stable machine codes |
| `SDK_API_VERSION` | Product freeze label |

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

Large bodies may be stored as chunked payloads; ordinary get returns complete
data only when every chunk verifies.

## Out of scope (this crate)

- TCP accept loop / authz / admission — [`dingo-server`](https://crates.io/crates/dingo-server)
- SDA examination of holes / recovery units — [`dingo-examine`](https://crates.io/crates/dingo-examine)
- Network Raft log shipping as a default path (quorum writes remain in-process
  `open_cluster`; experimental multi-process serve is separate)

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`dingo-store`](https://crates.io/crates/dingo-store) | MPL-2.0 | Single-node store |
| [`dingo-client`](https://crates.io/crates/dingo-client) | MIT | Wire framing (re-exported) |
| [`dingo-server`](https://crates.io/crates/dingo-server) | AGPL-3.0-or-later | TCP serve |
| [`dingo-cluster`](https://crates.io/crates/dingo-cluster) | AGPL-3.0-or-later | Partitions / Raft (`cluster` feature) |
| [`dingo-cli`](https://crates.io/crates/dingo-cli) | AGPL-3.0-or-later | Operator CLI |

## Documentation

- DX / product surface: [DX_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/DX_SPEC.md)
- Project overview: [README.md](https://github.com/frogfishio/dingodb/blob/main/README.md)
- Licensing: [doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md)

## License

MPL-2.0 for this crate's sources (default features). Enabling `cluster` adds
AGPL dependencies — see the install section above.

Part of [DingoDB](https://github.com/frogfishio/dingodb).
