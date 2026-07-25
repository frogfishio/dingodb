# DingoDB capability matrix

Status: living document for DEF-001 containment  
Audience: operators, reviewers, release notes  
Companion: [DEFECTS.md](../DEFECTS.md), [README.md](../README.md)

This matrix states what each advertised surface **actually provides today**,
with the acceptance evidence expected before a stronger label.

## Deployment profiles

| Profile | How to run | Durability / replication | Maturity | Evidence |
|---------|------------|---------------------------|----------|----------|
| Embedded single-node | `Dingo::open(path)` | Local store durability modes (`memory` / `buffered` / `durable`) | experimental / early-access | `dingo-store` / `dingo-sdk` stage suites |
| Single-node TCP | `dingo serve` (default `127.0.0.1`) | Local store only; **no** network quorum | development only | CLI `serve_*` tests; remote parity suite |
| In-process cluster | `Dingo::open_cluster` / `create_cluster` | Partition-local quorum **in one process** | integration-test harness | `dingo-cluster` stage8a–8f tests |
| Network multi-node | `dingo serve-cluster` + multi-seed connect | **Routing/advertise only**; each write hits the **serving node’s store** | experimental prototype (requires `--experimental-network-cluster`) | CLI bind/experimental gates; `stage8d_routing` |
| S3/GCS placement | `MediaLocator` + mirror env roots | Filesystem mirror of segments, not native cloud I/O SDK | experimental mirror | store tier / media tests |
| Erasure / lifecycle | scaffold APIs | Not production protection | scaffold | types/docs only until codecs land |

## Critical honesty rules

1. **Three `serve-cluster` processes do not provide replicated durability.**
   They advertise placement and endpoints; mutating RPCs still apply to the
   node that receives them unless/until network Raft (DEF-030+) lands.
2. **In-process quorum ≠ network cluster.** Prefer `Dingo::open_cluster` for
   tests that need partition-linearizable multi-replica behavior.
3. **Mirrors ≠ native cloud backends.** `s3://` / `gs://` parse and mirror
   paths are not a substitute for a production object-store connector.
4. **Draft wire.** `WIRE_PROFILE_LABEL = 1.0-draft` is not an interoperability
   freeze.
5. **Performance.** No Redis-class claim without
   [BENCHMARK_DISCLOSURE.md](BENCHMARK_DISCLOSURE.md) artifacts.

## Version label map

| Constant | Value (current) | Scope |
|----------|-----------------|--------|
| Crate / workspace semver | `0.1.0` | Packaging only |
| `SDK_API_VERSION` | `1.0` | Collection API surface freeze label |
| `CLUSTER_PROFILE_VERSION` | `v1` | In-process cluster profile |
| `WIRE_PROFILE_LABEL` | `1.0-draft` | On-disk/network frame draft |
| `CONFORMANCE_CORPUS_TAG` | `sda-standalone-v1.0` | SDA §14 corpus |

## Network bind policy (DEF-002)

| Bind | Plaintext without override | With `--allow-insecure-bind` |
|------|----------------------------|------------------------------|
| `127.0.0.1`, `::1`, `localhost` | allowed | allowed |
| `0.0.0.0`, `::`, LAN/public IPs | **refused** before accept | allowed (development only; no TLS yet) |

`serve-cluster` additionally requires `--experimental-network-cluster`.

## Writer ownership (DEF-020)

| Open path | Exclusive lock | Concurrent with serve |
|-----------|----------------|------------------------|
| `Store::open` / `Dingo::open` / CLI mutations / `dingo serve` | yes | second writer fails |
| `Store::open_inspect` / `Dingo::open_inspect` / `dingo doctor` | no | yes (read-only) |

Kill -9 releases the OS advisory lock; recovery rebuilds from segment bytes.

## Receipt honesty (DEF-014)

Remote write/delete receipts require server-proved `committed`, `acknowledgement`,
and non-zero identity fields. Missing fields yield `protocol_violation` rather
than optimistic defaults (`committed: true`, zero ids, requested durability).

## Scan completeness (DEF-012)

Ordinary `live_logical_entries` / collection scan-find paths fail closed when any
live payload is partial **or** tier coverage is incomplete (offline media). Use
`scan_live_logical` / `get_with_tier_coverage` / `get_payload` for explicit
partial maps. Secondary-index misses are authoritative only when the index
claims `complete_coverage`.

## Idempotent remote writes (DEF-010)

Mutating remote RPCs carry a client `operation_id`. Exact retries return the
original receipt; id reuse with different content yields `consistency_violation`.

## Salvage vs live export (DEF-011)

| Operation | CLI | What is preserved | Lineage |
|-----------|-----|-------------------|---------|
| Evidence salvage | `dingo salvage SRC --output DST` | Verified frames (byte-identical), history, tombstones; holes in recovery manifest | Frame event/item ids kept |
| Live-state export | `dingo export-live SRC --output DST` | Complete live payloads only | **New** store/event lineage |

Source is never mutated. Destination receives `recovery/salvage-manifest.v1.json`
in evidence mode.
Dedup evidence lives under `store-info/write_dedup.v1`.

## Durable-frontier catalogs (DEF-013)

Memory-mode publishes are visibility-only (no segment append). Persisted
collection catalogs are built from segment-derived durable state only.

## CI check

A lightweight workspace test asserts this file exists and still forbids the
disallowed production claim phrases used in public status tables. See
`crates/dingo-cli/tests/cli.rs` (`capability_matrix_document_present`).
