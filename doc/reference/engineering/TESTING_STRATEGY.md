# Residiuum testing and verification strategy

Status: normative engineering strategy v1.0-draft

Date: 2026-07-31

Audience: implementers, reviewers, release engineers, security engineers

Companions:
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md),
[FORMAT_SPEC.md](../storage/FORMAT_SPEC.md),
[HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md),
[DEFECTS.md](../../done/incidents/DEFECTS.md),
[CORE_STORAGE_QUALIFICATION_SPEC.md](../../todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md),
[doc/todo/verification/VERIFICATION_IMPLEMENTATION_PLAN.md](../../todo/verification/VERIFICATION_IMPLEMENTATION_PLAN.md),
and
[doc/wip/status/VERIFICATION_STATUS.md](../../wip/status/VERIFICATION_STATUS.md).

## 1. Decision

Residiuum does not use “the tests passed” as a product claim.

It uses a structured assurance case:

```text
product claim
    ↓
named invariant
    ↓
independent oracle
    ↓
bounded test/model domain
    ↓
reproducible execution
    ↓
immutable evidence artifact
    ↓
qualified capability label
```

A capability is releasable only when every mandatory claim has this chain.

Test quantity, source coverage, fuzz duration, proof count, and benchmark
volume are diagnostics. None independently proves correctness.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 3. What “exhaustive” means

### 3.1 Whole-system exhaustiveness is impossible

Residiuum accepts unbounded operation sequences, data, thread schedules, process
timings, device failures, network histories, topologies, and version histories.
The complete database state space cannot be enumerated. Documentation and
marketing MUST NOT describe the whole test program as exhaustive.

### 3.2 Finite-domain exhaustiveness is required where possible

An implementation MUST exhaustively enumerate a bounded domain when:

- the relevant state is finite and small;
- the bound covers a security or correctness kernel;
- a proof harness can explore every branch within the bound; or
- the specification defines a finite canonical corpus.

Examples include Heap admission/isolation, small Atomic state machines, cursor
transitions, crash-publication machines, canonical language vectors, bounded
replica models, and every byte position in a small canonical segment.

Every exhaustive claim MUST state its exact bound, assumptions, harness, and
properties. “Kani proved isolation” is not a sufficient report.

### 3.3 Systematic is not exhaustive

Failpoint enumeration, property generation, fuzzing, chaos, and soak are
systematic confidence techniques. Their reports MUST disclose:

- covered surface;
- generator/model;
- seed or corpus identity;
- duration/operation count;
- concurrency/topology;
- omitted states; and
- oracle.

## 4. Verification profiles

Testing is profile-specific.

| ID | Profile | Authority under test |
|---|---|---|
| `V0` | pure semantics | SDA, predicate, Heap, future Atomic/rank kernels |
| `V1` | format and examination | frames, segments, chunks, scanner, salvage, SDA projection |
| `V2` | embedded store | local store, collections, indexes, history, maintenance |
| `V3` | embedded Heap | Heap identity, authority, keys, isolation, lifecycle |
| `V4` | single-node server | TLS, admission, RPC, parity, concurrency, shutdown |
| `V5` | network cluster | consensus, replication, fencing, repair, coverage |
| `V6` | retention/archive | tiers, remote media, lifecycle, encryption, migration |

Passing a lower profile does not qualify a higher one. Every result manifest
identifies exactly one primary profile.

The normative specialization for `V1` format plus `V2` embedded-store
authority is
[CORE_STORAGE_QUALIFICATION_SPEC.md](../../todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md).
Where it is stricter than this general strategy for core storage, the
specialized specification controls.

## 5. Assurance levels

| Level | Meaning | Permitted claim |
|---|---|---|
| `A0` | builds or an example works | development only |
| `A1` | functional corpus and ordinary journeys pass | experimental |
| `A2` | adversarial, crash, damage, property and bounded model evidence pass | early access / self-assessed |
| `A3` | release matrix, compatibility, soak, security and reproducible evidence pass | production candidate for named profile |
| `A4` | A3 plus relevant independent review and finding disposition | independently assessed named claim |

Levels attach to a capability/profile pair, never to “Residiuum” globally.

## 6. Claim registry

Every stable or advertised capability MUST appear in:

```text
spec/verification/claims-v1.json
```

Each record contains:

```text
claim_id
profile
capability
claim_text
invariants[]
public_surfaces[]
observable_outcomes[]
forbidden_collapses[]
required_compositions[]
oracles[]
required_suites[]
required_platforms[]
required_failure_classes[]
required_evidence[]
assurance_level
known_exclusions[]
owner
```

