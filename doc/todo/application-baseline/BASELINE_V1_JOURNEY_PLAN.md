# Journey plan: Application Core query → `residiuum-application-baseline-v1`

Status: **v1 draft (2026-08-02)** — architecture review + deep planning first cut  
Board: Feature *Phase D+ — Journey to baseline-v1 complete* · labor task (see Kanban)  
Authority:

1. [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md) §0 / §0.8 / §7  
2. [MUST_ADD.md](./MUST_ADD.md) (APB-0…12)  
3. [APB_QUERY_ATOMICS_SEQUENCE.md](./APB_QUERY_ATOMICS_SEQUENCE.md)  
4. [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md)  
5. Host Kanban (labor SoT)

This document is the **missing post-query map**: how we get from “query spine
breathes / APB-7 accept path” to **application baseline complete** (APB-12 /
profile `residiuum-application-baseline-v1`). It is planning + architecture
review, not package accept and not product marketing.

---

## 0. Destination sentence

**Done for baseline-v1 complete** means (MUST_ADD §16):

```text
residiuum verify --profile residiuum-application-baseline-v1 --level A2
```

plus the packaged journey (create Heap/collection → mutations → bulk →
query/page/count under a read view → kill/reopen → partial enumerate →
historical version → watch resume → export/import → retire/restore →
local/remote semantic parity) with frozen evidence and no unexplained skips.

**Not** the destination of this plan alone:

| Explicit non-goal here | Owner later |
|---|---|
| Full RQL v1 (`enrich` / `within` / `at rank` / access) | Board placeholder `RQL-v1` — separate package(s) after Core product |
| M3 RRE product activation | RRE-1…4 after pure RRE-0 |
| M4 Atomics product API | ATM-1…5 after ATM-0 pure |
| M5 Direct Access / Order Wavelets | DDA/DOW after M3/M4 path |
| Cluster / vector / geo / FAS-5+ | Deferred / future |

Core-first still **builds toward** full RQL substrate; baseline-v1 does **not**
require full RQL language.

---

## 1. Where we are (2026-08-02 review)

### 1.1 Already closed (scoreboard / board)

| Area | State |
|---|---|
| CSQ-12 / C0 storage gate | accept (A2) |
| APB-0 contract freeze | accept |
| APP-4 predicates + plan | accept |
| APP-5 `rql-app-core-v1` compile | accept |
| APB-1 façade labor | active labor largely done on board; **no package accept** |
| Query spine labor | APB-7 T0–T4 + APP-6 T1–T2 + APB-6 T1–T2 **in_review** |
| Full RQL | placeholder only — out of APB-7 |

### 1.2 Board already covers (query finish line)

Open Query-spine cards take us to **Application Core product accept path**:

- residuals T5, T8–T11, APP-6 T3, APB-6 T3  
- HAR-4 dep + op 118 (APP-7/T6) for **honest remote**  
- T7 dual-pack + accept checklist  

**Ceiling of that Feature:** APB-7 package accept candidate — **not** baseline-v1.

### 1.3 Gap this plan fills

Between **APB-7 accept** and **APB-12 baseline gate** the board had **no**
architecture/planning card. Implementation packages APB-2…5, APB-8…11, HAR-2…7
are scoreboard-visible but **not** pre-staged as labor sequences with session
agendas, dependency honesty, and pull order after query.

---

## 2. Architecture sessions (recommended series)

Run as principal + labor working sessions (or sequential labor packages). Each
session produces **decisions recorded on board** (and amended here), not code.

| # | Session | Question | Exit artifact |
|---:|---|---|---|
| S0 | **Current-state freeze** | What is honestly product vs scaffold after APB-7 T0–T4? | Claim table (this §1 + scoreboard residual list) |
| S1 | **Remote posture** | HAR-4 HeapKey default + TLS accept path; when is op 118 legal? | Remote gate checklist; no fake dual-pack accept |
| S2 | **Read-view product bar** | What pin/retention/remote pin is enough for baseline journey “query under read view”? | APB-6 exit honesty vs DEF snapshot non-claims |
| S3 | **Mutation completeness** | APB-2 CAS + APB-3…5 path: min set for APB-12 journey | Must-have vs lag-allowed mutation matrix |
| S4 | **Aggregates (APB-8)** | Coverage/precision model vs Core scan | APB-8 package brief before code |
| S5 | **Watches + I/O (APB-9/10)** | Retention, resume, parity; import/export formats first cut | Scope freeze + lag rules |
| S6 | **Test kit (APB-11)** | Public harness surface vs private engine | Kit API sketch + grow-with-implementation rule |
| S7 | **Qualification journey (APB-12)** | Exact A2 profile steps, fixtures, local/remote | Journey script outline + evidence map |
| S8 | **M1 vs baseline** | What of HAR-5…7 / multi-Heap isolation is on critical path for baseline-v1 vs full M1 exit? | Two columns: baseline-critical vs M1-critical |
| S9 | **Full RQL fence** | Confirm enrich still **after** baseline product unless principal reorders | Keep `RQL-v1` card; no silent pull into APB |

**S0–S2 can start before APB-7 accept.** S3–S7 should not invent package accept
criteria that contradict MUST_ADD. S8 resolves “M1 exit vs baseline-v1” tension
honestly.

---

## 3. Dependency graph (post-query → baseline)

From MUST_ADD §17, redrawn as **labor waves**:

