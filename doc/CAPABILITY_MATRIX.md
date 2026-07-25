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
| `QUERY_PLAN_PROFILE` | `dingo-query-plan-v1` | Serializable filter/query plans (DEF-028) |
| `RESOURCE_PROFILE` | `dingo-resource-v1` | Query budgets + host resource limits (DEF-029) |
| `SERVER_PROFILE` | `dingo-server-v1` | Bounded TCP server admission + drain (DEF-030) |

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

## Control-document durability (DEF-021)

Mutable metadata uses `dingo_store::write_atomic` (temp → `sync_all` → rename →
parent dirsync). Non-trivial documents (`write_dedup.v1`, `lifecycle.json`,
`endpoints.json`, `cluster.json`, `placement.json`, recovery manifests) also
retain a `*.prev` generation. Endpoint upserts take a process + OS lock so
concurrent registrations cannot drop unrelated nodes. Parse failures surface
`StoreError::CorruptControl` (or cluster `CorruptMeta`) with a recovery action
rather than silently inventing state.

## Release content (DEF-003)

| Gate | What it checks | Evidence |
|------|----------------|----------|
| Clean work tree | No uncommitted slice fragments in CI | `git status --short` empty in `ci.yml` |
| Package lists | Every member `cargo package --list` is complete | `scripts/release_content.sh` |
| Package build | Workspace builds from package file lists only | same script, temp staging tree |
| Artifact policy | Crates vs specs/demos vs non-artifacts | [RELEASE_ARTIFACTS.md](RELEASE_ARTIFACTS.md) |

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

## Crash-consistency matrix (DEF-022)

| Surface | Status | Evidence |
|---------|--------|----------|
| Machine-readable matrix | shipped (hardened) | `crates/dingo-store/crash_matrix.v1.json` |
| Failpoint framework | shipped | `dingo_store::failpoint` (`Abort`, I/O faults, short-write) |
| Persistence-order docs | shipped | [CRASH_CONSISTENCY.md](CRASH_CONSISTENCY.md) + matrix `persistence_order` |
| CI subset | shipped | `stage_def_022_crash_matrix` (default) |
| Full matrix | nightly | `DINGO_CRASH_MATRIX_FULL=1` in nightly workflow / `scripts/nightly.sh` |
| Multi-process abort | shipped | `dingo-store-crash-child` + kill before-write / after-sync |
| ENOSPC / permission / short-write | shipped | failpoint I/O actions + instrumented write sites |
| Buffered power-loss equivalence | not yet | remaining DEF-022 work |

## Write-path derived state (DEF-023)

| Surface | Status | Evidence |
|---------|--------|----------|
| In-memory durable projection | shipped | `Store` keeps `durable_index` updated only after buffered/durable append |
| No full-store rescan on ack | shipped | Write path does not call `index_from_segments`; catalogs/index from durable projection |
| Frontier index cache (v2) | shipped | `indexes/primary.idx` records sealed fingerprint + active covered length |
| Open acceleration | shipped | Matching sealed frontier → apply active tail only; else rebuild from segments |
| Rate-limited checkpoints | shipped | Full cache rewrite every N durable ops / on seal / explicit `persist_index_cache` |
| Recovery without derived state | shipped | Wipe `indexes/` + `catalogs/` + `snapshots/` still reconstructs logical state |
| Tests | shipped | `stage_def_023_write_path` (+ bench disclosure skeleton) |

## Compaction reclaim (DEF-024)

| Surface | Status | Evidence |
|---------|--------|----------|
| Phased compact job | shipped | `planned → created → verified → activated → [retention_hold] → reclaimed` |
| Durable job record | shipped | `recovery/compaction/<job_id>.job.json` (+ `.prev`) |
| Default retains sources | shipped | `compact_live` activates only; history remains in sources |
| Safe reclaim | shipped | Requires `allow_history_loss` for live-projection; never deletes output/active |
| Byte metrics | shipped | estimated/actual read, write, retained, reclaimed on `CompactReport` |
| Restart recovery | shipped | `recover_compact_jobs` on open finishes or cancels incomplete phases |
| Cancel | shipped | Cancel pre-activate jobs; refuse after activate |
| Tests | shipped | `stage_def_024_compaction` |

## Identifier generation (DEF-025)

| Surface | Status | Evidence |
|---------|--------|----------|
| Profile tag | shipped | `dingo_store::ID_PROFILE = "dingo-id-v1"` |
| OS CSPRNG | shipped | `getrandom` via `dingo_store::random_id` / `fill_random` |
| Fail closed | shipped | `StoreError::RandomUnavailable` (no time-hash fallback) |
| Random identities | shipped | `event_id`, `store_id`, job/checkpoint ids, client `operation_id`, `ClusterId::generate` |
| Sortable segment ids | shipped | LE seq + store mix; seq recovered from disk on open |
| Content item ids | shipped | `blake3(subject)[..16]` (stable, not random) |
| Tests | shipped | `stage_def_025_identifiers` + `ids` unit tests |

## Bounded-memory cursors (DEF-026)

