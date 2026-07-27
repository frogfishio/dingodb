# dingo-server

**AGPL-3.0-or-later** networked serve path for DingoDB: bounded accept loop,
authorization, protocol admission, TLS bind policy, and network Raft control /
data-plane glue.

Use this crate when you embed a DingoDB TCP server in a Rust binary (the
`dingo` CLI does exactly that). Application collection APIs and remote clients
live in [`dingo-sdk`](https://crates.io/crates/dingo-sdk). Wire framing is MIT
[`dingo-client`](https://crates.io/crates/dingo-client).

## When to use this crate

| You want… | Use |
|-----------|-----|
| Embedded local store, no network | [`dingo-sdk`](https://crates.io/crates/dingo-sdk) only |
| CLI (`dingo serve`, `dingo doctor`, …) | [`dingo-cli`](https://crates.io/crates/dingo-cli) |
| Programmatic TCP serve from Rust | **`dingo-server`** (this crate) |
| Wire framing / handshake only | [`dingo-client`](https://crates.io/crates/dingo-client) |

## Install

```toml
[dependencies]
dingo-server = "0.1"
dingo-store = "0.1"   # open/create the store path you serve
```

Or: `cargo add dingo-server`

> **License note:** This crate is AGPL-3.0-or-later. Network use of a modified
> version triggers the AGPL source-offer obligation. Prefer MIT
> `dingo-client` + MPL `dingo-sdk` (embedded) if you only need a local store.

## Quick example

Single-node serve on loopback (development):

```rust
use dingo_server::{serve_store_with, ServeOptions};
use dingo_store::Store;
use std::path::Path;

fn run(store_path: &Path) -> Result<(), dingo_sdk::Error> {
    // Ensure the store exists before serving.
    let _ = Store::create(store_path)?;

    let opts = ServeOptions::default()
        .auth_token("SECRET"); // optional shared token

    // Blocks the current thread in the accept loop.
    // Default bind policy: loopback only for plaintext.
    serve_store_with(store_path, "127.0.0.1:7434", opts)
}
```

Clients then connect with the SDK:

```rust
use dingo_sdk::{ConnectOptions, Dingo};

let mut db = Dingo::connect_with(
    "dingo://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
# Ok::<(), dingo_sdk::Error>(())
```

## API surface

| API | Role |
|-----|------|
| `serve_store` / `serve_store_with` | Single-node TCP serve over a store directory |
| `serve_cluster_node` | Multi-node node process (experimental network cluster) |
| `ServeOptions` | Auth token, TLS, bind policy, admission, Raft, shutdown |
| `handle_connection` / `handle_connection_with` | Per-connection RPC dispatch |
| `ServerRuntime` / `ServerLimits` | Connection caps, idle timeout, drain |
| `AdmissionController` / `AdmissionLimits` | Rate limits, auth lockout, replay window |
| `AuthzPolicy` / `Principal` / `Privilege` | Principal privileges + audit chain |
| `validate_bind` / `ServeStartupReport` | Loopback-default bind policy (DEF-002) |
| `RaftServerState` / `TcpRaftTransport` | Network Raft RPC (vote / append / snapshot / read-index) |

## Serve options (highlights)

```rust
use dingo_server::ServeOptions;
use dingo_sdk::TlsServerOptions;

let opts = ServeOptions::default()
    .auth_token("SECRET")
    // .tls(TlsServerOptions { /* cert, key, optional client CA */ })
    .allow_insecure_bind(false) // refuse non-loopback plaintext
    .experimental_network_cluster(false);
```

**Bind policy:** plaintext binds default to loopback. Non-loopback plaintext
requires `allow_insecure_bind(true)`. Prefer TLS (`ServeOptions::tls`) for
public binds.

**`serve_cluster_node`:** experimental multi-process cluster serve. Requires
`experimental_network_cluster(true)`. When Raft attaches, data-plane put/delete
use partition propose (DEF-037) and control-plane `raft_*` RPCs (DEF-036).
Directory-only fallback if attach fails. Not production-ready; durable
rebalance, repair, and Jepsen gates remain open (DEF-038+).

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`dingo-sdk`](https://crates.io/crates/dingo-sdk) | MPL-2.0 | Collection + remote client API |
| [`dingo-client`](https://crates.io/crates/dingo-client) | MIT | Framed RPC + handshake |
| [`dingo-cluster`](https://crates.io/crates/dingo-cluster) | AGPL-3.0-or-later | Partitions, Raft, rebalance |
| [`dingo-cli`](https://crates.io/crates/dingo-cli) | AGPL-3.0-or-later | `dingo serve` binary |

## License

AGPL-3.0-or-later.

Part of [DingoDB](https://github.com/frogfishio/dingodb). Multi-tier license map:
[doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md).
