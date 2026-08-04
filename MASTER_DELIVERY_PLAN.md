# Residiuum master delivery plan

Status: **dependency and package reference v1.7; current priority superseded**

Current execution authority:
[CRITICAL_PATH.md](./CRITICAL_PATH.md).

The principal locked the active sequence on 2026-08-04 as
`RQL -> Atomics -> Cluster`. Where this historical stage plan or its earlier
NOW/THEN language conflicts with that sequence, `CRITICAL_PATH.md` wins. This
file remains useful for package definitions, dependencies and accumulated
delivery history; it no longer selects the next programme.

Effective: 2026-08-01

Owner: Residiuum product and engineering program

**v1.7 note:** Principal amendment — **Query + Atomics early risk discovery**
on the M-line (§0.8, §14, §19 record). Not impatience: prove expressiveness,
gotchas, and performance sinkholes while store put/get is already tested.
**v1.6 note:** Added **§0 Reader map** so stage boundaries, package families,
and “what connects to what” are visible without reading the whole file.
**v1.5 note:** External delivery record for scoreboard-accepted C0 (CSQ-12 /
A2) and FA0 foundation packages FAS-0…FAS-4 (MVP scopes). Living package
states remain in
[doc/wip/status/NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md).

Testing authority:
[TESTING_STRATEGY.md](./doc/reference/engineering/TESTING_STRATEGY.md),
[doc/todo/verification/VERIFICATION_IMPLEMENTATION_PLAN.md](./doc/todo/verification/VERIFICATION_IMPLEMENTATION_PLAN.md),
and
[doc/wip/status/VERIFICATION_STATUS.md](./doc/wip/status/VERIFICATION_STATUS.md).

---

## 0. Reader map (start here)

This section is the map. Everything below is detail. If the rest of the file
feels like a pile of acronyms, re-read this section — not every package plan.

### 0.1 What this document is (and is not)

| This document owns | This document does **not** own |
|---|---|
| **Program order** — which stage and package family comes next | Live package states (`accept` / `ready` / …) → **scoreboard** |
| **Stage boundaries** — what “M1 exit” or “C0 exit” means | Semantics of put/get/salvage → **specs** (`ARCHITECTURE.md` → named specs) |
| **Priority law** — what may preempt what | Day-to-day assignment → **Kanban** (workflow only) |
| **Entry / exit rules** for admitted packages | Full test matrices → package plans + `TESTING_STRATEGY` |

**Living “where are we now?”** is always:

1. **§0 + §20** of this file (map + NOW/THEN summary)
2. **[NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md)** (each package’s state)
3. Open **one** stage section below only when you are working that stage

Kanban columns are not package acceptance. Code in the tree is not acceptance.

### 0.2 Two layers (this is the usual confusion)

There are **two different name systems**. Mixing them makes the plan feel
self-referential.

```text
STAGES (release chapters — public program gates)
  M0  C0  M1  M2  M3  M4  M5  M6  E1+

PACKAGES (units of labor + qualification)
  CSQ-12   HAR-3   APB-0   FAS-4   PQH-9   DEF-100   …

LANES (parallel workstreams allowed inside / beside a stage)
  under/after C0:  PQ0 (PQH-*) , FA0 (FAS-*) , then M1 (HAR-* + APB-*)
```

| Word | Means | Example |
|---|---|---|
| **Stage** | A **release boundary**. Stages do not “accept”; their **packages** do. Public product claims advance when the stage **exits**. | “M1 exit” = app can create/secure/operate a Heap |
| **Package** | One scoreboard row with state `not_started`…`accept`. Smallest honest unit of exit evidence. | `CSQ-12`, `HAR-1`, `APB-0` |
| **Lane** | Work that may run **alongside** the critical path without renaming the stage. | PQ0 = measurement; FA0 = formal; neither is “M1” |

**Rule of thumb:** Stages answer *where is the product going?* Packages answer
*what do I implement or accept this week?*

### 0.3 Package ID dictionary (what the letters mean)

