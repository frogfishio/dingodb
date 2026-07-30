# DingoDB master delivery plan

Status: **definitive execution plan v1.1**

Effective: 2026-07-30

Owner: DingoDB product and engineering program

Testing authority:
[TESTING_STRATEGY.md](TESTING_STRATEGY.md),
[doc/VERIFICATION_IMPLEMENTATION_PLAN.md](doc/VERIFICATION_IMPLEMENTATION_PLAN.md),
and
[doc/VERIFICATION_STATUS.md](doc/VERIFICATION_STATUS.md).

## 1. Authority

This is the controlling document for **what DingoDB builds next and in what
order**.

Other documents retain their narrower authority:

- specifications define semantics;
- implementation plans define individual package contents;
- `DEFECTS.md` defines defects and production gates;
- capability documents define what may be claimed;
- this document defines program order, priority, entry, and exit.

If another roadmap conflicts with this document:

1. safety and correctness requirements still win;
2. normative feature semantics remain unchanged;
3. delivery order in this document wins; and
4. the conflict must be corrected rather than worked around.

No new product subsystem may enter active development merely because it has a
specification. It must be admitted by this plan or by an explicit amendment to
this plan.

## 2. Product target

The first product target is:

> A self-assessed, Heap-confined single-node database that an ordinary
> developer can safely choose instead of SQLite plus loose files.

That first adoption gate is reached at `M2`. DRE, Atomics, and exact navigation
then create the product-defining DingoDB proposition at `M3`–`M5`. `M6`
qualifies the combined operational product.

Cluster, vector search, geospatial search, and broad archive expansion are not
part of the immediate target.

## 3. Priority law

Work is selected using this strict order:

| Priority | Meaning | May preempt active work? |
|---|---|---|
| `P0-SAFETY` | Data loss, isolation failure, authority bypass, false durability, secret disclosure | Yes, immediately |
| `P0-GATE` | Blocks the active release exit | Yes |
| `P1-PATH` | Next package on the engine critical path | After P0 |
| `P1-TRUST` | Evidence, telemetry, crash, fuzz, recovery, compatibility | Alongside the active path |
| `P1-DX` | SDK, CLI, Studio path for an already real capability | Alongside the active path |
| `P2-NEXT` | Pure oracle/spec preparation for the next admitted release | Only within the stated allowance |
| `P3-FUTURE` | Cluster, retrieval, archive, research, polish | No |

Within the same priority, work with the earliest package number runs first.

A later package may not be pulled forward because it is interesting, easy, or
already partially implemented.

## 4. Package state

Every package has exactly one state:

```text
not_started
ready
active
blocked
accept
deferred
```

Rules:

- `ready` means every dependency and entry condition is satisfied.
- `active` means an owner is producing its required artifacts.
- `blocked` requires a named unsatisfied dependency or defect.
- `accept` requires every exit test and evidence item.
- code existing in the repository does not by itself mean `accept`.
- no package may be accepted with an unresolved release-gate defect.
- no more than one engine-critical package is `active` per implementation
  team.

The machine-readable/living scoreboard remains:
[doc/NEXT_BUILD_STATUS.md](doc/NEXT_BUILD_STATUS.md).

It MUST be expanded to contain every active-master-plan package and updated in
the same change that changes a package state.

## 5. Master stage order

```text
M0  Program Truth
 ↓
M1  Heap Application Ready
 ↓
M2  Trustworthy Core Early Access
 ↓
M3  Mathematical Documents
 ↓
M4  Atomic Integrity
 ↓
M5  Exact Navigation at Scale
 ↓
M6  Single-Node Production Candidate
 ↓
E1+ Competitive Expansion
```

Stages do not overlap at the public-release level. Limited package preparation
may overlap only where this document explicitly permits it.

## 6. M0 — Program Truth

Release outcome:

> The repository has one accurate execution queue and one honest account of
> what the Heap implementation has already proved.

Priority: `P0-GATE`

### M0-1 — Whole-database evidence inventory

State: **ready — first task**

Work:

