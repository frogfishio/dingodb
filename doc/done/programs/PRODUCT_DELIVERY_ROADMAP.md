# Residiuum critical product delivery roadmap

Status: **archived strategic summary — not an execution authority**

Execution order is governed by
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md).

Date: 2026-07-30

Audience: product owners, engineering leads, implementers

For package selection, stage entry, exact priority, and the current starting
queue, use [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md). If this strategic
summary conflicts with the master plan, the master plan wins.

Companions:
[NEXT_BUILD_PLAN.md](./NEXT_BUILD_PLAN.md),
[doc/wip/status/NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md),
[doc/done/programs/PRIME_TIME_PLAN.md](./PRIME_TIME_PLAN.md),
[doc/todo/heap-application-ready/HEAP_APPLICATION_READY_PLAN.md](../../todo/heap-application-ready/HEAP_APPLICATION_READY_PLAN.md),
[PRODUCT_DEFICIENCIES.md](../../reference/product/PRODUCT_DEFICIENCIES.md),
[MUST_ADD.md](../../todo/application-baseline/MUST_ADD.md),
[EVIDENCE_LEDGER_SPEC.md](../../todo/evidence/EVIDENCE_LEDGER_SPEC.md),
[TELEMETRY_SPEC.md](../../todo/telemetry/TELEMETRY_SPEC.md), and
[doc/todo/studio/STUDIO_IMPLEMENTATION_PLAN.md](../../todo/studio/STUDIO_IMPLEMENTATION_PLAN.md).

## 1. Decision

Residiuum has one engine critical path and two supporting delivery lanes:

```text
ENGINE
Heap Application Ready
        ↓
Application Baseline
        ↓
Trustworthy SQLite-replacement core
        ↓
document-local RRE
        ↓
LocalHeap Atomics
        ↓
relationships + cross-document RRE
        ↓
Direct Access
        ↓
Order Wavelets

TRUST                              PRODUCT / DX
minimum Evidence substrate        Rust SDK + CLI journey
bounded Telemetry path            Studio foundation
crash + fuzz + recovery           Studio Explorer
compatibility + release evidence  Studio Workbench
security qualification            Studio Integrity + Operations
```

The three lanes advance together, but only the engine arrows and the minimum
trust cuts named by the master plan are hard dependencies. Studio and broader
observability may begin early; they do not hold the engine idle.

The first strategic target is:

> A self-assessed, single-node, Heap-confined document database that is easy to
> operate, mathematically enforces declared integrity, navigates large result
> sets without offset walking, and exposes its evidence and uncertainty.

Cluster, text/vector/geospatial retrieval, and native archive scale are not on
the R0–R6 engine path. Massive retention is the first expansion program
because it is part of the founding proposition.

## 2. Why the previous plans need this document

The repository contains several correct but differently scoped plans:

- `DELIVERY_PLAN.md` records the completed Stages 0–9 construction program.
- `NEXT_BUILD_PLAN.md` orders the post-Heap mathematical engine.
- `PRIME_TIME_PLAN.md` orders production-readiness gates.
- the Evidence, Telemetry, and Studio documents each contain their own package
  order.

None of them alone answers which complete product slices should ship next.
This document is the integration layer. It does not replace their normative
package definitions.

## 3. Criticality classes

| Class | Meaning |
|---|---|
| **C0** | Blocks the next usable release or a required invariant |
| **C1** | Blocks the product-defining single-node proposition |
| **C2** | Blocks a strong production-candidate claim |
| **C3** | Competitive expansion after the single-node proposition is real |
| **Deferred** | Must not displace C0–C2 work |

Criticality is not synonymous with engineering difficulty. A small missing
journey or recovery test may be C0. A sophisticated ANN index may be Deferred.

## 4. Release sequence

### R0 — Program truth

Criticality: **C0**

Purpose:

> Make plans, capability labels, qualification data, and implementation state
> agree before starting another subsystem.

Deliver:

- complete `HAR-0`;
- update `doc/wip/status/NEXT_BUILD_STATUS.md` from actual Heap evidence;
- distinguish `implemented`, `self-assessed`, `machine-checked`, and
  `independently reviewed`;
- keep `qualified=false` while the mandatory qualification matrix says false;
- identify every old flat/raw entry point as disabled, explicit legacy, or
  test-only; and
- make one command produce the release-gate status.

Exit:

- no completed Heap package is shown as `not_started`;
- no unimplemented package is shown as accepted;
- Kani, Verus, architecture checks, and the qualification matrix agree; and
- the next implementer can select work from one accurate scoreboard.

R0 is deliberately short. It prevents expensive work from being selected from
stale documents.

### R1 — Heap Application Ready

Criticality: **C0**

Packages:

```text
HAR-1 → HAR-2 → HAR-3 → HAR-4 → HAR-5 → HAR-6 → HAR-7
```

The practical order is:

1. `HAR-1` — collection creation;
2. `HAR-2` — local Heap creation ceremony;
3. `HAR-3` — issue, inspect, blacklist, grace, and authority cycling;
4. `HAR-4` — qualified HeapKey listener becomes the normal remote posture;
5. `HAR-5` — Heap backup, restore, retire, scrub, and lifecycle status;
6. `HAR-6` — ordinary Rust, CLI, and remote journeys; and
7. `HAR-7` — release evidence and honest capability label.

Why this is first:

- every later artifact must be bound to one `HeapId`;
- RRE activation, Atomic decisions, evidence, telemetry disclosure, cursors,
  indexes, and Studio sessions all depend on the Heap boundary;
- implementing around incomplete creation/key/operation paths would create
  compatibility debt at the most security-sensitive boundary.

External review is not a code dependency and is not required to ship the
self-assessed release. Until it occurs, the permitted claim remains:

> Heap isolation is implemented, machine-checked, adversarially tested, and
> awaiting independent review.

R1 exit journey:

```text
create two Heaps
issue two application keys
create same-named collections
write different values under the same key
prove non-crossing
blacklist one key
cycle authority
back up and restore one Heap under a new identity
prove old keys and cursors inert
```

### R2 — Operable early access

Criticality: **C0**

Purpose:

> Turn the Heap engine into something a developer can operate, inspect, and
> diagnose without reading the source tree.

Three enabling slices run in parallel after the R1 protocol and authority
shapes freeze. They do not all block the RRE engine path; the exact blocking
minimum is defined by `MASTER_DELIVERY_PLAN.md`.

#### R2-A — Evidence foundation

Deliver first:

```text
DEL-0 → DEL-1 → DEL-2 → DEL-3
                  └────→ DEL-4
DEL-3 + DEL-4 ─────────→ DEL-7
```

This provides:

- closed event registries and canonical encodings;
- pure verification;
- survival-aware evidence records and checkpoints;
- one per-Heap durable ledger;
- signer identity and rotation; and
- Heap-confined read/export access.

`DEL-5` mandatory Atomic coupling waits for R4. `DEL-8` lifecycle depth,
`DEL-9` offline UX, and final qualification continue in later releases.

Evidence lands before RRE and Atomics so their activations and decisions do
not need an audit story retrofitted later.

#### R2-B — Telemetry foundation

Deliver:

```text
TEL-0 → TEL-1 → TEL-2 → TEL-3 → TEL-4 → TEL-8
```

This provides:

- closed topics, fields, enums, and buckets;
- bounded allocation-conscious emitters;
- the Ratatouille adapter;
- request/admission, process, transport, storage, and health telemetry;
- explicit overflow and disconnect behavior; and
- removal or development-only gating of the old logging path.

Telemetry is diagnostic and MUST remain unable to block authoritative work.

#### R2-C — Studio Explorer

Deliver:

```text
DST-000 → DST-001 → DST-002 → DST-003
       → DST-004 → DST-005 → DST-006
```

