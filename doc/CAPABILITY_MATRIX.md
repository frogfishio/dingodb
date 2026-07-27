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
| Network multi-node | `dingo serve-cluster` + multi-seed connect | Partition Raft propose when control plane attaches (DEF-036/037); directory-only if Raft attach fails | experimental (requires `--experimental-network-cluster`) | `stage_def_036_raft_rpc`, `stage_def_037_cluster_commit`; CLI bind gates |
| S3/GCS placement | `MediaLocator` + mirror env roots | Filesystem mirror of segments, not native cloud I/O SDK | experimental mirror | store tier / media tests |
| Erasure / lifecycle | scaffold APIs | Not production protection | scaffold | types/docs only until codecs land |

## Critical honesty rules

1. **Experimental network cluster is not production.** With Raft attached
   (default for `serve-cluster`), put/delete go through partition propose and
   acks report `committed` only after quorum + local apply (DEF-037). If Raft
   attach fails, the process falls back to **directory-only** routing and
   single-node apply — still not a release claim. Production gates remain
   DEF-041 follow-ons and §16 (durable rebalance DEF-038, anti-entropy repair
   DEF-039, query paging DEF-040, and in-process seeded verification DEF-041
   are shipped; multi-process Jepsen / long soak still open).
2. **In-process quorum ≠ production network cluster.** Prefer
   `Dingo::open_cluster` for deterministic multi-replica tests; use
   `serve-cluster` + DEF-037 suite for multi-process data-plane checks.
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
| `PROTOCOL_PROFILE` | `dingo-rpc-v1` | Framed RPC + handshake (DEF-031) |
| `RPC_WIRE_LABEL` | `1.0-draft` | Network RPC interoperability draft (DEF-031; not frozen) |
| `TLS_PROFILE` | `dingo-tls-v1` | TLS 1.3 transport + peer identity (DEF-032) |
| `AUTHZ_PROFILE` | `dingo-authz-v1` | Principal privileges + audit chain (DEF-033) |
| `ADMISSION_PROFILE` | `dingo-admission-v1` | Protocol admission: rate, auth lockout, churn, expensive budgets (DEF-034) |
| `RAFT_PERSIST_PROFILE` | `dingo-raft-persist-v1` | Durable Raft hard state, log, membership, snapshots (DEF-035) |
| `RAFT_RPC_PROFILE` | `dingo-raft-rpc-v1` | Network Raft control-plane RPCs (DEF-036) |
| `FEATURE_RAFT_RPC_V1` | `raft-rpc-v1` | Handshake feature when server serves `raft_*` ops |
| `CLUSTER_COMMIT_PROFILE` | `dingo-cluster-commit-v1` | Data-plane put/get via network Raft propose (DEF-037) |
| `FEATURE_CLUSTER_COMMIT_V1` | `cluster-commit-v1` | Feature token for quorum-committed collection ops |
| `REBALANCE_CONTROL_PROFILE` | `dingo-rebalance-control-v1` | Durable rebalance jobs + joint membership (DEF-038) |
| `ANTI_ENTROPY_PROFILE` | `dingo-anti-entropy-v1` | Hierarchical inventory + integrity-based repair (DEF-039) |
| `BACKUP_PROFILE` | `dingo-backup-v1` | Full single-node backup package + verified restore (DEF-050) |
| `SCRUB_PROFILE` | `dingo-scrub-v1` | Bounded integrity scrub + findings quarantine (DEF-051) |
| `MIGRATE_PROFILE` | `dingo-migrate-v1` | Phased format migration + version matrix (DEF-052) |
| `CONFIG_PROFILE` | `dingo-config-v1` | Versioned process config + validate-before-serve (DEF-054) |
| `LOG_PROFILE` | `dingo-log-v1` | Structured NDJSON process logs + correlation fields (DEF-060) |
| `METRICS_PROFILE` | `dingo-metrics-v1` | Bounded process metrics scrape (`metrics` RPC) (DEF-061) |
| `HEALTH_PROFILE` | `dingo-health-v1` | Liveness / readiness / detail health RPCs (DEF-061) |

## Raft persistence (DEF-035)