Illustrative claims:

```text
FMT-SURVIVE-001
  later verified frames remain discoverable after an earlier damaged region

HEAP-ISO-001
  a capability admitted for Heap A cannot observe or mutate Heap B

STORE-ACK-001
  a durable acknowledgement survives the declared crash boundary

QUERY-COVERAGE-001
  incomplete searched coverage cannot be represented as proven absence
```

A release verifier MUST fail when a public claim lacks a registry entry, an
invariant lacks an oracle, a required suite lacks current passing evidence,
evidence comes from another revision, or exclusions contradict the claim.

## 6A. Invariant discovery and semantic width

### 6A.1 A registry cannot contain what nobody thought to name

Closed registries prevent known obligations from disappearing; they do not
prove the obligation set is complete. Before a capability is registered, its
design review MUST perform an invariant-discovery pass over:

```text
authority       what bytes/state decide truth?
identity        what must never alias, merge, or cross scope?
time            what can be current, historical, stale, or reordered?
atomicity       what partial publication states can exist?
completeness    what is known, missing, unavailable, conflicting, or unknown?
derivation      what may be deleted/rebuilt and must never become authority?
projection      what meaning can be lost between format/store/SDK/RPC/UI?
composition     what fails only when two correct-looking features interact?
resources       what happens at every size/count/time/memory/disk boundary?
concurrency     what interleavings change visibility or ownership?
configuration   which layer supplies the effective policy and tightest limit?
compatibility   how do old/new/unsupported artifacts behave?
operations      which errors can be misread and which actions destroy evidence?
security        what crosses Heap, path, process, channel, or privilege scope?
recovery        what survives, who may select it, and can selection mutate it?
```

For every public method, reviewers answer:

1. What distinct physical/logical states can reach this method?
2. Can its return type represent every materially different state?
3. If not, does it fail closed without fabricating a stronger conclusion?
4. Can a convenience wrapper turn error, uncertainty, or partial coverage into
   `None`, `false`, zero, an empty collection, or a default value?
5. What prior state, second fault, or adjacent feature changes the answer?
6. Which independent oracle would detect an incorrect collapse?

A capability cannot leave design review with an unanswered question.

### 6A.2 Observation algebra

Residiuum distinguishes at least:

```text
VerifiedPresent(value)
VerifiedAbsent
VerifiedDeleted(tombstone)
Partial(surviving evidence)
Unavailable(known authority)
Conflicting(verified alternatives)
CoverageIncomplete(gaps)
Unsupported(version or kind)
ResourceStopped(bound)
PermissionDenied
OwnershipContended
UnknownOutcome
```

These values form an information/safety ordering, not a list of interchangeable
errors. Only `VerifiedAbsent` or a valid current tombstone can become ordinary
absence. A projection MAY remove detail only when the result remains no
stronger than its input and the loss is explicit.

Forbidden semantic collapses include:

```text
Partial/Unavailable/Conflicting       -> None or empty value
CoverageIncomplete                   -> empty or complete list
ResourceStopped/timeout/cancellation  -> complete result
OwnershipContended                   -> missing/new empty store
Unsupported                          -> corrupt, absent, or best-effort mutation
historical value                     -> current value
derived-cache miss                   -> authoritative absence
failed/unacknowledged write           -> acknowledged durable state
```

Every public projection registers its total mapping over reachable lower-layer
outcomes. An unregistered mapping or an unreachable/unhandled lower outcome
fails architecture CI.

### 6A.3 Compositional closure

Testing each component alone is insufficient. Registries identify feature
compositions that require direct histories, including:

```text
chunking × replacement × crash × history
partial body × key enumeration × pagination
coverage gap × filtering × limit/order/cursor
writer lock × force-quit × retry × application open
active log × stale/missing cache × reopen
large-value policy × SDK/server/RPC limit negotiation
compaction/tiering × chunk locator × historical read
damage × recovery selection × subsequent healthy write
```

All compatible pairs of P0 invariant domains receive a bounded composition
test or a reviewed proof of independence. Named incident compositions are
mandatory permanent journeys.

### 6A.4 Incident-driven expansion

Every field incident produces:

```text
incident artifact
→ classified or explicitly unclassified physical cause
→ missing/violated invariant
→ smallest reproducer
→ forbidden-collapse case
→ permanent regression
→ mandatory mutant or deliberately broken fixture
→ affected composition cells
→ capability revocation and rerun set
```