This provides:

- the hardened Tauri shell and closed IPC protocol;
- OS-vault credentials;
- immutable Heap-bound workspaces;
- collection and record exploration;
- safe document/bytes editing;
- history, damage, holes, and coverage; and
- no master-key or wildcard data path.

Studio is not engine authority. It is the first-class way to see the authority,
damage, uncertainty, and evidence that the engine already exposes. Studio S1
is a parallel DX release, not a prerequisite for beginning RRE.

#### R2-D — Early-access trust gate

Required alongside A–C:

- crash/kill matrix for the supported single-node profile;
- bounded-memory scans;
- backup, restore, scrub, and migration from released artifacts;
- continuous fuzzing for active untrusted parsers;
- reproducible benchmark disclosure;
- compatibility policy;
- packaged Rust crate and CLI journey; and
- one scripted “write → kill → reopen → damage → examine → restore” demo.

R2 exit:

> A careful outsider can install Residiuum, create a Heap, store data, kill it,
> reopen it, see damage honestly, inspect operational health, and restore data
> without understanding the repository architecture.

### R3 — Mathematical documents

Criticality: **C1**

Packages:

```text
DRE-0 → DRE-1 → DRE-2 → DRE-3 → DRE-4
```

Deliver:

- one frozen predicate semantics shared by RQL and RRE;
- the declarative, non-Turing-complete RRE language;
- canonical Invariant Core;
- independently verifiable compiled artifacts;
- document-local activation and enforcement;
- JSON Schema → RRE translation;
- stable RQL grammar and SQL-ish → RQL translation;
- evidence for ruleset activation and rejection; and
- Studio RQL/RRE editing, validation, impact preview, and activation UI.

R3 intentionally excludes:

- parent existence;
- uniqueness;
- transition predicates;
- delete restrictions; and
- cross-document cardinality.

Those require the Atomic publication boundary and must be rejected, not
emulated.

R3 exit:

> Every document committed under an active ruleset satisfies a finite,
> canonical, examinable invariant.

### R4 — Atomic integrity and relationships

Criticality: **C1**

Packages:

```text
ATM-0 → ATM-1 → ATM-2 → ATM-3 → ATM-4 → ATM-5
                          |
                          +→ REL-0 → REL-1 → REL-2 → REL-3 → REL-4
                          |
                          +→ DRE-5 → DRE-6
                          |
                          +→ DEL-5
```

Deliver:

- Key Atomic and bounded serializable LocalHeap Atomic;
- stable operation identity and deterministic retry;
- durable prepare/member/decision evidence;
- crash recovery to one allowed outcome;
- reference metadata and reverse indexes;
- parent-exists and `on delete restrict`;
- uniqueness and bounded cardinality;
- online activation/validation;
- transition and cross-document RRE;
- mandatory Evidence Ledger coupling; and
- Studio preview, evidence, recovery, relationship, and conflict views.

R4 must not become an open-ended SQL transaction project. The scope is:

```text
one Heap
bounded members
declared read/write set
finite validation
one serializable decision
```

R4 exit:

> Residiuum provides document flexibility with database-owned, mathematically
> specified cross-document integrity.

This is the release that creates the new competitive quadrant.

### R5 — Exact navigation at scale

Criticality: **C1**

Packages:

```text
DDA-0 → DDA-1 → DDA-2 → DDA-3 → DDA-4
                                      |
                                      v
DOW-0 → DOW-1 → DOW-2 → DOW-3 → DOW-4
```

Deliver:

- exact natural-order rank;
- exact predicate membership/count algebra;
- Heap/view/plan-bound selection artifacts and cursors;
- direct positioning without walking from record zero;
- deterministic scalar ordered access;
- counted order blocks and bounded mutation maintenance;
- explicit coverage, damage, and refusal;
- no hidden offset emulation;
- RQL integration; and
- Studio advanced query, rank, cursor, sort, and explain UX.

