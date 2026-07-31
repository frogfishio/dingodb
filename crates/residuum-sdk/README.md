# residuum-sdk

**Collection SDK** for ResiduumDB: the ordinary application surface.

Open a local store, connect to a remote server, or (optionally) open an
in-process multi-node cluster. Name a collection; put/get/delete JSON or bytes;
filter JSON documents; use pluggable query dialects that compile to pure SDA
(`dql` official human surface — [USER_GUIDE](../../doc/RQL/USER_GUIDE.md); also
`json` / `mongo` / `sql` mimicry / raw `sda`); manage secondary indexes;
inspect per-key history — without learning frames or segments for common paths.

Freeze label: `SDK_API_VERSION` = `1.0`.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Application put/get/find on a local file | **`residuum-sdk`** (this crate) |
| Same API over TCP (`residuum serve`) | **`residuum-sdk`** (`Residuum::connect`) |
| CLI | [`residuum-cli`](https://crates.io/crates/residuum-cli) |
| Raw subject store / salvage | [`residuum-store`](https://crates.io/crates/residuum-store) |
| Embed a TCP server | [`residuum-server`](https://crates.io/crates/residuum-server) |

## Install

```toml
[dependencies]
residuum-sdk = "0.2"   # MPL-2.0: embedded + remote (index + engine + RQL cut)
```

Optional in-process multi-node cluster (pulls AGPL `residuum-cluster`):

```toml
residuum-sdk = { version = "0.2", features = ["cluster"] }
```

Or: `cargo add residuum-sdk`

### License

| Feature set | Effective license of your dependency graph |
|-------------|--------------------------------------------|
| Default (embedded + remote client) | **MPL-2.0** (+ MIT `residuum-client` / `residuum-format` / `residuum-sda`) |
| `features = ["cluster"]` | Adds **AGPL-3.0-or-later** `residuum-cluster` |

Network **serve** is a separate AGPL crate (`residuum-server`), not a default
dependency of this SDK.

## Quick examples

### Embedded

```rust
use residuum_sdk::{json, Residuum, Filter};

# let dir = tempfile::tempdir().unwrap();
# let path = dir.path().join("app.dingo");
let mut db = Residuum::open(&path)?;
{
    let mut users = db.collection("users")?;
    users.put(
        "user-42",
        &json!({ "name": "Alice", "status": "active", "age": 30 }),
    )?;
    users.indexes()?.create("by-status", &["status"])?;

    let rows = users.find(&Filter::field("status").eq("active"))?;
    // SQL mimicry dialect → pure SDA predicate (doc/SDA/DIALECTS.md)
    let via_sql = users.find_dialect(
        "sql",
        "SELECT * WHERE status = 'active' AND age >= 18",
    )?;
    let hist = users.history("user-42")?;
    let _ = (rows, via_sql, hist);
}
# Ok::<(), residuum_sdk::Error>(())
```

### Remote

Requires a running server (`residuum serve ./app.dingo` or `residuum serve --token SECRET`):

```rust
use residuum_sdk::{json, ConnectOptions, Residuum};
use std::time::Duration;

let mut db = Residuum::connect("residuum://127.0.0.1:7434/app")?;
// With auth and timeouts:
let mut db = Residuum::connect_with(
    "residuum://127.0.0.1:7434/app",
    ConnectOptions::new()
        .auth_token("SECRET")
        .request_timeout(Duration::from_secs(10))
        .max_connect_attempts(5),
)?;
db.collection("users")?
    .put("user-42", &json!({ "name": "Alice" }))?;
# Ok::<(), residuum_sdk::Error>(())
```

### In-process cluster

Requires `features = ["cluster"]`:

```rust
use residuum_sdk::{json, ClusterConfig, Residuum, Filter, QueryOptions};

# let dir = tempfile::tempdir().unwrap();
let mut db = Residuum::create_cluster(
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
# Ok::<(), residuum_sdk::Error>(())
```

## What you get

| Area | Capability |
|------|------------|
| Embedded | `Residuum::open`, JSON/bytes put/get/delete, scan + streaming iter |
| Filters | SDK-native `Filter` / `find` / `query`, secondary field indexes, budgets |
| Multi-collection join | `Residuum::query().from(..).join(..).on(X,Y).collect()`; `.map_sda(..)` normalises |
| SDA/ENR text queries | `Collection::sda` / `filter_sda` (DX §7.6); multi-collection `Residuum::enr_query().bind(..).run` (`Match`/`enrich`) or `Residuum::sda(&[…], program)` |
| History | Per-key immutable event stream |
| Chunks | Completeness-aware `get_payload` for large bodies |
| Remote | `Residuum::connect("residuum://host:port")` framed `dingo-rpc-v1` TCP; auth token, deadline, retry |
| Parity | Remote put/get/delete/scan, history, indexes, `get_payload`, server-side find, `directory` |
| Cluster | Feature `cluster`: `create_cluster` / `open_cluster`, directory cache, `find_with_coverage` |

Application developers do not need to know about frames or segments.

## API surface

| API | Role |
|-----|------|
| `Residuum::open` | Create-or-open store directory with safe defaults |
| `Residuum::connect` / `connect_with` | Remote `residuum://host:port[/label]` or multi-seed |
| `Residuum::create_cluster` / `open_cluster` | In-process multi-node (`cluster` feature) |
| `Residuum::collection` | Lazy named collection handle |
| `Collection::put` / `get` / `delete` | JSON values (serde) |
| `Collection::put_bytes` / `get_bytes` | Opaque byte payloads |
| `Collection::get_payload` | Completeness-aware chunked read |
| `Collection::scan_keys` / `scan_json` / `scan_json_iter` / `scan_json_page` | Live scan |
| `Collection::find` / `find_json` / `query` | Filters + index acceleration |
| `Residuum::query` | Multi-collection equijoin (`from` / `join` / `on`) + optional SDA map |
| `Collection::sda` / `filter_sda` | Raw SDA/ENR1 text over one collection (DX §7.6) |
| `Residuum::enr_query` / `sda_query` / `sda` | Bind collections → free names + pure SDA/ENR1 text (`Match`/`enrich`) |
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

- TCP accept loop / authz / admission — [`residuum-server`](https://crates.io/crates/residuum-server)
- SDA examination of holes / recovery units — [`residuum-examine`](https://crates.io/crates/residuum-examine)
- Network Raft log shipping as a default path (quorum writes remain in-process
  `open_cluster`; experimental multi-process serve is separate)

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`residuum-store`](https://crates.io/crates/residuum-store) | MPL-2.0 | Single-node store |
| [`residuum-client`](https://crates.io/crates/residuum-client) | MIT | Wire framing (re-exported) |
| [`residuum-server`](https://crates.io/crates/residuum-server) | AGPL-3.0-or-later | TCP serve |
| [`residuum-cluster`](https://crates.io/crates/residuum-cluster) | AGPL-3.0-or-later | Partitions / Raft (`cluster` feature) |
| [`residuum-cli`](https://crates.io/crates/residuum-cli) | AGPL-3.0-or-later | Operator CLI |

## Documentation

- DX / product surface: [DX_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/DX_SPEC.md)
- Project overview: [README.md](https://github.com/frogfishio/dingodb/blob/main/README.md)
- Licensing: [doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md)

## License

MPL-2.0 for this crate's sources (default features). Enabling `cluster` adds
AGPL dependencies — see the install section above.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb).