| Prefix | Family | Stage / lane | One-line meaning |
|---|---|---|---|
| `M0-*` | Program truth | **M0** | Honest inventory + scoreboard + delivery check script |
| `DEF-*` | Defect remediation | before/with **C0** | Named P0 storage defects with permanent regressions |
| `CSQ-*` | Core storage qualification | **C0** | Store/format A2 evidence cells → `CSQ-12` closes C0 |
| `PQH-*` | Performance qualification harness | **PQ0** lane | Measurement machinery; not the M1 product gate |
| `FAS-*` | Formal assurance spine | **FA0** lane | Registry, tools, proofs; foundation `FAS-0…4` done; later waves deferred |
| `HAR-*` | Heap application ready | **M1** | Create/secure/operate/backup/restore one Heap |
| `APB-*` | Application baseline | **M1** | Product API baseline (contract → qualification) |
| `APP-*` | Application API slice | feeds **M1** | Older/core API plan; absorbed into APB/HAR where noted |
| `DEL-*` | Evidence (durable) | **M2+** | Evidence substrate (not “delivery plan”) |
| `TEL-*` | Telemetry | **M2+** | Bounded operational signals |
| `DST-*` | Studio | **M2+** | Explorer UI (parallel; not engine gate alone) |
| `RRE-*` | Document / math layer | **M3** | Residiuum Rules / document invariants |
| `ATM-*` | Atomics | **M4** | Atomic integrity / relationships |
| `VFY-*` | Verification IDs | cross-cutting | Claim/suite/profile identifiers |

Detail for a package lives in its **implementation plan** (linked from the
stage section). This file only admits order and exits.

### 0.4 Stage map — boundaries and “done means”

Stages are sequential for **public release**. Lanes may run in parallel only
where this plan says so.

```text
  [done] M0 Program Truth
           │  one honest queue + scoreboard + verify-delivery-status
           ▼
  [done] C0 Core Storage Qualification
           │  CSQ-0…12 accept; residiuum-core-storage-v1 / A2 verifies
           │
           ├─► PQ0  (lane)  PQH harness — alongside, not M1 exit
           ├─► FA0  (lane)  FAS foundation — alongside; FAS-5+ deferred
           │
           ▼
  [NOW]  M1 Heap Application Ready + Application Baseline
           │  HAR-0…7 + APB-0…12 accept; critical Heap journey
           ▼
         M2 Trustworthy Core Early Access   ← first “SQLite replacement” gate
           │  min Evidence + Telemetry + early-access trust
           ▼
         M3 Mathematical Documents (RRE)
           ▼
         M4 Atomic Integrity
           ▼
         M5 Exact Navigation (Direct Access + Order Wavelets)
           ▼
         M6 Single-Node Production Candidate
           ▼
         E1+ Competitive expansion (cluster, search, …) — not immediate target
```

| Stage | Product sentence (exit) | Package families | Detail section |
|---|---|---|---|
| **M0** | One accurate queue; Heap evidence not fantasized | `M0-1…M0-3` | §6 |
| **C0** | Format/store kernel A2-qualified | `DEF-098…104`, `CSQ-0…12` | §6A |
| **PQ0** | Performance can be measured honestly | `PQH-0…11` | §6B |
| **FA0** | Formal claim discipline foundation | `FAS-0…9` (0…4 accept MVP) | §6C |
| **M1** | App can create, secure, operate, back up, restore, retire a Heap | `HAR-*`, `APB-*` (+ `APP-*` absorbed) | §7 |
| **M2** | Careful outsider can replace SQLite + loose files | min `DEL`/`TEL`/`DST` + trust/distribution | §8 |
| **M3–M6** | Product-defining math → atomics → navigation → production candidate | `RRE`, `ATM`, DA/OW, gates | §9–§12 |
| **E1+** | Competitive expansion | cluster, retrieval, archive | §13 |

**Hard boundary rules (do not blur):**

1. **Stages do not overlap at the public-release level.** You do not “ship M2”
   while M1 is open. Limited **preparation** may overlap only where §7 / §14
   allow.
2. **C0 exit unlocks** post-C0 lanes: honest PQH entry, FA0, **APB-0** entry,
   and HAR feature labor per package deps — it does **not** mean M1 is done.
3. **M1 does not require full PQH** unless M1 work introduces or changes a
   **quantitative performance claim** (§7).
4. **FAS-5+ does not block** APB/HAR/M2 (principal: formal expansion deferred;
   foundation FAS-0…4 already accepted).
5. **Cluster / search / archive** stay later (`P3-FUTURE` / E1+) unless
   `P0-SAFETY`.

### 0.5 What connects to what (dependency spine)

Critical product spine (engine path):

```text
M0 ──► C0 ──► M1 (HAR + APB) ──► M2 ──► M3 ──► M4 ──► M5 ──► M6
```

Side lanes (do not redefine the spine):

```text
        ┌── PQ0 (PQH-*)  measurement ─────────────────────────┐
C0 ─────┤                                                     ├── may run in
        └── FA0 (FAS-*)  formal ──► FAS-5+ later / deferred ─┘    parallel
```