1. establish the `VFY-0` claim/suite/profile identifiers;
2. inventory every test, proof, fuzzer, chaos rig and CI lane;
3. run the quick and full Heap surfaces where infrastructure permits;
4. inspect `hp010-matrix-v1.json`;
5. map evidence to product claims and `HAR-0` through `HAR-7`;
6. mark each requirement `accept`, `partial`, `missing`, or `not_run`;
7. link the oracle, test, proof, fixture and source revision; and
8. do not infer acceptance from counts or documentation prose.

Required outputs:

- updated Heap qualification matrix;
- updated `doc/VERIFICATION_STATUS.md`;
- claim/suite gap report;
- list of genuinely missing work; and
- discrepancies raised as named defects.

Exit:

- Kani, Verus, architecture checks, tests, verification status, and the matrix
  tell the same story.

### M0-2 — Scoreboard reconciliation

Depends: `M0-1`

Work:

- update `doc/NEXT_BUILD_STATUS.md` with the observed HAR state;
- add `DEL`, `TEL`, `DST`, trust-gate, and release rows used by this plan;
- preserve source revision and evidence columns;
- add `last_verified` and `blocked_by` fields if the table remains Markdown;
  and
- reject invalid state transitions in the program verification script.

Exit:

- no completed work appears `not_started`;
- no partial work appears `accept`; and
- every `ready` package has satisfied dependencies.

### M0-3 — One program-status check

Depends: `M0-2`

Deliver:

```text
scripts/verify-delivery-status.sh
```

It checks:

- allowed package states;
- unique package IDs;
- dependency satisfaction;
- accepted-package evidence links;
- no active later engine stage while an earlier gate is open;
- capability labels against qualification truth; and
- required specification/plan links.

M0 exit:

- `verify-delivery-status.sh` passes in CI; and
- `HAR-1` is either `ready` or a more exact missing predecessor is named.

## 7. M1 — Heap Application Ready

Release outcome:

> An application can create, secure, operate, back up, restore, and retire one
> Heap without a global data path or manual fixture.

Priority: `P1-PATH`

Normative package plan:
[doc/HEAP_APPLICATION_READY_PLAN.md](doc/HEAP_APPLICATION_READY_PLAN.md).

The ordinary Rust API, collection provisioning, and DQL Application Core
vertical slice is governed by
[doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md).
Its `APP-1` implements `HAR-1`; `APP-2` through `APP-8` supply the application
portion of `HAR-6` and its release evidence. This does not alter the HAR gate
order: work may be prepared in parallel only where the APP plan and this
master plan both permit it.

### Required order

| Order | Package | Deliverable | Hard exit |
|---:|---|---|---|
| 1 | `HAR-0` | Heap truth cleanup | qualification sources agree |
| 2 | `HAR-1` | collection creation | embedded/remote parity, retry, crash, two-Heap tests |
| 3 | `HAR-2` | local Heap creation ceremony | clean CLI creation with phase crash recovery |
| 4 | `HAR-3` | application-key lifecycle | issue/blacklist/grace/cycle journey |
| 5 | `HAR-4` | qualified remote posture | HeapKey listener is default; legacy explicit |
| 6 | `HAR-5` | Heap operations | Heap-scoped backup/restore/retire/scrub |
| 7 | `HAR-6` | ordinary SDK and CLI journey | no legacy imports or architecture knowledge |
| 8 | `HAR-7` | release evidence | complete journey and honest label |

### M1 critical journey

The gate runs from an empty temporary location:

```text
create deployment
create Heaps A and B
issue separate CRUD keys
create collection "records" in both
put the same key with different values
query, page, inspect history, and manage an index
prove A cannot observe B
blacklist one key
cycle A authority and prove all old A keys inert
back up A
restore it as C with a new HeapId
prove A keys and cursors are inert against C
retire A
```

### M1 parallel allowance

After `HAR-0` and `APP-0` freeze the application contract:

- `APP-4` canonical predicate/plan work and `APP-5` pure compiler work may
  proceed alongside `HAR-1` through `HAR-3`;
- they may not publish a remote query surface or claim Application Core
  conformance before their APP dependencies and M1 security gates accept.

After `HAR-3` freezes certificate, authority epoch, and identity shapes:

- `DEL-0` registry drafting may begin;
- `TEL-0` registry drafting may begin; and
- `DST-000` repository/toolchain scaffolding may begin.

They may not publish a live product surface before M1 exits.

### M1 non-blocking external item