The incident is not closed merely because the symptom disappears.

### 6A.5 Semantic-width gate

A P0/P1 capability cannot enter implementation or qualification until:

- the reachable-state and public-outcome sets are enumerated;
- every format → engine → SDK → protocol projection is total;
- forbidden collapses are registered;
- every compatible P0-domain pair has a test/proof disposition;
- at least one deliberately broken fixture proves each critical oracle is
  capable of failing;
- a reviewer other than the implementation author performs the invariant
  discovery checklist; and
- unresolved questions become named defects, exclusions, or rejected claims.

The gate records candidate invariants considered and rejected, with reasons.
This prevents “we did not think of it” from being indistinguishable from “not
applicable.”

## 7. Oracle doctrine

### 7.1 Independent oracles

The preferred oracle is simpler than, and structurally independent from, the
implementation.

Examples:

- slow in-memory map/event-log model for collection state;
- complete scan and sort for indexes and ranked access;
- mathematical evaluator for RRE/Atomic plans;
- sequential specification for concurrent histories;
- byte-level reference decoder for canonical vectors;
- set-based replica model for repair/convergence; and
- independent evidence-package verifier.

Copying the production algorithm into a test helper is not independent.

### 7.2 Metamorphic properties

Where an expected result is difficult to calculate, assert relations:

- encode → decode preserves canonical value;
- rebuild derived state → same observations;
- forward/reverse scan → compatible verified islands and holes;
- backup → restore → same authoritative projection;
- migration → read → same supported interpretation;
- optimized query → slow query with identical rows/order/coverage;
- identical retry → same logical effect; and
- damage outside an authority unit → unchanged healthy-unit result.

### 7.3 Differential testing

Every accelerator MUST be compared to a slow authoritative path:

```text
secondary index       versus complete surviving scan
Hydra/Chimera         versus frame decode
Direct Access         versus complete ranked enumeration
Order Wavelet         versus complete stable sort
query compiler        versus direct predicate evaluator
repair result         versus replica-content model
```

Damage and incomplete coverage are part of the differential domain.

## 8. Required test families

### 8.1 Unit tests

Use for local deterministic behavior and stable errors. Unit tests MUST NOT be
the sole evidence for persistence, concurrency, security, or network claims.

### 8.2 Normative conformance corpora

Specification examples and counterexamples are executable. Corpora are
versioned, immutable after freeze, independently readable, run across relevant
backends, and retained across releases.

### 8.3 Property-based tests

Properties are required for codecs/canonicalization, domain separation,
predicates, index-versus-scan, cursor binding, backup/restore/migration,
Heap noninterference, Atomic retry/convergence, and coverage composition.

Every minimized counterexample becomes a permanent regression.

### 8.4 Model/state-machine tests

Reference models generate histories containing:

```text
create/open/close
put/delete/get/find/history
index create/rebuild/drop
backup/restore
compact/seal/checkpoint
authority issue/blacklist/cycle
crash/reopen
damage/salvage/scrub
```

After every step, implementation and model observations are compared,
including uncertainty and coverage—not merely payloads.

### 8.5 Crash-consistency tests

Every authoritative publication protocol declares its durable boundaries.
For every boundary, test:

- failure before;
- short/partial write;
- failure after write before sync;
- failure after file sync before directory sync;
- failure around rename/publication;
- process abort;
- reopen and repeated recovery;
- retry with the same operation identity; and
- subsequent healthy work.

Passing requires one specified old/new/unknown outcome, never a hybrid.
A conditionally skipped matrix reports `not_run`, not `pass`.

### 8.6 Filesystem and device faults

Required classes:

- ENOSPC and quota exhaustion;
- permission loss/read-only transition;
- short write and I/O error;
- torn/lost/duplicated/reordered regions;
- stale copied files and missing directory entries;
- metadata and payload corruption independently;
- unavailable tier/device; and
- unavailable key provider.

In-process injection is necessary but not sufficient. Release campaigns SHOULD
include OS/filesystem or block-device fault environments.

### 8.7 Corruption and survival campaigns

For small canonical artifacts, mutate every byte, every bit where affordable,
all structural boundaries, lengths, offsets, checksums, identities, tags and
versions. Include insertion, deletion, duplication, reorder, truncation,
garbage, and multiple separated holes.

The oracle proves:

- corrupt never becomes verified;
- every promised healthy island remains discoverable;
- holes and provenance remain honest;
- unsupported data stays unsupported;
- healthy units retain meaning; and
- forward/reverse observations reconcile.

### 8.8 Fuzzing

Every untrusted parser has an owned fuzz target or documented full coverage by
another target. Required surfaces include:

- frame, segment, chunk and canonical CBOR;
- forward/reverse scanner;
- Heap certificates, authority metadata and ownership;
- SDA/RQL/RRE source, artifacts and evaluation;
- RPC, configuration and URLs;
- indexes, catalogs, checkpoints and cursors;
- backup, salvage, scrub, migration and Evidence manifests; and
- cluster/control metadata.

Release evidence requires accumulated continuous or long scheduled fuzz time.
Crashes, hangs, OOMs, excessive allocation, and work bombs are findings.
Minimized inputs are retained.

### 8.9 Concurrency testing

Deterministic schedule exploration is required where practical for lock
ownership, publication/frontier updates, cursors, index-build/mutation,
authority snapshots, telemetry isolation, Evidence sequencing, and Atomic
decision. Loom, Shuttle, or an equivalent SHOULD cover small Rust kernels.

### 8.10 Distributed histories

Cluster qualification requires real multi-process histories with process
kill/restart, partitions, asymmetric loss, delay/duplication/reordering, stale
leaders/placements, disk stalls/damage, membership changes, coordinator
replacement, rolling upgrade, and control-plane reconstruction.

Independent linearizability, serializability, or convergence checkers verify
the advertised mode. In-process simulation is preparation, not production
network evidence.

### 8.11 Compatibility testing

Retain released format fixtures, backups, evidence packages, protocol vectors,
configuration, SDK/CLI JSON, and reproducible binaries/containers.

For each promised edge:

```text
old writer → new reader
new writer → permitted old reader behavior
upgrade → use → rollback where promised
backup old → restore new
damaged old → salvage new
```

A fixture generated by current code is not historical evidence.

### 8.12 Journey tests

Every release has a clean-machine journey using packaged artifacts:

```text
install
→ create Heap
→ write JSON and bytes
→ query/history
→ kill
→ reopen
→ damage
→ inspect holes
→ scrub
→ back up
→ restore under new identity
→ verify evidence
```

### 8.13 Scale, soak and resources

Test datasets larger than RAM, long histories, high collection/index counts,
mixed traffic, maintenance under load, bounded queues/cursors/connections,
slow/disconnected telemetry, near-full disk, and restart after long work.

Reports retain seed, configuration, time series, final verification, holes and
errors.

### 8.14 Performance correctness

A benchmark is invalid if durability degraded, verification/replication
silently changed, incomplete coverage looked complete, errors disappeared from
throughput, memory exceeded the claimed bound, or background completion was
excluded without disclosure.

### 8.15 Security testing

Required:

- authorization matrix and negative cases;
- cross-Heap differential noninterference;
- certificate/epoch/audience/channel-binding attacks;
- token forgery/replay/rotation;
- parser/resource denial;
- secret scanning of logs, telemetry, evidence, backups and diagnostics;
- path/symlink/permission attacks;
- dependency/advisory/license checks; and
- external review for independently assessed claims.

### 8.16 Cross-layer contract and forbidden-collapse testing

For every stable operation, generate the same logical scenarios through every
applicable surface:

```text
format/reference reader
→ Store
→ embedded Collection/Heap SDK
→ server RPC
→ remote SDK
→ cluster projection
→ CLI/Studio/reference application adapter
```

Assertions compare semantic outcome, identity, completeness, durability,
coverage, provenance, retryability, and bounds—not merely error strings.

Mandatory adversarial fixtures force each lower-layer outcome and prove that
every upper layer either preserves it or performs one registered safe
projection. Each layer has deliberately broken adapters that convert outcomes
to `None`, `[]`, defaults, success, or current state; the suite MUST kill them.

Reference application journeys additionally assert that:

- open/list/read errors do not initialize or overwrite an existing store;
- an incomplete list is visibly different from an empty complete list;
- a historical recovery result is visibly different from current state;
- read-only inspection cannot acquire writer authority;
- recovery/export never mutates source evidence; and
- a subsequent healthy write does not erase unresolved damage evidence.

## 9. Coverage model

Line and branch coverage are diagnostic. The authoritative matrix is:

```text
claim
× profile
× invariant
× prior state
× operation
× failure class/composition
× public surface/projection
× observable outcome
× oracle
× platform
× version/configuration
```

