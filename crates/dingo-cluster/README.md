# dingo-cluster

Cluster federation for DingoDB: partition routing, coverage records, placement
directory, multi-node stores built on ordinary [`dingo-store`](../dingo-store)
nodes, per-partition Raft-equivalent consensus, distributed find with coverage
honesty, and interruptible rebalance.

Normative sources: repository root [`CLUSTER_SPEC.md`](../../CLUSTER_SPEC.md);
[`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md) Stage 8.

## Status

**Shipped** (Stage **8a–8f** complete) for the development and dependable-local
profiles. Freeze label `CLUSTER_PROFILE_VERSION` = `v1`.

| Piece | Role |
|-------|------|
| Partition hash profile | Stable `partition_key` → virtual partition id |
| Consistency / read modes | `partition-linearizable`, `convergent-append`; linearizable / available / salvage reads |
| Coverage | Requested / completed / unavailable partitions on every multi-partition result |
| Placement directory | Partition → replica set, leader, term, placement epoch (`placement.json`) |
| In-process cluster | Development (1 node) and dependable-local (3 voting nodes) profiles |
| Raft (8b) | Per-partition elections, log matching, majority commit evidence |
| Raft persistence (DEF-035) | Durable hard state, log, membership, snapshots under `raft/node-*/p*` (`dingo-raft-persist-v1`) |
| Network Raft RPC (DEF-036) | RequestVote / AppendEntries / InstallSnapshot / ReadIndex over framed transport (`dingo-raft-rpc-v1`) |
| Write path (linearizable) | Propose → replicate → commit → apply to replica stores |
| Convergent-append (8c) | Any online replica may accept; dual-accept across splits; `reconcile` by content hash |
| Find / scan (8e) | `scan_with` / `find` + `FindResult` query id + resource-budget honesty |
| Rebalance (8f) | Interruptible §14 step machine; joint config; epoch activation |
| Node salvage | Each node directory remains an ordinary store without cluster software |

SDK surface (8d–8e): `dingo_sdk::Dingo::create_cluster` / `open_cluster` with
client directory cache; `find_with_coverage` / `allow_partial_coverage`.

### Network multi-node serve (experimental)

Process-per-node TCP beyond the in-process `Cluster` handle.

- `endpoints.json` — node index → `host:port` (`load_endpoints` / `upsert_endpoint`)
  — **routing hints only**; never write authority (DEF-036 fences epoch/cluster)
- `dingo serve-cluster CLUSTER_ROOT --node N --bind 127.0.0.1:PORT --experimental-network-cluster`
  — opens `nodes/node-N`, upserts this process into `endpoints.json`, and reloads
  endpoints on every `directory` RPC so late joiners appear without restart
- Server: `dingo_server::serve_cluster_node` (requires
  `ServeOptions::experimental_network_cluster(true)`); `RemoteClient` multi-hop
  routes keyed ops from the cached directory
- **Control-plane Raft RPC (DEF-036):** `raft_request_vote` /
  `raft_append_entries` / `raft_install_snapshot` / `raft_read_index` when
  `ServeOptions::raft` is attached (`RaftServerState` / durable peer stores).
  Profile `RAFT_RPC_PROFILE` = `dingo-raft-rpc-v1`.
- Profile freeze: `CLUSTER_PROFILE_VERSION` = `v1` (**in-process** profile)
- Bind policy: loopback by default; non-loopback plaintext needs
  `--allow-insecure-bind` (DEF-002)
- Demo: `scripts/demos/08_kill_a_node.sh`

**Data-plane** collection put/get on a served node still apply to that node's
store (single-node RPC dispatch) until DEF-037 routes client writes through
network Raft propose. In-process quorum remains `Dingo::open_cluster`.

Archive tiers: see `dingo-store` Stage 9 and `doc/RUNBOOK_RETENTION.md`
(filesystem, `object:local:`, and S3/GCS mirrors).

## Consensus rules (published)

See module docs on [`dingo_cluster::raft`](src/raft.rs),
[`dingo_cluster::raft_persist`](src/raft_persist.rs), and CLUSTER_SPEC §10:

- **Elections** — majority of the *configured* voter set; log up-to-date check.
- **Log matching** — AppendEntries prev term/index; truncate on conflict.
- **Commit** — majority `match_index` and entry term equals leader term.
- **Fencing** — higher term steps down leaders; wall clock alone does not fence.
- **Membership (8f)** — `set_voters` during rebalance; old placement or joint
  config remains explicit on interrupt.
- **Persistence (DEF-035)** — hard state and log flush before vote grant /
  AppendEntries success; torn tails and corrupt snapshots discarded; leadership
  role is volatile across restart; profile `RAFT_PERSIST_PROFILE` =
  `dingo-raft-persist-v1`.
- **Network RPC (DEF-036)** — same vote/append/snapshot/read-index rules over a
  transport; placement epoch and cluster id fence every request; endpoints are
  not authority; profile `RAFT_RPC_PROFILE` = `dingo-raft-rpc-v1`.

## Governing rule

> Consensus controls the right to write. The data remains able to speak for itself.

Losing the control plane or placement directory must not make surviving segments
unreadable. Open any node path with `dingo_store::Store` / `salvage` as usual.

## Quick example

```rust
use dingo_cluster::{Cluster, ClusterConfig, ConsistencyMode, ReadMode, ScanOptions};
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
// Full run (or use begin_rebalance + advance_rebalance per step).
let report = cluster
    .rebalance_partition(p, vec![NodeId::new(0), NodeId::new(1)])
    .unwrap();
assert_eq!(report.job.phase, RebalancePhase::Reclaimed);
```

## Profiles ([CLUSTER_SPEC §23](../../CLUSTER_SPEC.md))

- **Development** — one node, one replica, partition-linearizable without fault
  tolerance; `replicated_durability_available() == false`.
- **Dependable local** — three voting storage nodes, virtual partitions across
  all nodes, quorum durable acknowledgement; leader re-election when a majority
  remains online; rebalance may shrink/grow per-partition replica sets.

## Layout

```text
cluster-root/
  cluster.json          # cluster id, profile, virtual partition count, hash profile
  placement.json        # partition → replica set / leader / epoch
  endpoints.json        # optional node → host:port for network serve
  nodes/
    node-0/             # ordinary dingo-store directory
    node-1/
    ...
```