| Surface | Status | Evidence |
|---------|--------|----------|
| Profile tag | shipped | `dingo_store::CURSOR_PROFILE = "dingo-cursor-v1"` |
| Paged store scan | shipped | `Store::scan_live_page` — subject order, bounded bodies per page |
| Continuation tokens | shipped | MAC'd opaque tokens (store_id + generation + prefix + after) |
| Generation fence | shipped | BLAKE3(store_id ‖ segment_fp ‖ live_count); stale → `CursorStale` |
| Tamper / cross-store | shipped | `StoreError::CursorInvalid` |
| SDK streaming | shipped | `scan_json_page`, `scan_json_iter` / `scan_json_iter_paged` (embedded) |
| Find scan path | shipped | Embedded filter scan pages instead of full materialization |
| Remote page RPC | not yet | Follow-on; remote still uses list/find materialization |
| Tests | shipped | `stage_def_026_cursors` + cursor unit tests |

## Secondary index lifecycle (DEF-027)

| Surface | Status | Evidence |
|---------|--------|----------|
| Profile tag | shipped | `dingo_store::INDEX_LIFECYCLE_PROFILE = "dingo-index-lifecycle-v1"` |
| Durable states | shipped | building / ready / stale / partial / failed / rebuilding on `.six` v2 |
| Build metadata | shipped | build_id, source_frontier, resume_after_subject, failure_reason |
| Snapshot + catch-up | shipped | unfenced build pages + one frontier catch-up before Ready |
| Resume | shipped | create / `continue_build` resume mid-build; failpoints at plan/mid/ready |
| Absence honesty | shipped | only Ready+complete_coverage may prove miss; Partial hits-only |
| Stale marking | shipped | put/delete surface write failures (no silent drop) |
| Unique indexes | not yet | needs enforceable partition scope (follow-on) |
| Tests | shipped | `stage_def_027_index_lifecycle` + secondary unit tests |

## Filter / SDA alignment (DEF-028)

| Surface | Status | Evidence |
|---------|--------|----------|
| Profile tag | shipped | `dingo_sdk::QUERY_PLAN_PROFILE = "dingo-query-plan-v1"` |
| Filter → SDA | shipped | `Filter::to_sda` / `matches_sda` over portable vocabulary |
| Path helpers | shipped | SDA `getPath` / `startsWith` / `strContains` (pure stdlib) |
| Absence vs Null | shipped | missing/`None` ≠ stored `null`/`Some(null)` |
| Query plans | shipped | `QueryPlan` JSON round-trip; unknown profile rejected |
| Dual corpus | shipped | native ≡ SDA ≡ embedded find / force-scan |
| Remote plan RPC | not yet | wire still carries Mongo-style filter objects |
| Tests | shipped | `stage_def_028_filter_sda` + filter unit tests |

## Resource governance (DEF-029)

| Surface | Status | Evidence |
|---------|--------|----------|
| Profile tag | shipped | `dingo_sdk::RESOURCE_PROFILE = "dingo-resource-v1"` |
| Query budget | shipped | `max_docs_scanned` / `max_bytes_scanned` / `max_result_bytes` → `query_budget_required` |
| Partial budget stop | shipped | `allow_partial_coverage` returns matches so far instead of error |
| Host JSON depth | shipped | default 64; put paths fail closed with `resource_limit` |
| Host payload / RPC line | shipped | 16 MiB defaults; remote refuse oversized lines before parse |
| Result / sort memory | shipped | budget + 64 MiB host ceiling; no spill-to-disk in this profile |
| Cancellation | shipped | `CancelToken` on `QueryOptions` / builder (embedded find loops) |
| Frame length bounds | shipped | `dingo_format::SafetyLimits` (unchanged) |
| Conn / concurrent query admission | shipped | DEF-030 bounded server (`SERVER_PROFILE`) |
| Per-tenant work quotas | not yet | follow-on |
| Tests | shipped | `stage_def_029_resource_governance` + resource unit tests |

## Bounded TCP server (DEF-030)

| Surface | Status | Evidence |
|---------|--------|----------|
| Profile tag | shipped | `dingo_sdk::SERVER_PROFILE = "dingo-server-v1"` |
| Single store owner | shipped | one `Store::open` per serve process; shared via `Arc<Mutex<Store>>` |
| Concurrent connections | shipped | thread-per-connection workers; accept loop never blocks on client I/O |
| Connection limit | shipped | `ServeOptions::max_connections` / `ServerLimits` (default 64) |
| Overload response | shipped | unsolicited `resource_limit` line then close |
| Idle timeout | shipped | configurable; default 120s read/write |
| Graceful drain | shipped | `shutdown_flag` → stop accept, wait workers, report mutation counters |
| Mutation serialization | shipped | store mutex; not held across socket I/O |
| Worker pool reuse | not yet | thread-per-conn is the draft model |
| Concurrent read snapshots | not yet | reads still take the store mutex |
| Tests | shipped | `stage_def_030_bounded_server` + `server` unit tests |

## CI check

A lightweight workspace test asserts this file exists and still forbids the
disallowed production claim phrases used in public status tables. See
`crates/dingo-cli/tests/cli.rs` (`capability_matrix_document_present`).
