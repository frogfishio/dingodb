# ResiduumDB Core Storage Qualification Specification

Status: **normative design v1.0-draft — developer ready**

Profile identifier: `dingo-core-storage-v1`

Program identifier: `CSQ`

Audience: format, store, recovery, verification, release, and incident
engineering

Normative companions:

- [FORMAT_SPEC.md](FORMAT_SPEC.md)
- [OVERVIEW.md](OVERVIEW.md)
- [TESTING_STRATEGY.md](TESTING_STRATEGY.md)
- [DEFECTS.md](DEFECTS.md)
- [doc/VERIFICATION_IMPLEMENTATION_PLAN.md](doc/VERIFICATION_IMPLEMENTATION_PLAN.md)
- [doc/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md](doc/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md)

## 1. Decision

ResiduumDB's core proposition depends on one fact:

> Authoritative storage must tell the truth under ordinary operation, software
> defects, interrupted persistence, resource exhaustion, media damage, and
> recovery.

This specification defines the mandatory qualification suite for that fact.
It is the release authority for the format and embedded-store profiles. Heap,
server, query, cluster, and archive qualification build on it and cannot
compensate for its failure.

The suite does not claim that an unbounded software/hardware universe can be
exhaustively tested. It requires:

1. exhaustive enumeration of every declared finite domain;
2. exhaustive injection at every registered persistence/publication boundary;
3. systematic coverage of every declared failure class;
4. independent model, format, and recovery oracles;
5. generated transition histories rather than isolated feature examples;
6. mutation evidence that proves the suite detects broken invariants;
7. reproducible scale, soak, and real-filesystem evidence; and
8. a machine-verifiable evidence bundle bound to the released artifacts.

No test count, line-coverage percentage, proof count, fuzz duration, or soak
duration can substitute for a missing invariant/failure/oracle cell.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

Qualification states are:

```text
pass | fail | not_run | infrastructure_failure
```

Only `pass` satisfies a gate. Retry-to-green does not erase earlier attempts.

## 3. Scope

### 3.1 Included authority

`dingo-core-storage-v1` covers:

- frame and segment encoding, verification, forward/reverse scanning;
- subjects, event identity, logical lineage, current state, and history;
- inline and chunked payload publication/reassembly;
- durable, buffered, and memory acknowledgement boundaries;
- active/sealed segments and writer ownership;
- primary projection and every rebuildable store cache/catalog;
- checkpoints, secondary/Hydra/Chimera/chunk-locator derived structures;
- sealing, compaction, reclamation, tier movement, and coverage;
- backup, restore, scrub, salvage, export, and migration;
- resource admission and bounded progress;
- reopen, repeated recovery, and damaged-state operation; and
- compatibility with every format version claimed readable.

### 3.2 Excluded authority

This profile does not by itself qualify:

- Heap cryptographic admission or cross-Heap security;
- network protocols, TLS, remote retries, or server admission;
- distributed consensus, replication, or repair;
- RQL/RRE/Atomic semantics;
- native cloud object stores not used by the selected store profile; or
- application-specific structured recovery from arbitrary partial encodings.

Those profiles may reuse this evidence but require their own claims.

## 4. Qualification claim

A release may claim `dingo-core-storage-v1 / A2` only when:

> For every operation and failure class inside the registered profile, the
> implementation returns the exact acknowledged state, a permitted prior/new
> crash outcome, or explicit bounded uncertainty derived from surviving
> evidence. It never fabricates commitment, silently loses a durable
> acknowledgement, converts damage into absence, mixes logical identities or
> generations, or allows derived state to overrule authority.

Production-candidate `A3` additionally requires the platform/filesystem matrix,
compatibility matrix, mutation threshold, sustained campaigns, packaged
artifact journey, and release evidence in this specification.

## 5. Closed-world registries

The suite is controlled by checked-in registries:

```text
spec/verification/core-storage/claims-v1.json
spec/verification/core-storage/profiles-v1.json
spec/verification/core-storage/invariants-v1.json
spec/verification/core-storage/operations-v1.json
spec/verification/core-storage/boundaries-v1.json
spec/verification/core-storage/failures-v1.json
spec/verification/core-storage/failure-combinations-v1.json
spec/verification/core-storage/assumptions-v1.json
spec/verification/core-storage/oracles-v1.json
spec/verification/core-storage/proofs-v1.json
spec/verification/core-storage/outcomes-v1.json
spec/verification/core-storage/projections-v1.json
spec/verification/core-storage/compositions-v1.json
spec/verification/core-storage/incidents-v1.json
spec/verification/core-storage/suites-v1.json
spec/verification/core-storage/platforms-v1.json
spec/verification/core-storage/mutations-v1.json
spec/verification/core-storage/report-v1.schema.json
spec/verification/core-storage/vectors/
```

Every authoritative or derived operation has:

```text
operation_id
authority_inputs
preconditions
logical transition
durability modes
persistence steps
publication point
registered boundaries[]
permitted crash outcomes
invariants[]
oracles[]
failure classes[]
suites[]
resource bounds
```

The registries are closed:

- an implementation operation absent from the registry fails architecture CI;
- a registered operation without an implementation mapping fails CI;
- an authoritative write/sync/rename/remove/publication edge without a boundary
  entry fails CI;
- a boundary without an executable failpoint or justified external harness
  fails CI;
- an invariant without an independent oracle and mandatory suite fails CI;
- an error variant without a generated observation case fails CI; and
- a release claim with any unresolved registry cell fails qualification.

Adding a storage operation therefore necessarily adds its verification
obligations in the same change.

