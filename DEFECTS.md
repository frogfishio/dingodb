# ResiduumDB production-readiness defects and execution plan

Status: active remediation plan  
Scope: the complete ResiduumDB workspace
Primary inputs: repository review performed 2026-07-25, `OVERVIEW.md`,
`FORMAT_SPEC.md`, `DX_SPEC.md`, `CLUSTER_SPEC.md`, and current implementation

## 1. Purpose

This document turns the current product-review findings into an ordered,
testable engineering program.

ResiduumDB already has a credible damage-tolerant format, salvage scanner,
single-node store, Rust SDK, and unusually strong conformance tests. It is not
yet production-ready as a network database or distributed storage system.
Production readiness requires more than implementing missing APIs: acknowledged
writes, recovery evidence, distributed commitment, operational security, and
query completeness must remain truthful through crashes, retries, upgrades,
partial outages, and operator mistakes.

The work below is complete only when the release gates in §16 pass. A task is
not complete merely because an API or type exists.

For a short read on whether remaining labor is high-leverage vs polish thrash,
see [doc/WORK_HORIZON.md](doc/WORK_HORIZON.md). For product wedge, labor split,
and forbidden claims, see [doc/PRIME_TIME_PLAN.md](doc/PRIME_TIME_PLAN.md).
Stages 0–9 of the delivery plan are complete; most §16 gates and several P0/P1
defects are not.

## 2. Current deployment classification

Until this plan is complete, use these support labels:

- **Embedded single-node:** experimental/early-access.
- **Single-node TCP server:** development only.
- **In-process cluster:** deterministic integration-test harness.
- **`serve-cluster`:** experimental multi-process Raft (control plane DEF-036,
  data-plane commit DEF-037, durable rebalance DEF-038, in-process anti-entropy
  repair DEF-039 when attached, distributed query paging DEF-040, seeded
  in-process verification DEF-041); not production-ready (DEF-041 multi-process
  follow-ons + DEF-050/051/052 follow-ons + §16).
- **S3/GCS:** filesystem-mirror integration, not a native cloud backend.
- **Erasure coding and lifecycle automation:** scaffolds only.
- **Wire format:** `1.0-draft` with declared reader/writer matrix and phased
  migration (DEF-052); freeze checklist + policy published, not frozen
  (DEF-053 partial — `doc/WIRE_MAJOR1_FREEZE.md`).
- **Process config:** versioned `dingo-config-v1` validate-before-serve
  (DEF-054); live dynamic reload still follow-on.
- **Process logs:** versioned `dingo-log-v1` NDJSON on serve paths (DEF-060);
  client-side structured emission and config-driven sinks still follow-on.

Public documentation, CLI help, release notes, and crate READMEs MUST use these
labels until the corresponding gates pass.

## 3. Severity and completion rules

Priorities:

- **P0:** correctness or data-safety contract violation. Blocks every
  production release.
- **P1:** production reliability, security, scale, or operability blocker.
- **P2:** product completeness, packaging, maintainability, or confidence gap.

Every task must include:

1. a normative requirement or an explicit amendment to one;
2. implementation;
3. deterministic tests for success and failure;
4. fault-injection tests where persistence or networking is involved;
5. operator-visible diagnostics;
6. documentation and upgrade notes;
7. benchmark evidence when the task touches a hot path.

No task may weaken damage honesty, silently downgrade durability, or make
derived state authoritative.

## 4. Execution order

The critical dependency chain is:

```text
truthful single-node semantics
  → exclusive store ownership
  → crash-safe metadata
  → bounded concurrent server
  → secure and observable transport
  → persistent consensus
  → network replication
  → repair/rebalance automation
  → production packaging and release
```

Recommended phases:

1. **Containment:** correct claims and disable unsafe defaults.
2. **P0 correctness:** retries, salvage evidence, and incomplete-result
   handling.
3. **Single-node foundation:** locking, metadata atomicity, crash consistency,
   and bounded-memory APIs.
4. **Server foundation:** concurrency, protocol evolution, security, and
   observability.
5. **Distributed foundation:** durable Raft and real network replication.
6. **Fleet reliability:** repair, rebalancing, backup, upgrades, and deployment.
7. **Archive product:** native object storage, lifecycle, scrubbing, and
   erasure coding.
8. **Product completion:** SDK/CLI parity, packaging, benchmarks, and stable
   release qualification.

---

## 5. Immediate containment

### DEF-001 — Correct product maturity and capability claims

Priority: P0  
Dependencies: none  
Status: **remediated in-tree** (2026-07-25) — README / ARCHITECTURE /
CONTRIBUTING / `doc/CAPABILITY_MATRIX.md` + CLI matrix CI check

Work:

- Replace unqualified “shipped,” “done,” “cluster v1,” “live S3/GCS,” and
  “extreme speed” wording with capability-specific labels.
- State prominently that network `serve-cluster` writes only one node and that
  quorum replication exists only in the in-process harness.
- Distinguish native cloud I/O from filesystem mirrors.
- Remove or qualify Redis-class language until reproducible results are
  published.
- Explain the difference between crate semver `0.1.0`, `SDK_API_VERSION`,
  `CLUSTER_PROFILE_VERSION`, and `WIRE_PROFILE_LABEL`.
- Ensure README status tables link directly to known limitations.

Acceptance:

- A reader cannot reasonably infer that three `serve-cluster` processes provide
  replicated durability.
- Every advertised capability has a runnable acceptance test or is labeled
  experimental/scaffold/future.
- Release notes contain a generated capability matrix checked in CI.

### DEF-002 — Gate unsafe network binds

Priority: P0  
Dependencies: none  
Status: **remediated in-tree** (2026-07-25) — `dingo-sdk` bind policy +
`ServeOptions` + CLI flags + structured startup report + CLI tests

Work:

- Default `dingo serve` and `serve-cluster` to loopback.
- Refuse non-loopback plaintext binds unless the operator supplies an explicit
  development-only override.
- Print a structured startup warning showing transport security, authentication,
  durability, replication, and store-lock status.
- Make network cluster mode require an explicit experimental flag until
  DEF-030 through DEF-038 are complete (rebalance durability is in-process
  experimental; network production still gated by DEF-041 / §16).

Acceptance:

- An accidental `0.0.0.0` bind without TLS fails before accepting traffic.
- Startup output cannot claim replicated durability for routing-only mode.
- Integration tests cover loopback, public bind refusal, and override behavior.

### DEF-003 — Commit and release complete feature slices

Priority: P1  
Dependencies: none  
Status: **remediated in-tree** (2026-07-25) — clean-tree CI gate;
`scripts/release_content.sh` (`cargo package --list` + rebuild from package
file lists); `doc/RELEASE_ARTIFACTS.md`; session dirs ignored; path deps
versioned for packaging

Work:

- Resolve all currently untracked source, documentation, and demo files.
- Do not publish manifests that reference files absent from the release.
- Add a release-content test using `cargo package --list` and package builds.
- Define which demos and specs are release artifacts.

Acceptance:

- `git status --short` is empty in release CI.
- Every workspace crate packages and builds from its packaged tarball.
- Published documentation never references an omitted file.

Evidence:

- CI: `.github/workflows/ci.yml` runs clean-tree check +
  `./scripts/release_content.sh`.
- Policy: [doc/RELEASE_ARTIFACTS.md](doc/RELEASE_ARTIFACTS.md) classifies crate
  packages vs monorepo specs/demos vs non-artifacts (`.gremlin/`, proposals).
- Packaging: workspace path deps carry `version = "0.1.0"` so `cargo package`
  can rewrite them; the gate rebuilds from package file lists in a temp
  workspace (path deps stay local until crates.io publish).

---

## 6. P0 data correctness and recovery contracts

### DEF-010 — Make remote writes idempotent across ambiguous failures

Priority: P0  
Normative basis: `DX_SPEC.md` retry safety; `CLUSTER_SPEC.md` §13  
Status: **remediated in-tree** (2026-07-25) — client `operation_id` + content
identity + persisted `store-info/write_dedup.v1`; remaining follow-on: retention
/ compaction policy for dedup evidence, property tests for arbitrary response
loss points, network-cluster leader failover coverage

Problem:

`RpcRequest` carries no stable write/event identifier. `call_keyed` may retry a
write after a transport failure, while the server mints a new event ID. If the
first write committed and its response was lost, retry can append another
event.

Work:

- Add a required client-generated `operation_id`/`event_id` to every mutating
  RPC.
- Include a content identity covering operation, collection, key, mutation
  kind, and payload.
- Persist a deduplication record as part of the authoritative write boundary.
- Return the original receipt for an exact retry.
- Reject reuse of an ID with different content as `ConsistencyViolation`.
- Define retention and compaction rules for deduplication evidence.
- Preserve IDs across route refresh, reconnect, deadline, and gateway retry.
- Never automatically retry a mutation unless the protocol proves retry safety.

Acceptance:

- Kill the connection after durable append but before response; retry returns
  the original receipt and history contains one event.
- Reuse of an ID with altered content fails deterministically.
- Deduplication survives process restart, compaction, tier movement, and leader
  failover.
- Property tests cover arbitrary response-loss points.

In-tree so far:

- SDK mutations mint a client `operation_id` once per call so `call_keyed`
  transport retries reuse the same id.
- Content identity over `(op, collection, key, payload)` via
  `dingo_store::content_identity`.
- Server looks up `WriteDedupTable` before append; exact retry returns the
  original receipt; content-mismatched reuse → `ConsistencyViolation`.
- Dedup table persisted under `store-info/write_dedup.v1` after each accepted
  mutation; reloaded on `Store::open`.
- Tests: `stage_def_010_012_013`, `stage7_remote_parity::remote_put_is_idempotent_with_operation_id`.

### DEF-011 — Split evidence-preserving salvage from live-state export

Priority: P0  
Normative basis: `DX_SPEC.md` §13.4; `FORMAT_SPEC.md`; `SDA_PROFILE.md`  
Status: **remediated in-tree** (2026-07-25) — evidence `salvage_to` + hashed
recovery manifest + `export_live_state` / CLI `export-live`; remaining:
signed manifests, unsupported-format byte islands as opaque extents, chunk
partial-map examination parity beyond frame copy

Problem:

`Store::salvage_to` currently reads complete live values and re-appends them
with new lineage. It does not preserve raw verified frames, history, tombstones,
partial payloads, holes, conflicts, unsupported frames, or provenance.

Work:

- Rename the current behavior to an explicit operation such as
  `export_live_state`.
- Implement evidence-preserving salvage as the default `dingo salvage`.
- Copy verified frames without re-encoding whenever possible.
- Produce a signed or hashed recovery manifest containing:
  - source identity and scan parameters;
  - source byte ranges;
  - verified frames and hashes;
  - holes and corrupt candidates;
  - partial payload extents;
  - conflicts and unknown-commit evidence;
  - unsupported format bytes;
  - destination mapping;
  - tool/build/wire versions.
- Preserve event, item, segment, partition, term, and placement identities.
- Keep salvage non-destructive and destination-only.
- Expose an explicit materialization command for users who want a clean
  current-state database.

Acceptance:

- A damaged source with history, tombstones, partial chunks, conflicts, and
  holes produces a destination with equivalent examination evidence.
- Byte-identical verified frames remain byte-identical.
- Re-salvaging the result is deterministic.
- Current-state export is clearly distinguished in CLI help and JSON output.

In-tree so far:

- `Store::salvage_to` copies verified frame **bytes** into destination sealed
  segments (no re-encode); holes and scan parameters land in
  `recovery/salvage-manifest.v1.json` with a BLAKE3 content hash.
- Event/item identities inside frames are preserved (history and tombstones
  survive).
- `Store::export_live_state` keeps the old re-put materialization path with new
  lineage.
- CLI: `dingo salvage` = evidence mode; `dingo export-live` = live-state export;
  JSON reports `mode`, `frames_copied`, `holes_recorded`, `manifest_path`.
- Tests: `stage_def_011_salvage`, updated `salvage` suite.

### DEF-012 — Eliminate silent omission in reads and scans

Priority: P0  
Normative basis: `DX_SPEC.md` §§3.6, 3.7, 6.5; `CLUSTER_SPEC.md` §6.7  
Status: **largely remediated in-tree** (2026-07-25) — fail-closed logical
scans, offline-tier scan honesty, secondary-index miss authority; remaining:
unified query result envelope (holes/truncation fields), full cluster coverage
merge parity beyond Stage 8e

Problem:

`live_logical_entries` skips partial payloads, and remote JSON scans skip decode
failures. Offline tiers can also remove candidates from an ordinary index.
These paths can convert “unknown/incomplete” into a successful short result.

Work:

- Define one result envelope for scans and queries containing rows, coverage,
  holes, decode failures, partial payloads, unavailable tiers, and truncation.
- Make the simple API succeed only when completeness is established.
- Add an explicit partial-results API requiring caller opt-in.
- Distinguish:
  - proven absence;
  - no match in searched coverage;
  - unavailable data;
  - damaged data;
  - unsupported decoding;
  - resource-limited execution.
- Do not use `continue` to hide data-quality failures.
- Carry coverage through filtering, sorting, pagination, indexes, remote RPC,
  and cluster merge.
- Make secondary-index misses authoritative only with a complete frontier.

Acceptance:

- Every injected partial chunk, JSON decode failure, offline tier, damaged
  frame, unavailable partition, and budget exhaustion is visible.
- Ordinary `get`, `scan`, and `find` never return a complete empty result when
  required coverage is missing.
- Embedded, remote, and cluster conformance suites return equivalent semantics.

In-tree so far:

- `Store::live_logical_entries` returns `CoverageIncomplete` when any live
  subject is partial/unavailable/conflicting (no silent skip).
- `Store::scan_live_logical` returns `{entries, incomplete, complete,
  tier_coverage_incomplete}`; offline tiers force `complete = false`.
- Ordinary `find` refuses incomplete tier coverage unless
  `allow_partial_coverage`.
- Secondary-index empty lookups are authoritative only when
  `complete_coverage` is true (partial indexes fall through to scan).
- Remote `scan_json` fails closed on JSON decode failure instead of `continue`.
- Tests: `stage_def_020_021_lock_coverage`, `stage_def_010_012_013` offline
  scan cases.

### DEF-013 — Fix persisted collection-catalog contamination

Priority: P0  
Normative basis: authority-before-acceleration invariant  
Status: **remediated in-tree** (2026-07-25) — durable-frontier catalogs +
memory-mode visibility-only publishes; remaining follow-on: model-based mode
transition fuzzer

Problem:

`refresh_collection_catalog` persists a catalog derived from `self.index`.
That index may contain memory-mode writes that were never persisted. A later
durable operation can therefore write derived metadata describing nonexistent
authoritative data.

Work:

- Build every persisted catalog from a segment-derived index or a proven
  durable frontier.
- Track visibility and durability frontiers separately in memory.
- Prevent memory-only events from entering persisted indexes, catalogs,
  checkpoints, or tier summaries.
- Validate catalog fingerprints against both segment identity and durable
  frontier.
- Treat mismatched derived state as stale and rebuild automatically.

Acceptance:

- Memory write A followed by durable write B, crash, and reopen exposes only B
  unless A was later flushed.
- Catalogs, primary cache, secondary indexes, and checkpoints agree after every
  durability-mode sequence.
- A model-based test explores mode transitions and crashes.

In-tree so far:

- Memory-mode put/delete update the in-process visibility index only; they
  never append frames (so a later durable write cannot flush them via
  `write_segment_tail`).
- Persisted collection catalog is always rebuilt from a segment-derived
  durable index; `list_collections` may still reflect in-process memory
  visibility.
- Primary index cache already segment-derived (unchanged).
- Tests: `stage_def_010_012_013` memory/catalog/reopen cases.

### DEF-014 — Propagate achieved guarantees without optimistic defaults

Priority: P0  
Status: **remediated in-tree** (2026-07-25) — fail-closed receipt parsing +
`ProtocolViolation`; remaining follow-on: richer requested-vs-achieved fields
for cluster quorum receipts when network Raft lands

Work:

- Remove protocol parsing fallbacks that turn absent IDs into zero IDs,
  absent `committed` into `true`, or malformed durability into the requested
  mode.
- Make required receipt fields mandatory by protocol version.
- Validate store/cluster identity after reconnect.
- Return `ProtocolViolation` for missing or inconsistent guarantee fields.
- Include requested and achieved durability, consistency, replica count,
  commit status, partition, term, position, and placement epoch where relevant.

Acceptance:

- Malformed or old responses fail closed.
- A client never reports stronger durability or commitment than the server
  proved.
- Compatibility tests cover every supported protocol version.

In-tree:

- `write_receipt_from_resp` / `delete_receipt_from_resp` require `committed`,
  `acknowledgement`, `event_id`, `version`, `store_id`, `segment_id` (no
  `unwrap_or(true)` / durability fallback / zero IDs).
- Connect and reconnect require a non-zero `store_id` from `store_info`.
- `Error` / `ErrorCode::ProtocolViolation` for missing guarantee fields.
- Unit tests cover absent `committed`, absent durability, and missing event id.

---

## 7. Single-node persistence and concurrency

### DEF-020 — Enforce exclusive store ownership

Priority: P0  
Status: **remediated in-tree** (2026-07-25) — OS `flock` + in-process path
registry + `open_inspect` without lock; remaining: generation fencing token,
explicit NFS reject, Windows LockFileEx CI

Problem:

Multiple processes can open the same store for writing. There is no advisory
lock, lease, or fencing token.