| Concern | How | Maturity |
|---------|-----|----------|
| Hard state | `current_term`, `voted_for`, `commit_index`, `last_applied` in checksummed `hard_state.json` | **in-process cluster** shipped |
| Log | Append-only length-prefixed + blake3 records in `log.ndjson`; torn tails truncated | **in-process cluster** shipped |
| Membership | Checksummed `membership.json` (voters + placement epoch) | **in-process cluster** shipped |
| Snapshots | `snapshot.meta.json` + blake3 blob; atomic install truncates log | **in-process cluster** shipped |
| Persist-before-ack | Votes and AppendEntries flush before grant/success | **in-process cluster** shipped |
| Restart | `Cluster::open` restores peers as Followers; re-elect on demand | **in-process cluster** shipped |
| Evidence classes | `committed` / `prepared` / `conflicting` / `unknown_commit` | **in-process cluster** shipped |
| Durable rebalance jobs / joint membership | DEF-038 `rebalance_jobs.json` + joint `membership.json` | **in-process cluster** shipped |
| Anti-entropy inventory + replica repair | DEF-039 majority/integrity source select; `repair_audit.json` | **in-process cluster** shipped |
| Network multi-process Raft control plane | DEF-036 RequestVote / AppendEntries / snapshot / ReadIndex | **shipped** (see below) |
| Data-plane client writes via network Raft | DEF-037 propose + apply; `committed` after quorum | **shipped (experimental)** |
| Seeded in-process fault sim + lincheck | DEF-041 `dingo-cluster-verify-v1` | **shipped** (network Raft in-process) |
| CLUSTER_SPEC §22 core matrix (network Raft) | DEF-041 | **shipped** (§22.1–.8 + chaos) |
| In-process soak put/get after chaos | DEF-041 | **shipped** (`run_soak`; multi-process still follow-on) |
| Multi-process Jepsen-style partition histories | DEF-041 follow-on | **not yet** |
| Long-duration soak / rolling restart | DEF-041 follow-on | **not yet** |

Layout: `{cluster_root}/raft/node-{n}/p{partition}/`. User payloads remain in
ordinary `dingo-store` segments (salvage independent of Raft control plane).

Evidence: `stage_def_035_raft_persist`, `dingo_cluster::raft_persist` unit tests.

## Network Raft RPC (DEF-036)

| Concern | How | Maturity |
|---------|-----|----------|
| RequestVote | Framed `raft_request_vote`; term + log-up-to-date + epoch fences | **shipped** |
| AppendEntries | Framed `raft_append_entries`; bounded batch; match/conflict hints | **shipped** |
| InstallSnapshot | Framed `raft_install_snapshot`; blake3 meta + blob via DEF-035 store | **shipped** |
| ReadIndex / leadership | Framed `raft_read_index` | **shipped** |
| Transport | `RaftTransport` trait; `MemoryRaftNetwork` (tests); `TcpRaftTransport` (SDK) | **shipped** |
| Auth | Shared token / authz on peer connections when configured | **shipped** |
| Authority | Endpoints = routing only; epoch/cluster/term/membership fence writes | **shipped** |
| Retry identity | `operation_id` dedup → same log index | **shipped** |
| Client put/get via Raft propose | DEF-037 `propose_and_apply` + read-index barrier | **shipped (experimental)** |

Evidence: `stage_def_036_raft_rpc` (cluster + SDK), `dingo_cluster::raft_rpc` unit tests.

## Network data-plane commit (DEF-037)

| Concern | How | Maturity |
|---------|-----|----------|
| Put / delete path | Leader `propose_and_apply` on subject partition | **experimental** (`--experimental-network-cluster`) |
| Commit ack | `committed=true` only after quorum log commit + local apply | **experimental** |
| Linearizable get | Read-index barrier before local store read when Raft attached | **experimental** |
| Follower apply | After AppendEntries / InstallSnapshot, apply committed entries | **experimental** |
| Single-node serve | No Raft → local store commit (unchanged) | **development** |
| Directory-only fallback | If Raft attach fails, routing/advertise without quorum writes | **not quorum** |

Evidence: `stage_def_037_cluster_commit` (5 tests: labels, shared semantics,
kill seed after commit, op identity, solo serve).

## Durable rebalance control plane (DEF-038)

