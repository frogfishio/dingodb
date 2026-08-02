# residiuum (CLI)

Operator and everyday command-line interface for Residiuum.

Put and get JSON or bytes, list collections, inspect history, run read-only
`doctor`, evidence-preserving `salvage`, full `backup` / verified `restore`
(DEF-050), integrity `scrub` (DEF-051), format `migrate` (DEF-052), and start
a development TCP server (`serve`). Experimental multi-node `serve-cluster` is
available when Raft attaches (control plane + data-plane commit); not
production-ready.

Binary name: **`residiuum`**. Package name on crates.io: **`residiuum-cli`**.

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / ops: put, get, doctor, salvage, backup, restore, scrub, migrate, serve | **`residiuum`** (this binary) |
| Embed collections in a Rust app | [`residiuum-sdk`](https://crates.io/crates/residiuum-sdk) |
| SDA+ENR1 hybrid language CLI | [`residiuum-sda-cli`](https://crates.io/crates/residiuum-sda-cli) (`residiuum-sda` binary) |

## Install

From crates.io:

```sh
cargo install residiuum-cli
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/residiuum-cli
```

> **License:** AGPL-3.0-or-later (same track as `residiuum-server` and
> `residiuum-cluster`).

## Everyday data path

```sh
residiuum put ./app.residiuum users/user-42 --json '{"name":"Alice","status":"active"}'
residiuum get ./app.residiuum users/user-42
residiuum delete ./app.residiuum users/user-42
residiuum list ./app.residiuum
residiuum list ./app.residiuum users
residiuum collections ./app.residiuum
residiuum put-bytes ./app.residiuum artifacts/build-19 ./build.bin
residiuum history ./app.residiuum users/user-42
```

## Operator path

```sh
# Read-only health report (no repairs, no catalog writes)
residiuum doctor ./app.residiuum

# Evidence-preserving salvage: never mutates the source
residiuum salvage ./damaged.residiuum --output ./recovered.residiuum

# Live-state only materialization (new lineage; prefer salvage when history matters)
residiuum export-live ./damaged.residiuum --output ./live-only.residiuum

# Full backup package (content-hashed; distinct from salvage)
residiuum backup ./app.residiuum --output ./app.bak
residiuum restore ./app.bak --output ./restored.residiuum
# Clone with a new store identity:
residiuum restore ./app.bak --output ./clone.residiuum --reassign-identity

# Integrity scrub (bounded verification; findings under recovery/scrub/)
residiuum scrub ./app.residiuum
residiuum scrub ./app.residiuum --status
residiuum scrub ./app.residiuum --once --max-files 4
residiuum scrub ./app.residiuum --pause
residiuum scrub ./app.residiuum --resume

# Format migration (source preserved; new destination store)
residiuum migrate ./app.residiuum --output ./migrated.residiuum --preflight
residiuum migrate ./app.residiuum --output ./migrated.residiuum
residiuum migrate ./app.residiuum --status
# Abandon incomplete apply (refuses completed migrations):
# residiuum migrate ./app.residiuum --rollback
```

## Serve

### Product path — qualified HeapKey (HAR-4)

Product remote is TLS + HeapKey, not a shared token. Prefer an explicit
qualified config (or flags). Clients use `Residiuum::connect_heap`.

```sh
# Product listener shape (TLS + deployment id; registry from store/authority install)
residiuum serve ./app.residiuum \
  --bind 127.0.0.1:7434 \
  --qualified-heap-key \
  --tls-cert ./server.crt --tls-key ./server.key \
  --deployment-id 00000000-0000-4000-8000-000000000001

# Or apply a residiuum-config-v1 file with serve.qualified_heap_key=true,
# serve.deployment_id, and serve.tls.* secret refs (never inline secrets).
residiuum serve ./app.residiuum --config ./residiuum.json --bind 127.0.0.1:7434
```

Startup labels `auth_path=qualified-heap-key (product)`.  
Non-loopback plaintext still requires `--allow-insecure-bind` (prefer TLS).

SDK product client:

```rust
use residiuum_sdk::{
    HeapCredential, RemoteHeapOptions, Residiuum, TlsClientOptions,
};

// certificate_cose + HolderSigner from local authority (HAR-2/HAR-3).
let credential = HeapCredential::new(&certificate_cose, holder)?;
let options = RemoteHeapOptions::new(
    TlsClientOptions::new("localhost").ca_path(ca_path),
    credential,
)
.expected_heap_name("accounts");

let mut heap = Residiuum::connect_heap(
    "residiuum://127.0.0.1:7434/accounts",
    options,
)?;
# let _ = heap;
# Ok::<(), residiuum_sdk::Error>(())
```

Journey:  
[HAR4_T4_CONNECT_HEAP_JOURNEY.md](../../doc/todo/heap-application-ready/HAR4_T4_CONNECT_HEAP_JOURNEY.md).

### Appendix — legacy open/token (non-product)

Stage-7 / diagnostic shared-token path. Requires **`--legacy-token-server`**.
Incompatible with `--qualified-heap-key` and with token-on-qualified (fail-closed).

```sh
residiuum serve ./app.residiuum --bind 127.0.0.1:7434 --legacy-token-server
residiuum serve ./app.residiuum --bind 127.0.0.1:7434 --legacy-token-server --token SECRET
```

If auth path is **unset** in config, validate may default apply to
`legacy-token-server` with a **warning** — that is not a product claim; set
`serve.qualified_heap_key` + TLS + `deployment_id` for product.

Legacy SDK client (token — not product remote):

```rust
use residiuum_sdk::{ConnectOptions, Residiuum};

let mut db = Residiuum::connect_with(
    "residiuum://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
# let _ = db;
# Ok::<(), residiuum_sdk::Error>(())
```

### Experimental multi-node serve

```sh
# Requires a cluster root from Residiuum::create_cluster (cluster.json, nodes/, …)
residiuum serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster
residiuum serve-cluster ./cluster --node 1 --bind 127.0.0.1:7435 --experimental-network-cluster
```

**Experimental only.** When Raft attaches, put/delete use partition propose and
acks report `committed` only after quorum (DEF-037). If attach fails, writes
apply to the contacted node alone. Still not a production release claim.
In-process multi-replica tests remain `Residiuum::open_cluster` in the SDK. Prefer
the monorepo demo `scripts/demos/08_kill_a_node.sh` when exploring multi-node
behavior.

## Global flags

```sh
residiuum --version
residiuum --license
residiuum --json-out doctor ./app.residiuum   # stable machine-readable output
```

Auth path (HAR-4): product uses `--qualified-heap-key` + TLS + `--deployment-id`
(or config equivalents). Legacy shared token uses `--legacy-token-server` with
`--token` / `RESIDIUUM_TOKEN` / `serve.token_env` — never both paths together.

### Configuration (DEF-054)

```sh
# Validate a versioned residiuum-config-v1 document before deploy
residiuum config validate ./residiuum.json --mode serve
residiuum --json-out config show ./residiuum.json --mode serve

# Apply config at serve time (CLI flags still override the file)
residiuum serve ./app.residiuum --config ./residiuum.json --bind 127.0.0.1:7434
```

Secrets belong in the environment or secret files (`serve.token_env`,
`serve.token_secret_ref` as `env:NAME` / `file:PATH`) — never inline in JSON.
Product HeapKey credentials are holder-bound certificates (not serve tokens).

## Guarantees

| Command / policy | Guarantee |
|------------------|-----------|
| `doctor` | **Read-only** — no repairs, compact, or catalog writes |
| `salvage` | Never mutates the **source**; copies verified frames + recovery manifest |
| `export-live` | Materialises **current live state** only (new lineage) |
| `config validate` | Fails on schema errors and unsafe combinations (DEF-054) |
| `config show` | Redacts tokens/secrets in the effective report |
| `--json-out` | Stable machine-readable output (distinct from put `--json` body) |
| Exit status | Nonzero when an operation fails its guarantee |
| Bind policy | `serve` / `serve-cluster` default to loopback; non-loopback plaintext needs `--allow-insecure-bind` or TLS |

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`residiuum-sdk`](https://crates.io/crates/residiuum-sdk) | MPL-2.0 | Library API this CLI wraps |
| [`residiuum-server`](https://crates.io/crates/residiuum-server) | AGPL-3.0-or-later | Serve implementation |
| [`residiuum-store`](https://crates.io/crates/residiuum-store) | MPL-2.0 | Store open / salvage |
| [`residiuum-examine`](https://crates.io/crates/residiuum-examine) | MPL-2.0 | Doctor examination units |

## Documentation

- Project overview: [README.md](https://github.com/frogfishio/dingodb/blob/main/README.md)
- DX / operator surface: [DX_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/doc/reference/product/DX_SPEC.md)
- Licensing: [doc/reference/operations/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/reference/operations/LICENSING.md)

## License

AGPL-3.0-or-later.

Part of [Residiuum](https://github.com/frogfishio/dingodb).