Work:

- Acquire an OS-backed exclusive writer lock before opening an active segment.
- Keep read-only inspect/salvage modes separate from writer mode.
- Store lock metadata for diagnostics only; never trust it instead of the OS
  primitive.
- Define stale-lock and process-crash behavior for supported platforms.
- Add a store generation/fencing value to guard against split writer ownership.
- Ensure CLI mutations and servers use the same lock path.
- Document network-filesystem support and reject filesystems whose locking or
  durability semantics are unsupported.

Acceptance:

- A second writer fails before touching authoritative bytes.
- Read-only doctor can run concurrently without mutation.
- Kill -9 releases ownership through the OS and recovery preserves the valid
  prefix.
- Linux, macOS, and supported Windows behavior is tested.

In-tree:

- `WriterLock` on `store-info/writer.lock` via Unix `flock(LOCK_EX|LOCK_NB)`.
- In-process path registry so two handles in one process also collide.
- `Store::create` / `Store::open` acquire before active segment open;
  `Store::open_inspect` / `Dingo::open_inspect` do not.
- Diagnostic lock file text is not trusted for exclusion.
- Startup report claims exclusive-writer lock status.
- Tests: second writer, concurrent inspect, drop-release, cross-process flock.

### DEF-021 — Make all metadata writes atomic and durable

Priority: P1  
Status: **remediated in-tree** (2026-07-25) — shared atomic helper + parent
dirsync; previous generations for non-trivial control docs; endpoints lock +
checksum/generation; failpoint boundaries under `atomic.*`

Work:

- Introduce one atomic-file helper: create temp in same directory, write,
  `sync_all`, rename, and sync parent directory.
- Use it for `endpoints.json`, placement, cluster metadata, lifecycle policy,
  roots, catalogs, checkpoints, and every mutable control document.
- Add generation, checksum, and format version to each file.
- Keep previous known-good generations where reconstruction is not trivial.
- On parse failure, report corruption and use a documented recovery path rather
  than silently replacing state.

Acceptance:

- Fault injection at every write/fsync/rename boundary leaves either the old or
  new valid generation.
- Concurrent endpoint registration cannot lose unrelated endpoints.
- Recovery diagnostics identify the damaged generation and next action.

In-tree:

- `dingo_store::atomic_file` — `write_atomic` / `write_atomic_keep_previous`,
  `previous_path` / `read_with_previous`, failpoints
  `atomic.before_tmp_write` … `atomic.after_dir_sync`.
- Wired for store meta/descriptor, catalogs, index cache, secondary indexes,
  checkpoints, tier placement/roots, migration evidence, lifecycle policy,
  write dedup, recovery manifest; cluster `cluster.json`, `placement.json`,
  `endpoints.json`.
- Non-trivial docs keep `*.prev`; endpoints add `generation` + `content_blake3`
  and OS/process lock on upsert.
- `StoreError::CorruptControl` carries path, detail, and recovery action.
- Tests: `stage_def_021_atomic_meta`, `atomic_file` unit tests, endpoints unit
  tests.

### DEF-022 — Define and test crash-consistency boundaries

Priority: P0  
Status: **hardened in-tree** (2026-07-25) — matrix + failpoints + multi-process
abort + ENOSPC/permission/short-write injection + CI subset; remaining:
power-loss equivalence for buffered mode, and production-strength gates on
every seal/compact/tier cell under adversarial FS

Work:

- Document exact persistence ordering for store creation, append, chunked put,
  delete, seal, compaction, checkpoint, tier move, and catalog refresh.
- Build a failpoint framework around each persistent step.
- Run each operation with process termination after every failpoint.
- On reopen, assert:
  - no fabricated committed event;
  - no lost durable acknowledgement;
  - no duplicated event identity;
  - no derived state ahead of authority;
  - all surviving evidence remains salvageable.
- Test filesystem-full, short write, permission loss, I/O error, and rename
  failure.

Acceptance:

- A machine-readable crash matrix maps every operation and failpoint to an
  expected state.
- CI runs a bounded subset per PR and the full matrix nightly.

In-tree:

- `crates/dingo-store/crash_matrix.v1.json` — operations, ordered persistence
  steps, failpoint cells (`fault`: enospc / permission / short_write /
  process_abort), expected reopen state, CI subset flags.
- `dingo_store::failpoint` — `Panic`, `Abort`, `Error`/`Return`, `IoEnospc`,
  `IoPermission`, `ShortWrite`; `consume_short_write` for instrumented sites.
- Instrumented boundaries: create meta, active write_tail (before/after write/
  after sync/short_write), active dir_sync, seal dest write/remove, index cache,
  write dedup, catalog, checkpoint, compact segment sync, tier placement write,
  atomic tmp short_write.
- Multi-process harness: `dingo-store-crash-child` binary + parent reopen
  asserts (kill before write / after sync).
- Tests: `stage_def_022_crash_matrix` (document validation + CI subset + I/O
  suite + multi-process abort always; full matrix when
  `DINGO_CRASH_MATRIX_FULL=1`).
- Nightly workflow / `scripts/nightly.sh` run the full matrix.
- Doc: `doc/CRASH_CONSISTENCY.md`.

### DEF-023 — Remove full-store rescans from the write acknowledgement path

Priority: P1  
Status: **addressed** (frontier cache v2 + durable projection; see
`stage_def_023_write_path`, `doc/CAPABILITY_MATRIX.md`)

Problem:

Every buffered/durable write called `persist_index_cache`, which reconstructed
the index by scanning all segment files.

Work:

- Define an incremental, checksummed index journal or checkpoint+delta format.
- Publish visibility only after authoritative append succeeds.
- Update derived state incrementally outside the critical fsync path.
- Batch and rate-limit index/catalog checkpoints.
- Preserve rebuild-from-segments as the recovery path.
- Record a durable frontier so cache validation is O(number of changed
  segments), not O(total data).
- Bound active-segment memory and metadata work.

Implementation notes:

- In-memory `durable_index` updated only after buffered/durable append.
- `indexes/primary.idx` v2 records sealed-segment fingerprint + active covered
  length; open applies the active tail beyond that frontier (checkpoint+delta).
- Full index-cache rewrite **and** collection-catalog atomic writes are
  rate-limited together at a high ops watermark (`DERIVED_CHECKPOINT_EVERY_OPS`);
  **seal no longer forces a full index rewrite** (that O(N) body serialization
  caused an ~87% write-throughput drop over 1 GB). Explicit
  `persist_index_cache` still checkpoints. Per-put catalog fsync was removed so
  buffered mode is not fsync-bound.
- Seal updates the hierarchical segment catalog **incrementally** (scan only
  the newly sealed segment bytes already in memory) and registers placement
  with a precomputed content hash — no full-media rescan or second full-file
  hash on the hot path. Full catalog rebuild remains the recovery path.
- Collection names for durable subjects are maintained incrementally (no
  O(N) `from_index` walk on every checkpoint).
- Wipe of `indexes/` / `catalogs/` / `snapshots/` still rebuilds identical
  logical state from segments.

Measured confirmation (developer hardware; see `doc/BENCHMARK_DISCLOSURE.md`
and examples `write_latency_breakdown` / `write_scale_curve`):

- Steady-state buffered puts: ~9 µs (4 B) / ~20 µs (8 KiB); dual-index publish
  roughly **35–46%** on mid-size values, data encode/append/write dominating
  larger payloads.
- Scale curve late/early throughput typically **≳ 0.7** (was ~0.13 with the
  full-index-on-seal collapse).
- Remaining cost is **synchronous lifecycle** when it runs: `persist_index_cache`
  (O(live×body), tens of ms) and `seal_active` (O(segment), tens of ms) still
  sit on the put path at their rate limits / thresholds — not ordinary index
  insertion.

Follow-on (not blocking DEF-023 acceptance): move rate-limited checkpoints and
optional seal work off the put acknowledgement path (background / batch
lifecycle) so p99 is not coupled to lifecycle spikes. Preferred shape: dual (or
more) active segment slots with O(1) foreground rotate; finalize seal, BLAKE3
completion, and index checkpoints off the writer thread; bound pending seals
with backpressure only when workers lag. **Landed as DEF-096 Axis A**
(`seal_pipeline`, `active/pending/` rotate) **and Axis B**
(`create_with_shards`, `put_many`); sequencing in `doc/PARALLEL_INGEST.md`.

**Maximum-point note (2026-07-27 self-check):** ordinary in-memory index
insertion on the steady-state put path is past the asymptotic cliff — further
index micro-optimization is diminishing returns relative to lifecycle spikes.
Async lifecycle and sharded writers are landed (DEF-096 Axis A/B). Remaining
write-path leverage is harness measurement and Axis C capacity — not a new
primary index structure. See `doc/BENCHMARK_DISCLOSURE.md` (“Maximum point
self-check”) and `doc/PARALLEL_INGEST.md`.

Acceptance:

- Amortized write work is independent of total retained data.
- Deleting all derived state still reconstructs identical logical results.
- Benchmarks disclose write amplification, fsync count, p50/p95/p99, payload
  size, durability, and verification.

### DEF-024 — Make compaction reclaim space safely

