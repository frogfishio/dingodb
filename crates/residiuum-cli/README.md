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
residiuum put ./app.dingo users/user-42 --json '{"name":"Alice","status":"active"}'
residiuum get ./app.dingo users/user-42
residiuum delete ./app.dingo users/user-42
residiuum list ./app.dingo
residiuum list ./app.dingo users
residiuum collections ./app.dingo
residiuum put-bytes ./app.dingo artifacts/build-19 ./build.bin
residiuum history ./app.dingo users/user-42
```

## Operator path

```sh
# Read-only health report (no repairs, no catalog writes)
residiuum doctor ./app.dingo

# Evidence-preserving salvage: never mutates the source
residiuum salvage ./damaged.dingo --output ./recovered.dingo

# Live-state only materialization (new lineage; prefer salvage when history matters)
residiuum export-live ./damaged.dingo --output ./live-only.dingo

# Full backup package (content-hashed; distinct from salvage)
residiuum backup ./app.dingo --output ./app.bak
residiuum restore ./app.bak --output ./restored.dingo
# Clone with a new store identity:
residiuum restore ./app.bak --output ./clone.dingo --reassign-identity

# Integrity scrub (bounded verification; findings under recovery/scrub/)
residiuum scrub ./app.dingo
residiuum scrub ./app.dingo --status
residiuum scrub ./app.dingo --once --max-files 4
residiuum scrub ./app.dingo --pause
residiuum scrub ./app.dingo --resume

# Format migration (source preserved; new destination store)
residiuum migrate ./app.dingo --output ./migrated.dingo --preflight
residiuum migrate ./app.dingo --output ./migrated.dingo
residiuum migrate ./app.dingo --status
# Abandon incomplete apply (refuses completed migrations):
# residiuum migrate ./app.dingo --rollback
```

## Serve (development)

```sh
# Single-node TCP server (default bind is loopback)
residiuum serve ./app.dingo --bind 127.0.0.1:7434
residiuum serve ./app.dingo --bind 127.0.0.1:7434 --token SECRET

# TLS: --tls-cert / --tls-key (optional mTLS via --tls-client-ca)
# Non-loopback plaintext requires --allow-insecure-bind
```

SDK clients:

```rust
use residiuum_sdk::{ConnectOptions, Residiuum};

let mut db = Residiuum::connect("residiuum://127.0.0.1:7434/app")?;
let mut db = Residiuum::connect_with(
    "residiuum://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
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
residiuum --json-out doctor ./app.dingo   # stable machine-readable output
```

Auth token for serve: `--token` or environment variable `RESIDIUUM_TOKEN`.

### Configuration (DEF-054)

```sh
# Validate a versioned dingo-config-v1 document before deploy
residiuum config validate ./dingo.json --mode serve
residiuum --json-out config show ./dingo.json --mode serve

# Apply config at serve time (CLI flags still override the file)
residiuum serve ./app.dingo --config ./dingo.json --bind 127.0.0.1:7434
```

Secrets belong in the environment or secret files (`serve.token_env`,
`serve.token_secret_ref` as `env:NAME` / `file:PATH`) — never inline in JSON.

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
