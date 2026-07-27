# dingo (CLI)

Operator and everyday command-line interface for DingoDB.

Put and get JSON or bytes, list collections, inspect history, run read-only
`doctor`, evidence-preserving `salvage`, full `backup` / verified `restore`
(DEF-050), integrity `scrub` (DEF-051), format `migrate` (DEF-052), and start
a development TCP server (`serve`). Experimental multi-node `serve-cluster` is
available when Raft attaches (control plane + data-plane commit); not
production-ready.

Binary name: **`dingo`**. Package name on crates.io: **`dingo-cli`**.

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / ops: put, get, doctor, salvage, backup, restore, scrub, migrate, serve | **`dingo`** (this binary) |
| Embed collections in a Rust app | [`dingo-sdk`](https://crates.io/crates/dingo-sdk) |
| Pure SDA language CLI | [`sda`](https://crates.io/crates/sda) |

## Install

From crates.io:

```sh
cargo install dingo-cli
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/dingo-cli
```

> **License:** AGPL-3.0-or-later (same track as `dingo-server` and
> `dingo-cluster`).

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
dingo doctor ./app.dingo

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
dingo scrub ./app.dingo
dingo scrub ./app.dingo --status
dingo scrub ./app.dingo --once --max-files 4
dingo scrub ./app.dingo --pause
dingo scrub ./app.dingo --resume

# Format migration (source preserved; new destination store)
dingo migrate ./app.dingo --output ./migrated.dingo --preflight
dingo migrate ./app.dingo --output ./migrated.dingo
dingo migrate ./app.dingo --status
# Abandon incomplete apply (refuses completed migrations):
# dingo migrate ./app.dingo --rollback
```

## Serve (development)

```sh
# Single-node TCP server (default bind is loopback)
dingo serve ./app.dingo --bind 127.0.0.1:7434
dingo serve ./app.dingo --bind 127.0.0.1:7434 --token SECRET

# TLS: --tls-cert / --tls-key (optional mTLS via --tls-client-ca)
# Non-loopback plaintext requires --allow-insecure-bind
```

SDK clients:

```rust
use dingo_sdk::{ConnectOptions, Dingo};

let mut db = Dingo::connect("dingo://127.0.0.1:7434/app")?;
let mut db = Dingo::connect_with(
    "dingo://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
# Ok::<(), dingo_sdk::Error>(())
```

### Experimental multi-node serve

```sh
# Requires a cluster root from Dingo::create_cluster (cluster.json, nodes/, …)
dingo serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster
dingo serve-cluster ./cluster --node 1 --bind 127.0.0.1:7435 --experimental-network-cluster
```

**Experimental only.** When Raft attaches, put/delete use partition propose and
acks report `committed` only after quorum (DEF-037). If attach fails, writes
apply to the contacted node alone. Still not a production release claim.
In-process multi-replica tests remain `Dingo::open_cluster` in the SDK. Prefer
the monorepo demo `scripts/demos/08_kill_a_node.sh` when exploring multi-node
behavior.

## Global flags

```sh
dingo --version
dingo --license
dingo --json-out doctor ./app.dingo   # stable machine-readable output
```

Auth token for serve: `--token` or environment variable `DINGO_TOKEN`.

## Guarantees

| Command / policy | Guarantee |
|------------------|-----------|
| `doctor` | **Read-only** — no repairs, compact, or catalog writes |
| `salvage` | Never mutates the **source**; copies verified frames + recovery manifest |
| `export-live` | Materialises **current live state** only (new lineage) |
| `--json-out` | Stable machine-readable output (distinct from put `--json` body) |
| Exit status | Nonzero when an operation fails its guarantee |
| Bind policy | `serve` / `serve-cluster` default to loopback; non-loopback plaintext needs `--allow-insecure-bind` or TLS |

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`dingo-sdk`](https://crates.io/crates/dingo-sdk) | MPL-2.0 | Library API this CLI wraps |
| [`dingo-server`](https://crates.io/crates/dingo-server) | AGPL-3.0-or-later | Serve implementation |
| [`dingo-store`](https://crates.io/crates/dingo-store) | MPL-2.0 | Store open / salvage |
| [`dingo-examine`](https://crates.io/crates/dingo-examine) | MPL-2.0 | Doctor examination units |

## Documentation

- Project overview: [README.md](https://github.com/frogfishio/dingodb/blob/main/README.md)
- DX / operator surface: [DX_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/DX_SPEC.md)
- Licensing: [doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md)

## License

AGPL-3.0-or-later.

Part of [DingoDB](https://github.com/frogfishio/dingodb).