Inside **M1** only (order is package-level, not “all of M1 at once”):

```text
HAR-0 → HAR-1 → HAR-2 → HAR-3 → HAR-4 → HAR-5 → HAR-6 → HAR-7
APB-0  (after C0)  interleaves with HAR only where MUST_ADD deps permit
APP-0 / APP-1      feed collection create (HAR-1); later APP absorbed into APB
```

Inside **C0** only:

```text
DEF-098…104 → CSQ-0 → CSQ-1‖CSQ-2 → CSQ-3…11 → CSQ-12 (A2 bundle)
```

### 0.6 How to read the rest of this file

| Goal | Read |
|---|---|
| “What is Residiuum trying to ship first?” | §2 Product target + §0.4 M2 row |
| “What may I work on?” | §3 Priority + §17 Starting queue + **scoreboard** |
| “What does package state mean?” | §4 + scoreboard (not Kanban) |
| “What is stage M*?” | §5 diagram, then **one** of §6–§12 |
| “What is the current NOW?” | **§20** + scoreboard header / next-engine table |
| “May I prep X early?” | §14 Permitted preparation |
| “Is X deferred?” | §13 / §15 |
| “Change the order?” | §19 Amendment rule (required; chat is not enough) |

**Do not** treat §6–§12 as a novel to read end-to-end. Each stage section is a
**closed folder**: outcome, priority, package order, exit. Open the folder for
the stage you are in.

### 0.7 Companion files (narrow authority)

| File | Role |
|---|---|
| [NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md) | Living package scoreboard |
| [doc/README.md](./doc/README.md) | Doc lifecycle (todo/wip/done/reference) |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Map to normative specs (not a second roadmap) |
| Stage package plans under `doc/todo/…` | Contents of one package family |
| Kanban (`project_id` on board) | Who is doing which task; **not** accept |

### 0.8 Principal amendment — Query + Atomics earlier (risk management)

**Date:** 2026-08-01  
**Kind:** order / emphasis amendment under §19 (not a new architecture).

#### Intent

Store and retrieve are already under test. The remaining product risk is
**query expressiveness**, **Atomic compound transitions**, and the **gotchas**
that only show under wide corpora (hard-to-express shapes, refusal edges,
performance sinkholes). Principal wants those risks **surfaced earlier on the
M-line**, not deferred until after a long M1/M2 product digression.

#### New emphasis (order of attack)

```text
M1 spine (unchanged gate):  HAR + APB still required for product M1 exit
         │
         ├─ PRIORITIZE inside M1:  query spine
         │     APB-0 contract → APP-4/APP-5 (predicate/plan + RQL core)
         │     → APB-7 query baseline  (then page/history/index as needed)
         │
         ├─ RISK LANE (non-product until exits fire): pure oracles + corpora
         │     RRE-0 semantic oracle / corpus  (prep; no ruleset activation)
         │     ATM-0 oracle/profile after HAR-2 identity freeze
         │     adversarial “gotcha” + expressiveness matrices (diagnostic only)
         │
         └─ DE-EMPHASIZE until query spine breathes:
               APB watches / bulk import-export polish, Studio decoration,
               FAS-5+, cluster/search
```

**M3/M4 stage exits are not moved earlier.** Product claims for RRE enforcement
and LocalHeap Atomics still require their stage packages to `accept`. What
moves earlier is **risk discovery** (oracles, corpora, pure models, harness
stress) and **M1 query baseline** labor priority.

#### §19 fields

| Field | Statement |
|---|---|
| **1. New order** | Keep `M0→C0→M1→M2→M3→M4→…`. Inside M1, **query packages first** after APB-0. Admit **RRE-0 pure oracle** and **ATM-0 pure oracle** as preparation earlier (see §14). Do not accept M3/M4 from prep alone. |
| **2. Product reason** | Put/get is exercised; residual risk is query + atomics expressiveness and failure modes. Early discovery avoids late redesign of application baseline and document semantics. |
| **3. Dependency / evidence** | C0/`CSQ-12` accept; APP-4 precursor already in tree; plan already allowed APP-4/5 alongside HAR after APP-0 freeze; scoreboard already ties ATM-0 to **HAR-2 identity freeze** (not full M4). |
| **4. Delayed** | Non-query APB polish (e.g. watches/import-export) may lag the query spine; FAS-5+ remains deferred; full Studio/Evidence/Telemetry product surfaces stay on M2 schedule. |
| **5. Risk introduced** | Publishing live query/Atomic **product APIs** or capability claims before M1 security/isolation gates would be unsafe. Mitigation: prep is **pure/diagnostic only** until named package exits; no public claim language. |
| **6. Scoreboard / claims** | Next-engine order prioritizes APB-0 + query spine; APP-4 blocked_by drops satisfied `CSQ-12`; RRE-0/ATM-0 remain non-accept until exits; **no** new `qualified` claims. |