These files are a namespaced specialization of the general `VFY-0` registries,
not a competing evidence system. `claims-v1.json`, `suites-v1.json`, and
`profiles-v1.json` import/reference the core-storage records; the common
`dingo-verification-report-v1` envelope carries the
`dingo-core-storage-report-v1` attachment. One runner and one qualification
truth remain authoritative.

## 6. Independent oracle architecture

### 6.1 Three-oracle rule

P0 storage claims require three structurally different observations:

1. **Sequential logical model** — a small event/state machine that knows no
   segment, cache, production scanner, or production recovery algorithm.
2. **Independent format/recovery reader** — reads bytes without importing
   `residuum-store`; it does not call production frame scan or index code.
3. **Production implementation** — live, reopen, rebuild, inspect, salvage,
   compact, backup/restore, and migration observations.

All three must agree where the failure model provides enough evidence.
Disagreement is a failing minimized artifact, never resolved by majority vote.

### 6.2 Dependency firewall

The reference model MUST NOT depend on:

- `residuum-store`;
- store index, chunk, compaction, recovery, or catalog modules;
- production test helpers that calculate expected state; or
- production code hidden behind a test feature.

The independent byte reader MUST NOT call the production decoder/scanner.
It MAY use audited cryptographic primitives and standard serialization
libraries. CI enforces the dependency firewall.

### 6.3 Model state

The sequential model records:

```text
ModelStore {
    store_identity
    accepted_configuration
    authoritative_events_by_subject
    current_generation_by_subject
    receipts_by_operation_id
    acknowledged_durability
    known_damage
    unavailable_coverage
    logical_history
}
```

Derived catalogs, indexes, checkpoints, locators, summaries, and placement
hints are deliberately absent from model authority.

### 6.4 Observation

After every generated step, compare:

```text
point get / typed get / completeness-aware get
current key set and logical values
history ordering, identity, tombstones, and hole evidence
receipts and idempotent replay
scan pages and complete/incomplete coverage
catalog and index results versus authoritative scan
reopen and rebuild results
salvage/examination units and provenance
backup/restore projection
errors and uncertainty classification
```

Physical offsets and random identifiers are compared by invariant/relationship,
not by assuming the model can predict randomness.

## 7. Core invariant registry

Every invariant below is mandatory unless a narrower profile explicitly marks
it not applicable.

### 7.1 Identity and non-interference

| ID | Invariant |
|---|---|
| `CSQ-ID-001` | Store identity is stable across reopen, seal, compact, backup inspection, and derived rebuild. |
| `CSQ-ID-002` | Event IDs identify one logical event; identical physical duplicates do not double-apply. |
| `CSQ-ID-003` | Conflicting verified frames sharing an event ID remain conflicting; encounter order never chooses truth. |
| `CSQ-ID-004` | Item lineage is stable for one subject and cannot merge two subjects. |
| `CSQ-ID-005` | Activity on subject A cannot change the logical observation of subject B except declared global resource/maintenance evidence. |
| `CSQ-ID-006` | Binary SubjectV2 identities remain byte-exact; UTF-8 convenience paths cannot alias them. |
| `CSQ-ID-007` | Writer-shard routing is deterministic and does not change logical identity. |
| `CSQ-ID-008` | One process holds exclusive writer authority; rejected contenders create no durable effect. |
| `CSQ-ID-009` | Diagnostic lock-file contents never grant, retain, or break writer authority; OS/in-process ownership remains decisive. |

### 7.2 Acknowledgement and publication

| ID | Invariant |
|---|---|
| `CSQ-ACK-001` | A returned durable receipt implies the complete event crossed its declared stable-storage boundary. |
| `CSQ-ACK-002` | Every returned durable receipt is observable after crash/reopen on the qualified filesystem model. |
| `CSQ-ACK-003` | An operation without a returned receipt may resolve only to a registered old/new/unknown outcome; never a hybrid event. |
| `CSQ-ACK-004` | Buffered and memory acknowledgements never acquire stronger labels after reopen or logging. |
| `CSQ-ACK-005` | A receipt describes the actual event, item, store, segment, operation identity, and achieved durability. |
| `CSQ-ACK-006` | Exact mutation retry returns one logical effect and the original receipt. |
| `CSQ-ACK-007` | Reusing an operation ID with different content cannot mutate authority. |
| `CSQ-PUB-001` | Visibility is published only after the authoritative bytes required by the selected durability mode. |
| `CSQ-PUB-002` | A failed derived-cache/catalog update cannot roll back or conceal an authoritative event. |
| `CSQ-PUB-003` | Directory-entry creation, rename, replacement, and removal obey registered directory-sync assumptions. |

### 7.3 Current value, generations, and history

| ID | Invariant |
|---|---|
| `CSQ-GEN-001` | Current state is selected by authoritative event order, not physical encounter order. |
| `CSQ-GEN-002` | A value is reconstructed only from evidence named by its current generation. |
| `CSQ-GEN-003` | Older/newer orphan evidence cannot complete, damage, or conflict with another generation. |
| `CSQ-GEN-004` | Put→put, put→delete, delete→put, inline→chunked, and chunked→inline transitions yield exactly the latest committed state. |
| `CSQ-HIST-001` | History preserves every verified event, its order evidence, identity, and kind. |
| `CSQ-HIST-002` | Tombstones establish logical absence only at their valid position; missing data never becomes a tombstone. |
| `CSQ-HIST-003` | Known gaps before/between history events remain explicit. |
| `CSQ-HIST-004` | Compaction may change physical representation but cannot invent, reorder, or silently discard history promised by its profile. |
| `CSQ-HIST-005` | An exact historical-value read resolves only the requested event and labels it non-current when applicable. |
| `CSQ-HIST-006` | Last-complete recovery is bounded, preserves partial candidates and gaps, and never crosses a tombstone without explicit forensic policy. |
| `CSQ-ABS-001` | `None` means authoritative absence; damage, unavailability, unsupported format, resource stop, and conflict are not absence. |
| `CSQ-ABS-002` | Key/document scans claim completeness only when key-bearing authority coverage is complete; body damage cannot hide an independently surviving key. |