| Concern | How | Maturity |
|---------|-----|----------|
| Job persistence | Checksummed `rebalance_jobs.json` (atomic + `.prev`) after each phase | **in-process cluster** shipped |
| Joint consensus | `PartitionRaft::set_joint_voters`; `MembershipState.{joint,outgoing,incoming}` | **in-process cluster** shipped |
| Coordinator restart | `Cluster::open` reloads jobs; restores old or joint voters; resume advances | **in-process cluster** shipped |
| Degraded open | `Cluster::health` — missing stores, offline marks, in-flight phases | **in-process cluster** shipped |
| Missing placement | Multi-node open refuses silent synthetic `placement.json` | **in-process cluster** shipped |
| Endpoint registration | Atomic upsert; optional `registration_token_hash` + authenticated path | **in-process / serve-cluster** shipped |

Evidence: `stage_def_038_control_plane` (6 tests), Stage 8f rebalance suite.

## Anti-entropy and replica repair (DEF-039)

| Concern | How | Maturity |
|---------|-----|----------|
| Hierarchical inventory | Per-partition subject digests, log frontier, segment fingerprints | **in-process cluster** shipped |
| Source selection | Majority of healthy content hashes; corrupt never votes; no mtime | **in-process cluster** shipped |
| Missing / divergent repair | Verified body copy + post-put hash check | **in-process cluster** shipped |
| Conflicts | Equal-vote splits preserved (no wall-clock winner) | **in-process cluster** shipped |
| Irrecoverable holes | Explicit when no healthy readable body remains | **in-process cluster** shipped |
| Audit | Checksummed `repair_audit.json` (atomic + `.prev`) | **in-process cluster** shipped |
| Rate limit | `RepairOptions::{max_subjects,max_bytes,dry_run}` | **in-process cluster** shipped |
| Network multi-process repair RPC | Background peer exchange over `serve-cluster` | **not yet** (inventory/repair is in-process) |

Evidence: `stage_def_039_repair` (8 tests), `dingo_cluster::repair` unit tests.

## Distributed query paging (DEF-040)

| Concern | How | Maturity |
|---------|-----|----------|
| Coverage on every page | `FindResult.coverage` always attached (`scan_page` / `scan_with`) | **in-process cluster** shipped |
| Deterministic merge | Subject-ascending order; independent of `visit_order` / worker completion | **in-process cluster** shipped |
| Per-partition frontiers | `Coverage.frontiers` + `read_mode` on each page | **in-process cluster** shipped |
| Coordinator resume | MAC'd `QueryContinuation` (`dingo-query-continuation-v1`) bound to `cluster_id` | **in-process cluster** shipped |
| Index / tier / resource fields | `indexes_used`, `tiers_searched`/`tiers_excluded`, `resource_limit_reached` | **in-process cluster** shipped |
| Partial partition honesty | Unavailable partitions never look like empty complete success | **in-process cluster** shipped |
| Network multi-process page RPC | Remote worker page protocol over `serve-cluster` | **not yet** (coordinator is in-process) |

Evidence: `stage_def_040_query` (7 tests), `dingo_cluster::coverage` continuation unit tests.

## Backup and restore (DEF-050)

| Concern | How | Maturity |
|---------|-----|----------|
| Full package format | `backup-manifest.v1.json` + `store/` tree; profile `dingo-backup-v1` | **shipped** (single-node) |
| Content integrity | BLAKE3 per file + manifest `content_hash_hex` | **shipped** |
| Crash-consistent exclusive backup | Flush durable active under writer lock (`flushed_exclusive`) | **shipped** |
| Concurrent inspect backup | On-disk file copy without flush (`on_disk_inspect`) | **shipped** |
| Identity-preserving restore | Default: same `store_id` | **shipped** |
| Clone restore | `--reassign-identity` / `RestoreOptions::reassign_identity` | **shipped** |
| Distinct from salvage | Salvage = damage recovery; backup = intentional package | **shipped** |
| CLI | `dingo backup` / `dingo restore` | **shipped** |
| Incremental / encrypted / remote targets | — | **not yet** |
| Cluster-coordinated multi-node backup | — | **not yet** |

