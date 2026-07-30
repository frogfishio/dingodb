# DingoDB verification implementation plan

Status: developer-ready work-package plan v1.0-draft

Date: 2026-07-30

Normative strategy:
[TESTING_STRATEGY.md](../TESTING_STRATEGY.md)

Living status:
[VERIFICATION_STATUS.md](VERIFICATION_STATUS.md)

Program authority:
[MASTER_DELIVERY_PLAN.md](../MASTER_DELIVERY_PLAN.md)

## 1. Outcome

The program exits when DingoDB can run:

```text
dingo verify --profile embedded-heap --level A2
```

from a clean environment and produce a self-validating evidence bundle that
maps every advertised claim to passed invariants, or refuses qualification
with exact missing evidence.

The runner orchestrates meaningful suites and makes omissions impossible to
hide. It does not manufacture confidence.

## 2. Work-package order

```text
VFY-0 ┬→ VFY-1
      └→ VFY-2
          |
          +→ VFY-3 → VFY-4 → VFY-5 → VFY-6
          +→ VFY-7 → VFY-8
          +→ VFY-9 → VFY-10
          +→ VFY-11
all applicable ─────────────────────────→ VFY-12
```

`VFY-0`–`VFY-2` are M0 work and start immediately.

## 3. VFY-0 — Registries and schemas

Priority: `P0-GATE`

Deliver:

```text
spec/verification/claims-v1.json
spec/verification/suites-v1.json
spec/verification/profiles-v1.json
spec/verification/report-v1.schema.json
spec/verification/vectors/
```

Define claim, invariant, oracle and suite IDs; profiles; assurance levels;
result states; platform/resources; attachments/hashes; skips; infrastructure
failure; and compatibility rules.

Tests:

- positive/negative schema vectors;
- duplicate/unknown ID rejection;
- dependency-cycle rejection;
- missing oracle/suite rejection;
- canonical report encoding; and
- report tampering.

Exit:

- every currently advertised capability is registered at its honest level;
- no public claim is inferred; and
- Rust and JSON validators agree.

## 4. VFY-1 — Runner, preflight and evidence

Depends: `VFY-0`

Deliver:

```text
crates/dingo-verify/
scripts/dingo-verify.sh
```

The Rust crate owns registry validation, dependency resolution, preflight,
isolated work/artifact roots, supervision/deadlines, redacted capture, result
classification, attachment hashes, report generation, and report verification.

The shell wrapper only locates/builds the binary.

Required:

- fail before build when disk/inodes are below minima;
- never use the repository root as destructive storage;
- preserve partial evidence on interrupt;
- distinguish test and infrastructure failures;
- treat conditional skips as `not_run`;
- show the artifact/recovery location; and
- support deterministic suite selection.

Exit:

- synthetic pass/fail/skip/timeout/disk-full fixtures classify correctly;
- report verification detects mutation; and
- interrupted runs remain diagnosable.

## 5. VFY-2 — Repository evidence inventory

Depends: `VFY-0`

Priority: `P0-GATE`

Work:

1. enumerate every test, proof, fuzzer, chaos rig and CI job;
2. map it to claim/invariant/oracle/profile;
3. identify tests with no claim and claims with no tests;
4. identify implementation-shaped oracles;
5. identify conditional tests currently reporting pass when not run;
6. identify targets absent from CI;
7. update `VERIFICATION_STATUS.md`; and
8. update master-plan readiness from evidence.

Exit:

- every current artifact is classified;
- every M0/M1 claim gap has an owner/package;
- status is generated or mechanically checked from registries; and
- the next Heap gap is established from evidence, not prose.

## 6. VFY-3 — Pure semantics and proof

Depends: `VFY-2`

Scope:

- SDA;
- Heap admission/isolation;
- DQL/DRE predicate kernel when implemented;
- Atomic decision kernel when implemented;
- cursor/rank mathematics; and
- Evidence verification.

Deliver:

- proof obligations and bounds;
- slow executable oracles;
- Kani/Verus/TLA+ registry integration;
- required-tool enforcement; and
- hashed proof reports.

Tests:

- a deliberate false-lemma fixture rejects;
- assumption/bound changes invalidate evidence;
- unavailable proof tools produce `not_run`; and
- executable oracles and proof vectors agree.

Exit:

- every proof claim states exact scope/bounds;
- Heap proofs are connected to claims; and
- optional local tools cannot masquerade as completed proof.

## 7. VFY-4 — Format, corruption and salvage

Depends: `VFY-2`

Deliver:

- canonical frame/segment/chunk/store artifacts;
- every-byte and bounded every-bit mutation;
- structural insertion/deletion/duplication/reorder/truncation;
- multi-hole campaigns;
- forward/reverse reconciliation oracle;
- unsupported-version preservation; and
- SDA examination equivalence.

Exit:

- corrupt never becomes verified;
- every promised healthy island remains discoverable;
- hole bounds/provenance remain honest;
- scans terminate within resource bounds;
- failures retain minimized fixtures; and
- V1 reaches its declared A2 subset.

## 8. VFY-5 — Store model and crash

Depends: `VFY-2`, `VFY-4`

Deliver:

- independent sequential event/store model;
- generated operation histories;
- failpoint registry for every durable boundary;
- old/new/unknown crash oracle;
- ENOSPC/quota/permission/short-write/I/O-error environments;
- process-abort and repeated-reopen campaigns;
- derived-state rebuild differential checks; and
- near-full disk campaign.

Operations include create/open, put/delete/chunk, seal/checkpoint/compact,
indexes, backup/restore, scrub, migrate, and tier movement.

Exit:

- no publication boundary lacks a failpoint or justification;
- generated histories match the model;
- the full matrix cannot silently skip; and
- V2 A2 evidence is reproducible.

## 9. VFY-6 — Heap security and noninterference

Depends: `VFY-2`, `VFY-3`, `VFY-5`

Deliver:

- generated two/many-Heap histories;
- same-name/same-key differential cases;
- issue/expiry/blacklist/grace/cycle model;
- authority publication crash matrix;
- channel/audience/epoch replay corpus;
- no-existence-leak observation model;
- backup/restore/new-identity attacks;
- mixed-ownership salvage; and
- scheduled `heap_ownership` fuzzing.

Exit:

- every operation has wrong-Heap/right/epoch cases;
- foreign activity cannot change admitted observation class;
- key invalidation matches the model;
- the Heap fuzz target runs in CI; and
- V3 reaches its declared A2 scope.

## 10. VFY-7 — Query, index and cursor differential

Depends: `VFY-2`, `VFY-5`

Deliver:

- full-scan/filter/sort oracle;
- generated JSON/bytes/history datasets;
- index versus scan;
- embedded versus remote;
- page concatenation versus frozen result;
- damaged/offline/incomplete coverage;
- resource truncation;
- token forgery/binding/expiry; and
- future DRE/DDA/DOW adapters.

Exit:

- every accelerator has differential evidence;
- incomplete coverage never proves absence;
- paging neither duplicates nor loses rows;
- forged/cross-view cursors reject before execution; and
- query work remains bounded.

## 11. VFY-8 — Server and protocol

Depends: `VFY-2`, `VFY-5`–`VFY-7`

Deliver:

- framed protocol state-machine generator;
- malformed/oversized/slow-client corpus;
- connection churn/overload;
- TLS/mTLS/certificate matrix;
- auth/admission negative matrix;
- ambiguous response-loss/retry proxy;
- graceful drain; and
- packaged client/server parity journey.

Exit:

- hostile load remains bounded;
- ambiguous failure cannot duplicate a mutation;
- every active RPC has golden/adversarial vectors; and
- V4 reaches its declared level.

## 12. VFY-9 — Fuzz, sanitizer and concurrency

Depends: `VFY-2`

Deliver:

- fuzz ownership registry;
- targets for every untrusted parser;
- seed/minimized corpora;
- all targets in scheduled CI;
- continuous fuzz integration;
- Miri/sanitizer jobs;
- Loom/Shuttle models;
- mutation testing of critical decisions; and
- coverage publication.

Exit:

- no parser is unowned;
- cumulative fuzz duration is visible;
- crashes/hangs/OOM/work bombs become regressions;
- critical concurrency kernels have schedule evidence; and
- surviving critical mutations are defects.

## 13. VFY-10 — Compatibility and packaged journeys

Depends: `VFY-1`, `VFY-2`, `VFY-4`, `VFY-5`

Deliver:

- released fixture/archive repository;
- old/new reader/writer matrix;
- backup/evidence/migration fixtures;
- CLI/config/protocol golden outputs;
- install/upgrade/rollback journeys;
- OS/filesystem matrix; and
- clean packaged-artifact harness.

Exit:

- fixtures originate from real released binaries;
- promised edges run;
- unsupported behavior is explicit/evidence-preserving; and
- no source-private helper is required.

## 14. VFY-11 — Scale, soak and performance correctness

Depends: `VFY-1`, relevant functional lanes

Deliver:

- testrig in scheduled CI;
- datasets larger than RAM;
- sustained mixed workload;
- near-full disk;
- long history/index/collection counts;
- maintenance under foreground work;
- multi-hour restart/damage campaigns;
- telemetry/evidence failure campaigns;
- controlled benchmarks; and
- full final verification.

Exit:

- reports retain configuration, seed, time series and final state;
- performance includes achieved semantics;
- bounds are checked, not inferred; and
- regression thresholds are versioned.

## 15. VFY-12 — Qualification and release evidence

Depends: every lane required by selected profile/level

Deliver:

- release-candidate matrix runner;
- profile/level evaluator;
- evidence-bundle verifier;
- capability-matrix updater;
- release-note evidence links;
- exclusion validator; and
- retention/publication policy.

Exit:

- one command produces a bundle or exact missing evidence;
- `not_run`/infrastructure failure never satisfies a gate;
- capability labels cannot exceed evidence;
- the report is reproducible; and
- an independent reviewer can validate it without trusting CI prose.

## 16. Distributed extension

Deferred until cluster product admission:

```text
VFY-13 — multi-process history harness
VFY-14 — network/storage fault campaigns
VFY-15 — rolling upgrade and reconstruction
VFY-16 — independent checking and cluster qualification
```

In-process deterministic cluster tests remain active regression tests.

## 17. Immediate queue

```text
1. VFY-0 registries and report schemas
2. VFY-2 repository evidence inventory
3. VFY-1 preflight/runner skeleton
4. integrate M0 delivery-status checking
5. close discovered M1 Heap evidence gaps
```

`VFY-2` may start when the VFY-0 identifier/schema shape freezes; the runner
does not need to be complete.

## 18. CI target structure

```text
verify-registry
verify-pr-v0-v3
verify-nightly-crash-corruption
verify-nightly-fuzz
verify-nightly-sanitizers
verify-weekly-soak
verify-weekly-compat
verify-release-candidate
```

Each uploads `dingo-verification-report-v1` or a hash-addressed fragment merged
by `VFY-12`.

## 19. Completion rule

The program is not complete when the runner exists. It is complete for a
profile/level only when the strategy’s assurance case is populated and the
release gate accepts it.
