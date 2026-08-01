# Sequence: APB-0 → Query → Atomics (honest path)

Status: **normative labor sequence v1.1** (2026-08-01)  
Package context: `APB-0` accept · APP-4 accept · APP-5 active · principal: query spine + parallel HAR/client deps  

Authority (strict):

1. [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md) §0 / §0.8 / §7 / §14  
2. [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md)  
3. [MUST_ADD.md](./MUST_ADD.md)  
4. [spec/app/baseline-v1/README.md](../../../spec/app/baseline-v1/README.md) (APB-0 gaps)

This file is the **execution map** from the contract freeze to a usable query
system and atomics **without skipping gates**. It is not a second master plan.

---

## 0. One-page spine

```text
DONE     C0 / CSQ-12 accept · FAS-0…4 foundation · put/get tested
DONE     PHASE A — APB-0 contract freeze (accept 2026-08-01)
         │
NOW      PHASE B — Query spine (product path)
         │         APP-4 → APP-5 → APB-7 (+ min client/read)
         │         HAR-0…3 identity/keys in parallel as deps require
         │
         PHASE C — Risk discovery (non-product claims)
         │         RRE-0 pure oracle · ATM-0 after HAR-2
         │
         PHASE D — Finish M1 enough for real surface
         │         remaining HAR/APB needed for journey + remote
         │
         PHASE E — Product document + atomic stages
                   M3 RRE product · M4 ATM product
                   (oracles do not count as stage exit)
```

| Destination | What “there” means | First package that owns it |
|---|---|---|
| **Contract correct** | No inventable public types | `APB-0` accept |
| **Query works (app baseline)** | RQL Application Core, builder, explain, page, remote parity | `APB-7` accept (+ APP-4/5) |
| **Atomics risk seen** | Oracle, fixtures, hostile cases, gotcha catalog | `ATM-0` (pure; after HAR-2) |
| **Atomics product** | LocalHeap Atomic API + adversarial corpus | `ATM-5` / M4 exit |
| **RRE product** | Document ruleset activation | `RRE-4` / M3 path |

---

## 1. PHASE A — APB-0 sequence (contract freeze)

**Goal:** Freeze `spec/app/baseline-v1/` so later APB/APP packages implement, not invent.  
**Depends:** CSQ-12 accept (met); APP-0/APP-1 evidence reconciled.  
**Exit:** MUST_ADD §4 exit bullets + scoreboard `APB-0 = accept`.

| Step | Labor ID | Deliverable | Done when |
|---:|---|---|---|
| 1 | **T1** | Gap inventory + `baseline-v1/README` scaffold | **landed** (in_review) |
| 2 | **T2** | `operations-v1.json` — every APB-1…12 public op + wire op links; reserved ops named honestly | **draft landed** (47 ops; status still `draft` until T6 freeze) |
| 3 | **T3** | `outcomes-v1.json` — total map from lower conditions → public outcomes; amend APP-0 error_mapping | **draft landed** (38 projections; codes total) |
| 4 | **T4** | `projections-v1.json` — local vs remote parity rules | **draft landed** (12 rules; 47 ops) |
| 5 | **T5** | `capabilities-v1.schema.json` + `types-v1.json` cross-cut freeze; DEF-099/100 bind | **draft landed** |
| 6 | **T6** | Canonical fixtures + `scripts/verify-app-baseline-contract.sh` + freeze | **done** — script exit 0 `--require-frozen`; scoreboard **APB-0 accept** |

**Rules while in Phase A:**

- Amend APP-0; do not fork a second façade.  
- Reserved ≠ missing (e.g. op 118 registered, schemas null until query package).  
- No `residiuum-application-baseline-v1` product claim until APB-12.  
- Cross-Heap composition impossible by construction (MUST_ADD §4).

**Suggested Kanban:** tasks under feature *APB-0 — Application baseline contract freeze*.

---

## 2. PHASE B — Query spine (product path after APB-0)

**Goal:** Fully functional **application query** surface (RQL Application Core), not M5 Direct Access / Order Wavelets.  
**Priority:** principal §0.8 — first product emphasis after APB-0.

```text
APB-0 accept
    │
    ├─► HAR-0 (truth residual, ready) ─────────────────────┐
    │                                                      │
    ├─► APB-1  HeapClient / CollectionClient               │  min client
    │      depends: APB-0, collection provisioning (HAR-1/APP-1)
    │                                                      │
    ├─► APP-4  canonical predicate + plan   (scoreboard ready)
    │      may progress once APP-0 freeze + APB-0 types hold
    │                                                      │
    ├─► APP-5  RQL Application Core compiler               │
    │      depends: APP-4                                  │
    │                                                      │
    ├─► APB-6  stable bounded read views (needed for honest query)
    │      depends: APB-1                                  │
    │                                                      │
    └─► APB-7  RQL runtime, builder, explain, paging, remote parity
           depends: APB-1, APB-6, APP-4, APP-5
```

| Package | Role toward “query works” | May lag? |
|---|---|---|
| `APP-4` | Predicates + plans | **No** — spine |
| `APP-5` | Compiler | **No** — spine |
| `APB-1` | Unified client | **No** — need handle |
| `APB-6` | Read views | Prefer before APB-7 claim |
| `APB-7` | Query product baseline | **Gate for “query works”** |
| `APB-2`…`APB-5` | Mutations / path / bulk | Yes — after query breathes if forced to choose |
| `APB-8` | Aggregates | After APB-7 |
| `APB-9`…`APB-10` | Watches / import-export | **Yes lag** (§0.8) |
| `HAR-1…3` | Collection create, ceremony, keys | **No lag on identity** — needed for real/remote |
| `HAR-4+` | Default remote HeapKey posture | For remote parity of APB-7 |