Evidence: `stage_def_050_backup`, `dingo_store::backup` unit tests, CLI
`backup_and_restore_roundtrip`.

## Integrity scrub (DEF-051)

| Concern | How | Maturity |
|---------|-----|----------|
| Profile | `dingo-scrub-v1` under `recovery/scrub/` | **shipped** (single-node) |
| Bounded steps | `max_files` / `max_bytes` per `scrub_once` | **shipped** |
| Content integrity | Full-file BLAKE3 vs placement `content_hash` | **shipped** |
| Frame verification | Forward scan holes on sealed/active segments | **shipped** |
| Findings persistence | `findings.v1.json` open findings | **shipped** |
| Quarantine | Copy corrupt targets; never delete originals | **shipped** |
| Pause / resume | Durable `paused` flag on state | **shipped** |
| Operator metrics | coverage, bytes verified, failures, scrub age | **shipped** |
| CLI | `dingo scrub` / `--status` / `--pause` / `--resume` | **shipped** |
| Background interval daemon | — | **not yet** |
| Cluster repair integration | — | **not yet** (DEF-039 follow-on) |

Evidence: `stage_def_051_scrub`, `dingo_store::scrub` unit tests, CLI
`scrub_clean_store_and_status`.

## Format and protocol migration (DEF-052)

| Concern | How | Maturity |
|---------|-----|----------|
| Profile | `dingo-migrate-v1` job under `recovery/migration/` | **shipped** (single-generation) |
| Wire reader/writer matrix | `dingo-format::compat` (`SUPPORTED_READER_MAJORS`, current major 1) | **shipped** |
| Protocol policy snapshot | Declared `dingo-rpc-v1` / `1.0-draft` in job documents | **shipped** |
| Phases | preflight → plan → apply → verify; rollback of incomplete | **shipped** |
| Evidence-preserving copy | Never in-place rewrite; blake3 per file | **shipped** |
| Unsupported / unreadable segments | Preserve opaque bytes + plan notes | **shipped** |
| Failed migration | Source remains fully readable | **shipped** |
| CLI | `dingo migrate` / `--preflight` / `--plan-only` / `--status` / `--rollback` | **shipped** |
| Second wire major dual-read + rewrite | — | **not yet** (DEF-053) |
| Rolling mixed-cluster upgrade drills | — | **not yet** |

Evidence: `stage_def_052_migrate`, `dingo_format::compat` / `dingo_store::migrate`
unit tests, CLI `migrate_roundtrip_and_status`.

## Process configuration (DEF-054)

| Concern | How | Maturity |
|---------|-----|----------|
| Profile | `dingo-config-v1` JSON document | **shipped** |
| Validate before serve | `load_and_validate` / CLI `config validate` | **shipped** |
| Layering | defaults &lt; file &lt; env secrets &lt; CLI flags | **shipped** |
| Setting classes | static / restart-required / dynamic (`setting_class`) | **shipped** |
| Secrets | `token_env`, `token_secret_ref` (`env:` / `file:`); never inline | **shipped** |
| Redaction | effective report + `redact_json_value` | **shipped** |
| Unsafe combos | replication claim &lt; 3 nodes; public plaintext; serve-cluster gate | **shipped** |
| CLI | `dingo config validate\|show`, `serve --config` | **shipped** |
| Live dynamic reload + audit | — | **not yet** |

Evidence: `stage_def_054_config`, `dingo_server::config` unit tests, CLI
`config_validate_show_and_unsafe_reject`.

## Network bind policy (DEF-002 / DEF-032)

| Bind | Plaintext without override | With `--allow-insecure-bind` | With TLS (`--tls-cert`/`--tls-key`) |
|------|----------------------------|------------------------------|-------------------------------------|
| `127.0.0.1`, `::1`, `localhost` | allowed | allowed | allowed |
| `0.0.0.0`, `::`, LAN/public IPs | **refused** before accept | allowed (development-only plaintext) | **allowed** (production path) |

`serve-cluster` additionally requires `--experimental-network-cluster`.

## Transport security (DEF-032)