An independent Heap review remains desirable but does not block M1. Without
it, the release is explicitly:

> machine-checked, adversarially tested, self-assessed, awaiting independent
> review.

M1 exit:

- all applicable HAR packages are `accept`;
- the critical journey passes locally and in CI;
- `qualified` and public wording match the qualification matrix; and
- M2 foundation packages become `ready`.

## 8. M2 — Trustworthy Core Early Access

Release outcome:

> A careful outsider can replace SQLite plus loose JSON/blob files with
> DingoDB, then survive crash, damage, backup/restore, encryption-key
> operation, and upgrade without reading DingoDB internals.

M2 has one blocking product gate and three parallel enabling lanes. Evidence,
Telemetry, and Studio are important, but their complete feature sets do not
all block DRE. Only the minimum portions named below are M2 blockers.

### M2-A — Evidence foundation

Priority: `P1-TRUST`

Order:

```text
DEL-0 → DEL-1 → DEL-2 → DEL-3
                  └────→ DEL-4
DEL-3 + DEL-4 ─────────→ DEL-7
```

Required before M3 rule activation:

| Package | Result |
|---|---|
| `DEL-0` | registries, canonical schemas, golden vectors |
| `DEL-1` | pure types, canonicalizer, verifier |
| `DEL-2` | survival format and SDA examination |
| `DEL-3` | per-Heap durable store, recovery, head |

Parallel/non-blocking until required by their consumer:

- `DEL-4` must exit before signed evidence is advertised;
- `DEL-7` must exit before Studio Evidence or public audit browsing is
  advertised;
- `DEL-5` enters M4 because it requires Atomics;
- `DEL-6`, `DEL-8`, `DEL-9`, and applicable `DEL-11` close in M6;
- `DEL-10` enters the cluster program.

M2-A exit:

- the durable evidence substrate exists for later DRE/Atomic decisions;
- its pure verifier and damage behavior pass; and
- it is Heap-confined.

### M2-B — Telemetry foundation

Priority: `P1-TRUST`

Minimum M2 order:

```text
TEL-0 → TEL-1 → TEL-2
```

Required result:

- closed bounded registries;
- Ratatouille is the qualified telemetry path;
- no payloads, credentials, raw queries, or arbitrary error text;
- disconnect/full collector cannot block database work.

`TEL-3`, `TEL-4`, and `TEL-8` continue in parallel during M2–M5 and become M6
gates. Instrumentation for DRE, Atomics, Direct Access, and Order Wavelets lands
with its subsystem rather than in advance.

M2-B exit:

- the bounded emission path and its overhead/drop behavior pass; and
- later subsystems have one safe telemetry mechanism to integrate with.

### M2-C — Studio Explorer

Priority: `P1-DX`, **parallel product lane; not an M2 engine gate**

Order:

```text
DST-000 → DST-001 → DST-002 → DST-003
        → DST-004 → DST-005 → DST-006
```

Required result:

- signed/hardened local Tauri shell;
- generated closed IPC contract;
- credentials remain in Rust and the OS vault;
- immutable Heap-bound workspaces;
- collections, records, history, bytes, damage, holes, and coverage;
- safe edit/delete confirmation;
- no master-key, raw network escape, wildcard Heap, shell, file, or HTTP
  command.

M2-C release:

- a hostile document/package/server corpus cannot escape the renderer
  boundary; and
- the M1 critical journey can be observed through Studio without weakening it.

Failure to complete Studio S1 does not block M3. It blocks the Studio S1
release and remains visible as a DX gap.

### M2-D — Early-access trust and distribution

Priority: `P0-GATE`

Required existing work:

| Work | Source |
|---|---|
| crash and power-loss residuals | `DEF-022` and §16.1–16.2 |
| SDK MVP parity | `DEF-080` |
| executable outsider journeys | `DEF-082` |
| release packaging | `DEF-083` |
| compatibility/deprecation policy | `DEF-084` |
| hostile parser fuzzing | `DEF-091` |
| coverage/model evidence | `DEF-092` |
| reproducible benchmarks | `DEF-093` |

Additional mandatory product work:

- qualified encryption at rest for JSON, bytes, metadata, indexes, backup, and
  Evidence material in the supported profile;
