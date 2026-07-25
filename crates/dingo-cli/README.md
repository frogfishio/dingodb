# dingo (CLI)

Operator and everyday CLI for DingoDB (Stage 7 + network multi-node serve).

Install from this workspace:

```sh
cargo install --path crates/dingo-cli
```

## Commands

```text
# Everyday data path
dingo put ./app.dingo users/user-42 --json '{"name":"Alice"}'
dingo get ./app.dingo users/user-42
dingo delete ./app.dingo users/user-42
dingo list ./app.dingo
dingo list ./app.dingo users
dingo collections ./app.dingo
dingo put-bytes ./app.dingo artifacts/build-19 ./build.bin
dingo history ./app.dingo users/user-42

# Operator path
dingo doctor ./app.dingo
dingo salvage ./damaged.dingo --output ./recovered.dingo
dingo export-live ./damaged.dingo --output ./live-only.dingo

# Single-node TCP server (development; Dingo::connect)
# Default bind is loopback. Non-loopback plaintext needs --allow-insecure-bind.
dingo serve ./app.dingo --bind 127.0.0.1:7434
dingo serve ./app.dingo --bind 127.0.0.1:7434 --token SECRET

# Experimental multi-node routing/advertise (NOT network quorum replication)
dingo serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster
dingo serve-cluster ./cluster --node 1 --bind 127.0.0.1:7435 --token SECRET --experimental-network-cluster

# Global flags
dingo --version
dingo --license
dingo --json-out doctor ./app.dingo
```

## Guarantees

- `doctor` is **read-only** (`Store::open_inspect` + examination units) — no repairs, compact, or catalog writes.
- `salvage` never mutates the **source**; it copies verified frames and writes a recovery manifest (DEF-011).
- `export-live` materialises **current live state** only (new lineage); prefer `salvage` when history/holes matter.
- `--json-out` emits stable machine-readable output (distinct from put `--json` body).
- Nonzero exit status when an operation fails its guarantee.
- Auth token: `--token` or environment `DINGO_TOKEN`.
- **Bind policy (DEF-002):** `serve` / `serve-cluster` default to `127.0.0.1`.
  Non-loopback plaintext binds are refused unless `--allow-insecure-bind`
  (TLS not implemented yet). Startup prints transport/auth/durability/replication status.
- **`serve-cluster` is experimental:** requires `--experimental-network-cluster`.
  Writes apply to **this node only**; three processes do not equal replicated durability.

## Remote SDK clients

```text
dingo serve ./app.dingo --bind 127.0.0.1:7434
# Optional shared token (or env DINGO_TOKEN):
dingo serve ./app.dingo --token SECRET
```

```rust
use dingo_sdk::{ConnectOptions, Dingo};

let mut db = Dingo::connect("dingo://127.0.0.1:7434/app")?;
let mut db = Dingo::connect_with(
    "dingo://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
```

Cluster nodes (after `Dingo::create_cluster` laid down `cluster.json` /
`placement.json` / `nodes/`) — experimental routing only:

```text
dingo serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster
dingo serve-cluster ./cluster --node 1 --bind 127.0.0.1:7435 --experimental-network-cluster
```

SDK multi-hop routing uses the advertised `directory` + `endpoints.json`.
**In-process quorum** remains `Dingo::open_cluster`. Network Raft log shipping
is future work. Do not treat multi-process `serve-cluster` as replicated
storage. Demo: `scripts/demos/08_kill_a_node.sh`.

## Normative

DX_SPEC §§4.2, §§13–14; DELIVERY_PLAN Stage 7 (+ 7e/7f) and product follow-on
network multi-hop; CLUSTER_SPEC directory / endpoints.
