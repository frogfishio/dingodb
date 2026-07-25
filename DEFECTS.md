# DingoDB production-readiness defects and execution plan

Status: active remediation plan  
Scope: the complete DingoDB workspace  
Primary inputs: repository review performed 2026-07-25, `OVERVIEW.md`,
`FORMAT_SPEC.md`, `DX_SPEC.md`, `CLUSTER_SPEC.md`, and current implementation

## 1. Purpose

This document turns the current product-review findings into an ordered,
testable engineering program.

DingoDB already has a credible damage-tolerant format, salvage scanner,
single-node store, Rust SDK, and unusually strong conformance tests. It is not
yet production-ready as a network database or distributed storage system.
Production readiness requires more than implementing missing APIs: acknowledged
writes, recovery evidence, distributed commitment, operational security, and
query completeness must remain truthful through crashes, retries, upgrades,
partial outages, and operator mistakes.

The work below is complete only when the release gates in §16 pass. A task is
not complete merely because an API or type exists.

## 2. Current deployment classification

Until this plan is complete, use these support labels:

- **Embedded single-node:** experimental/early-access.
- **Single-node TCP server:** development only.
- **In-process cluster:** deterministic integration-test harness.
- **`serve-cluster`:** routing and endpoint-advertisement prototype; it does
  not provide network quorum replication.
- **S3/GCS:** filesystem-mirror integration, not a native cloud backend.
- **Erasure coding and lifecycle automation:** scaffolds only.
- **Wire format:** `1.0-draft`; not frozen for long-term interoperability.

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
  DEF-030 through DEF-038 are complete.

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
- Full cache rewrite is rate-limited (and forced on seal / explicit persist);
  catalog refresh uses the durable projection without a segment rescan.
- Wipe of `indexes/` / `catalogs/` / `snapshots/` still rebuilds identical
  logical state from segments.

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
  and page_size; MAC'd with a key derived from `store_id` (tamper / cross-store
  → `StoreError::CursorInvalid`).
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

Work:

- Either implement filter-to-SDA compilation as specified or amend the spec to
  define an independent but equivalent filter evaluator.
- Build a shared semantic corpus for absence, `Null`, numbers, ordering,
  containment, and failures.
- Run every portable filter through both paths and compare results.
- Version serialized query plans.

Acceptance:

- No semantic divergence exists in the shared vocabulary.
- Embedded, remote, cluster, indexed, and scan execution pass the same corpus.

### DEF-029 — Add resource governance

Priority: P1

Work:

- Enforce configurable limits for request bytes, JSON depth, frame lengths,
  scan bytes, decoded objects, sort memory, concurrent queries, open
  connections, and per-tenant work.
- Spill deterministic sorts only through a documented verified temp format.
- Return typed `ResourceLimit`/`QueryBudgetRequired` errors with partial
  coverage.
- Add cancellation propagation through storage and network loops.

Acceptance:

- Adversarial requests cannot cause unbounded memory, CPU, file descriptors, or
  disk growth.
- Limits are observable and tested at boundary values.

---

## 9. Production server and wire protocol

### DEF-030 — Replace the sequential TCP loop with a bounded server architecture

Priority: P1  
Dependencies: DEF-020

Work:

- Keep one coordinated store owner per store path.
- Use a bounded worker/runtime model for connections and read-only work.
- Serialize or shard mutations through explicit writer ownership.
- Add connection limits, idle timeouts, graceful shutdown, backpressure, and
  overload responses.
- Avoid holding a connection open in the accept loop.
- Add request cancellation and server draining.

Acceptance:

- One slow client cannot block unrelated clients.
- Load tests prove bounded memory and stable tail latency under overload.
- Graceful shutdown either completes or reports the outcome of every accepted
  mutation.

### DEF-031 — Version and frame the network protocol

Priority: P1

Work:

- Replace implicit line-delimited JSON compatibility with an explicit
  handshake and protocol version.
- Add maximum message lengths before allocation.
- Define feature negotiation and required receipt fields.
- Separate transport framing from application encoding.
- Preserve a human-debuggable mode only as a diagnostic profile.
- Add compatibility fixtures for supported versions.

Acceptance:

- Old/new clients fail clearly or negotiate a documented compatible subset.
- Oversized and malformed frames are rejected without unbounded allocation.
- Golden protocol fixtures run in CI.

### DEF-032 — Add TLS and authenticated peer identity

Priority: P0

Work:

- Support TLS 1.3 for client/server traffic.
- Support mTLS for node-to-node traffic.
- Verify hostname/service identity and cluster/node IDs.
- Define certificate reload and rotation without downtime.
- Remove credentials from request bodies and logs.
- Use constant-time secret comparison for any retained token mode.
- Make plaintext a loopback-only development profile.