#### How this changes day-to-day labor

1. Close **APB-0** (application contract) as soon as practical.  
2. Unblock / drive **APP-4 → APP-5 → APB-7** as the M1 query path.  
3. Keep **HAR-0…HAR-3** honest (identity and keys) so ATM-0 and remote query
   have a real surface later.  
4. Run **RRE-0 / ATM-0** as pure risk lanes when §14 allows — catalog gotchas,
   do not ship.  
5. M2 early-access still follows M1 exit; it is not skipped for query/atomics.

---

## 1. Authority

This is the controlling document for **what Residiuum builds next and in what
order**.

Document state and location are governed by
[doc/README.md](./doc/README.md).

Other documents retain their narrower authority:

- specifications define semantics;
- implementation plans define individual package contents;
- the archived emergency-defect record defines incident history and permanent
  regression authority;
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

That first adoption gate is reached at `M2`. RRE, Atomics, and exact navigation
then create the product-defining Residiuum proposition at `M3`–`M5`. `M6`
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
[doc/wip/status/NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md).

It MUST be expanded to contain every active-master-plan package and updated in
the same change that changes a package state.

## 5. Master stage order

```text
M0  Program Truth
 ↓
C0  Core Storage Qualification
 ├── PQ0 Performance Qualification Harness (measurement lane)
 ├── FA0 Formal Assurance Spine foundation
 └── M1  Heap Application Ready + Application Baseline
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
- updated `doc/wip/status/VERIFICATION_STATUS.md`;
- claim/suite gap report;
- list of genuinely missing work; and
- discrepancies raised as named defects.

Exit:

- Kani, Verus, architecture checks, tests, verification status, and the matrix
  tell the same story.

### M0-2 — Scoreboard reconciliation

Depends: `M0-1`

Work:

- update `doc/wip/status/NEXT_BUILD_STATUS.md` with the observed HAR state;
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

## 6A. C0 — Core Storage Qualification

Release outcome:

> The authoritative format/store kernel has passed a closed invariant ×
> operation × prior-state × persistence-boundary × failure-class ×
> recovery-oracle qualification, with independently verifiable evidence.

Priority: `P0-GATE`

**Live claim (post protocol identity reset, commit `02e1b0d`):**
`residiuum-core-storage-v1` / **A2** only — pre-reset profile ids are invalid for
qualification
([REBRAND_PROTOCOL_IDENTITY_RESET.md](./doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md)
§§3–5).

**Qualification command:**

```text
residiuum verify --profile residiuum-core-storage-v1 --level A2
```

Stand-in until the CLI subcommand lands:
[scripts/residiuum-verify-core-storage.sh](./scripts/residiuum-verify-core-storage.sh)
([scripts/lib/csq_evidence.py](./scripts/lib/csq_evidence.py)).

Normative specification:
[CORE_STORAGE_QUALIFICATION_SPEC.md](./doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md).

Implementation plan:
[doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md](./doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md).

Program scoreboard (package qualification state, not Kanban columns):
[doc/wip/status/NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md).

**Board materialization (Kanban = live stages):** Feature **C0 — Core Storage
Qualification** plus tasks `CSQ-0`…`CSQ-12` and docs track `CSQ-DOC`. Package
graph: `CSQ-0 → (CSQ-1 ‖ CSQ-2) → CSQ-3…CSQ-11 → CSQ-12`.

**Delivery record (2026-08-01 — scoreboard `accept`):** `CSQ-0`…`CSQ-12` are
**`accept`** on the program scoreboard. A2 verifies:
`bash scripts/residiuum-verify-core-storage.sh` (+ `--require-a2-pass`) exit 0;
`target/csq-evidence/a2-evaluation.json` **a2_pass=true** missing=0 (111 cells);
`residiuum-core-storage-report-v1.json` result=pass. **A3 residuals** (platform
matrix, 72h soak, full-mutation %) remain open and do **not** block A2. Board
stages for some CSQ tasks may still show `in_review` pending principal Kanban
`done` — package **qualification** state is scoreboard `accept`, not Kanban
column.

Required order:

```text
DEF-098 exact chunk generations/publication
→ DEF-099 exact historical recovery
→ DEF-100 coverage-aware scans
→ DEF-101 writer-lock contract
→ DEF-102 derived-index lifecycle diagnostics
→ DEF-103 large-value profile
→ DEF-104 executable crash/recovery contract
→ CSQ-0 registries
→ CSQ-1 independent oracles
→ CSQ-2 boundary/failure instrumentation
→ CSQ-3…CSQ-11 qualification lanes
→ CSQ-12 verified A2 evidence bundle
```

`CSQ-12 = accept` unlocks post-C0 lanes (PQH entry honesty, FA0, APB-0 entry
dependency). APP-0/APP-1 may complete principal review. HAR-0 truth-only
reconciliation may continue. APB/HAR feature labor still follows package
dependencies in §7 and the scoreboard.

Any P0 storage-invariant violation discovered during C0:

1. interrupts the qualification lane;
2. is remediated as a named defect;
3. adds a permanent regression and mandatory mutation;
4. reruns every affected CSQ matrix cell; and
5. prevents restoration of the qualification label until evidence verifies.

C0 exit:

- DEF-098 through DEF-104 and every other applicable P0 core-storage defect are
  accepted;
- `CSQ-0` through `CSQ-12` are accepted;
- `residiuum-core-storage-v1 / A2` independently verifies;
- no mandatory cell is skipped, flaky, infrastructure-blocked, or
  implementation-oracled; and
- core-storage capability language matches the evidence.

## 6B. PQ0 — Performance Qualification Harness

Release outcome:

> Residiuum can account for throughput and latency from the target filesystem
> envelope through Residiuum-shaped I/O, CPU transformation, queueing,
> indexing, lifecycle, durability and acknowledgement, then select tuning work
> from reproduced evidence.

Priority: `P1-TRUST` immediately after `C0`

Entry dependency: `CSQ-12 = accept`.

Normative specification:
[PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](./doc/todo/performance-qualification/PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md).

Implementation plan:
[PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md](./doc/todo/performance-qualification/PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md).

Required order:

```text
PQH-0 registries
  → PQH-1 safe runner/environment
  → PQH-2 deterministic workloads
  → PQH-3 metrics/result kernel
  → PQH-4 device envelope
  → PQH-5 Residiuum-shaped shadow writer
  → PQH-6 CPU pipeline/stage probes
  → PQH-7 complete database matrix
  → PQH-8 attribution analyzer
  → PQH-9 qualification campaign