```text
WAVE 0 — Finish query product path (already staged)
  APB-7 residuals + T7 accept checklist
  APP-6 T3 · APB-6 T3
  HAR-4 ──► APP-7 op 118 (honest remote query)
  Principal: package accepts only when exits met

WAVE 1 — Remote + mutation honesty (parallel after / with late query)
  HAR-4… (posture) · HAR-0 residual as hygiene
  APB-2 T5 CAS + T6 residual checklist
  APB-3 collection lifecycle (describe/rename/retire/…)
  APB-4 path lookup + single-doc atomic mutation
  APB-5 bounded bulk mutation
  (APB-1 package accept only if dual-pack exit truly met)

WAVE 2 — Query-adjacent baseline features
  APB-8 aggregates (depends APB-7)
  APB-9 watches (depends APB-2, APB-6) — may lag if forced
  APB-10 import/export (depends APB-3, APB-5, APB-6) — may lag

WAVE 3 — Kit + qualification
  APB-11 test kit (grows alongside; do not wait until end)
  APP-8 / journey evidence absorbed into APB-12
  APB-12 A2 profile + packaged journey + frozen fixtures

PARALLEL (non-blocking for baseline code, optional prep)
  RRE-0 pure · ATM-0 pure after HAR-2
  RQL-v1 placeholder — do not pull as baseline labor
```

### 3.1 Critical path (shortest honest path to baseline gate)

```text
APB-7 accept
  + HAR-4 (remote product honesty)
  + min mutations for journey (APB-2 CAS + APB-3…5 as journey requires)
  + APB-6 product bar for “query under read view”
  + APB-8 if journey requires count (MUST_ADD journey includes count)
  + APB-9 if journey requires watch resume
  + APB-10 if journey requires export/import
  + APB-11 kit growth
  + APB-12 verify profile
```

Anything not required by the **mandatory packaged journey** may lag **only** if
APB-12 exit still cannot be claimed (no silent skip). Prefer **implement
journey-required packages first**, polish later.

### 3.2 Suggested pull priority after APB-7 T7 is ready for principal

| Priority | Package / work | Why |
|---:|---|---|
| 1 | Close open **query spine** todos (T5–T11, T6/118, T7) | Unblocks “query works” |
| 2 | **HAR-4** product labor (not only dep card) | Remote parity / op 118 |
| 3 | **APB-2 CAS** + residual accept honesty | Versioned mutations in journey |
| 4 | **APB-3…5** inventory + first product slices | Journey mutations/bulk/lifecycle |
| 5 | **APB-8** aggregates | Journey includes count |
| 6 | **APB-9 / APB-10** as journey steps demand | Watch + I/O |
| 7 | **APB-11** continuous | External consumer tests |
| 8 | **APB-12** | Final gate |
| — | Full RQL / RRE product / ATM product | **Off** this critical path |

---

## 4. Deep planning notes (architecture)

### 4.1 One façade, two backends

Baseline qualification is **embedded + remote semantic parity**. Architecture
rule:

- All public baseline ops go through `HeapClient` / `CollectionClient` (or
  documented successors), not legacy-only paths.
- Remote must use **product wire** where claimed (e.g. op 118 for `rql_query`),
  not “collection-plane scan pretending to be remote product.”
- Dual-pack scenarios remain the evidence pattern (`apb1_facade_parity` style).

### 4.2 Read views vs snapshot isolation

APB-6 pin (segment fingerprint) is a **frontier observation** tool. Baseline
journey needs “query/page/count under a read view” — plan sessions must define:

- minimum pin stability for multi-page pages  
- fail-closed on drift  
- what **must not** be claimed (full snapshot isolation / DEF grade) until exits  

Do not overload APB-7 with snapshot marketing.

### 4.3 Query profile for baseline

Baseline ships **`rql-app-core-v1`**, plan profile `rql-plan-v1`.  
Advertise Core; reject enrich/`within`/`at rank` until full-RQL packages.

### 4.4 Coverage and incompleteness

DEF-100 / complete-by-default coverage is a **cross-cut**: scan, query pages,
aggregates, and qualification journey all inherit the same honesty rules.
APB-7 T9 and APB-8 precision/overflow policy must stay aligned.

### 4.5 Cursors and secrets

Product cursor MAC (APB-7 T10) is a **security boundary** for multipage query
and any remote continuation. Vector-lock test keys are not product.

### 4.6 Package accept discipline

- Labor stages cards to **`in_review`** with evidence in objectives.  
- Principal alone moves to **`done`** and scoreboard **accept**.  
- No package accept from partial façade labor (APB-1/2/6/7 same rule).

---

## 5. Deliverables of this planning Feature

| Deliverable | Owner | Done when |
|---|---|---|
| This journey plan (v1+) | Labor | File exists; linked from board task objective |
| Session S0–S9 decision log | Principal + labor | Amendments to this file or board memory |
| Pre-staged Kanban cards for Wave 1–3 | Labor **after principal approval of this plan** | Features + todos exist before code turns |
| Scoreboard next-engine note | Labor | Pointer only; Kanban remains human dashboard |

**Out of scope for the planning task itself:** implementing APB-8…12, accepting
packages, activating op 118 without HAR-4, pulling RQL-v1 enrich work.

---

## 6. Immediate next actions

1. Principal reviews this plan (board task → gate).  
2. Finish **query spine** open todos per existing board order (do not abandon).  
3. After plan approval: stage Wave 1 cards (HAR-4 package labor, APB-2 CAS,
   APB-3…5 inventories) as `todo` before pulling code.  
4. Keep **RQL-v1** and **RRE-0/ATM** as side/parallel — not baseline critical
   path unless principal reorders.

---

## 7. Changelog

| Date | Note |
|---|---|
| 2026-08-02 | v1 draft from principal request: missing architecture/planning card for path to baseline-v1 complete |