Acceptance:

- MITM, wrong-host, expired, revoked, and wrong-cluster certificates fail.
- Rotation tests keep healthy connections available.
- Security scans confirm secrets are not logged.

### DEF-033 — Implement authorization and audit

Priority: P1

Work:

- Separate authentication from permissions.
- Define database, collection, operation, administration, salvage, tier, and
  purge privileges.
- Make purge and force-reconfiguration high-friction, separately authorized
  operations.
- Write tamper-evident audit records for security- and recovery-sensitive
  actions.
- Bound audit labels and redact payloads/secrets.

Acceptance:

- A writer cannot administer, salvage, move tiers, or purge without explicit
  permission.
- Denied operations are tested and audited without exposing secrets.

### DEF-034 — Add protocol admission control

Priority: P1

Work:

- Add per-principal and global rate limits.
- Bound authentication failures and connection churn.
- Protect expensive scan/index/doctor operations with budgets.
- Add replay windows where credentials or signed requests require them.

Acceptance:

- Load and abuse tests show bounded resource use and useful overload errors.

---

## 10. Persistent distributed system

### DEF-035 — Persist Raft state correctly

Priority: P0  
Dependencies: DEF-021, DEF-022

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

Acceptance:

- Raft safety tests cover crash/restart at every persistence boundary.
- Jepsen-style histories show no lost acknowledged writes, split-brain
  commitments, or stale-leader acceptance.
- Consensus evidence can distinguish committed, prepared, conflicting, and
  unknown-commit frames after disaster.

### DEF-036 — Implement real network Raft RPC

Priority: P0  
Dependencies: DEF-031, DEF-032, DEF-035

Work:

- Implement authenticated RequestVote, AppendEntries, snapshot install, and
  leadership/read-index RPCs.
- Use bounded batching, flow control, retry, and per-peer backoff.
- Fence writes by term, membership, and placement epoch.
- Never treat endpoint routing data as authority to write.
- Separate control-plane and data-plane availability.

Acceptance:

- Three independent processes on separate storage roots provide quorum
  durability.
- Minority partitions cannot commit strong writes.
- Old leaders cannot write after a new term.
- Response loss and retries preserve one event identity.

### DEF-037 — Make cluster SDK requests actually use cluster commitment

Priority: P0  
Dependencies: DEF-036

Work:

- Route network writes through the partition leader's Raft proposal path.
- Return a replicated acknowledgement only after the configured durability and
  quorum conditions are proven.
- Implement linearizable reads using leader/read-index or documented quorum
  reads.
- Refresh stale routes on typed epoch/term errors, not only transport strings.
- Preserve operation IDs across redirects.
- Verify every endpoint belongs to the expected cluster.

Acceptance:

- Network and in-process cluster conformance suites share the same logical
  tests.
- Killing the contacted node after commit does not lose an acknowledged write.
- A routing-only acknowledgement is never labeled replicated.

### DEF-038 — Persist control-plane and rebalance workflows

Priority: P0  
Dependencies: DEF-035

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

Acceptance:

- Restart at every rebalance phase leaves old placement authoritative or a
  valid joint configuration.
- Loss of the coordinator does not lose the operation state.
- Missing nodes are visible in health, coverage, and operator output.

### DEF-039 — Implement anti-entropy and replica repair

Priority: P1  
Dependencies: DEF-036

Work:

- Exchange verified hierarchical inventories by partition, segment, region
  hash, log frontier, and chunk set.
- Detect missing, divergent, corrupt, and conflicting replicas.
- Select repair sources using integrity and consensus evidence, never mtime.
- Copy exact frames where possible and verify destination before placement
  activation.
- Preserve conflicts and audit every repair.
- Rate-limit repair and isolate it from foreground latency.

Acceptance:

- Corrupt/newer-mtime replicas never overwrite healthy evidence.
- Random deletion/corruption converges to policy while preserving explicit
  irrecoverable holes.

### DEF-040 — Complete distributed query semantics

Priority: P1  
Dependencies: DEF-026, DEF-037

Work:

- Attach coverage to every distributed page.
- Define deterministic merge order independent of worker completion.
- Preserve per-partition frontiers and read modes.
- Resume coordinator failure using authenticated continuation state.
- Carry index/tier/resource limitations end to end.

Acceptance:

- Randomized worker ordering produces identical sequence results.
- Coordinator failover neither silently duplicates nor omits rows.
- Partial partitions are never represented as empty complete partitions.

### DEF-041 — Build a distributed-system verification program

Priority: P0

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

---

## 11. Backup, restore, upgrades, and fleet operations

### DEF-050 — Productize backup and restore

Priority: P1

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