```

PQH is the first post-C0 measurement lane and may execute alongside M1. It
does not delay correctness fixes. It blocks:

- speculative performance optimization;
- a new quantitative performance claim;
- changing a tuning default on performance grounds without matched evidence;
  and
- describing multi-store capacity as single-store scaling.

PQ0 exit:

- `PQH-0` through `PQH-9` are accepted;
- macOS and Linux controlled campaigns are reproducible;
- correctness and durability interlocks remain green;
- observer overhead and stage-accounting residual meet the specification;
- the current 4 KiB/8 KiB underutilization observation has a registered,
  evidence-backed verdict; and
- subsequent tuning cards cite reproduced PQH run IDs.

## 6C. FA0 — Formal Assurance Spine

Release outcome:

> Residiuum’s principal consistency and security claims are named mathematical
> theorems with disclosed assumptions, machine-checkable proofs, production
> Rust refinement links, adversarial qualification evidence and reproducible
> release-bound proof bundles.

Priority: `P1-TRUST` immediately after `C0`

Entry dependency: `CSQ-12 = accept`.

Normative specification:
[FORMAL_ASSURANCE_SPEC.md](./doc/todo/formal-assurance/FORMAL_ASSURANCE_SPEC.md).

Implementation plan:
[FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md](./doc/todo/formal-assurance/FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md).

Execution waves:

```text
post-C0 foundation:
  FAS-0 → FAS-1/FAS-2 → FAS-3 → FAS-4 → FAS-5

with Atomics:
  FAS-6 → FAS-7

before/with cluster:
  FAS-8

incremental public proof product:
  FAS-9 after FAS-3 and each accepted theorem family
