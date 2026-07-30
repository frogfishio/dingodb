# Core Storage Qualification implementation plan

Status: **developer-ready v1.0-draft**

Program: `CSQ`

Normative specification:
[CORE_STORAGE_QUALIFICATION_SPEC.md](../CORE_STORAGE_QUALIFICATION_SPEC.md)

Program authority:
[MASTER_DELIVERY_PLAN.md](../MASTER_DELIVERY_PLAN.md)

## 1. Outcome

Implement the suite required to qualify `dingo-core-storage-v1`, then produce:

```text
dingo verify --profile dingo-core-storage-v1 --level A2
```

The command returns either:

- a self-verifying evidence bundle satisfying every mandatory core-storage
  claim; or
- an exact list of missing, failed, skipped, infrastructure-blocked, or
  unevidenced cells.

This package tests storage authority. It does not implement feature fixes.
Known P0 defects are repaired first; every repair contributes a permanent
regression and mutation to this suite.

## 2. Entry and interlock

Qualification implementation starts immediately after the observed
core-storage/recovery defect family DEF-098 through DEF-104 and any additional
active P0 core-storage defects are remediated.

Until `CSQ-12` accepts:

- no embedded/core-storage production or “unbreakable” claim is permitted;
- HAR/APP code may be specified, but new feature implementation does not
  preempt CSQ;
- a newly discovered P0 storage defect interrupts the program, is fixed, and
  returns through the affected CSQ packages.

## 3. Package order

```text
CSQ-0  Contract registries
  ├── CSQ-1  Independent oracles
  └── CSQ-2  Boundary/failure instrumentation
          |
          +── CSQ-3  Format exhaustive corpus
          +── CSQ-4  Store state machine
          +── CSQ-5  Crash/filesystem campaign
          +── CSQ-6  Chunk/large-value qualification
          +── CSQ-7  Damage/salvage/recovery
          +── CSQ-8  Derived/maintenance/backup/migration
          +── CSQ-9  Concurrency/resources
          +── CSQ-10 Mutation/fuzz
          +── CSQ-11 Compatibility/scale/soak
all mandatory packages ────────────────→ CSQ-12 Qualification evidence
```

At most one package that changes production storage semantics is active at a
time. Test/oracle packages may run in parallel when they do not share mutable
fixtures or redefine expected behavior.

## 4. CSQ-0 — Contract registries

Priority: `P0-GATE`

Deliver:

```text
spec/verification/core-storage/*.json
spec/verification/core-storage/report-v1.schema.json
scripts/verify-core-storage-registry.sh
```

Work:

- implement the minimum common `VFY-0` registry framework needed to import the
  core-storage namespace; do not create a second incompatible registry;
- encode every invariant, operation, failure class, boundary, oracle, suite,
  failure combination, assumption, proof, platform, mutation, and dependency
  from the specification;
- import/replace the hand-maintained `crash_matrix.v1.json` without losing
  historical IDs;
- define canonical result/evidence encoding;
- map every store error variant;
- make every assurance claim reference its complete assumption ledger;
- add positive/negative registry fixtures; and
- add architecture checks for missing/duplicate/dangling relationships.

Exit:

- general registry validation can resolve every core-storage ID;
- no production store operation or error is unregistered;
- every claim has invariant/oracle/suite paths;
- every proof obligation and failure-combination cell has an executable owner;
- Rust and JSON validators agree;
- dependency cycles and dishonest `not applicable` entries reject.

## 5. CSQ-1 — Independent oracles

Depends: `CSQ-0`

Deliver:

```text
crates/dingo-store-model/
tools/core-storage-reference-reader/
```

Work:

- sequential model with exact acknowledgement/uncertainty semantics;
- independent byte reader/scanner;
- canonical observation DTO;
- dependency-firewall CI;
- deliberate implementation/model/reader disagreement fixtures;
- model serialization for minimization/replay; and
- oracle self-tests with hand-built bytes and transitions.

Exit:

- neither oracle imports production store/recovery algorithms;
- known-bad production-like algorithms fail oracle fixtures;
- model and reader independently explain every canonical vector.

## 6. CSQ-2 — Boundary and failure instrumentation

Depends: `CSQ-0`

Deliver:

- generated boundary census;
- failpoints before/after every registered edge;
- short-write, syscall-error, allocator, cancellation, and process barriers;
- child-process crash controller;
- isolated filesystem-image harness;
- static boundary-to-source verification; and
- generated crash-matrix executor; and
- generated ordered-pair and t-wise composed-failure executor.

Exit:

- every boundary is injectable or has an approved external harness;
- adding an unregistered persistence/publication edge fails CI;
- every failpoint proves it was reached; unreachable injection is `fail`, not
  `pass`; and
- every registered compatible failure pair is scheduled or rejected by a
  checked feasibility constraint.

## 7. CSQ-3 — Format exhaustive corpus

Depends: `CSQ-1`, `CSQ-2`

Deliver:

- frozen canonical microframes/microsegments;
- every-bit/byte/truncation/insertion/deletion corpus;
- exhaustive bounded hole corpus;
- structural and multi-fault covering arrays;
- forward/reverse/independent-reader reconciliation; and
- minimized fixture retention.

Exit:

- `CSQ-FMT-*` and applicable `CSQ-DMG-*` pass;
- corpus generation is deterministic and hash-addressed;
- corrupt never becomes verified and healthy islands remain discoverable.

## 8. CSQ-4 — Store model/state machine

Depends: `CSQ-1`, `CSQ-2`

Regression authorities: `DEF-099`, `DEF-100`

Deliver:

- exhaustive publication kernel;
- machine-checked publication, parser-progress, range-safety, and recovery
  idempotence obligations with deliberately false harness controls;
- generated command/state histories;
- exact historical reads, bounded last-complete searches, and tombstone/gap
  policies;
- coverage-aware key/document scan model comparisons;
- model comparison after every step;
- reopen/rebuild/continue transformations;
- coverage report for every transition class; and
- shrinker retaining crash/damage prerequisites.

Exit:

- every registered ordinary state transition is reached;
- `CSQ-ID-*`, `ACK-*`, `PUB-*`, `GEN-*`, `HIST-*`, and `ABS-*` pass;
- every minimized failure is exactly replayable.

## 9. CSQ-5 — Crash, process, filesystem, and device faults

Depends: `CSQ-2`, `CSQ-4`

Regression authority: `DEF-101`

Deliver:

- every-boundary error/abort/short-write matrix;
- ordered compatible-failure pairs and required higher-order covering arrays;
- repeated reopen/retry/healthy-continuation oracle;
- ENOSPC/quota/inode/permission/EIO campaigns;
- memory-buffer corruption, clock discontinuity, path-alias, and mount
  substitution campaigns;
- stale/adversarial writer-lock diagnostics and bounded acquisition campaigns;
- Linux loopback/device-mapper lane;
- supported-platform abrupt-termination lanes;
- filesystem/mount evidence capture; and
- old/new/unknown outcome validator.

Exit:

- every applicable boundary/failure cell ran;
- every applicable composed-failure cell ran;
- every durable receipt survives;
- no failed/unacknowledged operation creates an impossible hybrid;
- no skip is hidden by platform conditions.

## 10. CSQ-6 — Chunk and large-value qualification

Depends: `CSQ-3`–`CSQ-5`

Primary regression authorities: `DEF-098`, `DEF-103`

Deliver:

- exhaustive bounded chunk-generation kernel;
- current-manifest exact-event selection tests;
- repeated large-value rewrite histories;
- chunk locator correctness/rebuild/performance tests;
- payload/manifest/allocation boundary matrix;
- partial/unavailable/conflicting diagnostics; and
- transcript survival journey.

Exit:

- all `CSQ-CHK-*` pass;
- every DEF-098 acceptance test is registry-linked;
- unrelated store growth does not increase point-read bytes examined;
- no old generation poisons current state.

## 11. CSQ-7 — Damage, salvage, and deterministic recovery

Depends: `CSQ-3`–`CSQ-5`

Deliver:

- damage-locality matrix;
- multi-hole/topology campaigns;
- live/reopen/rebuild/reference-reader/salvage differential;
- unsupported/conflicting/encrypted-unavailable cases;
- repeated recovery and healthy-continuation histories; and
- recovery provenance verifier.

Exit:

- all `CSQ-DMG-*` and `CSQ-REC-*` pass;
- absence is never inferred from damage/unavailability;
- all promised surviving units remain independently discoverable.