| Mode | How | Peer auth |
|------|-----|-----------|
| Plaintext | default on loopback | optional shared token (constant-time compare) |
| TLS 1.3 | `ServeOptions::tls` / `ConnectOptions::tls` | server cert; hostname/SNI verify |
| mTLS | server `--tls-client-ca` + client identity | mutual certs; cluster/node SAN URIs |

- Cluster/node identity: SAN URIs `urn:dingo:cluster:{id}` / `urn:dingo:node:{id}`.
- Operator revocation denylist: certificate serial hex on client/server options.
- Cert rotation: `TlsServerState::reload()` without downtime (new handshakes).
- Evidence: `stage_def_032_tls`.

## Authorization and audit (DEF-033)

| Concern | How |
|---------|-----|
| Authentication | Shared token → principal (constant-time); mTLS is transport identity (DEF-032) |
| Authorization | `PrivilegeSet` on principal; RPC op map in `requirement_for_op` |
| Roles | `reader`, `writer`, `dba`, `operator`, `superuser` |
| High-friction | `purge` needs `confirm=PURGE`; `force_reconfig` needs `confirm=FORCE_RECONFIG` |
| Audit | `AuditLog` hash chain; sensitive allow + all deny; no tokens/payloads |
| Legacy | `ServeOptions::auth_token` alone ⇒ single superuser principal |

- Profile: `AUTHZ_PROFILE = dingo-authz-v1`.
- Evidence: `stage_def_033_authz`.

## Protocol admission control (DEF-034)

| Concern | How |
|---------|-----|
| Global RPC rate | `AdmissionLimits::global_max_rps` (1s fixed window) → `resource_limit` |
| Per-principal rate | `per_principal_max_rps` after authz → `resource_limit` |
| Auth failure budget | Hashed token key; lockout → generic `authentication_failed` |
| Connection churn | Accept-time window before DEF-030 slot admission |
| Expensive ops | Concurrent budget for scan/find/index/salvage/admin scaffolds |
| Operation-id replay | Bounded TTL window; retries of same id admitted; full → `resource_limit` |
| Config | `ServeOptions::admission_limits` / `ServeOptions::admission` |

- Profile: `ADMISSION_PROFILE = dingo-admission-v1`.
- Complements DEF-029 host ceilings and DEF-030 connection slots (does not replace them).
- Evidence: `stage_def_034_admission` + `admission` unit tests.

## Network RPC framing (DEF-031)

| Mode | How | Handshake | Message shape |
|------|-----|-----------|---------------|
| Production (`dingo-rpc-v1`) | default `connect` / `serve` | client `hello` → server `welcome` | `u32` BE length + JSON payload |
| Diagnostic line | both sides set `diagnostic_line_protocol` | none | newline-delimited JSON |

- Negotiated features: `json-rpc-v1`, `receipts-v1`, `idempotency-v1`.
- `max_frame` is checked **before** allocating the payload buffer.
- Legacy bare-line clients against a production server fail with
  `protocol_violation` (clear error; no silent dual-mode).
- `RPC_WIRE_LABEL = 1.0-draft` is not an interoperability freeze (DEF-053).
- Golden fixtures: `crates/dingo-sdk/tests/fixtures/protocol/`.

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
| Rate-limited checkpoints | shipped | Index cache **and** collection catalog share one high rate limit (`DERIVED_CHECKPOINT_EVERY_OPS`); **not** forced on seal (seal is O(segment) incremental catalog + placement only); explicit `persist_index_cache` still checkpoints. Avoids O(N) full-index rewrite on the write scale path |
| Steady-state vs lifecycle attribution | shipped | Measured: ordinary put is data+dual-index µs-class; remaining spikes are synchronous `persist_index_cache` / `seal_active` (see `doc/BENCHMARK_DISCLOSURE.md`, examples `write_latency_breakdown` / `write_scale_curve`) |
| Index-path maximum point | documented | Steady-state index insertion past cliff (diminishing returns); next write-path leverage is async lifecycle, not new primary-index structure (`BENCHMARK_DISCLOSURE` maximum-point self-check) |
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
| Protocol rate / auth lockout / expensive budgets | shipped | DEF-034 admission (`ADMISSION_PROFILE`) |
| Per-tenant work quotas | partial | DEF-034 per-principal RPS; multi-tenant ACL store still follow-on |
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
