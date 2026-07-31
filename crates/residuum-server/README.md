# residuum-server

**AGPL-3.0-or-later** networked serve path for ResiduumDB: bounded accept loop,
authorization, protocol admission, TLS bind policy, and network Raft control /
data-plane glue.

Use this crate when you embed a ResiduumDB TCP server in a Rust binary (the
`residuum` CLI does exactly that). Application collection APIs and remote clients
live in [`residuum-sdk`](https://crates.io/crates/residuum-sdk). Wire framing is MIT
[`residuum-client`](https://crates.io/crates/residuum-client).

## When to use this crate

| You want… | Use |
|-----------|-----|
| Embedded local store, no network | [`residuum-sdk`](https://crates.io/crates/residuum-sdk) only |
| CLI (`residuum serve`, `residuum doctor`, …) | [`residuum-cli`](https://crates.io/crates/residuum-cli) |
| Programmatic TCP serve from Rust | **`residuum-server`** (this crate) |
| Wire framing / handshake only | [`residuum-client`](https://crates.io/crates/residuum-client) |

## Install

```toml
[dependencies]
residuum-server = "0.1"
residuum-store = "0.1"   # open/create the store path you serve
```

Or: `cargo add residuum-server`

> **License note:** This crate is AGPL-3.0-or-later. Network use of a modified
> version triggers the AGPL source-offer obligation. Prefer MIT
> `residuum-client` + MPL `residuum-sdk` (embedded) if you only need a local store.

## Quick example

Single-node serve on loopback (development):

```rust
use residuum_server::{serve_store_with, ServeOptions};
use residuum_store::Store;
use std::path::Path;

fn run(store_path: &Path) -> Result<(), residuum_sdk::Error> {
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
use residuum_sdk::{ConnectOptions, Residuum};

let mut db = Residuum::connect_with(
    "residuum://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
# Ok::<(), residuum_sdk::Error>(())
```

## API surface

| API | Role |
|-----|------|
| `serve_store` / `serve_store_with` | Single-node TCP serve over a store directory |
| `serve_cluster_node` | Multi-node node process (experimental network cluster) |
| `ServeOptions` | Auth token, TLS, bind policy, admission, Raft, logger, metrics, shutdown |
| `handle_connection` / `handle_connection_with` | Per-connection RPC dispatch |
| `ServerRuntime` / `ServerLimits` | Connection caps, idle timeout, drain |
| `AdmissionController` / `AdmissionLimits` | Rate limits, auth lockout, replay window |
| `AuthzPolicy` / `Principal` / `Privilege` | Principal privileges + audit chain |
| `validate_bind` / `ServeStartupReport` | Loopback-default bind policy (DEF-002) |
| `load_and_validate` / `DingoConfigFile` / `ValidatedConfig` | Versioned process config (DEF-054, `dingo-config-v1`) |
| `Logger` / `LogEvent` / `MemorySink` / `log_rpc_complete` | Structured NDJSON process logs (DEF-060, `dingo-log-v1`) |
| `MetricsRegistry` / `HealthReport` / `evaluate_health` | Process metrics + health probes (DEF-061) |
| `RaftServerState` / `TcpRaftTransport` | Network Raft RPC (vote / append / snapshot / read-index) |

## Process configuration (DEF-054)

```rust
use residuum_server::{load_and_validate, ConfigMode, ConfigOverrides, ServeOptions};
use std::path::Path;

let validated = load_and_validate(
    Some(Path::new("dingo.json")),
    ConfigMode::Serve,
    ConfigOverrides {
        store_path: Some(Path::new("/data/store").into()),
        ..Default::default()
    },
)?;
// Secrets never appear in effective reports:
let report = validated.effective_report(ConfigMode::Serve);
let opts: ServeOptions = validated.apply_to_serve_options(ServeOptions::new());
# Ok::<(), residuum_server::ConfigError>(())
```

JSON documents use profile `dingo-config-v1`. Put only secret *references*
(`serve.token_env`, `serve.token_secret_ref` as `env:NAME` or `file:PATH`) in
the file — never token values. Validation refuses unsafe combinations such as
`cluster.claim_replication=true` with fewer than three expected nodes.

## Structured logging (DEF-060)

Serve paths emit versioned NDJSON (`profile: dingo-log-v1`) on stderr by
default. Events use stable names (`rpc.complete`, `guarantee.failed`,
`connection.rejected`, …) and bounded correlation fields (`request_id`,
`operation_id`, `principal_id`, requested/achieved guarantees, latency).
Credentials and request/response bodies are never logged.

```rust
use residuum_server::{log_events, Logger, MemorySink, ServeOptions};
use std::sync::Arc;

// Tests: capture lines without touching stderr.
let sink = Arc::new(MemorySink::new(64));
let log = Logger::with_sink(sink).store("/data/store").mode("serve").shared();
let opts = ServeOptions::new().logger(log);
// Production: omit `.logger(...)` — serve installs stderr NDJSON automatically.
# let _ = opts;
```

## Metrics and health (DEF-061)

Serve installs an in-process `MetricsRegistry` (`profile: dingo-metrics-v1`)
and answers:

| RPC | Auth | Role |
|-----|------|------|
| `health_live` | public | Liveness — process is handling RPCs |
| `health_ready` | public | Readiness — fails when draining, store unavailable, or replication claimed without Raft |
| `health` | Read | Detailed operator health (`dingo-health-v1`) |
| `metrics` | Admin | Bounded scrape: per-op counters + latency histograms, guarantee/admission edges |

Op labels are a fixed known set plus `other` (no collection/key/token labels).
Public probes work without a token even when the data plane requires auth.

```rust
use residuum_server::{MetricsRegistry, ServeOptions};
use std::sync::Arc;

let metrics = MetricsRegistry::new().shared();
let opts = ServeOptions::new().metrics(Arc::clone(&metrics));
// Production: omit `.metrics(...)` — serve installs a default registry.
# let _ = opts;
```

## Serve options (highlights)

```rust
use residuum_server::ServeOptions;
use residuum_sdk::TlsServerOptions;

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
| [`residuum-sdk`](https://crates.io/crates/residuum-sdk) | MPL-2.0 | Collection + remote client API |
| [`residuum-client`](https://crates.io/crates/residuum-client) | MIT | Framed RPC + handshake |
| [`residuum-cluster`](https://crates.io/crates/residuum-cluster) | AGPL-3.0-or-later | Partitions, Raft, rebalance |
| [`residuum-cli`](https://crates.io/crates/residuum-cli) | AGPL-3.0-or-later | `residuum serve` binary |

## License

AGPL-3.0-or-later.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb). Multi-tier license map:
[doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md).
