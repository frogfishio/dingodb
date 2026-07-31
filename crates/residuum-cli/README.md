# dingo (CLI)

Operator and everyday command-line interface for ResiduumDB.

Put and get JSON or bytes, list collections, inspect history, run read-only
`doctor`, evidence-preserving `salvage`, full `backup` / verified `restore`
(DEF-050), integrity `scrub` (DEF-051), format `migrate` (DEF-052), and start
a development TCP server (`serve`). Experimental multi-node `serve-cluster` is
available when Raft attaches (control plane + data-plane commit); not
production-ready.

Binary name: **`residuum`**. Package name on crates.io: **`residuum-cli`**.

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / ops: put, get, doctor, salvage, backup, restore, scrub, migrate, serve | **`residuum`** (this binary) |
| Embed collections in a Rust app | [`residuum-sdk`](https://crates.io/crates/residuum-sdk) |
| SDA+ENR1 hybrid language CLI | [`residuum-sda-cli`](https://crates.io/crates/residuum-sda-cli) (`residuum-sda` binary) |

## Install

From crates.io:

```sh
cargo install residuum-cli
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/residuum-cli
```

> **License:** AGPL-3.0-or-later (same track as `residuum-server` and
> `residuum-cluster`).

## Everyday data path

```sh
dingo put ./app.dingo users/user-42 --json '{"name":"Alice","status":"active"}'
dingo get ./app.dingo users/user-42
dingo delete ./app.dingo users/user-42
dingo list ./app.dingo
dingo list ./app.dingo users
dingo collections ./app.dingo
dingo put-bytes ./app.dingo artifacts/build-19 ./build.bin
dingo history ./app.dingo users/user-42
```

## Operator path

```sh
# Read-only health report (no repairs, no catalog writes)
residuum doctor ./app.dingo

# Evidence-preserving salvage: never mutates the source
dingo salvage ./damaged.dingo --output ./recovered.dingo

# Live-state only materialization (new lineage; prefer salvage when history matters)
dingo export-live ./damaged.dingo --output ./live-only.dingo

# Full backup package (content-hashed; distinct from salvage)
dingo backup ./app.dingo --output ./app.bak
dingo restore ./app.bak --output ./restored.dingo
# Clone with a new store identity:
dingo restore ./app.bak --output ./clone.dingo --reassign-identity

# Integrity scrub (bounded verification; findings under recovery/scrub/)
residuum scrub ./app.dingo
residuum scrub ./app.dingo --status
residuum scrub ./app.dingo --once --max-files 4
residuum scrub ./app.dingo --pause
residuum scrub ./app.dingo --resume

# Format migration (source preserved; new destination store)
residuum migrate ./app.dingo --output ./migrated.dingo --preflight
residuum migrate ./app.dingo --output ./migrated.dingo
residuum migrate ./app.dingo --status
# Abandon incomplete apply (refuses completed migrations):
# residuum migrate ./app.dingo --rollback
```

## Serve (development)

```sh
# Single-node TCP server (default bind is loopback)
residuum serve ./app.dingo --bind 127.0.0.1:7434
residuum serve ./app.dingo --bind 127.0.0.1:7434 --token SECRET

# TLS: --tls-cert / --tls-key (optional mTLS via --tls-client-ca)
# Non-loopback plaintext requires --allow-insecure-bind
```

SDK clients:

```rust
use residuum_sdk::{ConnectOptions, Residuum};

let mut db = Residuum::connect("residuum://127.0.0.1:7434/app")?;
let mut db = Residuum::connect_with(
    "residuum://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
# Ok::<(), residuum_sdk::Error>(())
```

### Experimental multi-node serve

```sh
# Requires a cluster root from Residuum::create_cluster (cluster.json, nodes/, …)
residuum serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster
residuum serve-cluster ./cluster --node 1 --bind 127.0.0.1:7435 --experimental-network-cluster
```

**Experimental only.** When Raft attaches, put/delete use partition propose and
acks report `committed` only after quorum (DEF-037). If attach fails, writes
apply to the contacted node alone. Still not a production release claim.
In-process multi-replica tests remain `Residuum::open_cluster` in the SDK. Prefer
the monorepo demo `scripts/demos/08_kill_a_node.sh` when exploring multi-node
behavior.

## Global flags

```sh
dingo --version
dingo --license
dingo --json-out doctor ./app.dingo   # stable machine-readable output
```

Auth token for serve: `--token` or environment variable `RESIDUUM_TOKEN`.

### Configuration (DEF-054)

```sh
# Validate a versioned dingo-config-v1 document before deploy
residuum config validate ./dingo.json --mode serve
dingo --json-out config show ./dingo.json --mode serve

# Apply config at serve time (CLI flags still override the file)
residuum serve ./app.dingo --config ./dingo.json --bind 127.0.0.1:7434
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
| [`residuum-sdk`](https://crates.io/crates/residuum-sdk) | MPL-2.0 | Library API this CLI wraps |
| [`residuum-server`](https://crates.io/crates/residuum-server) | AGPL-3.0-or-later | Serve implementation |
| [`residuum-store`](https://crates.io/crates/residuum-store) | MPL-2.0 | Store open / salvage |
| [`residuum-examine`](https://crates.io/crates/residuum-examine) | MPL-2.0 | Doctor examination units |

## Documentation

- Project overview: [README.md](https://github.com/frogfishio/dingodb/blob/main/README.md)
- DX / operator surface: [DX_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/DX_SPEC.md)
- Licensing: [doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md)

## License

AGPL-3.0-or-later.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb).