### DEF-051 — Add integrity scrub and media-health automation

Priority: P1

Work:

- Implement bounded background frame/segment/chunk verification.
- Persist scrub frontier and findings.
- Integrate repair when redundancy exists.
- Quarantine corrupt evidence without hiding it.
- Expose scrub age, bytes verified, failures, and coverage.

Acceptance:

- Injected corruption is detected within the configured scrub interval.
- Scrub can pause/resume and does not starve foreground work.

### DEF-052 — Define format and protocol migration

Priority: P0

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

### DEF-053 — Freeze wire major 1 only after qualification

Priority: P0  
Dependencies: DEF-010 through DEF-014, DEF-022, DEF-052

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

### DEF-054 — Provide safe configuration management

Priority: P1

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

---

## 12. Observability and security operations

### DEF-060 — Implement structured logging

Priority: P1

Work:

- Use a structured logging facade throughout.
- Include operation, store, cluster, partition, event/operation ID, error code,
  requested/achieved guarantee, and latency where applicable.
- Redact credentials and payloads by default.
- Add stable event names and bounded field cardinality.

Acceptance:

- Every failed guarantee can be correlated across client, server, and replica
  logs without parsing prose.

### DEF-061 — Implement metrics and health endpoints

Priority: P1

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

Work:

- Threat-model local files, RPC, cluster membership, salvage, archive media,
  supply chain, admin operations, and malicious stored data.
- Review parsers and recovery tools as hostile-input surfaces.
- Commission an independent security audit and remediate findings.
- Establish vulnerability disclosure and supported-version policy.

Acceptance:

- No unresolved critical/high security finding remains at release.
- Fuzz targets cover every untrusted decoder and protocol parser.

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

### DEF-091 — Add property testing and fuzzing

Priority: P0

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

---

## 16. Production release gates

DingoDB may be called production-ready only when all applicable gates pass.

### 16.1 Data-safety gates

- [ ] Ambiguous remote retries produce exactly one authoritative event.
- [ ] Every acknowledged durable write survives the documented crash boundary.
- [ ] Every replicated acknowledgement proves configured quorum durability.
- [ ] Salvage preserves verified frames, partials, holes, conflicts, unsupported
      bytes, identities, and provenance.
- [ ] Ordinary reads and queries never convert incomplete coverage into absence.
- [ ] Derived state never gets ahead of authoritative durable state.
- [ ] Exclusive writer ownership is enforced.

### 16.2 Single-node gates

- [ ] Crash matrix passes for create, append, chunk, delete, seal, compact,
      checkpoint, tier transfer, and metadata update.
- [ ] True bounded-memory scans pass datasets larger than RAM.
- [ ] Concurrent server load remains bounded and graceful under overload.
- [ ] Backup, restore, scrub, and format migration are exercised from released
      artifacts.

### 16.3 Distributed gates

- [ ] Raft term, vote, log, commit, applied, snapshot, and membership state are
      durable.
- [ ] Network replication passes every cluster conformance test.
- [ ] Minority partitions cannot commit strong writes.
- [ ] Old leaders and stale placements are fenced.
- [ ] Rebalance, repair, and coordinator restart are resumable.
- [ ] Linearizability and convergence histories pass independent checkers.
- [ ] Complete control-plane destruction and reconstruction retain honest
      commitment/uncertainty evidence.

### 16.4 Security gates

- [ ] TLS is mandatory outside loopback and mTLS protects peer traffic.
- [ ] Authorization separates data, administration, salvage, and purge rights.
- [ ] Threat model and independent audit have no unresolved critical/high
      findings.
- [ ] Fuzzing covers all untrusted parsers.
- [ ] Secrets and payloads are absent from logs and metrics by default.

### 16.5 Operational gates

- [ ] Stable logs, metrics, traces, health, dashboards, and alerts exist.
- [ ] Signed packages, SBOMs, containers, and deployment examples are tested.
- [ ] Rolling upgrade, rollback, backup restore, and disaster drills pass.
- [ ] SLOs and capacity limits are published for supported profiles.
- [ ] On-call runbooks cover every advertised failure mode.

### 16.6 Product and compatibility gates

- [ ] Every stable SDK/CLI capability is executable and tested.
- [ ] Backend parity suite passes embedded, server, and network cluster.
- [ ] Wire, RPC, SDK, config, and CLI JSON compatibility policies are published.
- [ ] Performance claims link to disclosed reproducible evidence.
- [ ] No release documentation overstates scaffolded capability.

## 17. Suggested milestone cut lines

### Milestone A — Truthful embedded early access

Required: DEF-001, DEF-003, DEF-010–014, DEF-020–026, DEF-029, DEF-050,
DEF-060, DEF-061, and DEF-090–092.

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