R5 exit:

> Supported queries can move directly to exact positions and deterministic
> sorted regions without work proportional to the skipped prefix.

### R6 — Single-node production candidate

Criticality: **C2**

R6 is a qualification release, not another feature wave.

Complete:

- Evidence `DEL-6`, `DEL-8`, `DEL-9`, and the applicable part of `DEL-11`;
- Telemetry `TEL-5`, `TEL-6`, `TEL-9`, and `TEL-10`;
- Studio `DST-009` through `DST-013` and `DST-015`;
- every applicable `DEFECTS.md` §16.1, §16.2, §16.4, §16.5, and §16.6 gate;
- remote bounded cursors and concurrency for the supported server profile;
- signed packages, SBOM, release/rollback procedure, and support matrix;
- disaster, backup/restore, evidence verification, and authority-cycle drills;
- stable compatibility policy for format, RPC, SDK, configuration, and CLI;
  and
- an independent security review when the product claim requires one.

The external review may remain commercially deferred, but then the release
must remain `self-assessed production candidate`; it may not claim independent
qualification.

R6 exit:

> One explicitly supported single-node deployment profile has a complete,
> repeatable release case. Other profiles retain their lower maturity labels.

## 5. Competitive expansion after R6

These are ordered programs, not part of the immediate critical path.

### E1 — Massive-retention product

Criticality: **C3**

Deliver native object storage, encryption/KMS operation, lifecycle scheduling,
media refresh, restore drills, and long-retention evidence. Existing
filesystem mirrors and scaffolds do not satisfy this release.

### E2 — Deterministic text retrieval

Criticality: **C3**

Build the common Heap-bound derived-index substrate, then deterministic text
search. It follows the archive product so retained material gains an ordinary
rediscovery path and forces the shared index lifecycle to become real.

Text retrieval shares the derived-index substrate, but archived authority
must remain independently salvageable without any search index.

### E3 — Cluster product

Criticality: **C3**

Cluster work becomes a product only after:

- HP-011/HP-012 Heap placement and qualification;
- multi-process partition/leader histories;
- network repair and query paging;
- rolling restart and long soak;
- cluster-coordinated backup;
- cluster Evidence and Telemetry integration; and
- Studio cluster operations.

In-process consensus depth and experimental network Raft remain valuable
engineering foundations, not a production release claim.

The order between E1 and E3 may be reconsidered only through an explicit
market decision: choose archive-first for the fifteen-year retention market,
or cluster-first for the Couchbase/distributed-document market. The default
decision is **archive-first** because it is closer to Residiuum's core survival
thesis.

### E4 — Exact vector, then ANN/hybrid

Criticality: **C3**

Exact vector establishes semantics and the oracle. Segmented ANN and hybrid
text/vector search follow with explicit approximation and coverage labels.

### E5 — Geospatial

Criticality: **C3**

Basic point/radius/bounds operations precede advanced geometry.

## 6. What is critical now

The next work queue is:

| Order | Work | Class | Why now |
|---:|---|---:|---|
| 1 | R0 status and qualification reconciliation | C0 | current scoreboard is stale |
| 2 | `HAR-1` collection creation | C0 | blocks ordinary self-service Heap use |
| 3 | `HAR-2` local Heap creation | C0 | removes fixture/manual provisioning |
| 4 | `HAR-3` application-key lifecycle | C0 | completes the actual security product |
| 5 | `HAR-4` qualified remote default | C0 | prevents the legacy path defining DX |
| 6 | `HAR-5` Heap operations | C0 | backup/restore/retire must be Heap-correct |
| 7 | `HAR-6`/`HAR-7` journey and release evidence | C0 | turns implementation into a deliverable |
| 8 | Evidence `DEL-0`–`DEL-4`, then `DEL-7` | C0 | later integrity decisions need durable evidence |
| 9 | Telemetry `TEL-0`–`TEL-4`, `TEL-8` | C0 | performance and operation need bounded truth |
| 10 | Studio `DST-000`–`DST-006` | C0 DX | killer DX over the qualified Heap path |
| 11 | RRE `DRE-0`–`DRE-4` | C1 | first mathematical application invariant |
| 12 | Atomics + relationships + `DEL-5` | C1 | cross-document integrity differentiator |
| 13 | Direct Access | C1 | exact non-linear pagination |
| 14 | Order Wavelets | C1 | exact scalable filtered sorting |
| 15 | R6 qualification closure | C2 | one defensible production candidate |