### 7.4 Frames, chunks, and structured damage

| ID | Invariant |
|---|---|
| `CSQ-FMT-001` | A frame is verified only when every structural, length, version, envelope, suffix, and integrity condition succeeds. |
| `CSQ-FMT-002` | One damaged region cannot prevent discovery of later independently valid frames promised by the scan profile. |
| `CSQ-FMT-003` | Forward and reverse evidence reconcile without fabricating byte extents. |
| `CSQ-FMT-004` | Unsupported versions/kinds survive as unsupported evidence, not corruption or valid known data. |
| `CSQ-FMT-005` | All parsing terminates within declared time, depth, allocation, and progress bounds. |
| `CSQ-CHK-001` | A manifest is internally consistent and binds exact chunk event IDs, order, lengths, count, total length, and content hash. |
| `CSQ-CHK-002` | Complete requires every exact current-generation chunk and matching full hash. |
| `CSQ-CHK-003` | Partial/unavailable/conflicting states preserve exact surviving extents and causes. |
| `CSQ-CHK-004` | Same-lineage chunks from another generation are ignored for current reassembly. |
| `CSQ-CHK-005` | Chunk physical duplicates deduplicate only when their verified content agrees. |
| `CSQ-CHK-006` | A durable chunked acknowledgement covers all chunks and the manifest. |
| `CSQ-CHK-007` | Chunk lookup cost is bounded by manifest/referenced bytes, not total dataset size. |

### 7.5 Damage, recovery, and deterministic truth

| ID | Invariant |
|---|---|
| `CSQ-DMG-001` | Corrupt bytes are never returned as verified payload. |
| `CSQ-DMG-002` | Damage outside an authority unit cannot change the decoded meaning of a healthy unit. |
| `CSQ-DMG-003` | Holes have conservative ranges, causes, source identity, and provenance; exactness is never guessed. |
| `CSQ-DMG-004` | Multiple holes, garbage islands, duplicates, reorder, truncation, and stale copies do not cause nontermination or fabrication. |
| `CSQ-DMG-005` | Encryption/key unavailability is a semantic hole distinct from absent ciphertext. |
| `CSQ-REC-001` | Reopening identical bytes under identical declared availability produces identical logical conclusions. |
| `CSQ-REC-002` | Repeating reopen/rebuild/salvage is idempotent. |
| `CSQ-REC-003` | Live, clean reopen, cacheless rebuild, and independent recovery agree on authoritative observations. |
| `CSQ-REC-004` | Recovery never writes into the source unless an explicit separately tested repair operation was requested. |
| `CSQ-REC-005` | Healthy work can continue after recoverable damage without erasing the damage evidence. |

### 7.6 Derived state and maintenance

| ID | Invariant |
|---|---|
| `CSQ-DER-001` | Primary caches, catalogs, summaries, checkpoints, secondary indexes, Hydra, Chimera, and chunk locators are derived. |
| `CSQ-DER-002` | Missing, stale, corrupt, truncated, ahead, or foreign derived state is rejected/rebuilt and cannot change authority. |
| `CSQ-DER-003` | Rebuild from the same authoritative coverage is deterministic. |
| `CSQ-DER-004` | An accelerator result equals the slow complete authoritative path or reports incomplete coverage. |
| `CSQ-MNT-001` | Seal preserves every authoritative event and makes duplicates harmless. |
| `CSQ-MNT-002` | Compaction output is verified before activation; source authority remains until safe reclamation. |
| `CSQ-MNT-003` | Interrupted compaction/reclamation resumes or rolls back without losing acknowledged state. |
| `CSQ-MNT-004` | Tier copy/move preserves bytes and identity; unavailable tiers remain explicit coverage gaps. |
| `CSQ-MNT-005` | Scrub observes and records damage without changing authority unless explicit repair is invoked. |

### 7.7 Backup, restore, migration, and compatibility

| ID | Invariant |
|---|---|
| `CSQ-BAK-001` | Backup includes exactly its declared authoritative frontier and coverage. |
| `CSQ-BAK-002` | Backup manifests bind every included artifact and reject mutation/omission/substitution. |
| `CSQ-BAK-003` | Restore of a complete backup has the same logical projection and damage evidence promised by the profile. |
| `CSQ-BAK-004` | Partial backup/restore never masquerades as complete. |
| `CSQ-MIG-001` | Supported migration preserves logical state, identity rules, history, damage, and unsupported evidence. |
| `CSQ-MIG-002` | Interrupted migration resumes/rolls back according to its phase record. |
| `CSQ-COMPAT-001` | Every advertised old-writer→new-reader edge is exercised using artifacts made by the released old binary. |
| `CSQ-COMPAT-002` | Unsupported compatibility edges fail without modifying the source. |

### 7.8 Resource and concurrency safety