## 12. CSQ-8 — Derived state, maintenance, backup, and migration

Depends: `CSQ-4`, `CSQ-5`, `CSQ-7`

Regression authority: `DEF-102`

Deliver:

- delete/corrupt/stale/ahead/foreign derived-artifact corpus;
- scan-versus-every-accelerator differential;
- seal/compact/reclaim/tier job state machines;
- scrub non-mutation proof;
- backup/restore differential;
- migration phase interruption; and
- compatibility fixture hooks.

Exit:

- `CSQ-DER-*`, `MNT-*`, `BAK-*`, and `MIG-*` pass;
- derived state never changes authority;
- maintenance interruption retains acknowledged data and explicit coverage.

## 13. CSQ-9 — Concurrency, ownership, limits, and resources

Depends: `CSQ-2`, `CSQ-4`, `CSQ-8`

Deliver:

- Loom/Shuttle bounded kernels;
- native-thread and writer-shard campaigns;
- multi-process writer contention;
- allocator/FD/thread/disk resource injection;
- cancellation and shutdown histories;
- dataset-larger-than-RAM boundedness tests; and
- deadlock/livelock watchdog evidence.

Exit:

- all `CSQ-RES-*` and `CSQ-CON-*` pass;
- no mixed generation or half-publication is observed;
- point and scan operations meet declared work/memory bounds.

## 14. CSQ-10 — Mutation, fuzz, and sanitizers

Depends: `CSQ-3`, `CSQ-4`, `CSQ-6`–`CSQ-9`

Deliver:

- mandatory P0 mutant catalog;
- recurring sentinel-mutant job;
- full core parser fuzz ownership;
- minimized-corpus promotion;
- sanitizer/Miri jobs;
- mutation/equivalence adjudication workflow; and
- cumulative fuzz evidence.

Exit:

- 100% mandatory non-equivalent P0 mutants killed;
- at least 95% all non-equivalent core mutants killed;
- every core untrusted parser has owned scheduled fuzzing;
- no unexplained sanitizer/fuzz finding.

## 15. CSQ-11 — Compatibility, packaged journey, scale, and soak

Depends: `CSQ-5`–`CSQ-10`

Regression authority: `DEF-104`

Deliver:

- immutable released-writer fixture repository;
- old/new/backup/damage/migration matrix;
- clean packaged-artifact torture journey;
- supported platform/filesystem matrix;
- 24-hour weekly campaign;
- 72-hour/billion-operation release campaign; and
- full final reconciliation.

Exit:

- all claimed compatibility/platform edges pass;
- scale exceeds RAM and exercises every registered failure class;
- final rebuild/scrub/backup/restore agrees with the model;
- no unexplained flake, skip, leak, or unreconciled state.

## 16. CSQ-12 — Qualification and evidence

Depends: every mandatory `CSQ-0`–`CSQ-11` package

Deliver:

- `dingo verify --profile dingo-core-storage-v1`;
- canonical evidence-bundle builder;
- versioned assumption/impossibility ledger in the bundle;
- independent bundle verifier;
- qualification evaluator for A2/A3;
- capability/status updater;
- retention/publication policy; and
- release-candidate report.

Exit:

- one clean command produces a valid bundle or exact missing cells;
- every invariant/failure/combination/boundary/platform/version/proof claim is
  evidenced;
- `not_run`, infrastructure failure, retry-to-green, and prose cannot satisfy
  the gate;
- an independent reviewer can validate the result without trusting CI output.

## 17. CI schedule

```text
PR:
  core registry + bounded kernels + changed boundaries + sentinel mutants

nightly:
  all failpoints + generated model + corruption + fuzz/sanitizers + resources

weekly:
  real filesystem faults + mutation + compatibility + 24h torture

release candidate:
  packaged artifacts + platform matrix + full evidence + 72h/1B operations
```

Every lane uploads a report fragment consumable by `CSQ-12`.

## 18. Completion rule

The program is not complete when tests exist. It is complete only when:

```text
CSQ-12 = accept
and
the evidence bundle independently verifies
and
the capability language matches the proven A2/A3 profile
```

Any later P0 storage invariant violation immediately reopens the relevant CSQ
packages and revokes the affected qualification result.