```

**Delivery record (2026-08-01 — scoreboard; living detail in NEXT_BUILD_STATUS):**

| Package | Scoreboard | Exit evidence (summary) | Honest residual |
|---|---|---|---|
| `FAS-0` | **accept** | Closed §12 catalogue + schemas; `formal/registry/FAS0_CLOSED`; `check-formal-registry.sh` → `fas0-registry-report.json` | Linter/schema depth; no `machine_proved` status inflation |
| `FAS-1` | **accept** | Pinned Verus/Kani/Lean/TLC; four-tool smokes; `check-formal-toolchain.sh` → `fas1-toolchain-report.json` | TLAPS deferred; CI job wiring |
| `FAS-2` | **accept** | Lean `Residiuum` State/Observation kernel; `check-formal-foundation.sh` → `fas2-foundation-report.json` | Stronger WF/put preservation; feature ops stubs |
| `FAS-3` | **accept** | Entrypoint census + type-map; vertical slice `FAS-BRIDGE-AUTHORITY-BINDING-001`; `check-formal-refinement.sh` → `fas3-refinement-report.json` | Store put/get full forward simulation |
| `FAS-4` | **accept (MVP)** | 8× `FAS-CON-*` Lean theorems; connections + negatives; CSQ A2 links; `check-formal-consistency.sh` → `fas4-consistency-report.json`; profile `mvp_abstract_plus_csq_links` | Full `physically_qualified` consistency profile |
| `FAS-5`…`FAS-9` | not_started / ready / deferred | — | Per scoreboard |

FA0 continues alongside PQH and M1. It does not delay ordinary application work
unless that work makes or changes a theorem-bearing claim.

No feature may claim mathematical or formal assurance merely because:

- documentation contains equations;
- a model checker explored an undisclosed finite bound;
- an abstract theorem exists without a Rust refinement;
- a proof file exists but was skipped; or
- tests happen to agree with the intended property.

The formal spine is cumulative:

- consistency establishes authoritative truth;
- security proves who may affect or observe it;
- Atomics proves compound transitions and exact isolation;
- cluster proves distributed agreement, fencing and convergence; and
- every later family proves preservation of earlier accepted invariants.

FA0 foundation exit:

- `FAS-0` through `FAS-5` are accepted for their exact declared scopes;
- theorem, assumption, TCB and public-claim registries are closed;
- at least the consistency and achieved Heap security profiles independently
  reproduce;
- applicable proofs are connected to the released Rust entrypoints;
- negative controls demonstrate live proof harnesses;
- CSQ/adversarial evidence is linked; and
- the public proof bundle refuses every overclaim fixture.

## 7. M1 — Heap Application Ready

Release outcome:

> An application can create, secure, operate, back up, restore, and retire one
> Heap without a global data path or manual fixture.

Priority: `P1-PATH`

Entry dependency: `C0` exit. PQH begins first as the post-C0 measurement lane
and then runs alongside M1; M1 does not require `PQH-9` unless it introduces or
changes a quantitative performance claim.

Immediate Must-Add program:
[MUST_ADD.md](./doc/todo/application-baseline/MUST_ADD.md), packages `APB-0` through `APB-12`.

Normative package plan:
[doc/todo/heap-application-ready/HEAP_APPLICATION_READY_PLAN.md](./doc/todo/heap-application-ready/HEAP_APPLICATION_READY_PLAN.md).

The ordinary Rust API, collection provisioning, and RQL Application Core
vertical slice is governed by
[doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](./doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md).
Its `APP-1` implements `HAR-1`; `APP-2` through `APP-8` supply the application
portion of `HAR-6` and its release evidence. This does not alter the HAR gate
order: work may be prepared in parallel only where the APP plan and this
master plan both permit it.

The product-gap authority is
[PRODUCT_DEFICIENCIES.md](./doc/reference/product/PRODUCT_DEFICIENCIES.md). Before `APP-2` begins, the
`APB-0` contract closure MUST reconcile and amend the existing APP plan.
Existing APP-0 fixtures remain evidence; they do not freeze the known omissions
as the final baseline. APP-2…APP-8 implementation is absorbed into the
corresponding APB packages rather than duplicated.

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

The APB lane begins with `APB-0` immediately after C0 and interleaves with HAR
only where [MUST_ADD.md](./doc/todo/application-baseline/MUST_ADD.md) dependencies permit. `APB-12` additionally
depends on the qualified remote Heap posture and therefore cannot accept before
the applicable HAR packages.

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
- `APB-0` through `APB-12` are `accept`;
- `residiuum-application-baseline-v1 / A2` independently verifies;
- the critical journey passes locally and in CI;
- `qualified` and public wording match the qualification matrix; and
- M2 foundation packages become `ready`.

## 8. M2 — Trustworthy Core Early Access

Release outcome:

> A careful outsider can replace SQLite plus loose JSON/blob files with
> Residiuum, then survive crash, damage, backup/restore, encryption-key
> operation, and upgrade without reading Residiuum internals.

M2 has one blocking product gate and three parallel enabling lanes. Evidence,
Telemetry, and Studio are important, but their complete feature sets do not
all block RRE. Only the minimum portions named below are M2 blockers.

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

- the durable evidence substrate exists for later RRE/Atomic decisions;
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
gates. Instrumentation for RRE, Atomics, Direct Access, and Order Wavelets lands
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
- `residiuum-application-baseline-v1` remains green against packaged artifacts;
- `DEL-0`–`DEL-3` pass before M3 rule activation;
- `TEL-0`–`TEL-2` pass before new performance claims;
- Rust and CLI quickstarts use the same qualified path; and
- the release label is no stronger than the passed evidence.

Studio S1, richer Evidence browsing, and full telemetry instrumentation
continue in parallel but do not hold the mathematical engine idle.

## 9. M3 — Mathematical Documents

Release outcome:

> Every committed document under an active RRE ruleset satisfies a finite,
> canonical, independently examinable invariant.

Priority: `P1-PATH`

Normative plan:
[doc/todo/rre/RRE_IMPLEMENTATION_PLAN.md](./doc/todo/rre/RRE_IMPLEMENTATION_PLAN.md).

Order:

| Order | Package | Result |
|---:|---|---|
| 1 | `RRE-0` | semantic oracle and executable corpus |
| 2 | `RRE-1` | parser and canonical AST |
| 3 | `RRE-2` | normalization and Invariant Core |
| 4 | `RRE-3` | canonical artifact and independent verifier |
| 5 | `RRE-4` | document-local activation and enforcement |

Mandatory integrations:

- shared predicate semantics with RQL;
- JSON Schema → RRE translation;
- SQL-ish → RQL translation against the frozen RQL grammar;
- Evidence records for validate/activate/replace/reject;
- `TEL-5` RRE collection points;
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

> Within one Heap, Residiuum commits bounded serializable changes with durable
> decision evidence and enforces declared cross-document integrity.

Priority: `P1-PATH`

Normative plan:
[doc/todo/atomics/ATOMICS_IMPLEMENTATION_PLAN.md](./doc/todo/atomics/ATOMICS_IMPLEMENTATION_PLAN.md).

Order:

```text
ATM-0 → ATM-1 → ATM-2 → ATM-3 → ATM-4 → ATM-5
                          |
                          +→ REL-0 → REL-1 → REL-2 → REL-3 → REL-4
                          |
                          +→ RRE-5 → RRE-6
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
- transition/cross-document RRE equals its oracle; and
- every completed decision remains independently examinable.