| ID | Invariant |
|---|---|
| `CSQ-RES-001` | Size/depth/count/arithmetic violations fail before durable effect. |
| `CSQ-RES-002` | Reader and writer limits are compatible; a supported writer cannot emit unreadable-by-policy authority. |
| `CSQ-RES-003` | Scans, reads, rebuilds, salvage, and maintenance obey declared memory/file/time budgets or stop explicitly. |
| `CSQ-RES-004` | ENOSPC, quota, inode exhaustion, permission loss, allocation failure, and descriptor exhaustion preserve prior authority. |
| `CSQ-RES-005` | Cancellation/deadline cannot publish a half-operation or conceal committed authority. |
| `CSQ-CON-001` | Concurrent reads observe a permitted complete generation, never a mixed generation. |
| `CSQ-CON-002` | Concurrent writers are serialized/fenced according to the selected writer model. |
| `CSQ-CON-003` | Seal, compact, checkpoint, index rebuild, scrub, backup, and tier movement cannot race authority into an impossible state. |
| `CSQ-CON-004` | Async lifecycle completion is fenced before shutdown/reopen claims completion. |
| `CSQ-CON-005` | No panic, poison, deadlock, livelock, use-after-free, or data race becomes a successful storage result. |

### 7.9 Observation and public projection

These invariants apply to every public surface included by the selected
profile. Higher profiles repeat them across their additional RPC, cluster, and
user-interface projections.

| ID | Invariant |
|---|---|
| `CSQ-OBS-001` | Every reachable engine outcome has one registered public projection; unknown mappings fail closed. |
| `CSQ-OBS-002` | Partial, unavailable, conflicting, unsupported, resource-stopped, and incomplete-coverage outcomes cannot become absence, empty success, or a complete result. |
| `CSQ-OBS-003` | Ownership contention cannot become missing-store discovery, creation, initialization, or an empty logical store. |
| `CSQ-OBS-004` | Historical/recovered values remain explicitly non-current and recovery reads do not mutate or promote authority. |
| `CSQ-OBS-005` | Key coverage and body completeness remain separate through paging, filtering, decoding, and convenience APIs. |
| `CSQ-OBS-006` | Error code, structured detail, retryability, provenance, and achieved durability remain semantically compatible across projections. |
| `CSQ-OBS-007` | Every critical forbidden-collapse mutant is rejected by at least one independent oracle and public journey. |

## 8. Operation transition domain

Generated histories draw from:

```text
create/open/open_inspect/close/reopen
put-inline/put-chunked/put-many
get/get-payload/get-via-derived/get-version/find-last-complete
delete/history
scan-keys-page/scan-partial-page/scan/page/resume
seal/start-active
checkpoint/load/delete-cache/rebuild
secondary-index create/rebuild/use/drop
Hydra/Chimera/chunk-locator rebuild/use/delete/corrupt
compact/activate/reclaim/cancel/recover-job
tier-copy/tier-move/tier-offline/tier-online
backup/inspect/restore
scrub/pause/resume
salvage/salvage-to/export
migration preflight/run/resume
writer-contender
crash/abort/reopen
inject-damage/inject-resource-fault
```

Each generated history is observed through every applicable path:

```text
independent byte reader
production Store
embedded Collection/Heap projection
packaged CLI/reference application adapter
```

The outcome comparison includes value, current/historical status, coverage,
completeness, durability, ownership, retryability, provenance, and resource
termination.

State generation MUST include:

- absent, live inline, live chunked, deleted, rewritten, long-history,
  partially damaged, conflicting, unsupported, and coverage-incomplete keys;
- zero, one, and many subjects;
- same prefixes and binary SubjectV2 identities;
- active plus multiple sealed segments;
- derived state absent/current/stale/ahead/corrupt/foreign;
- one and multiple writer shards;
- every durability mode;
- boundary payload/key/count sizes; and
- maintenance jobs at every durable phase.

The generator weights transitions for coverage but cannot omit legal
transitions. Registry coverage reports every attempted and unattempted
pre-state × operation × post-observation class.

## 9. Exhaustive bounded kernels

### 9.1 Publication model

A pure model checker exhaustively explores:

```text
subjects:             2
values per subject:   absent, A, B, tombstone
durability modes:     memory, buffered, durable
operations:           put, delete, retry
persistence steps:    every registered boundary
crash choice:         before/after each step
recovery repetitions: 0, 1, 2
history depth:        up to 6 logical mutations
```

Properties: `CSQ-ACK-*`, `CSQ-PUB-*`, `CSQ-GEN-*`, `CSQ-ABS-001`.

The report states the exact state count and transition count. Any reduction,
symmetry rule, or partial-order reduction is documented and equivalence-tested.

### 9.2 Chunk-generation kernel

Exhaustively enumerate:

```text
generations:          0..3
chunks/generation:    1..4
payload alphabet:     {A, B}
missing slots:        every subset
duplicate slots:      none, identical, conflicting
physical order:       every permutation within the bounded corpus
current manifest:     each generation
```

The only complete result is the exact current manifest generation.

### 9.3 Small concurrent kernels

Loom/Shuttle or equivalent explores bounded schedules for:

- writer ownership/acquisition/drop;
- publication versus read;
- async seal completion versus close/reopen;
- compaction activation/reclamation versus read;
- derived-cache publication versus authoritative append; and
- chunk-locator update versus current-manifest publication.

Every kernel declares threads, operations, preemption bound, state bound, and
unexplored schedules.

### 9.4 Machine-checked proof obligations

Tests establish behavior in exercised executions. The following bounded,
pure-kernel properties are additionally proved with Kani, Verus, or an
equivalent machine checker:

- all decoded lengths, offsets, counts, and ranges are checked before use;
- every parser step either consumes input, returns a complete item, or returns
  a classified terminal result;