- protected local key-provider operation, rotation, backup, and loss behavior;
- disk-full and read-only recovery;
- compaction and interrupted-maintenance recovery;
- bounded streaming larger than RAM;
- ordinary stable errors and durability receipts;
- a five-minute Rust journey with zero mandatory service/configuration; and
- explicit “when SQLite is still the better choice” documentation.

Required release drill:

```text
install
→ create Heap
→ write JSON and bytes
→ kill process
→ reopen
→ punch controlled damage
→ inspect explicit holes
→ run scrub
→ back up
→ restore under a new Heap identity
→ verify Evidence
```

M2 exit:

- M2-D passes;
- `DEL-0`–`DEL-3` pass before M3 rule activation;
- `TEL-0`–`TEL-2` pass before new performance claims;
- Rust and CLI quickstarts use the same qualified path; and
- the release label is no stronger than the passed evidence.

Studio S1, richer Evidence browsing, and full telemetry instrumentation
continue in parallel but do not hold the mathematical engine idle.

## 9. M3 — Mathematical Documents

Release outcome:

> Every committed document under an active DRE ruleset satisfies a finite,
> canonical, independently examinable invariant.

Priority: `P1-PATH`

Normative plan:
[doc/DRE_IMPLEMENTATION_PLAN.md](doc/DRE_IMPLEMENTATION_PLAN.md).

Order:

| Order | Package | Result |
|---:|---|---|
| 1 | `DRE-0` | semantic oracle and executable corpus |
| 2 | `DRE-1` | parser and canonical AST |
| 3 | `DRE-2` | normalization and Invariant Core |
| 4 | `DRE-3` | canonical artifact and independent verifier |
| 5 | `DRE-4` | document-local activation and enforcement |

Mandatory integrations:

- shared predicate semantics with DQL;
- JSON Schema → DRE translation;
- SQL-ish → DQL translation against the frozen DQL grammar;
- Evidence records for validate/activate/replace/reject;
- `TEL-5` DRE collection points;
- Studio `DST-007`, plus the document-local subset of `DST-013`; and
- exact impact preview before activation.

Stable rejection in M3:

- uniqueness;
- parent existence;
- delete restriction;
- cross-document cardinality;
- transition predicates; and
- arbitrary user code.

M3 exit:

- every source form normalizes deterministically;
- fast enforcement equals the slow oracle;
- unknown/damaged ruleset state fails closed;
- activation is Heap-confined and crash-safe; and
- the Studio and Rust journeys show the same decision evidence.

## 10. M4 — Atomic Integrity

Release outcome:

> Within one Heap, DingoDB commits bounded serializable changes with durable
> decision evidence and enforces declared cross-document integrity.

Priority: `P1-PATH`

Normative plan:
[doc/ATOMICS_IMPLEMENTATION_PLAN.md](doc/ATOMICS_IMPLEMENTATION_PLAN.md).

Order:

```text
ATM-0 → ATM-1 → ATM-2 → ATM-3 → ATM-4 → ATM-5
                          |
                          +→ REL-0 → REL-1 → REL-2 → REL-3 → REL-4
                          |
                          +→ DRE-5 → DRE-6
                          |
                          +→ DEL-5
```

Required semantics:

```text
one Heap
bounded members
declared read/write set
finite validation
stable operation identity
one serializable decision
```

Required integrations:

- mandatory Evidence Ledger coupling;
- `TEL-5` Atomic/relationship collection points;
- Studio Atomic preview/status/evidence and relationship views;
- crash recovery at every publication phase;
- replay returns the original decision;
- damaged/missing decision evidence produces `unknown`, never guessed commit;
  and
- no cross-Heap or cross-partition implication.

M4 exit:

- LocalHeap Atomic linearizability/model checks pass;
- parent-exists, optional reference, restrict delete, uniqueness, and bounded
  cardinality pass their adversarial corpus;
- transition/cross-document DRE equals its oracle; and
- every completed decision remains independently examinable.

## 11. M5 — Exact Navigation at Scale

Release outcome:

> Supported queries move directly to exact ranked and sorted regions without
> walking the skipped prefix or sorting the complete result set.

Priority: `P1-PATH`