**Query “gotcha” work (allowed once APP-4/5 pure work exists):**

- Expressiveness / refusal matrices (diagnostic)  
- Wide-case corpora against pure compiler (APP-5)  
- **No** public “query is qualified” until APB-7 + evidence chain  

**Suggested Kanban (board SoT; filled 2026-08-01):**

| Feature | Labor IDs | Stage intent |
|---|---|---|
| *Phase B — Query spine* | APP-5 **T2** §9 surface · **T3** corpus/reject · **T4** budget+fuzz · **T5** scoreboard accept | Primary path; do T2→T5 before APB-7 |
| *Phase B∥ — HAR identity + min client* | HAR-0 **T1** residual · HAR-1 **T1** op-106 reconcile · APB-1 **T1** client gap inventory | Parallel deps for APB-7 |
| *Phase C — RRE-0 pure risk* | RRE-0 **T1** oracle scaffold | Diagnostic only; never blocks APP-5 |

APP-4 T1 + APP-5 T1 labor are **in_review** (APP-4 scoreboard accept landed).

---

## 3. PHASE C — Atomics + RRE risk discovery (parallel, non-product)

**Goal:** See hard-to-express cases, sinkholes, and adversarial edges **before** M3/M4 product exits.  
**Not:** product Atomic API or ruleset activation.

| When | Package / work | Allowed claim |
|---|---|---|
| After C0 (now) | `RRE-0` pure semantic oracle + adversarial document corpus | Diagnostic only |
| After `HAR-2` identity freeze | `ATM-0` oracle, profiles, fixtures, hostile cases | Diagnostic only |
| With APP-4/5 | Shared predicate semantics notes for RRE/RQL alignment | Spec prep |
| Never from Phase C alone | M3/M4 accept, `qualified`, product marketing | **Forbidden** |

```text
HAR-0 → HAR-1 → HAR-2 ──► ATM-0 (pure) ──► (later) ATM-1…5 product
                │
RRE-0 (pure, early) ─────────────────────► (later) RRE-1…4 product
```

Product order for stages remains:

```text
… → M1 exit → M2 early access → M3 RRE product → M4 Atomics product → …
```

---

## 4. PHASE D — Complete M1 enough for a real system

After query spine breathes, close remaining M1 obligations **in dependency order**, not “whatever is fun”:

| Track | Packages | Why |
|---|---|---|
| Heap app-ready | `HAR-4…HAR-7` | Remote posture, ops, journey, release evidence |
| Baseline completeness | `APB-2…5`, `APB-8…12` as MUST_ADD deps require | Mutations, aggregates, test kit, A2 bundle |
| M1 exit | HAR + APB accept + critical journey | Unlocks honest M2 |

M1 critical journey (plan §7) still requires multi-Heap isolation, keys, backup/restore — not only query.

---

## 5. PHASE E — Product atomics and document rules (later stages)

| Stage | Packages | Outcome sentence |
|---|---|---|
| **M3** | `RRE-0…RRE-4` (+ later RRE-5/6 with ATM) | Committed docs under active ruleset satisfy finite invariants |
| **M4** | `ATM-0…ATM-5`, relationships, RRE-5/6 as plan | LocalHeap atomics + bounded relationships |

Prep from Phase C **feeds** these packages; it does not replace them.

---

## 6. What is deliberately *not* on this critical path

| Deferred | Why |
|---|---|
| FAS-5+ | Principal: more formal later |
| Cluster / vector / geo / archive expansion | P3-FUTURE / E1+ |
| Full Studio product | Parallel DX; not engine gate alone |
| M5 Direct Access / Order Wavelets | After M3/M4 product path |
| Skipping APB-0 | Invents semantics under pressure — rejected |

---

## 7. Dependency diagram (packages only)

```text
CSQ-12 ──► APB-0 ──► APB-1 ──┬──► APB-2…5 (mutations; may lag spine)
              │              ├──► APB-6 ──► APB-7 ──► APB-8…12
              │              │              ▲
              │              │         APP-4 ──► APP-5
              │              │
              └──► (types freeze enable APP-4/5 labor)

HAR-0 ──► HAR-1 ──► HAR-2 ──► HAR-3 ──► HAR-4…7
              │         │
              │         └──► ATM-0 (pure) ──► ATM-1…5 (product M4)
              │
APP-1 ────────┘ (collection create evidence)

RRE-0 (pure, early) ──► RRE-1…4 (product M3)
```

---

## 8. Definition of done (milestones)

| Milestone | Evidence |
|---|---|
| **A — Contract frozen** | Scoreboard `APB-0 = accept`; `verify-app-baseline-contract.sh` exit 0 |
| **B — Query usable** | Scoreboard `APB-7 = accept` (+ APP-4/5); wide-case corpus green as diagnostic; no false `qualified` |
| **C — Atomics risks catalogued** | ATM-0 fixtures + hostile suite; gotcha notes; still no product Atomic claim |
| **D — M1 exit** | HAR/APB plan exits + critical journey |
| **E — Product atomics/query-at-doc-layer** | M3/M4 package accepts |

---

## 9. Immediate next actions (ordered)

1. **APB-0 T2** — draft `spec/app/baseline-v1/operations-v1.json`  
2. **APB-0 T3–T6** — outcomes → projections → capabilities/types → verify + accept  
3. Open **APP-4** package labor (precursor exists) once APB-0 types stop moving  
4. **HAR-0 / HAR-1** progress so collection + identity stay real  
5. **RRE-0** pure corpus in parallel (low claim surface)  
6. After HAR-2: **ATM-0**  

---

## 10. Document maintenance

- Update this file when package order or principal emphasis changes (cite plan §19 if order changes).  
- Update [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md) in the **same change** as package state moves.  
- Kanban tracks labor; this file + scoreboard track truth.