Priority: P1  
Status: **addressed** (phased job + optional reclaim; see
`stage_def_024_compaction`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Separate compaction creation, verification, activation, retention window, and
  source reclamation into durable phases.
- Persist a compaction job record and recovery generation.
- Never delete the only surviving conflict, unknown frame, or recovery
  evidence required by retention policy.
- Carry tombstone and deduplication horizons explicitly.
- Support cancellation and restart from every phase.
- Report estimated and actual bytes read, written, retained, and reclaimed.

Implementation notes:

- Durable job records under `recovery/compaction/<job_id>.job.json` with phases
  `planned → created → verified → activated → retention_hold → reclaimed`
  (plus `cancelled` / `failed`).
- Default `compact_live` stops at **activated** with sources retained (history
  preserved). Reclaim requires `allow_history_loss` for live-projection coverage.
- Open runs `recover_compact_jobs`: incomplete plan → cancel; created/verified →
  finish activate; activated/reclaimed left for the operator.
- Byte estimates and actuals on `CompactReport` / job; failpoints at plan,
  create, verify, activate, and reclaim boundaries.

Acceptance:

- Crash at every compaction phase preserves an authoritative old or verified
  new generation.
- Reclaimed bytes are measurable.
- Salvage and history semantics remain correct after compaction.

### DEF-025 — Strengthen identifier generation

Priority: P1  
Status: **addressed** (`dingo-id-v1` + `getrandom`; see
`stage_def_025_identifiers`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Use a standard OS CSPRNG abstraction on all supported platforms.
- Fail closed if secure randomness is required and unavailable.
- Define stable sortable IDs separately from random event identity.
- Persist monotonic counters only where the protocol depends on monotonicity.
- Add collision tests and format/version tags.

Implementation notes:

- Profile tag `ID_PROFILE = "dingo-id-v1"` in `dingo_store::ids`.
- Random identities (`event_id`, `store_id`, job/checkpoint/operation ids,
  cluster id) use `getrandom` via `random_id()`; `StoreError::RandomUnavailable`
  on failure (no wall-clock/hash fallback).
- Sortable `segment_id`: LE `u64` seq + store mix (`mint_sortable_segment_id`);
  seq recovered from on-disk segment names on open.
- Content-derived `item_id` remains `blake3(subject)[..16]`.
- SDK remote `operation_id` and `ClusterId::generate` share the same path.

Acceptance:

- No time-hash fallback is used for security- or correctness-sensitive IDs.
- IDs remain unique across restart, clone, restore, and concurrent writers.

---

## 8. Query, indexing, and memory behavior

### DEF-026 — Implement true bounded-memory cursors

Priority: P1  
Status: **addressed** (embedded paged cursors; see `stage_def_026_cursors`,
`doc/CAPABILITY_MATRIX.md`)

Problem:

`scan_json_iter` materializes the complete logical live set before returning an
iterator.

Work:

- Implement segment/index cursors that read bounded pages.
- Define deterministic order and snapshot/frontier semantics.
- Add authenticated continuation tokens containing query identity, scope,
  order, coverage, and frontier.
- Bound token size and reject replay against incompatible query generations.
- Stream remote pages without building a complete `Vec`.
- Support cancellation and deadlines.
- Preserve partial-result evidence across page boundaries.

Implementation notes:

- Profile tag `CURSOR_PROFILE = "dingo-cursor-v1"` in `dingo_store::cursor`.
- `Store::scan_live_page` walks the primary index in subject order, loading at
  most one page of complete bodies (default 64, cap 4096). Incomplete subjects
  are reported per page and still advance the cursor.
- Continuation tokens encode store_id, scan generation, prefix, after-subject,
  and page_size. The current keyed tag is derived from public `store_id`; it
  detects accidental mutation and cross-store use but is forgeable by a
  malicious client (DEF-097).
- Scan generation = BLAKE3(store_id ‖ segment_fingerprint ‖ live_count).
  Concurrent mutations that change the fence → `StoreError::CursorStale`.
  Declared model: read-committed paging with generation fencing (not MVCC).
- SDK: `Collection::scan_json_page`, `scan_json_iter` / `scan_json_iter_paged`
  page on demand; embedded `find` scan path uses the same pager.
- Follow-ons (not blocking this defect): remote/cluster page RPCs, cancellation
  deadlines, authenticated remote tokens across restarts.

Acceptance:

- Scan a dataset larger than process memory under a fixed memory budget.
- Resume after client/server restart without duplicates or omitted rows under
  the declared consistency model.
- Tampered, expired, and cross-query tokens fail.

### DEF-027 — Make index lifecycle online and truthful

Priority: P1  
Status: **addressed** (durable lifecycle + resume; see
`stage_def_027_index_lifecycle`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Persist index build state, source frontier, coverage, and failure reason.
- Build indexes concurrently with writes using snapshot+catch-up.
- Implement resumable `building`, `ready`, `stale`, `partial`, `failed`, and
  `rebuilding` states.
- Never use a stale/partial index to prove absence.
- Propagate stale-marking failures to diagnostics and health metrics.
- Add unique indexes only after defining enforceable partition scope.

Implementation notes:

- Profile tag `INDEX_LIFECYCLE_PROFILE = "dingo-index-lifecycle-v1"`.
- `.six` format v2 records `build_id`, `source_frontier`, `resume_after_subject`,
  and `failure_reason` (v1 files still load).
- Create persists `building`/`rebuilding` before the live walk; checkpoints every
  32 subjects; failpoints `index.build.after_plan` / `.mid` / `.before_ready`.
- Unfenced `Store::scan_live_bodies_for_build` + one catch-up pass when the
  segment fingerprint drifts; otherwise Ready, or Partial if still drifting /
  incomplete payloads.
- Absence only via Ready + `complete_coverage` (`may_prove_absence`); Partial
  may accelerate non-empty hits only.
- Put/delete surface stale-marking I/O errors (no longer `let _ =`).
- Follow-on: unique indexes with enforceable partition scope (not in this cut).

Acceptance:

- Kill an index build at every phase and resume without blocking writes.
- Indexed and forced-scan results are equivalent under randomized workloads.
- Dropping all indexes never changes correctness.

### DEF-028 — Align native filters and SDA semantics

Priority: P1  
Status: **addressed** (filter→SDA + dual corpus; see
`stage_def_028_filter_sda`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Either implement filter-to-SDA compilation as specified or amend the spec to
  define an independent but equivalent filter evaluator.
- Build a shared semantic corpus for absence, `Null`, numbers, ordering,
  containment, and failures.
- Run every portable filter through both paths and compare results.
- Version serialized query plans.

Implementation notes:

- Profile tag `QUERY_PLAN_PROFILE = "dingo-query-plan-v1"` for serializable
  [`QueryPlan`] (filter + options JSON).
- `Filter::to_sda` / `matches_sda` compile the portable vocabulary to boolean
  SDA over `input`, using host helpers `getPath`, `startsWith`, `strContains`.
- `getPath` matches native path rules: missing or non-object intermediate →
  `None`; stored JSON `null` → `Some(null)`.
- Comparison / type failures do not match (SDA non-bool/`Fail` → false).
- Corpus: native `matches` ≡ `matches_sda` ≡ embedded `find` / force-scan.
- Follow-on: remote/cluster plan RPC carrying the profile tag; index path is
  already constrained to equality and re-checks with the same filter.

Acceptance:

- No semantic divergence exists in the shared vocabulary.
- Embedded, remote, cluster, indexed, and scan execution pass the same corpus.

### DEF-029 — Add resource governance

Priority: P1  
Status: **addressed** (query budgets + host limits + cancel; see
`stage_def_029_resource_governance`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Enforce configurable limits for request bytes, JSON depth, frame lengths,
  scan bytes, decoded objects, sort memory, concurrent queries, open
  connections, and per-tenant work.
- Spill deterministic sorts only through a documented verified temp format.
- Return typed `ResourceLimit`/`QueryBudgetRequired` errors with partial
  coverage.
- Add cancellation propagation through storage and network loops.

Implementation notes:

- Profile tag `RESOURCE_PROFILE = "dingo-resource-v1"`.
- Explicit [`QueryBudget`]: `max_docs_scanned`, `max_bytes_scanned`,
  `max_result_bytes` — exceed → `QueryBudgetRequired` unless
  `allow_partial_coverage` returns matches collected so far.
- Hard host [`ResourceLimits`]: JSON depth (default 64), payload bytes
  (16 MiB), RPC line bytes (16 MiB), result materialisation ceiling (64 MiB)
  → `ResourceLimit`. Frame length bounds remain in `dingo_format::SafetyLimits`.
- Cooperative [`CancelToken`] on `QueryOptions` / builder; checked between
  scan pages and index probes (not serialized in query plans).
- Sort/materialise fail closed when over budget/ceiling; spill-to-disk sort
  is **not** enabled in this profile (documented).
- Follow-ons: per-tenant work quotas, verified temp spill format; connection
  admission addressed in DEF-030.

Acceptance:

- Adversarial requests cannot cause unbounded memory, CPU, file descriptors, or
  disk growth.
- Limits are observable and tested at boundary values.

### DEF-095 — Locator-first primary index (stop O(dataset) RSS)

Priority: P0  
Status: **addressed (cut)** (2026-07-27) — slim PrimaryIndex + v3 cache

Problem:

The 10 GiB `dingo-testrig` campaign drove process RSS to ~10 GiB and forced the
host into swap, poisoning latency metrics. Root cause:

1. **`PrimaryIndex` stored full payload bodies** for every live subject
   (`LiveValue.body`), so resident memory tracked dataset size (~3.5 GiB of
   8 KiB payloads for 450k keys).
2. **Dual maps** (`index` + `durable_index`) each held a full body copy via
   `apply_durable_event` → ~2× bodies.
3. **`indexes/primary.idx` v2** serialized every body (~3.5 GiB on disk for a
   3.5 GiB segment set). Open + `fs::read` + decode + clone into both maps
   peaked near **dataset × 3**.
4. Checkpoint encode built another full body `Vec` while dual maps still held
   payloads → additional multi-GB peak during pump rate-limited checkpoints.

Evidence from `/var/tmp/dingo-testrig-10g/store`:

| Component | On-disk |
|-----------|---------|
| `segments/` | ~3.5 GiB |
| `indexes/primary.idx` (fat v2) | ~3.5 GiB |
| `indexes/chimera/*.cmr` | ~3.5 GiB |
| **Total** | ~10.5 GiB |

Chimera layouts are a **disk** amplification issue (derived full-value
sidecars); they were not on the hot get path, but fat primary index memory was.

Work:

- Store **frame locators** (`segment_id` + `frame_offset`) in the primary index
  for durable puts; drop ordinary payload bodies from resident maps.
- Keep resident only: memory-mode publishes, chunk **manifests** (small).
- `Store::get` / `get_payload`: map lookup → resident body **or** bounded frame
  pread at `frame_offset` (active segment bytes first, then disk).
- Primary cache **v3** (`DIDX0003`): frontier + offsets + slim bodies only.
- Refuse to load legacy fat v1/v2 caches above a 64 MiB resident-body budget
  (force slim rebuild instead of re-inflating multi-GB RSS).
- Compaction / checkpoint / Chimera pair builders resolve bodies via locator.

Implementation notes:

- `index::LiveValue::{frame_offset, body}` with `slim_put_body_for_index`.
- `index_cache` write path always v3; `try_load_primary_index_frontier` prefers v3.
- `compact::{pread_item_body, resolve_live_body}` for disk re-read.
- `Store::resident_index_body_bytes()` for operators / tests.
- Follow-ons (not blocking this cut): stop dual full-value Chimera sidecars on
  seal (disk bloat); streaming rebuild without holding all event bodies;
  optional tiny-body inline threshold; process RSS gauge on metrics.

Acceptance:

- Durable pump of multi-GiB data keeps primary-index resident bodies ≪ dataset
  (metadata + manifests only).
- `get` after buffered/durable put returns correct payloads via frame pread.
- Reopen with v3 cache does not load O(dataset) body bytes into RSS.
- Existing store unit/integration suites pass.

**Re-verification (2026-07-27):** second 10 GiB `dingo-testrig` campaign
(`--seed 2`, 8 KiB payloads, buffered) **PASS**; peak RSS ~0.92 GiB (vs ~10 GiB
pre-cut). See `doc/BENCHMARK_DISCLOSURE.md` (10 GiB snapshot) and
`doc/WORK_HORIZON.md` (memory-eat self-check).

### DEF-096 — Parallel ingest: multi-core write path (async lifecycle → sharded writers)

Priority: P1  
Dependencies: DEF-023 (follow-on), DEF-020, DEF-095  
Status: **Axis A + Axis B + Axis C harness addressed and measured**; **product
cluster capacity path started** (async lifecycle + sharded writers + multi-store
testrig + `Cluster::put_many` multi-partition fan-out; 1 GiB comparative +
256 MiB integrity + **clean multi-store 10 GiB PASS** 2026-07-27, ~17.7k ops/s
wall / CPU% sum ~376 with free disk) — see `doc/PARALLEL_INGEST.md` /
`doc/BENCHMARK_DISCLOSURE.md`. Network multi-process product capacity still open.

**Post-measure residual (not blocking DEF-096 acceptance):** Axis B wall lift is
modest (~7.4k → ~8.1k ops/s) because PrimaryIndex publish after `put_many` stays
serial (process CPU% still ~1-core class). Optional follow-on: shrink that serial
section so shard count and wall ops/s move together (only with before/after
numbers). **Ordered next steps toward maximum performance** (ingest residual →
product cluster → durable disclosure → Hydra hot get → Chimera worker → DEF-093)
are frozen in `doc/WORK_HORIZON.md` (“Next steps towards maximum performance”
self-check, 2026-07-27) and `doc/PARALLEL_INGEST.md` §7 / §10. Strategy order
after this cut: gate-driven readiness by default; performance labor only against
those measured residuals.

**Product capacity path (S3, 2026-07-27):** `Cluster::put_many` groups items by
virtual partition, ensures leadership once per partition, and returns honest
per-item `ClusterWriteAck` (replica_acks / committed / leader). Tests:
`stage_def_096_product_capacity.rs` (dev + dependable-local multi-leader
spread). Not a substitute for multi-process OS Jepsen (DEF-041 follow-on).

Problem:

After DEF-095, a 10 GiB buffered pump on Apple M4 keeps RSS ~0.92 GiB and does
not thrash, but process CPU sits mostly in the ~50% range with peaks near one
full core (~97%). Memory and SSD headroom are unused. The store is still a
**single exclusive writer** over **one active segment**, with `seal_active` and
rate-limited `persist_index_cache` on the put acknowledgement path. Hydra/Chimera
compile runs synchronously at seal. Product doctrine already requires parallel
ingestion (OVERVIEW §1 / §12, USP “sharded writers”); the implementation does
not yet use the spare cores.

Work (sequenced — do **not** multi-thread one `Store::put` first):

1. **Axis A — Async lifecycle (first cut):** dual (or more) active segment slots;
   O(1) foreground rotate; background seal (rewrite, BLAKE3, Hydra, Chimera,
   tier/catalog); background derived checkpoints; backpressure when workers lag.
   **Done:** `seal_pipeline` worker; `active/pending/` rotate; `drain_lifecycle` /
   recover-on-open; tests `stage_def_096_async_lifecycle`. Explicit `seal_active`
   remains sync (failpoint-compatible).
2. **Axis B — Sharded writers (done):** N active segments by subject hash
   (`create_with_shards`); per-shard append + auto-seal; shared PrimaryIndex;
   `put_many` parallel appends; tests `stage_def_096_sharded_writers`.
3. **Axis C — Horizontal (harness done; product path started):** testrig
   `--stores N` multi-process harness (capacity upper bound). Product path:
   `Cluster::put_many` multi-partition batch on dingo-cluster with independent
   partition leaders (dependable-local multi-node). Network multi-process
   serve-cluster capacity still open.
4. **Harness (done for Axis B + C):** testrig `--writer-shards N` creates with
   `create_with_shards`, pumps via `put_many` when N>1; `--stores N` multi-process
   capacity; discloses `concurrency` / `writer_shards` / `store_count` /
   `writer_model` plus peak RSS / process CPU% (`ps` samples).

Anti-goals:

- Rayon over a single active segment / exclusive lock.
- PrimaryIndex micro-opts as a substitute for lifecycle offload.
- Publishing multi-core claims without BENCHMARK_DISCLOSURE concurrency fields.

Acceptance:

- Design doc `doc/PARALLEL_INGEST.md` is the sequencing authority until cuts land.
- Axis A: put p99 not coupled to seal/checkpoint duration; crash matrix preserved;
  unit/integration tests for rotate+get+recover. **10 GiB re-measure follow-on**
  (ops/s + multi-core CPU%) not required to close Axis A code cut.
- Axis B: multi-shard create/open/put/get/seal/put_many tests pass; legacy
  single-shard layout unchanged; subject home-shard routing stable.

---

## 9. Production server and wire protocol

### DEF-030 — Replace the sequential TCP loop with a bounded server architecture

Priority: P1  
Dependencies: DEF-020  
Status: **addressed** (bounded worker server; see
`stage_def_030_bounded_server`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Keep one coordinated store owner per store path.
- Use a bounded worker/runtime model for connections and read-only work.
- Serialize or shard mutations through explicit writer ownership.
- Add connection limits, idle timeouts, graceful shutdown, backpressure, and
  overload responses.
- Avoid holding a connection open in the accept loop.
- Add request cancellation and server draining.

Implementation notes:

- Profile tag `SERVER_PROFILE = "dingo-server-v1"`.
- `serve_store_with` opens **one** `Store` (exclusive writer), wraps it in
  `Arc<Mutex<Store>>`, and admits up to `ServerLimits::max_connections`
  (default 64) worker threads. Accept loop is non-blocking and never runs
  request I/O inline.
- Over-limit / draining admissions get an immediate `resource_limit` line and
  close (backpressure). Idle sockets use configurable read/write timeouts
  (default 120s).
- Mutations serialize on the store mutex (writer ownership); mutex is not held
  across socket I/O. Mutation start/finish counters are reported on drain.
- Graceful shutdown via `ServeOptions::shutdown_flag`: stop accept, drain
  workers up to `drain_timeout`, report stats; mismatch or timeout is an error.
- Follow-ons: concurrent read snapshots without mutex, worker pool reuse
  (instead of thread-per-connection), load-test CI artifacts (DEF-050), TLS
  (DEF-032).

Acceptance:

- One slow client cannot block unrelated clients.
- Load tests prove bounded memory and stable tail latency under overload.
- Graceful shutdown either completes or reports the outcome of every accepted
  mutation.

Evidence:

- `crates/dingo-sdk/tests/stage_def_030_bounded_server.rs` — concurrent clients,
  connection limit overload, single store owner under load, graceful shutdown.
- Unit: `server::tests` admission/drain/mutation accounting.

### DEF-031 — Version and frame the network protocol

Priority: P1  
Status: **addressed** (framed `dingo-rpc-v1` handshake; see
`stage_def_031_protocol`, `tests/fixtures/protocol/`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Replace implicit line-delimited JSON compatibility with an explicit
  handshake and protocol version.
- Add maximum message lengths before allocation.
- Define feature negotiation and required receipt fields.
- Separate transport framing from application encoding.
- Preserve a human-debuggable mode only as a diagnostic profile.
- Add compatibility fixtures for supported versions.

Implementation notes:

- Profile tag `PROTOCOL_PROFILE = "dingo-rpc-v1"`; draft label
  `RPC_WIRE_LABEL = "1.0-draft"`.
- Transport frame: big-endian `u32` length + UTF-8 JSON payload. Application
  bodies remain `RpcRequest` / `RpcResponse` (encoding separate from framing).
- Connect path: client `hello` → server `welcome` (or `reject`) with
  `max_frame`, feature tokens (`json-rpc-v1`, `receipts-v1`, `idempotency-v1`),
  and `required_receipt_fields` for `receipts-v1`.
- Length is checked against negotiated `max_frame` **before** payload
  allocation. Legacy clients that send bare `{...}` lines get a clear
  `protocol_violation` reject (or connection close).
- Diagnostic profile: `ConnectOptions` / `ServeOptions::diagnostic_line_protocol`
  restores newline-delimited JSON without handshake for local debugging only.
- Overload/drain admission rejects are framed handshake `reject` messages with
  `code=resource_limit`.

Acceptance:

- Old/new clients fail clearly or negotiate a documented compatible subset.
- Oversized and malformed frames are rejected without unbounded allocation.
- Golden protocol fixtures run in CI.

Evidence:

- `crates/dingo-sdk/tests/stage_def_031_protocol.rs` — handshake, legacy
  rejection, version reject, oversized frames, diagnostic mode, fixtures.
- `crates/dingo-sdk/tests/fixtures/protocol/*.json` — golden payloads.
- Unit: `protocol::tests` frame limits and feature negotiation.

### DEF-032 — Add TLS and authenticated peer identity

Priority: P0  
Status: **addressed** (TLS 1.3 + mTLS + identity SANs; see
`stage_def_032_tls`, `dingo_sdk::tls`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Support TLS 1.3 for client/server traffic.
- Support mTLS for node-to-node traffic.
- Verify hostname/service identity and cluster/node IDs.
- Define certificate reload and rotation without downtime.
- Remove credentials from request bodies and logs.
- Use constant-time secret comparison for any retained token mode.
- Make plaintext a loopback-only development profile.

Implementation notes:

- Profile tag `TLS_PROFILE = "dingo-tls-v1"` (rustls, TLS 1.3 only).
- `ServeOptions::tls` / `ConnectOptions::tls` with PEM cert/key/CA paths;
  CLI `--tls-cert` / `--tls-key` / `--tls-client-ca` / `--tls-cluster-id`.
- Peer identity via certificate SAN URIs:
  `urn:dingo:cluster:{id}` and `urn:dingo:node:{id}`.
- Hot reload: `TlsServerState::reload()` re-reads PEM paths; new handshakes
  use rotated material without process restart (`tls_state_slot` for callers).
- Shared application tokens use constant-time compare; `redact_secret` for
  logs. Tokens remain optional application auth (authorization is DEF-033).
- Non-loopback plaintext still requires `--allow-insecure-bind`; non-loopback
  with TLS is allowed (DEF-002 updated).

Acceptance:

- MITM, wrong-host, expired, revoked, and wrong-cluster certificates fail.
- Rotation tests keep healthy connections available.
- Security scans confirm secrets are not logged.

Evidence:

- `crates/dingo-sdk/tests/stage_def_032_tls.rs` — happy path, wrong host,
  wrong cluster, expired, wrong CA (MITM), revoked serial, mTLS, rotation,
  plaintext loopback, public bind policy with TLS.
- Unit: `tls::tests` constant-time compare + URN helpers.

### DEF-033 — Implement authorization and audit

Priority: P1  
Status: **addressed** (privilege model + audit chain; see `stage_def_033_authz`,
`dingo_sdk::authz`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Separate authentication from permissions.
- Define database, collection, operation, administration, salvage, tier, and
  purge privileges.
- Make purge and force-reconfiguration high-friction, separately authorized
  operations.
- Write tamper-evident audit records for security- and recovery-sensitive
  actions.
- Bound audit labels and redact payloads/secrets.

Implementation notes:

- Profile tag `AUTHZ_PROFILE = "dingo-authz-v1"`.
- `Privilege` / `PrivilegeSet` roles: reader, writer, dba, operator, superuser.
- `AuthzPolicy` maps shared tokens → principals (public id + privilege set);
  constant-time token compare; never stores tokens in audit.
- `ServeOptions::authz` + `ServeOptions::audit`; legacy `auth_token` synthesizes
  a single superuser principal.
- RPC op → privilege map; `purge` / `force_reconfig` also require `confirm`
  strings `PURGE` / `FORCE_RECONFIG`.
- `AuditLog` hash-chained records (seq, prev_hash, principal_id, op, decision);
  labels bounded (`MAX_AUDIT_LABEL_LEN`); secrets redacted from reasons.
- Scaffold privileged RPCs (`admin_stats`, `salvage_export`, `tier_move`,
  `purge`, `force_reconfig`) so gates are testable before full engines land.

Acceptance:

- A writer cannot administer, salvage, move tiers, or purge without explicit
  permission.
- Denied operations are tested and audited without exposing secrets.

Evidence:

- `crates/dingo-sdk/tests/stage_def_033_authz.rs` — writer denial matrix, high-
  friction confirm, reader cannot write, auth vs authz error codes, legacy
  token superuser, secret non-leakage.
- Unit: `authz::tests` chain integrity, open mode, constant-time auth.

### DEF-034 — Add protocol admission control

Priority: P1  
Status: **addressed** (rate / auth lockout / connect churn / expensive budgets /
operation-id replay; see `stage_def_034_admission`, `dingo_sdk::admission`,
`doc/CAPABILITY_MATRIX.md`)

Work:

- Add per-principal and global rate limits.
- Bound authentication failures and connection churn.
- Protect expensive scan/index/doctor operations with budgets.
- Add replay windows where credentials or signed requests require them.

Implementation notes:

- Profile tag `ADMISSION_PROFILE = "dingo-admission-v1"`.
- `AdmissionLimits` / `AdmissionController` shared process-wide via
  `ServeOptions::admission` (built from `admission_limits` at serve start).
- Fixed 1s windows for global and per-principal RPC rates; overload →
  `resource_limit`.
- Auth-failure keys are hashes of the presented token (secrets never stored);
  after `max_auth_failures` within the window, lockout returns generic
  `authentication_failed` without re-scanning tokens.
- Connection churn limited at accept (`try_admit_connect`) before simultaneous
  connection slots (DEF-030); rejects use framed handshake `resource_limit`.
- Expensive ops (`find`, `scan_json`, `index_*`, salvage/admin scaffolds, …)
  take a concurrency budget (`ExpensiveGuard`).
- Mutation `operation_id` values enter a bounded TTL replay window: retries of
  the same `(principal, operation_id)` are admitted; capacity pressure returns
  `resource_limit`. Store-level idempotency (DEF-010) remains authoritative for
  content matching.

Acceptance:

- Load and abuse tests show bounded resource use and useful overload errors.

Evidence:

- `crates/dingo-sdk/tests/stage_def_034_admission.rs` — global rate, auth
  lockout, connect churn, expensive concurrency, operation-id replay, SDK
  client `resource_limit` surface.
- Unit: `admission::tests` rate windows, lockout key hygiene, expensive guard,
  replay fresh/retry/full.

---

## 10. Persistent distributed system

### DEF-035 — Persist Raft state correctly

Priority: P0  
Dependencies: DEF-021, DEF-022  
Status: **addressed** (durable hard state / log / membership / snapshots;
in-process cluster restore on open; see `stage_def_035_raft_persist`,
`dingo_cluster::raft_persist`, `doc/CAPABILITY_MATRIX.md`)

Problem:

Terms, votes, logs, and commit indexes exist only in memory.

Work:

- Adopt a proven Raft library or subject the current protocol to an independent
  formal/safety review before extension.
- Persist current term and voted-for before granting votes.
- Persist log entries before acknowledging append.
- Persist commit/applied frontiers and membership configuration.
- Define state-machine application idempotency.
- Add snapshots with checksum, last-included term/index, and atomic install.
- Recover from torn log tails and corrupt snapshots without fabricating
  commitment.
- Keep user payload frames independently salvageable.

Implementation notes:

- Profile tag `RAFT_PERSIST_PROFILE = "dingo-raft-persist-v1"`.
- On-disk layout: `{cluster_root}/raft/node-{n}/p{partition}/` with
  checksummed `hard_state.json`, `membership.json`, append-only
  length-prefixed `log.ndjson`, and optional `snapshot.meta.json` +
  `snapshot.blob`.
- `PartitionRaft::attach_store` / `flush_peer`: votes and AppendEntries success
  flush hard state + log before ack (fail closed on I/O error).
- `Cluster::create` / `open` seed or restore peer stores; leadership role is
  always volatile (Follower on reopen).
- Snapshots: `install_local_snapshot` writes blake3-checked meta/blob and
  truncates the durable log past `last_included_index`.
- Torn log tails and corrupt snapshot blobs are discarded; commit never
  advances past validated durable evidence.
- `ConsensusEvidenceClass`: `committed` | `prepared` | `conflicting` |
  `unknown_commit`.
- User payload frames remain in ordinary `dingo-store` segments (salvage
  independent of Raft control plane).

Acceptance:

- Raft safety tests cover crash/restart at every persistence boundary.
- Jepsen-style histories show no lost acknowledged writes, split-brain
  commitments, or stale-leader acceptance.
- Consensus evidence can distinguish committed, prepared, conflicting, and
  unknown-commit frames after disaster.

Evidence:

- `crates/dingo-cluster/tests/stage_def_035_raft_persist.rs` — process restart
  with committed write, vote hard-state durability, torn tail, corrupt
  snapshot discard, snapshot compact+recover, uncommitted not promoted,
  term restore across nodes.
- Unit: `raft_persist::tests` hard state/.prev fallback, log append/torn
  tail, snapshot install, evidence classes, peer save/load.

Remaining (out of this cut; tracked by DEF-036+ / DEF-041):

- Full multi-process Jepsen-style network partition histories.
- Independent formal/safety review of the purpose-built protocol (or adopt a
  proven Raft library) before freezing network Raft.

### DEF-036 — Implement real network Raft RPC

Priority: P0  
Dependencies: DEF-031, DEF-032, DEF-035  
Status: **addressed** (control-plane RequestVote / AppendEntries /
InstallSnapshot / ReadIndex over framed transport; term / membership /
placement-epoch fences; see `stage_def_036_raft_rpc`,
`dingo_cluster::raft_rpc`, `dingo_sdk::raft_server`, `doc/CAPABILITY_MATRIX.md`)

Work:

- Implement authenticated RequestVote, AppendEntries, snapshot install, and
  leadership/read-index RPCs.
- Use bounded batching, flow control, retry, and per-peer backoff.
- Fence writes by term, membership, and placement epoch.
- Never treat endpoint routing data as authority to write.
- Separate control-plane and data-plane availability.

Implementation notes:

- Profile tag `RAFT_RPC_PROFILE = "dingo-raft-rpc-v1"` (SDK feature
  `FEATURE_RAFT_RPC_V1 = "raft-rpc-v1"`).
- `NetworkRaftNode` — single-peer Raft actor over a `RaftTransport`;
  `MemoryRaftNetwork` for in-process multi-peer tests; `TcpRaftTransport`
  for framed `raft_*` ops between processes.
- Wire ops: `raft_request_vote`, `raft_append_entries`,
  `raft_install_snapshot`, `raft_read_index` on `serve_store` /
  `serve_cluster_node` when `ServeOptions::raft` is set.
- Fences: `cluster_id`, `placement_epoch`, voter membership, and term;
  endpoint maps are routing hints only (`raft_fenced` on epoch mismatch).
- Bounded batching (`DEFAULT_MAX_APPEND_BATCH`), per-peer backoff, and
  replicate retries; `operation_id` dedup preserves one log index on retry.
- Persist-before-ack (DEF-035) remains on each peer when a store is attached.
- Auth: shared token / authz path required for peer RPCs when configured.

Acceptance:

- Three independent processes on separate storage roots provide quorum
  durability.
- Minority partitions cannot commit strong writes.
- Old leaders cannot write after a new term.
- Response loss and retries preserve one event identity.

Evidence:

- `crates/dingo-cluster/tests/stage_def_036_raft_rpc.rs` — three-peer quorum
  commit, minority no-elect, stale leader fence, operation_id retry,
  placement-epoch fence, multi-root durable peers.
- `crates/dingo-sdk/tests/stage_def_036_raft_rpc.rs` — TCP RequestVote /
  AppendEntries / ReadIndex across three listeners; unauthenticated reject.
- Unit: `raft_rpc::tests` (campaign/propose, offline peers, snapshot install).

Remaining (out of this cut; tracked by DEF-037 / DEF-041):

- ~~Data-plane collection put/get routed through network Raft propose
  (DEF-037)~~ — **addressed** (see DEF-037).
- Full multi-process Jepsen-style partition histories (DEF-041).
- Independent formal/safety review (or proven Raft library) before freezing
  network Raft as the only production path.

### DEF-037 — Make cluster SDK requests actually use cluster commitment

Priority: P0  
Dependencies: DEF-036  
Status: **addressed** (2026-07-27) — network put/delete go through partition
Raft propose when `serve-cluster` attaches Raft; `committed=true` only after
quorum + local apply; linearizable get via read-index barrier; see
`stage_def_037_cluster_commit`, `RaftServerState::propose_and_apply`,
`doc/CAPABILITY_MATRIX.md`

Work:

- Route network writes through the partition leader's Raft proposal path.
- Return a replicated acknowledgement only after the configured durability and
  quorum conditions are proven.
- Implement linearizable reads using leader/read-index or documented quorum
  reads.
- Refresh stale routes on typed epoch/term errors, not only transport strings.
- Preserve operation IDs across redirects.
- Verify every endpoint belongs to the expected cluster.

Implementation notes:

- Profile tag `CLUSTER_COMMIT_PROFILE = "dingo-cluster-commit-v1"` and
  feature token `FEATURE_CLUSTER_COMMIT_V1 = "cluster-commit-v1"`.
- `serve_cluster_node` attaches `RaftServerState` by default (best-effort;
  directory-only fallback if attach fails).
- Data-plane put/delete call `propose_and_apply` on the subject's partition;
  followers apply committed log entries after AppendEntries / InstallSnapshot.
- Reads use `linearizable_read_barrier` (read-index) when Raft is attached.
- Plain `serve_store` without Raft keeps single-node local-commit semantics.
- Still requires `--experimental-network-cluster` (DEF-002); not a production
  release claim.

Acceptance:

- Network and in-process cluster conformance suites share the same logical
  tests.
- Killing the contacted node after commit does not lose an acknowledged write.
- A routing-only acknowledgement is never labeled replicated.

Evidence:

- `crates/dingo-sdk/tests/stage_def_037_cluster_commit.rs` — feature/profile
  labels; in-process and network share commit semantics; kill contacted seed
  after committed puts (survivor reads + new commit); distinct operation
  events; single-node serve without Raft still local-commits.
- Server path: `dingo_server::raft_server::RaftServerState::propose_and_apply`,
  `serve.rs` mutate + linearizable read barrier.

Remaining (out of DEF-037 cut; DEF-038 addressed separately):

- ~~Durable rebalance / joint consensus across restarts (DEF-038)~~ — **addressed**.
- ~~Anti-entropy / repair (DEF-039)~~ — **addressed** (in-process).
- ~~Distributed query pages (DEF-040)~~ — **addressed**.
- Full multi-process Jepsen-style partition histories (DEF-041).
- Formal/safety review before freezing network Raft as production-only path.

### DEF-038 — Persist control-plane and rebalance workflows

Priority: P0  
Dependencies: DEF-035  
Status: **addressed** (2026-07-27) — durable rebalance jobs, joint membership
persist/restore, degraded open health, authenticated endpoint registration;
see `stage_def_038_control_plane`, `REBALANCE_CONTROL_PROFILE`,
`doc/CAPABILITY_MATRIX.md`

Work:

- Replicate membership, placement, policy, and epochs through durable
  consensus.
- Persist rebalance job phase, source/destination frontiers, hashes, and safety
  windows.
- Use joint consensus for voter changes.
- Resume or safely roll back after coordinator restart.
- Write endpoint registrations atomically and authenticate them.
- Refuse silent cluster open when expected nodes or metadata are missing;
  expose degraded state explicitly.

Implementation notes:

- Profile tag `REBALANCE_CONTROL_PROFILE = "dingo-rebalance-control-v1"`.
- In-flight jobs persisted as checksummed `rebalance_jobs.json` (atomic +
  `.prev`) after every `begin_rebalance` / `advance_rebalance`.
- `MembershipState` records joint flag + outgoing/incoming voter sets;
  `PartitionRaft::set_joint_voters` flushes before return.
- `Cluster::open` reloads jobs, re-attaches learners, restores joint or old
  voters so directory ownership never invents a gap mid-job.
- `Cluster::health` exposes expected/online/offline/missing stores and
  in-flight phases (`degraded` when incomplete).
- Multi-node open refuses missing `placement.json` (no silent synthetic map).
- Optional `registration_token_hash` on `cluster.json`; when set,
  `upsert_endpoint` refuses and `upsert_endpoint_authenticated` verifies
  the secret (atomic endpoint writes retained from DEF-021).

Acceptance:

- Restart at every rebalance phase leaves old placement authoritative or a
  valid joint configuration.
- Loss of the coordinator does not lose the operation state.
- Missing nodes are visible in health, coverage, and operator output.

Evidence:

- `crates/dingo-cluster/tests/stage_def_038_control_plane.rs` — restart at
  every phase + resume; joint membership restore; health missing nodes;
  missing placement refused; authenticated endpoints.
- Unit: `RebalanceJobsFile` roundtrip; Stage 8f suite still green.

### DEF-039 — Implement anti-entropy and replica repair

Priority: P1  
Dependencies: DEF-036  
Status: **addressed** (2026-07-27) — hierarchical inventory, integrity-based
source selection (never mtime), verified copy + audit, rate-limited passes;
see `stage_def_039_repair`, `ANTI_ENTROPY_PROFILE`, `doc/CAPABILITY_MATRIX.md`

Work:

- Exchange verified hierarchical inventories by partition, segment, region
  hash, log frontier, and chunk set.
- Detect missing, divergent, corrupt, and conflicting replicas.
- Select repair sources using integrity and consensus evidence, never mtime.
- Copy exact frames where possible and verify destination before placement
  activation.
- Preserve conflicts and audit every repair.
- Rate-limit repair and isolate it from foreground latency.

Implementation notes:

- Profile tag `ANTI_ENTROPY_PROFILE = "dingo-anti-entropy-v1"`.
- `Cluster::inventory_partition` / `inventory_cluster` collect per-replica
  subject content hashes, Raft log frontier, and store segment fingerprints.
- `select_repair_source` tallies only readable (healthy) bodies; corrupt
  observations never vote; strict majority wins; equal splits are explicit
  conflicts; leader preferred only when on the majority hash.
- `Cluster::repair_partition` / `repair_cluster` / `anti_entropy_once` copy
  verified bodies, re-hash destination after put, and append
  checksummed `repair_audit.json` (atomic + `.prev`).
- `RepairOptions::{max_subjects,max_bytes,dry_run}` bounds a pass so repair
  stays operator-invoked and isolated from the foreground put path.
- Local inject helpers (`store_put_local` / `store_delete_local`) support fault
  injection without claiming quorum commitment.

Acceptance:

- Corrupt/newer-mtime replicas never overwrite healthy evidence.
- Random deletion/corruption converges to policy while preserving explicit
  irrecoverable holes.

Evidence:

- `crates/dingo-cluster/tests/stage_def_039_repair.rs` — missing replica
  converges; divergent newer body loses to majority; 1-1 conflict preserved;
  rate limit; audit survives restart; segment fingerprints in inventory.
- Unit: `select_repair_source` majority / corrupt / conflict / irrecoverable;
  `RepairAuditFile` roundtrip.

### DEF-040 — Complete distributed query semantics

Priority: P1  
Dependencies: DEF-026, DEF-037  
Status: **addressed except token authentication DEF-097** (2026-07-27) —
multi-page distributed find with integrity-tagged continuation
(`dingo-query-continuation-v1`), deterministic
subject merge independent of visit/worker order, coverage on every page
including frontiers / read mode / indexes / tiers / resource limits;
coordinator resume without silent dup/omit

Work:

- Attach coverage to every distributed page.
- Define deterministic merge order independent of worker completion.
- Preserve per-partition frontiers and read modes.
- Resume coordinator failure using continuation state; qualify cryptographic
  authentication through DEF-097.
- Carry index/tier/resource limitations end to end.

Acceptance:

- Randomized worker ordering produces identical sequence results.
- Coordinator failover neither silently duplicates nor omits rows.
- Partial partitions are never represented as empty complete partitions.

Evidence:

- `crates/dingo-cluster/src/coverage.rs` — `QueryContinuation` keyed tags
  whose public key derivation remains DEF-097;
  `Coverage` indexes/tiers fields; `FindResult.has_more` / `continuation`.
- `Cluster::scan_with` / `scan_page` — paged merge by subject order;
  `ScanOptions::{page_size, continuation, visit_order, after_subject}`.
- Tests: `stage_def_040_query.rs` (visit-order identity, multi-page sequence,
  coordinator reopen resume, partial-partition honesty, budget, tamper,
  frontiers); unit tests for token roundtrip / wrong cluster / tamper.
- SDK: `ClusterFindResult` carries `has_more` + `continuation`;
  `ClusterBackend::scan_page`.

### DEF-041 — Build a distributed-system verification program

Priority: P0  
Status: **partial** — in-process cut (2026-07-27) + **multiproc OS chaos / short
soak labor (DEF-041-N, 2026-07-31)**. Full Jepsen PORC against live
`serve-cluster` TCP + multi-hour soak remain for production multi-node claims.

Work:

- Add deterministic simulation for elections, delays, duplication, reordering,
  partitions, and crashes.
- Add multi-process chaos tests.
- Add linearizability checking for strong mode.
- Add convergence checking for convergent append.
- Test every `CLUSTER_SPEC.md` §22 case against the network implementation.
- Run long-duration soak and rolling-restart tests.

Acceptance:

- The complete cluster conformance matrix runs before a production release.
- Failures retain seeds and histories for deterministic replay.

Evidence:

- `crates/dingo-cluster/src/sim.rs` — `SeedRng`, `FaultModel`, `SimTransport`,
  `SimWorld` (`client_put` / `client_get` / `run_chaos` / `run_soak` /
  `campaign_with_epoch`), `check_partition_linearizable`,
  `check_convergent_preserved`, `run_conformance_matrix`, profile
  `VERIFY_PROFILE` = `dingo-cluster-verify-v1`.
- Tests: `stage_def_041_verify.rs` (seed replay, §22.1–.8, chaos + soak
  linearizability, dump retains seed); module unit tests under `sim::tests`.
- **DEF-041-N labor (2026-07-31):** multiproc OS harness
  `dingo-cluster-multiproc-v1` (`multiproc` module + `dingo-cluster-multiproc-child`
  binary + `stage_def_041n_multiproc`): rolling restart, abort-after-ack, cross-
  process writer lock, short soak, seed+history JSON dumps. Long soak via
  `DINGO_MULTIPROC_LONG_SOAK=1`.
- Remaining: full Jepsen PORC against live `serve-cluster` TCP partitions,
  multi-hour soak, CLUSTER_SPEC §22.9–.20 network surface; serve-cluster stays
  experimental until those pass.

---

## 11. Backup, restore, upgrades, and fleet operations

### DEF-050 — Productize backup and restore

Priority: P1  
Status: **addressed (single-node full cut)** (2026-07-27) — content-hashed
full backup package (`dingo-backup-v1`), exclusive-flush and inspect
consistencies, verified restore with optional identity reassignment; CLI
`dingo backup` / `dingo restore`. Incremental, encrypted, remote-target,
cluster, and tiered multi-volume remain follow-ons.

Work:

- Define crash-consistent online backup boundaries.
- Support full and incremental backup with verified manifests.
- Preserve store/cluster identity, wire version, keys, segment hashes, and
  consensus evidence.
- Support restore into a new generation and explicit identity reassignment.
- Add retention, encryption, and remote target policies.
- Keep salvage distinct from backup restore.

Acceptance:

- Restore tests cover single-node, cluster, tiered data, partial backup,
  interrupted upload, and missing encryption keys.
- Recovery point and recovery time are measured.
- Quarterly restore drills are documented and automated.

Evidence (this cut):

- `crates/dingo-store/src/backup.rs` — `BACKUP_PROFILE` = `dingo-backup-v1`,
  `write_full_backup` / `restore_full_backup`, manifest + per-file blake3,
  `BackupConsistency::{FlushedExclusive,OnDiskInspect}`.
- `Store::backup_to` flushes durable active under exclusive writer; inspect
  opens copy on-disk files without flush.
- Restore verifies manifest hash and every file; optional
  `RestoreOptions::reassign_identity` for clones (new `store_id`, wipe keyed
  derived state).
- CLI: `dingo backup STORE -o PKG`, `dingo restore PKG -o STORE
  [--reassign-identity]`.
- Tests: `stage_def_050_backup.rs`, module unit tests, CLI
  `backup_and_restore_roundtrip` / `restore_reassign_identity_clone`.
- Salvage / export-live remain separate paths (no backup package confusion).

Remaining (out of this cut):

- Incremental backup chains; encryption at rest for packages; remote targets
  (S3/GCS native).
- Cluster-coordinated backup (Raft snapshots + multi-node identity).
- Tiered multi-volume / offline archive inclusion policy.
- Automated quarterly restore drills and RPO/RTO measurement harness.

### DEF-051 — Add integrity scrub and media-health automation

Priority: P1  
Status: **addressed (single-node cut)** (2026-07-27) — bounded scrub with
durable frontier/findings, placement hash checks, frame-hole detection,
quarantine-without-hide, pause/resume, CLI `dingo scrub`. Background daemon
scheduling, cluster repair hook-in, and media SMART/health remain follow-ons.

Work:

- Implement bounded background frame/segment/chunk verification.
- Persist scrub frontier and findings.
- Integrate repair when redundancy exists.
- Quarantine corrupt evidence without hiding it.
- Expose scrub age, bytes verified, failures, and coverage.

Acceptance:

- Injected corruption is detected within the configured scrub interval.
- Scrub can pause/resume and does not starve foreground work.

Evidence (this cut):

- `crates/dingo-store/src/scrub.rs` — `SCRUB_PROFILE` = `dingo-scrub-v1`,
  `scrub_once` / pause / resume / status, plan over sealed + active + chunks,
  BLAKE3 + placement `content_hash` + `scan_forward` holes.
- Durable state: `recovery/scrub/state.v1.json`, `findings.v1.json`,
  `quarantine/` (copy only; original retained).
- `Store::scrub_once`, `scrub_to_completion`, `scrub_status`, `pause_scrub`,
  `resume_scrub`, `list_scrub_findings`.
- CLI: `dingo scrub STORE [--once|--status|--pause|--resume] [--max-files]
  [--max-bytes] [--no-quarantine]`.
- Tests: `stage_def_051_scrub.rs`, module unit tests, CLI
  `scrub_clean_store_and_status`.

Remaining (out of this cut):

- Automatic interval scheduler / background worker process.
- Hook scrub findings into DEF-039 replica repair when redundancy exists.
- Native media-health (SMART, cloud object integrity APIs).
- Multi-volume / offline-tier scrub scheduling policies.

### DEF-052 — Define format and protocol migration

Priority: P0

Status: **addressed (single-generation cut)** (2026-07-27) — declared wire
reader/writer matrix (`dingo-format::compat`), protocol policy snapshot,
phased store migration `dingo-migrate-v1` (preflight / plan / apply / verify /
rollback), evidence-preserving copy (never in-place rewrite), unsupported and
unreadable segment bytes preserved as opaque evidence, durable job under
`recovery/migration/`, CLI `dingo migrate`; multi-major dual-read writers and
rolling mixed-cluster upgrade drills remain follow-ons (DEF-053 freeze).

Work:

- Define supported reader/writer version matrix.
- Keep old readers for every supported wire generation.
- Make migration append/copy evidence-preserving, never in-place blind rewrite.
- Add preflight, plan, apply, verify, and rollback phases.
- Preserve unsupported bytes and provenance.
- Test rolling upgrades across adjacent supported versions.

Acceptance:

- A mixed-version cluster follows an explicit compatibility policy.
- Failed migration leaves the prior generation readable.
- Golden corpora survive upgrade and downgrade where promised.

Evidence (this cut):

- `dingo-format::compat` — `wire_compat_matrix`, `SUPPORTED_READER_MAJORS`,
  `wire_reader_supports` / `wire_writer_emits`.
- `dingo-store::migrate` — `migrate_preflight`, `migrate_plan`, `migrate_apply`,
  `migrate_verify`, `migrate_rollback`, `migrate_store`; `Store::migrate_to`.
- Tests: `stage_def_052_migrate`, `migrate::tests`, CLI
  `migrate_roundtrip_and_status`.
- CLI: `dingo migrate STORE --output DEST` (`--preflight`, `--plan-only`,
  `--status`, `--rollback`, `--skip-verify`).

Remaining (out of this cut):

- Second wire major with dual-read + rewrite-to-current plan actions.
- Rolling multi-node cluster upgrade drills (mixed protocol majors).
- Golden corpus upgrade/downgrade suite across promised support windows
  (ties to DEF-053 freeze).

### DEF-053 — Freeze wire major 1 only after qualification

Priority: P0  
Dependencies: DEF-010 through DEF-014, DEF-022, DEF-052

Status: **partial — freeze gap inventory + policy cut** (2026-07-31) — freeze
**not** declared; `WIRE_PROFILE_LABEL` remains `1.0-draft`. Published
`doc/WIRE_MAJOR1_FREEZE.md` (criteria F1–F16, canonical encodings inventory,
compatibility / relabel procedure), `dingo-format::compat` freeze readiness API
(`wire_freeze_criteria`, `wire_is_frozen`, `wire_freeze_summary`) with DEF-053
guard test preventing silent stable relabel. Implemented Met rows: framing,
integrity, limits, CBOR, conflict, salvage §13, encodings inventory, compat
policy. Residual Open/Partial: external review, long fuzz, production soak,
clean-room multi-impl, multi-window golden upgrade/downgrade, stable label.

Work:

- Complete external review of framing, integrity, limits, chunk manifests,
  envelopes, conflict identity, and recovery ordering.
- Add fuzzing and multi-implementation fixtures.
- Run production-scale soak and corruption campaigns.
- Publish canonical encodings and compatibility policy.
- Assign a new label if semantics changed after `1.0-draft`.

Acceptance:

- `WIRE_PROFILE_LABEL` becomes stable only after all freeze criteria pass.
- Every historical corpus remains readable by the promised support window.

Evidence (this cut):

- `doc/WIRE_MAJOR1_FREEZE.md` — policy id `dingo-wire-major1-freeze-v1`.
- `dingo-format::compat` — freeze criteria table + `wire_is_frozen() == false`
  guard while draft.
- Tests: `compat::tests::def_053_wire_remains_draft_until_freeze`.

Remaining (blocks freeze declaration):

- External review of wire surfaces (ties DEF-063 independent audit).
- OSS-Fuzz / multi-hour fuzz accumulation (DEF-091-F residual).
- Production-scale soak + long corruption campaigns (DEF-041-N residual).
- Second clean-room implementation of golden vectors.
- Multi-major dual-read + golden upgrade/downgrade suite (DEF-052 remaining).
- Principal freeze declaration and stable `WIRE_PROFILE_LABEL` (only when all Met).

### DEF-054 — Provide safe configuration management

Priority: P1

Status: **addressed (process config cut)** (2026-07-27) — versioned
`dingo-config-v1` JSON schema, load/validate before serve bind, static vs
restart-required vs dynamic setting classes, secret refs (`env:` / `file:`)
with redacted effective reports, unsafe-combination gates (replication claim
with one local copy, public plaintext bind, serve-cluster without experimental
opt-in), CLI `dingo config validate|show` and `serve`/`serve-cluster --config`;
live atomic admission reload + audit chain remain follow-ons.

Work:

- Define a versioned configuration schema.
- Validate startup config before opening writers.
- Separate static, dynamic, and restart-required settings.
- Redact secrets and support external secret providers.
- Detect unsafe combinations such as replicated claims with one local copy.
- Record effective configuration in diagnostics.

Acceptance:

- Invalid configuration fails with actionable typed errors.
- Dynamic reload is atomic and audited.

Evidence (this cut):

- `dingo-server::config` — `DingoConfigFile`, `load_and_validate`,
  `validate_document`, `ConfigOverrides`, `ValidatedConfig`,
  `EffectiveConfigReport`, `resolve_secret_ref`, `redact_json_value`,
  `setting_class`; profile `CONFIG_PROFILE` = `dingo-config-v1`.
- Typed errors: `ConfigError::Validation` / `Unsafe` / `UnsupportedFormat` /
  `Secret` with stable `code` strings.
- CLI: `dingo config validate|show [--mode serve|serve-cluster|validate]`,
  `dingo serve --config`, `dingo serve-cluster --config` (flags override file).
- Tests: `stage_def_054_config`, `config::tests`, CLI
  `config_validate_show_and_unsafe_reject`.

Remaining (out of this cut):

- Live atomic reload of dynamic admission limits with audit log entries
  (no restart) while serve is running.
- Hot-reload of TLS material beyond existing `TlsServerState::reload`.
- Multi-document / directory include layering and remote config providers.
- Persist effective config snapshot under store diagnostics on every open.

---

## 12. Observability and security operations

### DEF-060 — Implement structured logging

Priority: P1

Status: **addressed (process NDJSON cut)** (2026-07-27) — versioned
`dingo-log-v1` structured facade (`dingo-server::slog`), stable event names,
bounded field cardinality, redaction of credentials/payloads by construction,
RPC completion + guarantee-failed correlation fields on the serve path;
client-side log emission and distributed replica join remain follow-ons.

**Superseded target (2026-07-30):** the synchronous per-RPC stderr design is a
legacy implementation only. [TELEMETRY_SPEC.md](TELEMETRY_SPEC.md) replaces it
with aggregate in-memory measurement and bounded Ratatouille-only export. File,
stdout/stderr, syslog, and direct synchronous network sinks are not production
targets.

Work:

- Use a structured logging facade throughout.
- Include operation, store, cluster, partition, event/operation ID, error code,
  requested/achieved guarantee, and latency where applicable.
- Redact credentials and payloads by default.
- Add stable event names and bounded field cardinality.

Acceptance:

- Every failed guarantee can be correlated across client, server, and replica
  logs without parsing prose.

Evidence (this cut):

- `dingo-server::slog` — `Logger`, `LogEvent`, `LogSink` / `StderrSink` /
  `MemorySink`, `log_rpc_complete`, profile `LOG_PROFILE` = `dingo-log-v1`.
- Stable events: `server.start`, `server.drain`, `connection.rejected`,
  `connection.error`, `rpc.complete`, `guarantee.failed`, `raft.attach_failed`.
- Correlation fields: `request_id`, `operation_id`, `principal_id`, `op`,
  `collection`, `store` / `cluster` / `node_index` / `partition`, `error_code`,
  `guarantee_requested` / `guarantee_achieved`, `committed`, `event_id`,
  `latency_ms` — no `token` / body / `bytes_b64` keys.
- Serve wiring: `ServeOptions::logger`, default stderr NDJSON on
  `serve_store_with` / `serve_cluster_node`; every application RPC emits
  `rpc.complete` (+ `guarantee.failed` when requested durability is missed).
- Tests: `slog::tests`, `stage_def_060_logging` (correlation, redaction, churn
  reject, SDK e2e).

Remaining (out of this cut):

- Client SDK structured log emission of the same correlation ids (join client ↔
  server without custom app code).
- Replica / Raft data-plane peer logs with shared `operation_id` on propose
  apply paths beyond control-plane attach failures.
- Configurable log level / sink via `dingo-config-v1` (file/syslog; dynamic).
- OpenTelemetry log export bridge (ties to DEF-062 tracing).

### DEF-061 — Implement metrics and health endpoints

Priority: P1

Status: **addressed (process metrics + health cut)** (2026-07-27) — versioned
`dingo-metrics-v1` registry and `dingo-health-v1` probes on the serve path;
public liveness/readiness RPCs, authenticated detail/metrics scrapes, bounded
op labels + latency histograms + guarantee/admission counters. Store-tier
bytes, index lag, scrub/backup series, and dashboard packages remain follow-ons.

**Superseded export target (2026-07-30):** the in-memory registry remains a
measurement source, but Ratatouille periodic snapshots are the production
export. The metrics RPC is compatibility/test-only in the qualified profile;
health probes and authenticated health detail remain.

Required metrics:

- operation throughput and latency histograms;
- durability/consistency/commit outcomes;
- active/sealed/tier bytes;
- append, fsync, and write amplification;
- index state and lag;
- cache hit ratio;
- holes, corrupt frames, partial payloads;
- partition, leader, quorum, replica, repair, and rebalance health;
- query scan/read amplification and coverage;
- backup, scrub, compaction, and lifecycle status;
- resource-limit and admission-control events.

Work:

- Export stable metrics with bounded labels.
- Add liveness, readiness, and detailed authenticated health endpoints.
- Readiness must fail when the node cannot provide its advertised guarantees.

Acceptance:

- Dashboards and alerts exist for every supported deployment profile.
- Metric cardinality and overhead are load-tested.

Evidence (this cut):

- `dingo-server::metrics` — `MetricsRegistry`, `MetricsSnapshot`,
  `evaluate_health` / `HealthReport`, profiles `METRICS_PROFILE` /
  `HEALTH_PROFILE`.
- RPCs: `health_live` and `health_ready` (public probes, no token);
  `health` (Read, detailed); `metrics` (Admin scrape).
- Readiness fails when draining, store not open, or replication is claimed
  without Raft attached; public probes still answer during drain.
- Bounded labels: fixed known-op set + `other` overflow; fixed latency
  buckets; no collection/key/token labels.
- Counters: per-op total/ok/err + latency histogram; guarantee miss /
  committed; connection reject; admission rate/auth/expensive mirrored on
  scrape.
- Tests: `metrics::tests`, `stage_def_061_metrics_health`.

Remaining (out of this cut):

- Store/media gauges (active/sealed/tier bytes, holes, scrub age).
- Index lag, cache hit ratio, query amplification series.
- Partition/leader/quorum/replica/repair/rebalance gauges from cluster.
- Prometheus/OpenMetrics text exposition and example dashboards/alerts.
- Cardinality + overhead load tests under high RPS.

### DEF-062 — Implement distributed tracing

Priority: P2

Work:

- Trace routing, queueing, append, fsync, quorum wait, index work, tier fetch,
  verification, decoding, SDA evaluation, and response.
- Propagate trace context securely.
- Sample without losing all failure spans.

Acceptance:

- A slow request can be decomposed into application, network, storage, and
  consensus latency.

### DEF-063 — Perform threat modeling and independent security review

Priority: P0  
Dependencies: DEF-032 through DEF-034

Status: **partial** — threat-model first cut (2026-07-27) + **disclosure /
supported-version / audit package labor (DEF-063-A, 2026-07-31)**. Independent
external audit + remediation of critical/high findings remain for acceptance.

Work:

- Threat-model local files, RPC, cluster membership, salvage, archive media,
  supply chain, admin operations, and malicious stored data.
- Review parsers and recovery tools as hostile-input surfaces.
- Commission an independent security audit and remediate findings.
- Establish vulnerability disclosure and supported-version policy.

Acceptance:

- No unresolved critical/high security finding remains at release.
- Fuzz targets cover every untrusted decoder and protocol parser.

Evidence:

- `doc/THREAT_MODEL.md` — structured first-cut model (`dingo-threat-model-v0`).
- `SECURITY.md` — public vulnerability disclosure process.
- `doc/SUPPORTED_VERSIONS.md` — supported-version policy
  (`dingo-supported-versions-v1`).
- `doc/SECURITY_AUDIT_PACKAGE.md` — auditor evidence pack linking fuzz,
  threat model, and CI entrypoints.
- README security section links the above.
- Fuzz expansion/schedule: DEF-091 / DEF-091-F.

Remaining:

- Independent **external** audit engagement + signed report.
- Remediation of critical/high findings from that audit (or published residual).
- Production maturity claims still blocked until residual list is clear.

---

## 13. Archive and long-retention product

### DEF-070 — Implement native object-store backends

Priority: P1

Work:

- Implement native S3 and GCS clients behind `MediaBackend`.
- Support credentials, endpoint selection, timeouts, retries, multipart upload,
  conditional writes, checksums, range reads, and cancellation.
- Verify object identity/content after upload.
- Handle eventual-consistency assumptions explicitly.
- Keep mirror mode as a separately named adapter.

Acceptance:

- Emulator and real-provider integration suites pass.
- Interrupted multipart transfers resume or clean up safely.
- Missing/offline providers produce incomplete coverage, never false absence.

### DEF-071 — Implement durable lifecycle scheduling

Priority: P1

Work:

- Persist scheduler jobs and policy generation.
- Add plan/apply separation and transfer budgets.
- Verify copy before source deletion.
- Respect holds, retention, backup, repair, and scrub state.
- Expose progress and cancellation.

Acceptance:

- Restart at every move phase is safe and resumable.
- Policies cannot delete the last required copy.

### DEF-072 — Implement production erasure coding

Priority: P1

Work:

- Select and version a reviewed codec/profile.
- Encode independently verifiable shards with manifest checksums.
- Place shards across declared independent failure domains.
- Reconstruct from any valid threshold while rejecting conflicting shards.
- Repair degraded sets and rotate coding profiles through evidence-preserving
  migration.

Acceptance:

- Every combination of up to `m` shard losses reconstructs identical bytes.
- More than `m` losses produce explicit unrecoverable extents.
- Corrupt shards are detected and never treated as zero data.

### DEF-073 — Add encryption and key-lifecycle support

Priority: P1

Work:

- Define envelope and payload encryption profiles without weakening independent
  framing.
- Support key IDs, rotation, rewrapping, unavailable-key evidence, and audit.
- Keep ciphertext salvageable and self-identifying when keys are absent.
- Integrate cloud KMS through a provider-neutral interface.

Acceptance:

- Missing keys yield `encryption-unavailable`, not corruption or absence.
- Rotation does not require rewriting unrelated store content.

### DEF-074 — Validate the multi-decade retention claim

Priority: P2

Work:

- Publish format support windows and migration cadence.
- Maintain golden archives from every released wire generation.
- Run periodic media-loss and catalog-loss exercises.
- Document dependency-free recovery tooling and reproducible builds.

Acceptance:

- A clean environment can identify, verify, and examine historical segments
  without the original application or control plane.

---

## 14. SDK, CLI, documentation, and packaging

### DEF-080 — Complete the Rust SDK MVP contract

Priority: P1

Implement or explicitly remove from the stable specification:

- create-if-absent;
- version-conditional replace;
- inspection with evidence;
- generated-key add;
- bulk streaming writes;
- partition-scoped batches;
- version/time reads;
- watches/change streams;
- raw SDA examination;
- explain;
- continuation-token pagination;
- deadlines and cancellation;
- complete embedded/remote/cluster parity.

Acceptance:

- Every stable method documents atomicity, consistency, durability, retry
  safety, ordering, memory behavior, and uncertain outcomes.
- One conformance suite runs against every backend.

### DEF-081 — Complete operator CLI workflows

Priority: P1

Add task-oriented commands for:

- open/info;
- find and explain;
- inspect item/recovery evidence;
- index create/list/drop/rebuild;
- tier status/plan/move/copy;
- lifecycle plan/apply;
- scrub;
- backup/restore;
- cluster status/member/rebalance/repair;
- metrics/health;
- format migration.

Rules:

- Human output by default and stable JSON through one consistent flag.
- Exit status must agree with the JSON success/guarantee state.
- Destructive commands require explicit target and intent.
- Long operations support plan, progress, cancellation, and resume.

### DEF-082 — Replace pseudocode-first onboarding with executable journeys

Priority: P2

Work:

- Lead with the shipped Rust SDK and CLI.
- Label TypeScript examples as product-shape pseudocode until an SDK exists.
- Add install → put → get → find → index → bytes → serve → doctor → salvage
  quickstarts.
- Add all files referenced by SDA tutorials or remove broken references.
- Make every example compile/run in CI.
- Turn tier and node-failure demos into real operator narratives rather than
  wrappers around unit tests.

Acceptance:

- A fresh user can install and complete local put/get in under one minute on a
  supported platform.

### DEF-083 — Provide production distribution

Priority: P1

Work:

- Publish signed checksummed binaries for supported platforms.
- Publish crate artifacts with verified package contents.
- Add a minimal container image running as non-root.
- Provide systemd and Kubernetes examples with health probes, persistent
  volumes, security contexts, disruption budgets, and resource limits.
- Generate SBOMs and provenance attestations.

Acceptance:

- Installation and upgrade tests run from released artifacts, not the source
  tree.

### DEF-084 — Define compatibility and deprecation policy

Priority: P1

Work:

- Version SDK API, RPC protocol, wire format, configuration, CLI JSON, and
  examination schemas independently.
- State support windows and migration requirements.
- Add compatibility tests and deprecation diagnostics.

Acceptance:

- No “1.0” label exists without a written compatibility promise and test
  matrix.

---

## 15. Engineering quality and performance evidence

### DEF-090 — Make CI enforce the documented quality bar

Priority: P1

Status: **addressed** (2026-07-27) — required PR/push CI enforces fmt, strict
clippy, clean-tree, build/test all targets, docs, release-content package lists,
MSRV 1.88, OS matrix (ubuntu + macos), and cargo-deny; local mirror
`scripts/quality.sh`; expensive suites on `nightly.yml`.

Work:

- Fix current strict clippy failures in `dingo-format`.
- Add to required CI:
  - `cargo fmt --check`;
  - build/test all targets;
  - strict clippy;
  - documentation build and doctests;
  - package build;
  - dependency/license/advisory audit;
  - minimum supported Rust version;
  - supported OS matrix.
- Keep expensive corruption and chaos suites nightly, with release-blocking
  status.

Acceptance:

- Main cannot merge with a failed required gate.

Evidence:

- `.github/workflows/ci.yml` — `quality` (ubuntu/macos), `msrv`, `audit` jobs.
- `.github/workflows/nightly.yml` — corruption corpus, crash matrix, benches.
- `scripts/quality.sh` — local full bar including DEF-091 property tests.
- `deny.toml` — advisories / licenses / bans for `cargo deny check`.
- Workspace `rust-version = "1.88.0"` kept in sync with MSRV job.

Remaining (out of this cut):

- Branch-protection proof that all required checks are mandatory on the default
  branch (repo settings; not in-tree).
- Windows matrix (optional; exclusive-writer Windows lock path is a separate
  DEF-020 follow-on).

### DEF-091 — Add property testing and fuzzing

Priority: P0

Status: **partial — format cut addressed; continuous expansion (DEF-091-F)
shipped labor (2026-07-31)** — proptest + cargo-fuzz for format, SDA, RPC
framing, chunk manifest, item envelope, backup JSON, cursor tokens; scheduled
`fuzz_smoke` + `scripts/fuzz-smoke.sh` continuous policy. OSS-Fuzz / multi-hour
accumulation residual.

Targets:

- frame decode/verify and forward/reverse scanners;
- deterministic CBOR;
- segment descriptor/trailer;
- chunk manifest and reassembly;
- store/event envelopes;
- index/catalog/checkpoint decoders;
- SDA lexer/parser/evaluator;
- RPC and URL parsers;
- cluster/control metadata;
- salvage and migration manifests.

Acceptance:

- Fuzzers run continuously or on a scheduled service.
- Every discovered crash receives a minimized permanent regression corpus.

Evidence (this cut):

- `crates/dingo-format/tests/stage_def_091_properties.rs` — proptest in PR CI.
- `fuzz/` — cargo-fuzz package (not a workspace member) + README corpus policy.
- `.github/workflows/nightly.yml` job `fuzz_smoke` — 30s per target.
- `scripts/quality.sh` runs the property test binary.
- Hostile CBOR map/array length bound + regression
  `hostile_map_len_does_not_allocate_or_panic` (OOM class found by proptest).

Remaining / DEF-091-F residual after continuous expansion cut:

- OSS-Fuzz (or equivalent long-running hosted service) beyond nightly 30s smoke.
- Raft wire bodies / full migration job JSON as dedicated targets (backup
  manifest + RPC framing covered; cluster Raft residual).
- Accumulated multi-hour corpora on release gates.

### DEF-092 — Add coverage, sanitizers, and model checking

Priority: P1

Work:

- Publish line/branch coverage as a diagnostic, not a substitute for
  conformance.
- Run sanitizers/Miri where supported.
- Model durability-mode transitions, index/catalog frontiers, and compaction
  state machines.
- Model or formally review Raft membership and fencing.

Acceptance:

- Critical persistence and protocol state transitions have explicit model
  invariants.

### DEF-093 — Publish reproducible performance results

Priority: P1  
Dependencies: DEF-023, DEF-026, DEF-030

Required benchmark classes:

- embedded point read;
- append for memory/buffered/durable;
- network single-node;
- replicated quorum write/read;
- scan/filter/index;
- chunked payload;
- salvage and rebuild;
- compaction;
- warm/archive retrieval;
- repair and rebalance under foreground load.

Every report must disclose the fields in `doc/BENCHMARK_DISCLOSURE.md`,
including version, hardware, dataset, working set, payload distribution,
concurrency, durability, verification, replication, p50/p95/p99, throughput,
and recovery state.

Acceptance:

- README performance language links to reproducible commands and raw results.
- CI guards catastrophic regressions; stable release criteria use controlled
  hardware rather than noisy shared runners.

### DEF-094 — Establish release and incident processes

Priority: P1

Work:

- Define release checklist, rollback, support window, and emergency patch flow.
- Maintain changelog and migration notes.
- Add incident runbooks for disk full, corruption, quorum loss, certificate
  expiry, bad release, stalled repair, and lost control plane.
- Require restore and disaster-reconstruction drills.

Acceptance:

- A release candidate completes the full production qualification in §16.

### DEF-097 — Replace publicly derivable continuation-token keys

Priority: P0
Dependencies: DEF-032, DEF-054
Status: **partial — secret keyrings + v2 tokens shipped (labor in_review);
token-payload encryption, full Heap binding, and control-plane key
distribution beyond local cluster root remain residual**

Finding:

Both `dingo-store::cursor` and
`dingo-cluster::coverage::QueryContinuation` previously derived their keyed
BLAKE3 keys solely from the store ID or cluster ID, while that public ID is
included in the token. A client could therefore derive the same key and forge a
token. The tag detected accidental corruption and cross-store/cluster
use; it was not cryptographic authentication against a malicious client.

#### Labor shipped (dingo-token-key-v1 / cursor-v2 / query-continuation-v2)

- Store-local secret keyring (≥256-bit CSPRNG) at
  `store-info/cursor_token_keys.v1`; minted on create; load-or-mint on open.
- Cluster control-plane keyring at `{cluster_root}/cluster_token_keys.v1`.
- MAC domain separation includes secret + public id + generation id; public id
  alone is insufficient to forge.
- Wire profiles: `dingo-cursor-v2` (`DCSR0002`), `dingo-query-continuation-v2`
  (`DQRY0002`) embed `key_generation_id`.
- Rotation + one-generation grace + explicit retire; zeroize on drop; Debug
  redacts secrets.
- APIs: `Store::rotate_continuation_keys` / `retire_previous_continuation_key`,
  `Cluster::rotate_continuation_keys` / `continuation_keyring`.
- Tests: unit keyring/cursor, `stage_def_097_token_keys`, updated
  `stage_def_026_cursors` / `stage_def_040_query`.

#### Residual

- Encrypt token body when subject/metadata is sensitive.
- Richer binding (Heap authority gen, expiry, multi-gen grace policy config).
- Fleet key distribution beyond single cluster root file.
- Re-qualify marketing/spec “authenticated continuation” language against
  qualified profile (DIRECT_ACCESS_SPEC §19).

Acceptance (labor):

- Knowledge of store/cluster ID + token bytes is insufficient to forge a valid
  token without the secret keyring.
- Cross-store/cluster, bit-flip, and retired-generation tokens fail before
  scan/query execution.
- Rotation keeps previous gen during grace; retire invalidates it.
- Restart reloads retained generations from durable keyring files.
- Secret material is redacted from Debug and not returned from doctor fields.

### DEF-098 — Make chunked values generation-exact, bounded, and directly addressable

Priority: **P0**
Dependencies: none for correctness containment; DEF-022 for crash evidence;
DEF-029 for resource-profile integration; DEF-095 for locator-cache conventions
Status: **partial — generation-exact current get shipped (labor in_review); remaining:
bounded hot-path scan-free preads under all open paths, logical-value admission
ceiling, full crash/performance/application journey acceptance, transcript guidance**

Release impact:

- blocks embedded early access and every later production milestone;
- blocks any claim that an acknowledged large value remains ordinarily readable
  until actual damage is observed;
- requires immediate application guidance for append-heavy documents such as
  transcripts.

#### Finding

Large logical values are split into independently verified payload-chunk frames
and one item-event manifest. The current manifest correctly records, in order:

```text
content_hash
logical_total_len
chunk_count
for each logical slot:
    chunk_event_id
    logical_len
```

The current read path does not use that generation identity. It calls
`collect_chunk_pieces(item_id)`, scans every segment, and collects every
payload chunk with the subject's stable item-lineage ID. The item ID is retained
when the same key is replaced. Consequently, two chunked writes to the same key
may contribute different verified bodies for the same chunk index.

The generic reassembler correctly classifies those bodies as conflicting.
Ordinary `get` then collapses partial, unavailable, and conflicting results to
`PayloadPartial`. Thus a later, fully durable value can become unreadable even
when:

- every chunk of the current value is present and verifies;
- the current manifest is present and verifies;
- no storage byte was lost or corrupted; and
- the only additional evidence is an older valid version of the same key.

This is a correctness defect, not merely a poor large-document recommendation.
The manifest already contains the information required to select the correct
generation, but the read algorithm discards it.

The exact regression shape is:

```text
put(K, large A, durable) -> acknowledged
put(K, large B, durable) -> acknowledged
get(K)                   -> PayloadPartial/Conflicting
expected                 -> exactly B
```

The risk is especially high for transcripts, timelines, snapshots, and other
documents repeatedly replaced under one stable key after crossing the chunk
threshold.

#### Adjacent defects exposed by the same incident

##### A. No coherent logical-value size contract

Current effective boundaries differ by surface:

- direct `dingo-store` chunks values above the soft threshold without first
  applying one declared logical-value ceiling;
- the ordinary SDK applies a 16 MiB payload ceiling;
- remote use is also bounded by payload and negotiated RPC-frame limits;
- frame scanners default to a 16 MiB stored-body ceiling;
- a chunk manifest grows by 24 bytes per chunk in addition to its fixed
  header;
- chunk splitting, frame construction, and complete reassembly allocate memory
  proportional to the logical value.

“Limited by available disk” is therefore not a valid contract. In particular,
the low-level chunk writer uses the parts append path and can construct values
whose manifest or memory requirement is outside the normal scanner/resource
profile. A writer must never acknowledge an object that the same supported
profile cannot subsequently scan and reconstruct.

##### B. Chunked point reads are proportional to the store

`collect_chunk_pieces` reads every candidate segment file and scans every
verified frame for each chunked `get`. Its effective cost is:

```text
O(total physical segment bytes + logical payload bytes)
```

The required hot-path cost is:

```text
O(manifest chunk count + logical payload bytes)
```

This is a severe scale defect. A single transcript open must not rescan a
multi-terabyte store.

##### C. Ordinary errors erase the distinction between damage classes

The completeness-aware path distinguishes:

- partial;
- unavailable; and
- conflicting.

Ordinary `get` maps all three to `PayloadPartial`. This hides the difference
between missing media and contradictory verified evidence, impeding correct
repair, incident diagnosis, and application policy.

##### D. Byte survival is not structured-document survival

Arbitrary 16 KiB byte chunks are independently recoverable, but arbitrary
surviving fragments of a JSON encoding are not necessarily parseable JSON.
A monolithic transcript may therefore retain most physical bytes while losing
all ordinary document-level readability after one missing chunk.

This does not violate the byte-survival format rule: `get_payload` can expose
verified surviving extents. It does mean ResiduumDB must not imply that chunking
alone makes every field or array element independently examinable.

#### Immediate containment

Until this defect closes:

1. document that repeatedly replaced chunked values are unsafe for ordinary
   reads;
2. do not diagnose `PayloadPartial` as physical chunk loss without inspecting
   the completeness-aware result;
3. do not silently overwrite a partial/conflicting value with an empty or
   reconstructed default;
4. store append-heavy transcripts as independently addressed turns or bounded
   blocks rather than one ever-growing JSON document;
5. keep the large-value profile experimental and exclude it from performance
   claims; and
6. add telemetry counters for `partial`, `unavailable`, and `conflicting`
   without logging subjects, values, chunk bodies, or secret Heap material.

The 64 KiB threshold is a storage-layout switch, not a supported-document-size
promise. With the current strict comparison, an encoded body is chunked only
when `len > DEFAULT_CHUNK_THRESHOLD`; exactly 64 KiB remains inline under
defaults.

#### Normative invariants

For manifest `M`, let:

```text
M.slots[i] = (event_id_i, logical_len_i)
Current(M) = the verified chunk frame whose frame event ID is event_id_i
             and whose decoded slot is exactly i
```

A conforming read MUST satisfy:

```text
Selected(M) =
  [ Current(M)[0], Current(M)[1], ..., Current(M)[n-1] ]
```

It MUST NOT select a chunk merely because its `item_id`, subject, index, total,
segment, or content resembles a required chunk.

For every selected slot `i`, validate all of:

```text
frame.kind              = PayloadChunk
frame.event_id          = M.slots[i].event_id
piece.item_id           = current manifest item_id
piece.index             = i
piece.total             = M.slots.len
piece.logical_len       = M.slots[i].logical_len
piece.logical_len       = decoded payload length
```

For the manifest, validate:

```text
M.logical_total_len = sum(M.slots[*].logical_len)
M.slots.len         fits the declared chunk-count profile
encoded_len(M)      fits max_body_len
reassembled length  = M.logical_total_len
hash(reassembled)   = M.content_hash
```

Failure classification is exact:

| Evidence | Result |
|---|---|
| every exact required event verifies and full hash agrees | `Complete` |
| some exact required events verify and some are absent/unavailable | `Partial` |
| no exact required event is available | `Unavailable` |
| one required event ID has contradictory verified frames, or its decoded metadata disagrees with its slot | `Conflicting` |
| unrelated older/newer chunks exist | ignored for current read; retained for history/salvage |

No fallback may substitute a same-index chunk from another generation.

#### Exact implementation

##### 1. Make manifest membership part of the reassembly type

Replace the unqualified store call:

```text
collect_chunk_pieces(item_id)
```

with a manifest-qualified operation:

```text
resolve_manifest_chunks(current_item_id, manifest, read_budget)
```

The resolved input to reassembly must retain both identities:

```text
ResolvedChunk {
    frame_event_id
    segment_id
    frame_offset
    piece
    verified_body_hash
}
```

Change `reassemble_with_manifest` to accept resolved chunks containing frame
event IDs. It validates membership and slot metadata itself. Do not depend only
on callers to pre-filter correctly.

Build a bounded expected map from the manifest:

```text
chunk_event_id -> expected slot index and logical length
```

Reject duplicate event IDs in one manifest. While reading frames, ignore event
IDs absent from this map. For an expected event ID, preserve all physically
distinct verified candidates long enough to detect identical duplicates versus
conflicting evidence.

The existing on-disk manifest already contains event IDs, so the correctness
fix requires no wire-major or frame-format migration.

##### 2. Add a derived chunk locator

Introduce a rebuildable locator:

```text
ChunkEventId ->
    one or more {
        segment_id,
        media/tier identity,
        frame_offset,
        frame_len,
        item_id,
        chunk_index,
        chunk_total,
        logical_len,
        verified_body_hash
    }
```

Rules:

- build it during the same authoritative scan that rebuilds the primary index;
- update it after a chunk append becomes durably visible;
- fingerprint/fence persisted locator caches using the same source-generation
  discipline as other derived indexes;
- update or rebuild it after compaction, tier movement, restore, and salvage;
- retain multiple locators for physical duplicates;
- retain a conflict marker when one event ID has differing verified content;
- never make the locator authoritative;
- never infer absence from a stale or incomplete locator.

The ordinary read path:

1. resolves the current item-event manifest;
2. performs one locator lookup for each manifest event ID;
3. uses bounded frame preads for the located frames;
4. independently verifies every frame and slot;
5. returns the exact completeness class.

If the locator cannot prove coverage, the read returns explicit incomplete
coverage or uses a separately budgeted diagnostic rebuild. It MUST NOT perform
an unbounded full-store scan invisibly on the point-read hot path.

##### 3. Enforce logical-value admission before effect

Add `max_logical_payload_bytes` to the store/resource profile.

For the stable v1 application profile:

```text
default max logical encoded payload = 16 MiB
default chunk threshold             = 64 KiB
default chunk payload size          = 16 KiB
```

The logical ceiling includes the SDK JSON/bytes type tag. The same effective
ceiling applies to embedded Heap, ordinary SDK, qualified remote server, and
cluster admission. The effective limit is the minimum of client, server,
store, and negotiated transport policy and is reported in diagnostics.

Before minting event IDs, starting/rotating a segment, allocating all pieces,
or appending a frame:

1. checked-convert the logical length;
2. reject `logical_len > max_logical_payload_bytes`;
3. checked-compute `ceil(logical_len / chunk_size)`;
4. require a non-zero valid chunk size;
5. require the chunk count to fit the manifest and `u32`;
6. checked-compute manifest encoded length;
7. require every chunk frame and the manifest frame to satisfy the active
   `SafetyLimits`;
8. require estimated peak write/reassembly memory to fit the host operation
   budget; and
9. return `PayloadTooLarge`/`ResourceLimit` with zero durable effect on failure.

Low-level deployments that explicitly raise the 16 MiB default use a labelled
experimental large-value profile. The raised value is still bounded by all
checked manifest, frame, allocation, and address-space constraints. Raising an
RPC limit alone does not raise the store limit.

Existing readable values above a newly configured admission ceiling remain
readable and salvageable. The ceiling governs new/replacement writes. Opening
an existing store MUST NOT discard a large value merely because current write
policy is tighter.

##### 4. Preserve atomic publication and make it exhaustive

For `Durable`:

```text
append all exact chunk frames
append their manifest item event
write complete pending tail
fsync the containing durability domain
publish current primary-index event
return receipt
```

No durable success receipt is returned before every current-generation chunk
and the manifest cross the same documented durability boundary.

Recovery rules:

- chunks without a complete published manifest are orphan evidence, not a
  current value;
- a torn/absent unacknowledged manifest leaves the prior current value visible;
- a complete manifest with missing current-generation chunks is explicit
  partial evidence;
- old-generation chunks cannot complete a new manifest;
- a durable acknowledged rewrite reopens as exactly the new value.

`Buffered` and `Memory` retain their weaker documented failure boundaries and
must never be described as durable.

##### 5. Expose exact damage semantics

Keep the completeness-aware API as the recovery authority. Ordinary `get`
maps:

```text
Partial       -> PayloadPartial
Unavailable   -> PayloadPartial with completeness=unavailable
Conflicting   -> DataDamaged with damage_kind=payload_conflict
```

If changing the public error shape requires a compatibility phase, add
structured detail first and retain the old broad code temporarily. Telemetry,
doctor, scrub, and Residuum Studio show the exact class, missing slot count,
conflict slot, expected event IDs, and affected tier—never payload bytes.

Repair may fetch an exact missing `chunk_event_id` from an authenticated replica
or backup. It MUST NOT repair using “same item and index” alone.

##### 6. Define the structured-survival boundary

Documentation and APIs distinguish:

```text
physical chunk survival
    verified byte extents can survive independently

structured document usability
    requires a complete valid encoding unless a separate structured-segment
    profile defines independently decodable components
```

For transcripts and append-heavy timelines, the supported pattern is:

```text
transcript/{id}/meta
transcript/{id}/turn/{monotonic-id}
transcript/{id}/timeline/{bounded-block-id}
```

Each turn/block is an independently meaningful ResiduumDB value. Optional aggregate
snapshots are derived and replaceable. Losing one unit does not make surviving
units unqueryable.

A future structured-segment format may make top-level fields or array blocks
independently decodable, but it requires a normative format and must not be
invented inside this defect. Arbitrary byte chunks are not relabelled as
structured fragments.

#### Required tests

Correctness:

- inline → chunked replacement;
- chunked A → different chunked B under the same key;
- same-size, larger, and smaller chunked replacements;
- replacement where only one chunk changes;
- identical replacement and physical duplicate frames;
- chunked → inline → chunked history;
- many consecutive chunked generations;
- old conflicting chunks present while the current generation is complete;
- current event ID duplicated identically;
- current event ID duplicated with different verified content;
- wrong item ID, index, total, length, or event ID;
- malformed/duplicate manifest event IDs;
- full body present but content hash mismatch;
- missing first, middle, last, many, and all current chunks.

Crash/fault injection:

- before and after every chunk append;
- before and after manifest append;
- partial tail write in chunks and manifest;
- before and after flush/fsync;
- before and after primary-index publication;
- crash during replacement with a prior inline value;
- crash during replacement with a prior chunked value;
- reopen, index-cache deletion, full rebuild, compaction, and tier move after
  every acknowledged/unacknowledged outcome.

Limits:

- threshold minus one, exactly threshold, and threshold plus one;
- logical limit minus one, exactly limit, and limit plus one;
- manifest length exactly at and one byte beyond frame limits;
- zero/tiny/oversized chunk-size configuration;
- checked arithmetic near integer boundaries without allocating the claimed
  size;
- client/server/store limit mismatch uses the tightest value;
- rejection produces no chunk, manifest, receipt, catalog, or index effect;
- pre-existing above-policy values remain readable.

Performance and boundedness:

- point-read bytes examined are bounded by manifest plus referenced frames;
- point-read work is invariant when unrelated store size grows;
- no full segment `fs::read` occurs on the ordinary chunked-get path;
- peak memory stays within the declared operation budget;
- cold/offline tier locators produce honest partial coverage;
- locator deletion/corruption rebuilds without changing logical results;
- compaction and restore update/rebuild locators correctly.

Application journey:

- repeatedly append a long transcript using independent turn records;
- inject damage into one turn;
- all unaffected turns remain queryable and ordered;
- aggregate snapshot failure does not become transcript absence;
- no recovery path overwrites partial evidence with an empty document.

#### Acceptance

DEF-098 is accepted only when:

- the deterministic chunked-overwrite regression returns exactly the latest
  value before and after reopen;
- current-manifest event IDs are enforced by the reassembler, not merely by one
  caller;
- old generations cannot cause current partial/conflict results;
- every supported write surface reports and enforces one effective logical
  payload limit before effect;
- a supported writer cannot acknowledge a value outside its reader/scanner
  profile;
- ordinary chunked point reads use exact locators and do not scan the dataset;
- partial, unavailable, and conflicting evidence remain distinguishable;
- crash/rebuild/compaction/tier tests preserve the same result;
- transcript guidance and an executable survival journey are published;
- the chunked payload benchmark reports p50/p95/p99 latency, bytes read, frames
  verified, peak RSS, payload size/chunk count, durability, and unrelated
  dataset size; and
- the incident cause is recorded as generation conflict, real missing chunks,
  weaker durability, or another evidenced class—never guessed from the broad
  `PayloadPartial` error alone.

#### Incident evidence: rewrite-heavy desktop transcript

A real embedded desktop workload reported a force-quit followed by
`PayloadPartial`/`CoverageIncomplete` while opening a repeatedly replaced
200–500 KiB transcript. The key and metadata remained visible and application
code converted the read/list failure into an apparently empty UI.

This incident is a permanent DEF-098 regression input, but its physical cause
MUST remain `unclassified` until exact manifest event IDs, chunk observations,
durability mode, receipt outcome, and store bytes identify one of:

```text
cross-generation conflict
missing current-generation chunk
unavailable tier
torn or corrupt bytes
unacknowledged weaker-durability loss
another evidenced class
```

Force-quit timing and the broad `PayloadPartial` code are not causal proof.

---

### DEF-099 — Add exact historical-value reads and explicit last-complete recovery

Priority: **P1**
Dependencies: DEF-098 exact manifest resolution and chunk locator
Status: **partial — embedded historical get / last-complete shipped (labor in_review);
remote/cluster/CLI/Studio parity and full required-test matrix residual**

#### Finding

Per-key history exposes verified item events, but a historical chunked put is
currently projected as its manifest body rather than its reconstructed logical
value. An application faced with a partial current generation therefore has no
safe high-level way to:

- read one exact historical event;
- locate the newest earlier generation that is independently complete;
- distinguish a deliberately selected historical value from the current value;
  or
- preserve current damage evidence while exporting a usable prior version.

Applications will otherwise invent unsafe recovery by treating manifests as
documents, choosing physical encounter order, crossing tombstones, or writing
the recovered value over damaged evidence.

#### Normative rules

- Ordinary `get` remains current-generation and fail-closed. It MUST NOT
  silently fall back.
- Historical selection is by exact authoritative `event_id`, not array index,
  timestamp, segment order, or `item_id`.
- Chunked historical reconstruction uses only chunk event IDs named by that
  historical manifest.
- Returned results disclose current event, selected event, completeness,
  history gaps, and whether a tombstone boundary was crossed.
- A read-only recovery call never mutates, repairs, promotes, or hides current
  authority.
- “Last complete” is bounded and deterministic. Its default search stops at the
  first tombstone so a deleted value is never accidentally resurrected.
- Searching across a tombstone requires an explicit forensic option and the
  result is labelled non-current historical evidence.

#### Exact implementation

Store API:

```text
get_payload_version(subject, event_id, ReadBudget)
  -> VersionedPayloadResult

find_last_complete_version(subject, BeforeEvent, RecoveryReadOptions)
  -> HistoricalSearchResult
```

SDK/API projection:

```text
Collection::get_version(key, event_id)
Collection::find_last_complete(key, options)
```

`VersionedPayloadResult` contains:

```text
subject identity
selected event_id / item_id / segment_id
selected event kind
current event_id and current completeness
selected PayloadResult
known_gap_before / history_coverage_complete
tombstone_crossed
provenance and bytes examined
```

Implementation:

1. resolve the exact item event through history/event locators;
2. verify its frame independently;
3. return delete events as tombstones, never empty values;
4. for inline puts, return the verified logical body;
5. for chunked puts, decode that event's manifest and invoke the DEF-098
   manifest-qualified resolver;
6. enforce caller read budgets and return an explicit bounded-search result;
7. retain partial/unavailable/conflicting candidates in the search report;
8. never update the primary index or write recovered bytes; and
9. expose a separate, explicitly authorized restore/copy operation later if
   promotion is desired.

#### Required tests

- exact inline and chunked event reads across many replacements;
- current partial with previous complete;
- several previous partial/conflicting generations before a complete one;
- delete boundary before an older complete put;
- explicit forensic crossing of the delete boundary;
- duplicated physical event, conflicting same event ID, and history holes;
- unavailable tier and strict read-budget exhaustion;
- rebuild, compaction, backup/restore, and migration preserve selection;
- ordinary `get` still fails on the current partial and never auto-falls back;
- recovery calls leave every source byte and current index entry unchanged; and
- embedded, remote, cluster, CLI, and Studio projections preserve the same
  provenance.

#### Acceptance

- exact event reads reconstruct the same value as when that event was current;
- `find_last_complete` returns the newest provably complete permitted event;
- tombstones and history gaps are never silently crossed;
- no recovery read mutates or promotes authority;
- runtime work obeys the declared budget; and
- the transcript incident can display/export an evidenced prior value without
  representing it as the current value.

---

### DEF-100 — Make key enumeration and document scans coverage-aware

Priority: **P0**
Dependencies: DEF-012 coverage truth; DEF-026 cursor paging
Status: **partial — embedded key/document coverage pages shipped (labor in_review);
remote/cluster page parity and full required-test matrix residual**

#### Finding

`Collection::scan_keys` avoids payload reassembly, which is correct for a
partially available body. However, its plain `Vec<String>` result cannot state
whether unavailable/damaged authoritative regions may contain additional
keys. It can therefore present an incomplete set as a complete key list.

Conversely, `Collection::scan_json` and `scan_json_page` fail the entire
logical scan when one body's payload is incomplete. The lower store layer
already has partial-aware pages, but the ordinary Collection, remote, and
cluster surfaces do not expose the useful result.

The required distinction is:

```text
body incomplete, key event verified
    key remains listable; document appears in incomplete[]

key-bearing authority unavailable or damaged
    known keys remain listable; key coverage is explicitly incomplete
```

#### Normative rules

- Value-body damage MUST NOT suppress an independently verified key.
- Unknown key coverage MUST NOT be represented as a complete empty or partial
  vector.
- A malformed/partial document cannot abort enumeration of unrelated healthy
  documents in the partial-aware API.
- Legacy fail-closed methods may return `CoverageIncomplete`; they may not
  silently discard rows or gaps.
- Continuations bind store, Heap, collection, scan generation, prefix, page
  size, and coverage frontier.
- No API may infer the identity of a key whose key-bearing bytes were destroyed.

#### Exact implementation

Add common response types:

```text
KeyScanPage {
    keys,
    continuation,
    has_more,
    coverage_complete,
    coverage_gaps,
    examined
}

DocumentScanPage<T> {
    rows,
    incomplete: [{ key, completeness, detail }],
    undecodable: [{ key, error_class }],
    continuation,
    has_more,
    key_coverage_complete,
    coverage_gaps,
    examined,
    bytes_examined
}
```

Add:

```text
Store::scan_live_keys_page
Collection::scan_keys_page
Collection::scan_json_partial_page
remote list_keys_page / scan_json_partial_page
cluster parity for both operations
```

Rules for existing methods:

- `scan_keys()` drains `scan_keys_page` only when coverage is complete;
  otherwise it returns `CoverageIncomplete` with a resumable diagnostic.
- `scan_json()` remains fail-closed compatibility behavior.
- `scan_json_partial_page()` returns all healthy rows and per-key failures.
- neither method materializes the entire dataset internally;
- body resolution is never performed by key-only scanning; and
- all backends use the same response and error vocabulary.

Coverage gaps use stable typed causes and conservative authority ranges. They
do not leak Heap identifiers or subjects to unauthorized callers.

#### Required tests

- one missing chunk does not remove its key or unrelated documents;
- missing first/middle/last body chunks and conflicting chunks;
- corrupt JSON with a verified key;
- damaged key-bearing item event;
- unavailable tier containing zero, one, or many possible key events;
- corrupt/missing/stale primary index and collection catalog;
- empty collection with complete coverage versus zero known keys with
  incomplete coverage;
- pagination through pages containing only incomplete documents;
- cursor tamper, stale generation, cross-store/Heap/collection reuse;
- bounded work with long runs of incomplete entries;
- remote and cluster parity; and
- application regression: no error or incomplete result can become `[]`
  without an explicit caller policy.

#### Acceptance

- verified keys survive body failure;
- complete key lists are claimed only with complete key-bearing coverage;
- healthy documents remain streamable around damaged documents;
- all omissions appear in typed per-key or coverage evidence;
- legacy APIs fail closed; and
- the incident's sidebar can list the affected chat while showing its body as
  unavailable rather than making the database appear empty.

---

### DEF-101 — Define a truthful writer-lock acquisition and diagnostic contract

Priority: **P1**
Dependencies: DEF-020 exclusive writer ownership
Status: **partial — structured observation + open_with_options/try_open/doctor
lock-status shipped (labor in_review); multi-process SIGKILL soak residual**

#### Finding

The OS advisory lock is authoritative and the text in
`store-info/writer.lock` is diagnostic only. Process death releases the OS
lock; stale text cannot legitimately keep the store locked. Current errors do
not provide enough structured information to distinguish:

- another handle in the same process;
- a live external OS-lock holder;
- an acquisition race;
- an unsupported filesystem/locking failure; or
- stale diagnostic text while the OS lock is actually free.

Applications may consequently treat a retryable open failure as an empty store
or instruct operators to delete a harmless file.

#### Normative rules

- Only successful OS/in-process ownership establishes writer authority.
- Diagnostic PID/text is advisory and never grants, retains, or breaks a lock.
- ResiduumDB MUST NOT delete a lock file or kill a process to acquire ownership.
- Writer-lock failure is never database absence.
- Read-only inspection uses `open_inspect` and cannot mutate derived state.
- Waiting is bounded, cancellable, observable, and defaults to the existing
  non-blocking behavior unless explicitly requested.

#### Exact implementation

Add:

```text
OpenOptions {
    writer_lock_wait,
    writer_lock_poll_interval,
    cancellation
}

WriterLockObservation {
    class: in_process | external_holder | unsupported | io_failure
    diagnostic_pid
    diagnostic_pid_liveness: alive | dead | unknown
    diagnostic_acquired_time
    os_lock_authoritative
    waited
    retryable
}
```

Expose:

```text
Store::open_with_options
Store::try_open
Store::open_inspect
dingo doctor lock-status
```

On contention, reread advisory metadata for diagnostics but do not trust it.
PID liveness is reported as advisory because PID reuse and permissions make it
racy. Timeout returns structured `WriterLockHeld`, not `NotFound` or an empty
store. SDK/server errors preserve the class and retryability.

#### Required tests

- two handles in one process;
- live holder in another process;
- SIGKILL/abort/normal exit releases the OS lock;
- deliberately stale/dead/reused PID text with OS lock free;
- text naming a live PID that does not hold the OS lock;
- contention disappears during bounded wait;
- timeout and cancellation create no store effect;
- read-only inspect while a writer is live;
- unsupported/network filesystem behavior; and
- application regression forbidding `WriterLockHeld -> empty database`.

#### Acceptance

- stale diagnostics never block a free authoritative lock;
- a real holder cannot be bypassed;
- callers can choose immediate, bounded-wait, or inspect behavior;
- all failure classes remain distinguishable and retryable as appropriate; and
- operator guidance never recommends deleting `writer.lock` as a lock-breaking
  mechanism.

---

### DEF-102 — Explain and expose the derived-index/active-log lifecycle

Priority: **P2**
Dependencies: DEF-023 frontier cache; DEF-095 locator-only primary index
Status: **partial — Store + doctor diagnostics shipped (labor in_review); Studio UI residual**

#### Finding

A live store may contain megabytes in `active/`, empty `segments/` and
`chunks/`, and a very small `indexes/primary.idx`. This can be correct:
authoritative events remain in the active log while `primary.idx` is only a
derived, checksummed checkpoint/frontier cache.

The current product surface does not make that interpretation obvious.
Operators cannot readily distinguish a healthy minimal cache from a stale,
rejected, truncated, or insufficient cache.

#### Normative rules

- `primary.idx` is never authority.
- Its byte size alone is never a health signal.
- Missing, stale, foreign, unsupported, or corrupt cache state is rejected and
  rebuilt/replayed from authoritative coverage.
- Store diagnostics report both authority coverage and derived-cache status.
- “Index healthy” cannot mean only “file exists.”

#### Exact implementation

Extend `StoreInfo`, doctor output, and Studio with:

```text
primary_cache {
    present
    format_version
    byte_len
    validation: accepted | absent | stale | corrupt | foreign | unsupported
    sealed_fingerprint
    active_segment_id
    active_covered_len
    active_actual_len
    replay_bytes
    resident_entries
    resident_body_bytes
    authoritative: false
}

lifecycle {
    active_shards
    pending_seals
    sealed_segments
    checkpoint_reason
    last_checkpoint_time
}
```

Document create → active append → checkpoint/frontier → rotate/pending seal →
sealed segment → compaction, including which artifacts are authoritative and
which are disposable/rebuildable.

#### Required tests

- minimal/empty cache with a non-empty active log;
- cache absent, truncated, hash-corrupt, stale frontier, ahead frontier,
  foreign store ID, and unsupported version;
- active tail before/after the recorded frontier;
- delete all derived directories and obtain identical logical state;
- seal/async seal/compaction lifecycle transitions;
- doctor output matches independently measured files/frontiers; and
- no diagnostic state changes ordinary logical results.

#### Acceptance

- the observed 124-byte-cache scenario is classified unambiguously;
- every rejected cache reports why and what replay/rebuild occurred;
- deleting the cache is demonstrated as logically neutral;
- documentation prevents cache size from being interpreted as stored-data
  size; and
- diagnostics never elevate cache presence to authoritative health.

---

### DEF-103 — Define one large-value profile and safe rewrite-heavy workload contract

Priority: **P1**
Dependencies: DEF-029 resource profiles; DEF-098 logical-size admission
Status: **partial — LargeValuePolicy + admit-before-effect + rewrite_heavy helpers
shipped (labor in_review); cross-layer negotiation / full perf matrix residual**

#### Finding

The 64 KiB threshold and 16 KiB chunk size are storage-layout defaults, not a
document-size promise. Transcript, agent, timeline, and snapshot workloads
cross the threshold routinely and repeatedly replace the same logical key.
Configuration is currently test-oriented and the effective client/server/store
limits and layout choice are not obvious to application developers.

Simply raising the threshold is not a correctness solution: it trades more
chunks for larger individual frames, allocations, and damage blast radius.

#### Normative rules

- One versioned store/resource profile defines maximum logical payload, chunk
  threshold, chunk size, frame limits, memory budget, and transport limit.
- The effective write ceiling is the minimum across all participating layers.
- Admission occurs before event IDs, allocation, append, rotation, or derived
  effect.
- Threshold changes layout only; they do not weaken atomicity or verification.
- Existing above-policy values remain readable and salvageable.
- Defaults are changed only with crash, damage, memory, and performance
  evidence—not workload anecdotes.
- Rewrite-heavy compound documents receive a first-class recommended data
  model of independently meaningful records plus optional derived snapshots.

#### Exact implementation

Add a validated, persisted/profile-bound configuration:

```text
LargeValuePolicy {
    max_logical_payload_bytes
    chunk_threshold_bytes
    chunk_payload_bytes
    max_manifest_bytes
    max_reassembly_bytes
    max_write_peak_memory
}
```

Expose effective policy through store information, SDK connection information,
server negotiation, CLI, Studio, and write errors. `PutOptions` may request
stricter per-operation bounds but cannot raise the effective profile.

Write receipts/diagnostics expose non-secret layout facts:

```text
inline | chunked
logical bytes
chunk count
effective limit/profile id
```

Publish the supported transcript pattern:

```text
transcript/{id}/meta
transcript/{id}/turn/{monotonic-id}
transcript/{id}/timeline/{bounded-block-id}
transcript/{id}/snapshot/{generation}   # derived/rebuildable
```

#### Required tests

- every threshold and maximum at `-1`, exact, and `+1`;
- incompatible client/server/store/RPC limits choose the tightest;
- invalid zero/oversized chunk and manifest settings reject before effect;
- policy change does not make old data unreadable;
- inline versus chunked crash/damage matrices;
- peak allocation and bytes examined remain within policy;
- large repeated replacement benchmark with unrelated dataset growth;
- transcript survival journey with one damaged turn/block; and
- no configuration or telemetry leaks subjects or bodies.

#### Acceptance

- every write surface reports the same effective policy;
- over-limit rejection has zero authoritative or derived effect;
- profile values are inspectable before an application writes;
- the default threshold is retained or changed only by recorded evidence; and
- the recommended rewrite-heavy model is executable in SDK examples and
  qualification journeys.

---

### DEF-104 — Publish one executable crash-and-recovery contract

Priority: **P1**
Dependencies: DEF-098 through DEF-103
Status: **partial — contract page + executable journeys shipped (labor in_review);
remote/Studio/policy-negotiation residuals tracked under 098–103**

#### Finding

Chunk format, store behavior, scan completeness, history, lock ownership,
derived indexes, and application guidance currently live in separate sources.
An application developer cannot consult one normative page to determine:

- what a returned durability receipt proves;
- what may happen when termination occurs before a receipt;
- how current partial/conflicting evidence is represented;
- how to enumerate surviving keys and documents;
- how to select historical evidence safely;
- which files are authoritative or rebuildable; and
- which application reactions are forbidden.

Fragmented documentation is a safety defect when the predictable fallback is
`error -> []`, empty overwrite, guessed repair, or deletion of diagnostic files.

#### Exact implementation

Create one normative, versioned page:

```text
doc/CRASH_AND_RECOVERY_CONTRACT.md
contract id: dingo-crash-recovery-v1
```

It contains:

1. durability-mode acknowledgement table;
2. inline and chunked publication state diagrams;
3. old/new/unknown/partial/unavailable/conflicting decision table;
4. exact Collection and Store recovery APIs;
5. key coverage versus body completeness;
6. historical-version selection and tombstone rules;
7. writer-lock recovery and inspect pattern;
8. authority-versus-derived artifact map;
9. large/rewrite-heavy document modelling guidance;
10. operator decision tree and forbidden actions; and
11. capability limitations and assumption ledger.

Every example is compiled or executed in CI. The page links exact error codes,
receipts, response schemas, CLI commands, and the corresponding CSQ invariant
and suite IDs.

#### Required journeys

- durable put acknowledged then kill/reopen;
- kill at every unacknowledged chunk publication phase;
- current partial with explicit prior-version export;
- key list and partial document scan around one damaged value;
- force-killed writer followed by immediate/bounded/inspect reopen;
- missing/corrupt derived cache with authoritative replay;
- transcript stored as independent turns with one damaged unit; and
- deliberately unsafe sample reactions rejected by lint/test fixtures.

#### Acceptance

- no normative statement conflicts with implementation, DEF-098–103, or the
  core qualification profile;
- every code example runs against packaged artifacts;
- every outcome maps to a typed application decision;
- no example converts uncertainty/damage/lock failure to absence; and
- the original incident can be diagnosed and recovered using only this page
  and stable public APIs.

---

## 16. Production release gates

ResiduumDB may be called production-ready only when all applicable gates pass.

### 16.1 Data-safety gates

- [ ] Ambiguous remote retries produce exactly one authoritative event.
- [ ] Every acknowledged durable write survives the documented crash boundary.
- [ ] Every replicated acknowledgement proves configured quorum durability.
- [ ] Salvage preserves verified frames, partials, holes, conflicts, unsupported
      bytes, identities, and provenance.
- [ ] Ordinary reads and queries never convert incomplete coverage into absence.
- [ ] Derived state never gets ahead of authoritative durable state.
- [ ] Exclusive writer ownership is enforced.
- [ ] Replacing a chunked value selects only the exact chunk event IDs named by
      its current manifest; old generations cannot create false partials or
      conflicts (DEF-098).
- [ ] Every write surface rejects an over-limit logical payload before durable
      effect, and no supported writer can emit a value its reader profile
      rejects (DEF-098).
- [ ] Exact historical reads and last-complete recovery preserve current damage,
      tombstones, history gaps, and provenance without mutation (DEF-099).
- [ ] Key enumeration never represents incomplete key-bearing coverage as a
      complete list, and body damage does not hide a surviving key (DEF-100).

### 16.2 Single-node gates

- [ ] Crash matrix passes for create, append, chunk, delete, seal, compact,
      checkpoint, tier transfer, and metadata update.
- [ ] True bounded-memory scans pass datasets larger than RAM.
- [ ] Chunked point reads are proportional to the referenced payload, not total
      store size, and preserve exact partial/unavailable/conflicting evidence
      (DEF-098).
- [ ] Partial-aware Collection scans return healthy rows plus typed incomplete
      evidence with backend parity (DEF-100).
- [ ] Concurrent server load remains bounded and graceful under overload.
- [ ] Backup, restore, scrub, and format migration are exercised from released
      artifacts.

### 16.3 Distributed gates

- [x] Raft term, vote, log, commit, applied, snapshot, and membership state are
      durable for the **in-process** cluster (DEF-035; network multi-process
      durability still depends on DEF-036 RPC).
- [ ] Network replication passes every cluster conformance test.
- [ ] Minority partitions cannot commit strong writes.
- [ ] Old leaders and stale placements are fenced.
- [ ] Rebalance, repair, and coordinator restart are resumable.
- [ ] Linearizability and convergence histories pass independent checkers.
- [ ] Complete control-plane destruction and reconstruction retain honest
      commitment/uncertainty evidence.

### 16.4 Security gates

- [x] TLS is available outside loopback and mTLS protects peer traffic (DEF-032;
      plaintext non-loopback still requires explicit insecure override).
- [x] Authorization separates data, administration, salvage, and purge rights
      (DEF-033; full salvage/tier/purge engines still scaffolded).
- [x] Protocol admission control bounds rate, auth failures, connection churn,
      expensive ops, and operation-id replay (DEF-034).
- [ ] Every continuation token is authenticated by a non-public rotating
      secret or an equally strong issuer signature; public deployment/cluster
      identifiers are never token keys (DEF-097).
- [ ] Threat model and independent audit have no unresolved critical/high
      findings. *(threat-model first cut: `doc/THREAT_MODEL.md`; audit open)*
- [ ] Fuzzing covers all untrusted parsers. *(format first cut: DEF-091 properties
      + cargo-fuzz smoke; remaining surfaces open)*
- [x] Secrets and payloads are absent from **serve** structured logs by default
      (DEF-060; client logs follow-on).
- [x] Process metrics + liveness/readiness/detail health RPCs on serve
      (DEF-061 cut; store/cluster gauges + dashboards follow-on).

### 16.5 Operational gates

- [ ] Stable logs, metrics, traces, health, dashboards, and alerts exist.
- [ ] Writer-lock contention, timeout, stale diagnostics, and read-only inspect
      have structured, tested operator behavior (DEF-101).
- [ ] Store diagnostics distinguish authoritative active/log coverage from
      disposable primary-cache lifecycle state (DEF-102).
- [ ] Signed packages, SBOMs, containers, and deployment examples are tested.
- [ ] Rolling upgrade, rollback, backup restore, and disaster drills pass.
- [ ] SLOs and capacity limits are published for supported profiles.
- [ ] On-call runbooks cover every advertised failure mode.

### 16.6 Product and compatibility gates

- [ ] Every stable SDK/CLI capability is executable and tested.
- [ ] Backend parity suite passes embedded, server, and network cluster.
- [ ] Wire, RPC, SDK, config, and CLI JSON compatibility policies are published.
- [ ] Performance claims link to disclosed reproducible evidence.
- [ ] One effective large-value policy is inspectable and consistent across
      store, SDK, server, and transport surfaces (DEF-103).
- [ ] The executable crash-and-recovery contract matches packaged behavior
      (DEF-104).
- [ ] No release documentation overstates scaffolded capability.

## 17. Suggested milestone cut lines

### Milestone A — Truthful embedded early access

Required: DEF-001, DEF-003, DEF-010–014, DEF-020–026, DEF-029, DEF-050,
DEF-060, DEF-061, DEF-090–092, and DEF-098–104.

Qualification gate:
`dingo-core-storage-v1 / A2` under
[CORE_STORAGE_QUALIFICATION_SPEC.md](CORE_STORAGE_QUALIFICATION_SPEC.md).

Outcome: a supportable embedded store with explicit early-access limits.

### Milestone B — Production single-node server

Required: Milestone A plus DEF-030 through DEF-034, DEF-051, DEF-052, DEF-054,
DEF-063, DEF-080 through DEF-084, DEF-093, DEF-094.

Outcome: secure, observable, concurrent single-node service with backup and
upgrade support.

### Milestone C — Production local cluster

Required: Milestone B plus DEF-035 through DEF-041.

Outcome: independently running nodes with persistent consensus, real quorum
replication, repair, and verified failover.

### Milestone D — Production archive platform

Required: Milestone C plus DEF-070 through DEF-074.

Outcome: native cloud tiers, automated lifecycle/scrub, erasure protection,
encryption, and documented long-retention compatibility.

## 18. Tracking template

Create one issue per `DEF-NNN`. Each issue should use:

```text
Title:
Owner:
Priority:
Normative sections:
Dependencies:

Design:
Failure model:
Persistence/network boundaries:
Compatibility impact:
Observability:
Security considerations:

Implementation checklist:
Test/fault-injection checklist:
Benchmark checklist:
Documentation/runbook checklist:

Acceptance evidence:
Release gate(s) satisfied:
```

Do not close an issue with only unit tests or an API stub. Attach the exact
conformance, fault-injection, interoperability, and benchmark evidence required
by the task.