Items 8–10 may run in parallel after R1 public shapes freeze. Items 11–14 may
have pure oracle/spec work overlap, but their public activation remains in the
listed order.

## 7. What must not interrupt the queue

Until R5 exits, do not allow these to displace C0/C1 work:

- broad cluster expansion;
- ANN/vector implementation;
- geospatial implementation;
- native cloud archive or erasure coding;
- Studio cluster screens;
- new general-purpose scripting languages;
- another authorization hierarchy or RBAC store;
- arbitrary offset pagination;
- adaptive index research without a named product gate;
- cosmetic dashboard breadth; or
- primary-index micro-optimization without a reproduced bottleneck.

Security fixes, data-loss defects, and compatibility emergencies always
preempt this rule.

## 8. Parallelism rules

Safe parallel work:

- Evidence pure formats/verifier may start while final Heap journeys close,
  once Heap identity and authority certificate shapes are frozen.
- Telemetry registries and bounded emitter may start independently of RRE.
- Studio shell/security/IPC may start while R1 finishes; live Heap Explorer
  activation waits for the qualified APIs.
- RRE semantic oracle may start while R2 product work proceeds.
- Atomic oracle work may start after the RRE invariant core and Heap object
  identity are frozen.
- Direct Access oracle work may start after shared predicate semantics freeze.

Unsafe parallel work:

- relationship enforcement before LocalHeap Atomic publication;
- durable Evidence coupling invented independently inside each subsystem;
- Studio invoking unfinished server internals through a raw escape hatch;
- Order Wavelets publishing before Direct Access identities/cursors freeze;
- cluster variants of a feature before its single-node semantics qualify.

## 9. Resource allocation

Recommended allocation until R2 exits:

```text
45%  Heap critical path and release journey
25%  Evidence + telemetry foundation
20%  Studio foundation and Explorer
10%  continuous trust gates: crash, fuzz, packaging, compatibility
```

Recommended allocation during R3–R5:

```text
50%  current engine critical-path release
20%  trust, evidence, telemetry, and recovery qualification
20%  Studio/DX integration for the capability just delivered
10%  next release's pure oracle, vectors, and spec preparation
```

Do not allocate a permanent percentage to cluster or future retrieval until
R5 exits or a conscious product strategy changes this roadmap.

## 10. Release discipline

Every release must include:

1. one complete outsider journey;
2. exact capability and maturity labels;
3. crash/damage behavior;
4. resource bounds;
5. evidence and telemetry behavior;
6. upgrade, backup, restore, and rollback treatment;
7. a Studio or CLI way to inspect the new capability;
8. executable conformance fixtures; and
9. reproducible performance disclosure where performance changed.

No release is complete merely because its API compiles.

## 11. Decision summary

The immediate sequence is:

```text
finish Heaps
→ close the trustworthy SQLite-replacement core
→ install the minimum Evidence and bounded Telemetry substrates
  while Studio Explorer proceeds in parallel
→ ship document-local RRE
→ ship Atomics and relationships
→ ship exact direct navigation and counted order
→ qualify one single-node production candidate
→ expand into archive, text, cluster, vector, and geospatial product work
```

The important correction is that **boring product trust sits between Heaps and
the mathematical feature program**, while Studio is a parallel product lane.
Residiuum should neither accumulate brilliant backend capabilities that cannot
be operated nor hold those capabilities behind completion of a graphical tool.