- publication recovery selects only the complete old state or complete new
  state, never a hybrid;
- a chunked value can be assembled only from the exact event identities named
  by its authenticated manifest;
- conflicting identities or duplicate slots cannot be accepted as one value;
- derived state cannot become authoritative merely because authoritative
  state is absent or damaged; and
- applying recovery repeatedly reaches the same observable fixed point as
  applying it once.

Each proof records its bounded domain, assumptions, unwinding bounds, checker
version, source hash, and result. Each proof suite includes a deliberately
false companion property that the checker must reject, proving that the
harness is live. Machine proof complements rather than replaces independent
readers, real filesystem campaigns, fuzzing, and soak tests.

## 10. Failure model

### 10.1 Registered failure classes

| ID | Class |
|---|---|
| `F-SW-CRASH` | returned error, panic, abort, SIGKILL, process restart |
| `F-WRITE` | zero/short/partial write, delayed error, interrupted write |
| `F-SYNC` | sync error, lost unsynced data, reordered persistence within declared filesystem model |
| `F-META` | rename/link/unlink/create/remove failure; missing directory sync |
| `F-CAPACITY` | ENOSPC, quota, inode exhaustion, file-size limit |
| `F-PERM` | permission loss, read-only transition, ownership/mode change |
| `F-IO` | EIO, device disappearance, transient/permanent read/write failure |
| `F-DAMAGE` | bit flip, byte overwrite, tear, hole, truncation, insertion, deletion |
| `F-TOPOLOGY` | duplicate/reordered/stale/missing files and tier roots |
| `F-RESOURCE` | allocation failure, memory/FD/thread exhaustion, cancellation, timeout |
| `F-MEMORY` | transient corruption of buffers, cached metadata, and bytes between read and validation |
| `F-CLOCK` | wall-clock rollback/jump, monotonic discontinuity after restart, timestamp collision |
| `F-PATH` | path aliasing, symlink substitution, case/Unicode collision, mount replacement, traversal attempt |
| `F-CONCURRENCY` | adverse schedule, simultaneous maintenance/read/write, writer contention |
| `F-CONFIG` | invalid/incompatible limits, shard count, format/profile, paths |
| `F-VERSION` | old/new artifacts, unsupported version/kind, interrupted migration |
| `F-KEY` | encryption key unavailable/wrong/rotated where applicable |
| `F-OPERATOR` | copied partial directory, restored wrong file, removed cache/source, wrong permissions |

Every operation registry entry declares applicable classes. `not applicable`
requires a reason reviewed with the invariant.

### 10.2 Persistence boundary completeness

The boundary census covers every storage-path call or abstraction that can:

```text
create/open/truncate/extend a file
write bytes
flush/sync data
sync a directory
rename/link/unlink/remove
publish an in-memory authoritative pointer/frontier
persist or activate a manifest/head/job phase
delete or reclaim former authority
acknowledge a caller
```

Static architecture checking compares source call sites and approved wrapper
APIs to `boundaries-v1.json`. A new or moved edge without registry coverage
fails CI.

For each boundary, execute:

- injected error immediately before;
- injected error immediately after;
- process abort before and after;
- short write for byte-producing edges;
- sync failure/lost-unsynced model where applicable;
- reopen once and repeatedly;
- retry with same and different operation identity;
- subsequent unrelated healthy write/read; and
- independent recovery observation.

### 10.3 Real persistence environments

In-process failpoints are necessary but not sufficient. Release qualification
runs subprocess and, where supported, privileged block/filesystem campaigns:

- SIGKILL at externally observed boundary barriers;
- loopback filesystem images;
- device-mapper flakey/error/delay targets or an equivalent harness;
- forced detach/remount/read-only transitions;
- quota, inode, and near-full conditions;
- actual process restart and clean-machine reopen; and
- abrupt VM termination where the platform lane supports it.

No filesystem is supported by a release profile until its required campaign
passes. The report records mount options, cache/barrier settings, device model,
kernel/OS, and virtualization.

### 10.4 Assumption and impossibility ledger

Qualification is valid only relative to a versioned, evidence-bundled ledger
of assumptions. At minimum it names:

- filesystem ordering, atomicity, rename, sync, and directory-sync semantics;
- OS/kernel, storage driver, device, controller, volatile-cache, and power-loss
  assumptions;
- RAM/ECC and CPU execution assumptions;
- cryptographic assumptions where authentication or encryption is used;
- the number, independence, placement, and failure domains of durable copies;
- the trusted computing base, including production code, independent oracles,
  proof tools, compilers, and test harnesses; and
- excluded Byzantine behavior for components outside ResiduumDB's control.

The suite does not claim recovery when every information-bearing copy of a
datum and its redundancy has been destroyed. It also cannot prove correctness
against a malicious or faulty kernel/device that returns a coherent forged
history, a broken cryptographic primitive, a CPU and all independent oracles
making the same undetected error, total destruction of all declared failure
domains, or deployment outside the qualified platform profile.

Those are explicit limits, not silent skips. The guarantee is:

```text
for every registered state, operation, fault, schedule, and qualified
platform inside the ledger assumptions, the registered invariant holds
or the result is a registered, truthful degradation state.
```

Any field failure not represented by the current registries is a qualification
incident. It must:

1. create a new failure-class or combination entry;
2. preserve the smallest reproducer and affected artifacts;
3. add a regression, mutant, and—where finite—a bounded exhaustive case;
4. rerun every transitively affected suite; and
5. revoke or narrow affected qualification claims until the new evidence
   passes.

### 10.5 Composed-failure closure