Source coverage cannot close an invariant gap.

Mutation testing MUST target integrity verification, coverage/absence,
authorization, durability receipts, cursor binding, error classification and
recovery selection. It also targets every registered forbidden collapse. A
surviving critical mutation is a finding.

## 10. Platform matrix

The supported release profile declares operating systems, architectures,
filesystems, media, Rust versions, containers, TLS/key providers, and
capacity/free-space assumptions.

PR CI covers at least Linux and macOS. Production qualification adds the
actual supported filesystem/platform matrix. Untested means unsupported.

## 11. Execution lanes

### 11.1 Pull request

Formatting/lint/build, unit/integration suites, normative corpora, short
properties, critical compile-fail/security checks, architecture checks, and
changed-surface fuzz smoke where affordable.

### 11.2 Nightly

Full crash matrix, destructive corpora, longer properties, all fuzz targets,
testrig smoke, sanitizer/Miri subsets, packaged journeys, and deterministic
cluster simulation.

### 11.3 Weekly/continuous

Long fuzzing, multi-hour soak, large datasets, filesystem faults,
multi-process histories, compatibility matrix, and controlled benchmarks.

### 11.4 Release candidate

Every mandatory suite for one named profile/level runs from clean packaged
artifacts and produces one evidence bundle. It cannot silently skip.

## 12. Runner and evidence

Canonical entry point:

```text
residiuum verify --profile <V0..V6> --level <A0..A4>
```

Until the binary exists, `scripts/residiuum-verify.sh` MAY implement the contract.

### 12.1 Preflight

Before compilation or mutation, check free disk/inodes, writable isolated
roots, tools, CPU/memory/time/file-descriptor minima, platform/filesystem,
source state, privileges, and that destructive targets are dedicated.

Preflight failure is `infrastructure_failure`, never test failure or pass.

### 12.2 Result states

Every suite ends in exactly one:

```text
pass
fail
not_run
infrastructure_failure
```

Qualification requires `pass`. Retries retain all attempts.

### 12.3 Evidence manifest

Every run emits `residiuum-verification-report-v1.json` containing:

```text
run_id
source_revision
dirty_tree_hash
profile and assurance level
runner/platform/filesystem/resources
suite versions and commands
seeds/corpora
start/end/duration
result per suite
claim coverage
failures/skips/infrastructure failures
artifact hashes
known exclusions
```

Large logs, histories, minimized failures and benchmarks are hashed
attachments.

## 13. Flake and failure policy

- A flaky correctness test is a defect.
- Retry-to-green cannot satisfy a release gate.
- Quarantine requires owner, defect, expiry, affected claims and downgrade.
- Unknown hangs/timeouts are failures until classified.
- Infrastructure failures are repaired/rerun and retained.
- Generated failures print and retain their seed.
- Tests leave no daemon, port, key, or destructive directory.

## 14. Test-code quality

- Critical helpers receive tests.
- Oracles are reviewed separately.
- Expected values are not regenerated by code under test.
- Fixtures have hashes/provenance.
- Failpoints map to publication protocols.
- Test-only bypasses cannot enter release builds.
- Semantic time is controlled.
- Random campaigns disclose and replay seeds.

## 15. Current truth

The repository already has SDA conformance, format corruption/scanner corpora,
store crash/I/O-fault matrices, Heap isolation plus Kani/Verus, server
protocol/TLS/admission tests, deterministic cluster histories,
backup/scrub/migration suites, eleven cargo-fuzz targets (format + SDA + RPC +
store control decoders) under scheduled smoke (`scripts/fuzz-smoke.sh` /
nightly `fuzz_smoke`), and a scale/chaos rig.

It does not yet satisfy this strategy. See
[doc/wip/status/VERIFICATION_STATUS.md](../../wip/status/VERIFICATION_STATUS.md).

## 16. Completion definition

A named profile reaches a named assurance level only when:

1. public claims are registered;
2. every claim has invariants and independent oracles;
3. mandatory suites pass without silent skip;
4. crash, damage, security, compatibility and resource classes pass;
5. platform/version requirements pass;
6. evidence binds exact source and artifacts;
7. exclusions are compatible with the claim;
8. capability documentation matches; and
9. every public projection is total and every forbidden collapse is killed;
10. required feature compositions and incident journeys pass; and
11. the release verifier independently validates the bundle.

That is how Residiuum knows what works—and what has not yet been proved.