Normative plans:
[doc/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md](doc/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md)
and
[doc/ORDER_WAVELET_IMPLEMENTATION_PLAN.md](doc/ORDER_WAVELET_IMPLEMENTATION_PLAN.md).

### M5-A — Direct Access

```text
DDA-0 → DDA-1 → DDA-2 → DDA-3 → DDA-4
```

Exit:

- exact natural and supported filtered rank;
- Heap/view/plan-bound selection identity;
- authenticated cursor;
- explicit damage and coverage;
- stable refusal outside the admitted domain; and
- no offset-walk fallback.

### M5-B — Order Wavelets

```text
DOW-0 → DOW-1 → DOW-2 → DOW-3 → DOW-4
```

Exit:

- exact counted scalar order;
- immutable and mutable paths;
- bounded base/delta compaction;
- deterministic ties;
- damage-aware coverage; and
- differential equality with the slow sort oracle.

Mandatory integrations:

- DQL query/profile identity;
- SDK and remote cursors;
- Studio `DST-008`;
- query/index telemetry; and
- disclosed `rank k` work and memory benchmarks.

M5 exit:

- Direct Access and Order Wavelet release evidence pass;
- `start=100001` examples use Direct Access or reject explicitly;
- no public API describes cursor walking as direct rank; and
- Studio can explain the plan, view, coverage, cursor, and refusal reason.

## 12. M6 — Single-Node Production Candidate

Release outcome:

> One named single-node deployment profile has a repeatable, supportable,
> evidence-backed release case.

Priority: `P0-GATE`

This stage adds no new product subsystem.

Required completion:

### Evidence

```text
DEL-4 → DEL-6 → DEL-7 → DEL-8 → DEL-9 → applicable DEL-11
```

### Telemetry

```text
TEL-3 → TEL-4 → TEL-5 → TEL-6 → TEL-8 → TEL-9 → TEL-10
```

### Studio

```text
DST-009 → DST-010 → DST-011 → DST-012 → DST-013 → DST-015
```

`DST-014` cluster remains deferred.

### Production gates

Close every applicable item in:

- `DEFECTS.md` §16.1 data safety;
- §16.2 single node;
- §16.4 security;
- §16.5 operations; and
- §16.6 product/compatibility.

Also require:

- bounded remote cursor/server concurrency;
- wire/RPC compatibility decision;
- signed packages and SBOM;
- install, upgrade, rollback, backup, restore, authority cycle, evidence
  verification, damage, and incident drills;
- supported OS/runtime/MSRV matrix;
- capacity and SLO disclosure;
- public security contact/disclosure process; and
- no unresolved critical/high finding in the review level being claimed.

External-review rule:

- no external review: `self-assessed production candidate`;
- completed and dispositioned external review: the exact reviewed claim may be
  used;
- the software is not technically blocked from progressing while procurement
  is unavailable.

M6 exit:

- one profile can be released and supported without relying on unpublished
  repository knowledge;
- every other profile retains its lower maturity label; and
- the expansion program may begin.

## 13. Competitive expansion

Expansion begins only after M6, unless a written market decision amends this
plan.

Default order:

| Order | Program | Reason |
|---:|---|---|
| `E1` | massive-retention product | core fifteen-year survival proposition |
| `E2` | deterministic text search | rediscover retained material through a general derived-index substrate |
| `E3` | cluster product | Couchbase/distributed-document competitive tier |
| `E4` | exact vector, then segmented ANN/hybrid | semantics before approximation |
| `E5` | geospatial | narrower general workload |

`E1` and `E3` may swap only through an explicit choice:

- archive-first targets the long-retention market;
- cluster-first targets the distributed-document market.

Archive-first is the default. Text follows immediately because “remember it
for fifteen years” is incomplete without a practical way to rediscover it.

## 14. Permitted preparation

The following preparation may occur without changing release order:

| When | Permitted |
|---|---|
| after `HAR-3` | `DEL-0`, `TEL-0`, `DST-000` drafting/scaffold |
| after shared DRE predicate semantics freeze | `DDA-0` oracle work |
| after `DRE-2` | `ATM-0` oracle/profile work |
| after `DDA-3` order identity freezes | `DOW-0` oracle work |
| during M6 | E1 archive-adapter/profile specification and E2 common-index substrate specification only |