## 11. M5 — Exact Navigation at Scale

Release outcome:

> Supported queries move directly to exact ranked and sorted regions without
> walking the skipped prefix or sorting the complete result set.

Priority: `P1-PATH`

Normative plans:
[doc/todo/direct-access/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md](./doc/todo/direct-access/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md)
and
[doc/todo/order-wavelets/ORDER_WAVELET_IMPLEMENTATION_PLAN.md](./doc/todo/order-wavelets/ORDER_WAVELET_IMPLEMENTATION_PLAN.md).

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

- RQL query/profile identity;
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
| after `C0` accept + `APP-0` freeze in progress | **Query risk:** expand APP-4/APP-5 pure predicate/plan/compiler corpora; expressiveness and refusal matrices; **no** remote query product claim |
| after `HAR-0` + application contract draft | **RRE-0 pure semantic oracle** and adversarial document-invariant corpus (diagnostic); **no** ruleset activation, no product RRE claim |
| after `HAR-2` identity freeze | **`ATM-0` oracle/profile** work (pure crate, fixtures, hostile cases); **no** LocalHeap Atomic product API claim |
| after `HAR-3` | `DEL-0`, `TEL-0`, `DST-000` drafting/scaffold |
| after shared RRE predicate semantics freeze | `DDA-0` oracle work |
| after `RRE-2` | further ATM integration prep beyond ATM-0 (still pure until M4 packages accept) |
| after `DDA-3` order identity freezes | `DOW-0` oracle work |
| during M6 | E1 archive-adapter/profile specification and E2 common-index substrate specification only |

Preparation means pure models, corpora, schemas, harness stress, and design
validation. It does **not** mean publishing product APIs, starting migrations,
or claiming capability. See §0.8 (principal query + atomics risk amendment).

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