Qualification cannot assume failures arrive one at a time. The
`failure-combinations-v1.json` registry defines compatibility, order, injection
phase, expected observation class, and required suite for composed faults.

Required coverage is:

- every registered single fault at every applicable boundary;
- every ordered pair of compatible failure classes in the bounded publication,
  recovery, chunk, and maintenance kernels;
- every pair of concrete boundary injections where the first can leave durable
  or externally observable state;
- deterministic 3-wise and 4-wise covering arrays over class, boundary phase,
  artifact role, operation, durability mode, and restart count at system scale;
- targeted named scenarios from field incidents and threat analysis; and
- repeated occurrence of the same fault before, during, and after recovery.

Mandatory named combinations include:

```text
short write -> error handling -> crash
durable data sync -> directory sync failure -> crash
ENOSPC/inode exhaustion -> cleanup or retry -> crash
partial publication -> restart -> second write or delete
media damage -> inspect/recovery/salvage -> second damage or crash
stale/duplicate topology -> rebuild -> cancellation or crash
chunk rewrite -> manifest publication -> old/new chunk loss
compaction activation -> source reclamation -> crash or EIO
backup/migration -> partial copy -> restore/open on another version
memory corruption -> checksum/identity validation -> retry
clock rollback -> retry/dedup/retention decision
path alias or mount replacement -> open/write/recovery
```

An infeasible combination requires a machine-checked registry constraint or a
reviewed construction argument. “Unlikely” is not an exclusion. Every failure
must remain truthful after the second and later fault; recovery code is part of
the fault surface, not a trusted escape hatch.

## 11. Damage and survival campaigns

### 11.1 Canonical artifacts

Freeze canonical artifacts containing:

- empty and minimal stores;
- adjacent inline events;
- repeated puts/deletes;
- multiple chunk generations;
- active and sealed segments;
- unknown frame kinds/versions;
- indexes/catalogs/checkpoints;
- compacted/source pairs;
- backups and migrations; and
- mixed healthy/damaged regions.

Fixtures include byte hashes and provenance.

### 11.2 Exhaustive finite mutation

For every small canonical frame/segment:

- flip every individual bit;
- replace every byte with `0x00`, `0xff`, and one different deterministic byte;
- truncate at every byte boundary;
- insert one byte at every boundary for the canonical insertion alphabet;
- delete every single byte;
- duplicate every structural field/frame;
- mutate every length, kind, flag, identity, checksum, hash, version, count,
  offset, and terminator boundary to canonical edge values; and
- punch every contiguous hole `(start,end)` in the bounded micro-segment corpus.

For larger artifacts:

- cover every structural boundary and every byte position with single-bit
  mutation;
- use deterministic pairwise/t-wise covering arrays for multiple faults;
- generate separated multi-hole patterns;
- delete/reorder/duplicate/stale-copy every file-role combination within the
  bounded topology corpus; and
- retain minimized counterexamples.

Oracles assert corruption rejection, healthy-island discovery, honest holes,
termination, and bounded work. A cryptographically forged alternative valid
frame is outside accidental-damage claims and belongs to the security profile.

### 11.3 Damage locality matrix

For every authority unit `U` and every other unit `V`:

```text
damage(U) must not change decode(V)
```

The matrix covers frame, chunk, item generation, subject, segment, tier, store,
backup member, and derived artifact boundaries. Expected global coverage
evidence is compared separately from value meaning.

## 12. State-machine and metamorphic campaigns

### 12.1 Generated histories

Nightly and release campaigns generate long operation histories. After every
operation and injected failure:

1. observe live state;
2. compare the sequential model;
3. close without graceful cleanup where selected;
4. reopen;
5. delete every derived artifact and rebuild;
6. compare the independent byte reader;
7. optionally compact, back up/restore, or migrate;
8. compare again; and
9. continue healthy work.

Every failure retains the seed, minimized command sequence, pre-state fixture,
post-state bytes, and all oracle outputs.

### 12.2 Mandatory metamorphic relations

```text
decode(encode(x))                         = canonical(x)
reopen(reopen(S))                         = reopen(S)
rebuild(delete_derived(S))                = authoritative_projection(S)
compact(S)                                observationally_equivalent(S)
restore(backup(S))                        = promised_projection(S)
migrate_supported(S)                      = promised_projection(S)
scan_pages(S).concatenate                 = complete_scan(S)
index_query(S)                            = complete_scan_filter(S)
salvage(forward,S) and salvage(reverse,S) reconcile evidence
add_unrelated_healthy_event(S)            preserves prior observations
duplicate_identical_frame(S)              preserves logical event count
reorder_physical_files(S)                 preserves event-order semantics
damage_derived(S)                         preserves authority
damage_unrelated_unit(S,U)                preserves healthy unit U
```

### 12.3 Cross-layer semantic-collapse campaign

For every registered lower outcome, inject or construct it beneath each public
projection and compare with `outcomes-v1.json` and `projections-v1.json`.

Mandatory negative adapters deliberately implement:

```text
Err(_)                    -> None
CoverageIncomplete       -> []
PayloadPartial           -> empty body
WriterLockHeld           -> create/open empty
historical recovered     -> current
cache absent/corrupt     -> key absent
timeout/resource stop    -> complete prefix
```

The suite passes only when each adapter is killed. It additionally exercises
every named incident composition in `incidents-v1.json`, including
DEF-098–DEF-104, through the lowest responsible layer and the highest public
surface in the profile.

## 13. Limits, allocation, and progress

Test the Cartesian boundary set:

```text
0
1
minimum - 1 / minimum / minimum + 1
default threshold - 1 / threshold / threshold + 1
maximum - 1 / maximum / maximum + 1
integer conversion and checked-arithmetic boundaries
```

Apply it to:

- subject, envelope, frame, body, logical payload, chunk, manifest, segment;
- history, collection, index, catalog, partition/tier counts;
- scan page, result, cursor, backup member, migration record;
- paths, filenames, nesting, arrays/maps, and configuration values; and
- memory, files, descriptors, threads, queue, time, and disk budgets.

Requirements:

- invalid requests fail before authoritative effect;
- no integer overflow/wrap/truncation;
- no preallocation based on untrusted counts before bounds;
- no infinite/zero-progress scan;
- no hidden O(dataset) point operation;
- no unbounded collection in recovery/maintenance;
- cancellation leaves authority valid; and
- errors name the effective bound without exposing secrets.

Allocator-failure injection covers every large allocation class. Large-data
tests measure peak RSS and bytes read/written, not merely completion.

## 14. Concurrency and process ownership

Required workloads combine:

```text
read/get-payload/history/scan
put/delete/put-many
seal/async lifecycle
checkpoint/index rebuild
compact/activate/reclaim
scrub/backup
tier movement/availability change
close/reopen and writer contention
```

Run:

- exhaustive bounded schedule kernels;
- deterministic randomized scheduler campaigns;
- native-thread stress with 1, 2, 4, and 8 writer shards;
- multiple-process writer-lock contention;
- sanitizer/Miri-supported subsets;
- forced cancellation/panic/poison paths; and
- shutdown/restart during background work.

Every history is checked against the advertised single-writer consistency
model. Deadlock/livelock watchdogs retain thread/task state.

## 15. Mutation testing

Mutation testing proves the suite would detect plausible implementation rot.

The mandatory P0 mutation catalog includes:

- remove or move each durable sync;
- return a durable receipt before sync;
- publish an index before authority;
- skip directory sync;
- accept a torn/bad-hash/bad-length frame;
- turn damage/unavailability/conflict into `None`;
- choose physical encounter order;
- mix item generations or ignore manifest event IDs;
- accept a same-index chunk from another generation;
- omit content-hash verification;
- let a derived cache override segments;
- drop/reorder/duplicate history events;
- reclaim compaction sources before verified activation;
- treat unavailable tier as empty;
- skip size/count/arithmetic checks;
- convert short write/ENOSPC/EIO to success;
- reuse an operation ID with different content;
- remove writer fencing; and
- disable cancellation checks in bounded loops.

Requirements:

- 100% of non-equivalent mandatory P0 mutants are killed;
- at least 95% of all non-equivalent core-storage mutants are killed;
- equivalent mutants require reviewed proof/justification;
- every surviving non-equivalent P0 mutant is a release-blocking defect;
- a fixed sentinel-mutant set runs regularly to verify the mutation harness;
  and
- mutation results bind exact source and suite revisions.

## 16. Fuzzing

Owned targets include:

```text
frame encode/decode
forward/reverse scan
deterministic CBOR envelopes
chunk manifest/piece/reassembly
primary index/cache/catalog/checkpoint
segment summary/catalog/tier placement
history and dedup evidence
backup/salvage/scrub/migration manifests
store configuration and paths
generated operation sequence decoder
```

Fuzz findings include panic, abort, memory unsafety, data race, hang,
non-progress, excessive allocation, excessive CPU, inconsistent oracle,
fabricated verification, and silent uncertainty loss.

Every minimized finding becomes:

- a permanent regression fixture;
- a registry-linked invariant/failure case; and
- a named defect when the root cause is not immediately closed.

Release-candidate evidence includes at least 24 cumulative CPU-hours per
untrusted core parser since the candidate's common ancestor, with all corpus
hashes and sanitizer configurations retained. Duration does not replace corpus
or invariant coverage.

## 17. Compatibility

Retain immutable artifacts from every released writer. For each supported edge:

```text
old writer -> current ordinary reader
old writer -> current inspect/salvage
old backup -> current restore
old damaged bytes -> current recovery
old store -> current migration -> current reader
current writer -> declared old-reader outcome
```

Artifacts generated by the current source tree are not historical evidence.
The released binary/package/container hash and generation command accompany
every fixture.

Unsupported edges must preserve the source and produce explicit
`FormatUnsupported`, never attempt an undocumented best-effort mutation.

## 18. Scale and soak

Qualification datasets include:

- working set larger than available RAM;
- many small values and boundary-sized values;
- repeated large/chunked rewrites;
- long histories and tombstone-heavy histories;
- many segments, collections, indexes, tiers, and derived files;
- fragmented/damaged media;
- near-full disk/inode conditions; and
- maintenance concurrent with foreground operations.

Minimum release-candidate campaign:

```text
duration:                 72 continuous hours per supported store profile
logical operations:       at least 1,000,000,000 or 72 hours, whichever is later
forced process restarts:  at least 1,000
injected registered faults: every applicable class, with no unreported skip
final observations:       model sample + full rebuild + scrub + backup/restore
```

The workload remains reproducible through a root seed and deterministic
sub-seed derivation. Reports include throughput, latency, peak RSS, disk,
inodes, file descriptors, background backlog, errors, partials, conflicts,
recovery time, and final invariant verdict.

A soak that merely remains running does not pass. Its final authoritative state
must reconcile.

## 19. Test execution lanes

### 19.1 Pull request

Maximum intended wall budget: 20 minutes on the reference CI class.

Required:

- registry/architecture closure;
- invariant unit and canonical corpus;
- exhaustive small publication/chunk kernels;
- changed-operation state-machine histories;
- all changed persistence boundaries;
- sentinel P0 mutants;
- parser fuzz smoke;
- one crash/reopen/rebuild journey; and
- no conditional skip reported as pass.

### 19.2 Nightly

Required:

- every in-process/subprocess failpoint;
- complete generated transition matrix accumulation;
- at least 1,000,000 model-checked logical operations;
- canonical every-bit/truncation/deletion campaigns;
- all fuzz targets and sanitizers;
- concurrency scheduler campaigns;
- near-full disk and resource faults;
- packaged backup/restore/migration journey; and
- evidence report upload.

### 19.3 Weekly/continuous

Required:

- real-filesystem/block-device fault lanes;
- exhaustive bounded contiguous-hole corpus;
- multi-fault covering arrays;
- mutation campaign;
- compatibility matrix;
- dataset-larger-than-RAM and maintenance-under-load;
- long fuzz accumulation; and
- at least one 24-hour torture run.

### 19.4 Release candidate

Required:

- clean packaged artifacts only;
- every mandatory platform/filesystem/profile cell;
- complete failpoint/failure/operation/invariant matrix;
- mutation thresholds;
- fuzz budget;
- 72-hour/billion-operation campaign;
- independent report verification; and
- zero unexplained skip, survivor, flake, mismatch, or infrastructure failure.

## 20. Evidence bundle

Every run emits a canonical `dingo-core-storage-report-v1` containing:

```text
run_id
profile and assurance level
source revision and dirty-tree content hash
released package/binary hashes
registry and specification hashes
runner/oracle binary hashes
platform/kernel/filesystem/mount/device facts
CPU/memory/disk/inode/FD/time budgets
suite IDs and exact commands
operation/invariant/failure/boundary coverage matrices
failure-combination coverage and infeasibility constraints
outcome/projection/forbidden-collapse/composition coverage
model bounds and explored state/transition counts
proof obligations, bounds, assumptions, checker versions, and results
seeds, generator versions, corpora, mutation set
start/end/duration and result
all retries/attempts
failures, not_run, infrastructure failures, flakes
resource and performance measurements
known exclusions
attachment hashes
```

Attachments include minimized histories, corrupted fixtures, raw tool output,
crash images where retainable, coverage, mutation reports, fuzz corpora, soak
time series, and final reconciliation.

The report is signed or placed in the Residuum Evidence Ledger when that subsystem
is available. Until then it is deterministically encoded, hash-addressed, and
verified by a separately built verifier.

## 21. Failure retention and flake policy

- Every failure is retained before minimization.
- Minimization never replaces the original artifact.
- Seeds and operation histories are printed before destructive execution.
- A correctness flake fails qualification.
- Reruns are additional evidence, not replacement evidence.
- Quarantine requires a defect, owner, expiry, affected invariant/claim, and
  capability downgrade.
- Timeout/hang is failure until independently classified.
- Infrastructure failure is retained and rerun; it cannot satisfy a gate.
- No suite may modify or delete a non-dedicated source directory.

## 22. Change-impact law

Every pull request touching:

```text
format/store/recovery/chunks/history/index/cache/catalog/checkpoint
compaction/tiering/backup/scrub/migration/failpoints/resource limits
```

must declare affected operation, invariant, boundary, failure, oracle, and suite
IDs. CI rejects an empty impact declaration unless the architecture checker
proves no storage-semantic change.

A code reviewer cannot waive a missing P0 invariant cell. Waiver requires a
normative profile amendment and explicit capability downgrade.

## 23. Qualification gates

### 23.1 A2 self-assessed core

- all registry relations valid;
- independent model/reader firewall valid;
- all P0 invariants pass;
- all registered in-process and subprocess boundaries pass;
- all mandatory composed-failure cells pass;
- exhaustive bounded kernels pass with disclosed bounds;
- all mandatory machine-checked obligations pass with live negative controls;
- every reachable outcome has a total registered projection through every
  public surface in the profile;
- every critical forbidden-collapse adapter/mutant is killed;
- canonical corruption/survival campaigns pass;
- generated state-machine coverage reaches every registered transition class;
- P0 mutation catalog is fully killed;
- no false absence, fabricated commit, lost durable acknowledgement, or
  cross-generation contamination;
- no unexplained skip/flake/survivor/infrastructure failure; and
- evidence bundle independently verifies.

### 23.2 A3 production candidate

A2 plus:

- every advertised platform/filesystem cell;
- the bundled assumption ledger exactly matches every advertised platform;
- real device/filesystem fault campaign;
- full compatibility matrix;
- all-parser fuzz budget;
- overall mutation threshold;
- scale/resource gates;
- 72-hour/billion-operation reconciliation;
- packaged-artifact damage/recovery journey; and
- published limitations and capacity bounds.

### 23.3 Immediate revocation

Any reproducible violation of a P0 invariant:

1. revokes the affected qualification result;
2. opens a P0 defect;
3. adds the minimized regression and mutation if applicable;
4. blocks release until the complete affected matrix reruns; and
5. requires correction of public capability language.

## 24. Non-negotiable acceptance statement

The suite is complete only when ResiduumDB can produce a verified bundle proving:

```text
for every registered core-storage invariant
for every applicable operation and prior-state class
for every declared failure class
for every registered persistence/publication boundary
for every required recovery observation
for every supported platform/filesystem/version cell

result = exact truth | explicitly permitted uncertainty
```

Anything unregistered, unrun, skipped, infrastructure-blocked, flaky,
implementation-oracled, or unevidenced is not qualified.