Preparation means pure models, corpora, schemas, and design validation. It
does not mean publishing APIs, starting migrations, or claiming capability.

## 15. Explicit deferrals

Before M6, the following are `P3-FUTURE`:

- cluster feature expansion not required to fix a safety defect;
- vector and geospatial implementation;
- native cloud archive, broad erasure coding, and lifecycle expansion;
- Studio cluster views;
- global/cross-Heap transactions;
- RBAC database;
- arbitrary scripting;
- arbitrary offset pagination;
- new query languages;
- adaptive-index research without a current release gate;
- dashboard decoration beyond the qualified telemetry registry;
- unmeasured primary-index micro-optimization; and
- marketing work that describes an unaccepted package as delivered.

## 16. Work allocation

Until M1 exits:

```text
70%  active HAR package
15%  qualification, crash, fuzz, and release evidence
15%  permitted Evidence/Telemetry/Studio preparation
```

During M2:

```text
55%  trustworthy embedded/Heap release gate
15%  minimum Evidence foundation
10%  minimum Telemetry foundation
10%  Studio Explorer parallel lane
10%  crash/fuzz evidence beyond the active gate
```

During M3–M5:

```text
50%  active engine stage
20%  Evidence/Telemetry/trust integration
20%  SDK/CLI/Studio integration
10%  permitted next-stage oracle and corpus
```

During M6:

```text
60%  unresolved safety/security/recovery/release gates
20%  Evidence and Telemetry qualification
20%  Studio Operations/Integrity and packaging
```

For a single developer, percentages become interleaving order:

1. engine/gate package;
2. its evidence and telemetry;
3. its SDK/CLI/Studio surface;
4. its qualification;
5. only then the next engine package.

## 17. Starting queue

This is the current executable queue:

| Queue | Package | State now | Action |
|---:|---|---|---|
| 1 | `M0-1` | `accept` | evidence inventory done |
| 2 | `M0-2` | `accept` | scoreboard reconciled |
| 3 | `M0-3` | `accept` | `verify-delivery-status.sh` + CI/quality wire-up |
| 4 | `APP-0` | `active` | **principal track:** CORE_APPLICATION_API contract freeze |
| 5 | `APP-1` | board backlog | after APP-0; implements collection create (HAR-1 capability) |
| 6 | `HAR-0` | `ready` (board backlog) | residual; do not steal labor from APP-0 |
| 7 | `HAR-2`…`HAR-7` | board backlog | re-queue after principal APP track needs them |

The next task is **`APP-0`** (CORE plan). Live scoreboard: [doc/NEXT_BUILD_STATUS.md](doc/NEXT_BUILD_STATUS.md).

No developer should start DRE, Atomics, Direct Access, Order Wavelets, search,
or cluster product work from this queue.

## 18. Package handoff template

Before starting any package, record:

```text
package:
owner:
state: active
started:
source_base:
dependencies:
normative_sections:
deliverables:
required_tests:
required_evidence:
known_defects:
capability_label_before:
capability_label_after:
```

At acceptance, add:

```text
completed:
source_revision:
evidence_links:
benchmark_links:
open_follow_ons:
release_gate_effect:
```

Follow-ons do not keep a package active unless they violate its stated exit.
They enter the queue at their real priority.

## 19. Amendment rule

Changing the order requires a patch to this document that states:

1. the new order;
2. the product reason;
3. the dependency or market evidence;
4. what is delayed;
5. what compatibility/security risk is introduced; and
6. which scoreboards and capability claims change.

Chat agreement, partial implementation, or developer preference alone does not
change the plan.

## 20. Definitive summary

```text
NOW:
M0-1 evidence inventory
→ M0-2 scoreboard
→ M0-3 enforcement
→ HAR-0…HAR-7

THEN:
trustworthy SQLite-replacement core
+ minimum Evidence substrate
+ minimum bounded Telemetry path
+ Studio Explorer in parallel, not as an engine gate

THEN:
DRE document invariants
→ LocalHeap Atomics and relationships
→ Direct Access
→ Order Wavelets
→ single-node production-candidate qualification

ONLY AFTER THAT:
massive retention
→ text
→ cluster product
→ vector
→ geospatial
```

Start with `M0-1`. Its output determines which apparent Heap gaps are real and
prevents already completed work from being rebuilt.
