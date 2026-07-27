# dingo-cluster

**Cluster federation** for DingoDB: partition routing, coverage records,
placement directory, multi-node stores built on ordinary
[`dingo-store`](https://crates.io/crates/dingo-store) nodes, per-partition
Raft-equivalent consensus, convergent-append, distributed find with coverage
honesty, and interruptible rebalance.

Most applications should open a cluster through
[`dingo-sdk`](https://crates.io/crates/dingo-sdk) (`features = ["cluster"]`).
Use this crate directly for lower-level partition/Raft/rebalance control.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Collection API on a multi-node cluster | [`dingo-sdk`](https://crates.io/crates/dingo-sdk) with `cluster` |
| Partition, Raft, coverage, rebalance APIs | **`dingo-cluster`** (this crate) |
| Single-node store only | [`dingo-store`](https://crates.io/crates/dingo-store) |
| TCP serve of a cluster node | [`dingo-server`](https://crates.io/crates/dingo-server) / `dingo serve-cluster` |

## Install

```toml
[dependencies]
dingo-cluster = "0.1"
```

Or: `cargo add dingo-cluster`

> **License:** AGPL-3.0-or-later. Network use of a modified version triggers the
> AGPL source-offer obligation. The SDK's default features do **not** depend on
> this crate; only `dingo-sdk` with `features = ["cluster"]` pulls it in.

## Status

**Shipped** (Stage 8a–8f) for development and dependable-local profiles.
Freeze label `CLUSTER_PROFILE_VERSION` = `v1`.

| Piece | Role |
|-------|------|
| Partition hash profile | Stable `partition_key` → virtual partition id |
| Consistency / read modes | `partition-linearizable`, `convergent-append`; linearizable / available / salvage reads |
| Coverage | Requested / completed / unavailable partitions on every multi-partition result |
| Placement directory | Partition → replica set, leader, term, placement epoch |
| In-process cluster | Development (1 node) and dependable-local (3 voting nodes) |
| Raft | Per-partition elections, log matching, majority commit evidence |
| Raft persistence | Durable hard state, log, membership, snapshots |
| Network Raft RPC | RequestVote / AppendEntries / InstallSnapshot / ReadIndex |
| Convergent-append | Dual-accept across splits; reconcile by content hash |
| Find / scan | `find` / `scan_page` + coverage on every page; authenticated continuation (`dingo-query-continuation-v1`); deterministic subject merge |
| Rebalance | Interruptible step machine; joint config; epoch activation |
| Anti-entropy / repair | Hierarchical inventory; majority/integrity source select (never mtime); audited, rate-limited copies (`dingo-anti-entropy-v1`) |

Each node directory remains an ordinary `dingo-store` and can be salvaged
without cluster software.

## Quick example

```rust
use dingo_cluster::{Cluster, ClusterConfig, ReadMode, ScanOptions};
use dingo_store::DurabilityMode;

# let dir = tempfile::tempdir().unwrap();
let cfg = ClusterConfig::development(dir.path().join("dev-cluster"))
    .with_virtual_partitions(16);
let mut cluster = Cluster::create(cfg).unwrap();

let ack = cluster
    .put("users/alice", br#"{"ok":true}"#, DurabilityMode::Durable)
    .unwrap();
assert!(ack.committed);
assert_eq!(ack.replica_acks, 1);

let got = cluster
    .get("users/alice", ReadMode::Linearizable)
    .unwrap();
assert_eq!(got.value.as_deref(), Some(br#"{"ok":true}"#.as_slice()));
assert!(got.coverage.is_complete());

let found = cluster.find(ScanOptions::new().prefix("users/")).unwrap();
assert!(found.coverage.is_complete());
assert!(found.query_id.starts_with("q-"));
```

## Rebalance

```rust
use dingo_cluster::{Cluster, ClusterConfig, NodeId, PartitionId, RebalancePhase};

# let dir = tempfile::tempdir().unwrap();
let mut cluster = Cluster::create(
    ClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(4),
).unwrap();

let p = PartitionId::new(0);
let report = cluster
    .rebalance_partition(p, vec![NodeId::new(0), NodeId::new(1)])
    .unwrap();
assert_eq!(report.job.phase, RebalancePhase::Reclaimed);
```

## Profiles

- **Development** — one node, one replica, partition-linearizable without fault
  tolerance; `replicated_durability_available() == false`.
- **Dependable local** — three voting storage nodes, virtual partitions across
  all nodes, quorum durable acknowledgement; leader re-election when a majority
  remains online.

## Layout

```text
cluster-root/
  cluster.json          # cluster id, profile, virtual partition count
  placement.json        # partition → replica set / leader / epoch
  endpoints.json        # optional node → host:port for network serve
  nodes/
    node-0/             # ordinary dingo-store directory
    node-1/
    ...
```

## Governing rule

> Consensus controls the right to write. The data remains able to speak for itself.

Losing the control plane or placement directory must not make surviving segments
unreadable. Open any node path with `dingo_store::Store` / `salvage` as usual.

## Network multi-node serve (experimental)

Process-per-node TCP beyond the in-process `Cluster` handle:

```text
dingo serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster
```

- `endpoints.json` holds routing hints only (not write authority).
- In-process quorum remains `Dingo::open_cluster` / `Cluster::create`.
- Demo: `scripts/demos/08_kill_a_node.sh` in the monorepo.

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`dingo-store`](https://crates.io/crates/dingo-store) | MPL-2.0 | Per-node store |
| [`dingo-sdk`](https://crates.io/crates/dingo-sdk) | MPL-2.0 | Collection API + `cluster` feature |
| [`dingo-server`](https://crates.io/crates/dingo-server) | AGPL-3.0-or-later | Network serve + Raft RPC glue |
| [`dingo-cli`](https://crates.io/crates/dingo-cli) | AGPL-3.0-or-later | `dingo serve-cluster` |

## Documentation

- Cluster spec: [CLUSTER_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/CLUSTER_SPEC.md)
- Licensing: [doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md)

## License

AGPL-3.0-or-later.

Part of [DingoDB](https://github.com/frogfishio/dingodb).