Until M1 exits (principal §0.8 emphasis):

```text
45%  active HAR / APB gate package on the query spine
     (APB-0 → APP-4/APP-5 → APB-7; HAR identity/keys as required)
20%  remaining M1 packages needed for M1 exit (non-query APB, HAR ops)
15%  qualification, crash, fuzz, and release evidence
10%  permitted query/RRE/ATM pure risk discovery (§14)
10%  permitted Evidence/Telemetry/Studio preparation
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

This is the current executable queue (as of **2026-08-01**):

| Queue | Package | State now | Action |
|---:|---|---|---|
| 1 | `M0-1`…`M0-3` | `accept` | program truth + delivery status check |
| 2 | `DEF-098`…`DEF-104` | `accept` | emergency remediation complete; permanent regressions |
| 3 | `CSQ-0`…`CSQ-12` | **scoreboard `accept`** | C0/A2 delivered; A3 residuals only |
| 4 | `FAS-0`…`FAS-4` | **scoreboard `accept`** (FAS-4 = MVP) | FA0 foundation through consistency MVP; FAS-5… deferred (principal: more FAS later) |
| 5 | `PQH-0`…`PQH-11` | scoreboard `active`; board largely `in_review` | **principal accept** PQH labor; qualification residual on controlled hosts |
| 6 | `APP-0` / `APP-1` | board `in_review` | principal review only; preserve produced work |
| 7 | `APB-0` then **query spine** | `not_started` / mixed | **M1 priority:** APB-0 → APP-4/APP-5 → APB-7 (query baseline); other APB may lag |
| 8 | `HAR-0`…`HAR-3` (identity/keys) | mixed | keep honest for remote query + ATM-0 gate; full HAR-4…7 still M1 exit |
| 9 | Risk prep `RRE-0` / `ATM-0` | prep only | pure oracles/corpora when §14 allows; **no** M3/M4 product accept |
| 10 | `FAS-5`…`FAS-9` | deferred | formal expansion later; does not replace M1 query path |

**Critical path (re-check scoreboard before labor):** M1 with **query spine
first** (APB-0 → APP-4/5 → APB-7) + HAR identity; pure RRE/ATM risk discovery
alongside under §14; then complete M1 exit → M2. Principal PQH accept remains
honest measurement hygiene, not a blocker for query prep. **APB-0** entry
(`CSQ-12 = accept`) is satisfied.

Live scoreboard:
[doc/wip/status/NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md).

No developer should start **product** RRE activation, **product** Atomics APIs,
Direct Access, Order Wavelets, search, or cluster work from this queue. Pure
oracle/corpus prep for RRE-0/ATM-0 is permitted only under §14 / §0.8 and never
counts as stage exit.

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
DELIVERED (scoreboard accept — 2026-08-01):
DEF-098…DEF-104 accepted
→ C0 / CSQ-0…CSQ-12 accepted
→ residiuum-core-storage-v1 / A2 verifies (a2_pass; A3 residual)
→ FA0: FAS-0…FAS-4 accepted (FAS-4 consistency profile = MVP abstract+CSQ links)
→ reports under target/formal-assurance/fas{0..4}-*.json

NOW (critical path — M-line + §0.8 query/atomics risk):
PQH principal accept (hygiene; not a hard gate for pure query prep)
→ APB-0 contract
→ M1 query spine: APP-4 / APP-5 → APB-7 (wide-case / gotcha discovery)
→ HAR-0…HAR-3 identity/keys (in parallel as deps require)
→ pure RRE-0 / ATM-0 risk oracles when §14 allows (no product claims)
→ finish remaining HAR/APB for M1 exit
→ verified residiuum-application-baseline-v1 / A2
→ M2 early access (Heap single-node vs SQLite+files)
(Principal: past FAS expansion; FAS-5… deferred.
 Product M3/M4 exits not skipped — risk prep is earlier, claims are not.)

THEN:
trustworthy SQLite-replacement core (M2)
+ minimum Evidence substrate
+ minimum bounded Telemetry path
+ Studio Explorer in parallel, not as an engine gate

THEN (product stages — after M1/M2 gates as written):
RRE document invariants (M3 product)
→ LocalHeap Atomics and relationships (M4 product)
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

Authoritative living package states:
[doc/wip/status/NEXT_BUILD_STATUS.md](./doc/wip/status/NEXT_BUILD_STATUS.md).
Do not treat code presence or board `in_review` alone as package `accept`.
