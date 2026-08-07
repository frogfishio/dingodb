# RQL-Q0 — Tier A/B/C capability matrix

Status: **labor complete · principal freeze pending**

Package: RQL-Q0 deliverable 2
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §2–§3
Grounding: [RQL_SPEC.md](../../wip/query/RQL_SPEC.md), [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md)
Board task: Q0.2
Effective: 2026-08-07

## 0. Classification legend

| Class | Meaning |
|---|---|
| `exact` | RQL exposes the same result semantics directly |
| `document-native-equivalent` | Different expression, same practical result |
| `deliberate-exclusion` | Outside the frozen Gate-1 profile; stable refusal required |
| `blocker` | Required by frozen Tier A but not yet product-complete |

**Impl state** (honesty, not Gate-1 accept): `implemented` · `partial` · `absent`
from the gap ledger. Impl state ≠ matrix class.

**Owner** names the normative section or package that must freeze meaning before
behavior changes.

**Law:** no mandatory Tier-A semantic may remain `TBD`. Aggregates are Tier A in
the qualification programme even though RQL_SPEC v1 currently excludes them —
they are frozen here as **`blocker`** (SPEC must be amended in Q2 before product
syntax lands).

---

## 1. Tier A — mandatory Gate-1 surface

| id | Capability | Class | Impl | Owner | Residual / notes |
|---|---|---|---|---|---|
| TA-KEY | Key lookup / point get | `exact` | implemented | RQL_SPEC §3; Core path | Product Core path green |
| TA-SEL-EQ | Equality selection (root) | `exact` | partial | RQL_SPEC; gap ledger root `where` | Range/multi-field multipage residual |
| TA-SEL-RANGE | Range predicates | `exact` | partial | RQL_SPEC; index pushdown | Compound/range multipage residual |
| TA-SEL-COMPOUND | Compound predicates | `exact` | partial | RQL_SPEC; planner | Partial pushdown |
| TA-NULL | Total absent / null / value semantics | `exact` | partial | RESIDIUUM_PREDICATE_SPEC; SDA | Must not collapse holes to empty complete pages |
| TA-TYPE | Type-aware predicates | `exact` | partial | Predicate profile | Wrong-type adversarial cases required in Q3 |
| TA-NESTED | Nested-field predicates | `exact` | partial | RQL_SPEC; SDA path | |
| TA-ARRAY | Array predicates | `exact` | partial | RQL_SPEC; ENR/SDA | Completeness residual → Q2 |
| TA-BOOL | Boolean composition (and/or/not) | `exact` | implemented | Predicate / Core where | |
| TA-PARAM | Named parameter binding `$` | `exact` | implemented | Core | Cursor param MAC residual separate |
| TA-PROJ-FLAT | Flat projection | `exact` | implemented | Core project | On wire op 118 |
| TA-PROJ-NEST | Nested / brace projection | `exact` | partial | Full RQL project | Local façade; wire refuse full language |
| TA-PROJ-COMP | Computed projection | `blocker` | absent | RQL_SPEC amend + Q2 | SPEC v1 excludes arbitrary computed proj; programme requires practical shaping |
| TA-PROJ-COND | Conditional projection / shaping | `blocker` | absent | RQL_SPEC amend + Q2 | Same as computed for Gate-1 practical surface |
| TA-ORDER | Deterministic multi-field order + immutable key tie-break | `exact` | implemented | Core order | |
| TA-TOPK | Top-k / limit | `exact` | implemented | Core limit/page | |
| TA-CURSOR | Cursor continuation without offset-prefix discard | `exact` | partial | APP-6 cursor-v1 | Heap-confined secrets residual; offset deliberately refused |
| TA-IDX-EQ | Equality index eligibility | `exact` | partial | APB-7 index pushdown | Admitted paths only; scan fallback honest |
| TA-IDX-RANGE | Range index eligibility | `exact` | partial | Planner / index | Residual multipage |
| TA-IDX-COMPOUND | Compound index eligibility | `exact` | partial | Planner / index | |
| TA-ENRICH-1 | Enrich `exactly_one` | `exact` | partial | Full enrich; ENR | Local path; **op 118 refuse**; root eq-index partial |
| TA-ENRICH-OPT | Enrich `optional` | `exact` | partial | Full enrich | Same wire residual |
| TA-ENRICH-MANY | Enrich `many` | `exact` | partial | Full enrich | Same; within still scan |
| TA-WITHIN | Nested `within` carrier | `exact` | partial | Full within | Depth bound; wire refuse |
| TA-GROUP | Grouping | `blocker` | absent | RQL_SPEC amend; APB-8 lane | SPEC v1 excludes GROUP BY; programme Tier A requires it |
| TA-AGG-COUNT | Count accumulator | `blocker` | absent | RQL_SPEC amend | sql+ refuses aggregates today |
| TA-AGG-SUM | Sum accumulator | `blocker` | absent | RQL_SPEC amend | |
| TA-AGG-MIN | Min accumulator | `blocker` | absent | RQL_SPEC amend | |
| TA-AGG-MAX | Max accumulator | `blocker` | absent | RQL_SPEC amend | |
| TA-AGG-AVG | Average accumulator | `blocker` | absent | RQL_SPEC amend | |
| TA-COMPOSE | Reusable composition / subplans required by corpus | `blocker` | partial | RQL_SPEC; plan reuse | Named reusable components incomplete |
| TA-BUDGET | Query budgets (docs/bytes/result) | `exact` | implemented | Core budget | |
| TA-CANCEL | Cancellation / deadline | `exact` | partial | Resource / deadline codes | Cooperative cancel residual honesty |
| TA-CONSIST | Consistency modes | `exact` | implemented | Core consistency | |
| TA-COVER | Coverage policy + incomplete honesty | `exact` | implemented | Core coverage | Incomplete fail-closed |
| TA-EXPLAIN | Explain of programme actually executed | `exact` | partial | Core + full explain | Full explain not on op 118; must describe physical strategy honestly |
| TA-SQL-SUBSET | Deterministic SQL subset → RQL/QVM | `document-native-equivalent` | partial | SQL_TO_RQL_SPEC; sql+ scaffold | Emit or refuse; never guess; joins currently refuse |

**Tier A blocker summary (must close before Q2 exit):** TA-PROJ-COMP, TA-PROJ-COND,
TA-GROUP, TA-AGG-*, TA-COMPOSE (to corpus bar), plus elevating all `partial` rows
to expressible-without-app-scan for their Tier-A cases.

---

## 2. Tier B — important expansion (non-blocking unless promoted pre-Q1 freeze)

| id | Capability | Class | Impl | Owner | Notes |
|---|---|---|---|---|---|
| TB-AGG-RICH | Richer accumulators beyond count/sum/min/max/avg | `deliberate-exclusion` until promoted | absent | Future SPEC | Measured only if promoted |
| TB-ARRAY-XFORM | Array transformation pipelines | `deliberate-exclusion` until promoted | partial | ENR/SDA | |
| TB-ENRICH-FANOUT | Larger / multi-hop enrich fan-out | `deliberate-exclusion` until promoted | partial | Full attach | |
| TB-DISTINCT | Distinct | `deliberate-exclusion` until promoted | absent | SPEC | |
| TB-NAMED-COMP | Named reusable query components (library) | `deliberate-exclusion` until promoted | absent | DX / plan | |
| TB-COVERING-IDX | Partial/covering index improvements | `deliberate-exclusion` until promoted | partial | Index planner | |
| TB-SQL-AGG | SQL++/Mongo aggregation conveniences beyond subset | `deliberate-exclusion` until promoted | absent | SQL_TO_RQL | |

Promotion of any Tier B row into Tier A before Q1 corpus freeze requires principal
amendment of this matrix and the programme §2 tables.

---

## 3. Tier C — explicitly deferred

| id | Capability | Class | Owner |
|---|---|---|---|
| TC-FTS | Full-text search | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-VEC | Vector search | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-GEO | Geospatial search | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-GRAPH | Recursive graph traversal | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-CHANGE | Change streams / live queries | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-SPILL | Analytics-scale external-spill pipelines | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-WRITE-Q | Server-side write/update query pipelines | `deliberate-exclusion` | RQL read-only doctrine |
| TC-ML | Predictive / ML query operators | `deliberate-exclusion` | FUTURE_ROADMAP |
| TC-OFFSET | SQL OFFSET silent prefix discard | `deliberate-exclusion` | RQL_SPEC §3 deliberate |
| TC-DDA | Ranked `at rank` / direct access policies | `deliberate-exclusion` for Gate-1 unless promoted | DIRECT_ACCESS_SPEC |
| TC-ACCESS-POL | sequential/direct/build access policies | `deliberate-exclusion` | DDA-linked |

Tier C is named product backlog, not an unspoken deficiency.

---

## 4. Frontend and runtime surfaces (profile freeze)

| Surface | Gate-1 role | Notes |
|---|---|---|
| Application Core RQL | Primary product syntax | op 118 + embedded |
| Full RQL (enrich/within/brace project) | Tier A semantics required; wire may lag | Local path until wire parity |
| SQL-ish+ (`sql+`) | Declared subset only | refuse outside subset |
| JSON / Mongo dialect → QVM | Portable filter path | Not full Mongo aggregation |
| Rust builder | Equivalent frontend → same QVM | Q2 identity exit |
| Raw SDA / dialect `sda` | **Not** Gate-1 product query path | Explicit raw API only |
| Test-only semantic oracle | Q3 only | Never product path |

---

## 5. Exit (Q0.2)

- [x] Every §2.1 programme surface has a row and class
- [x] Tier B and C named
- [x] No Tier A `TBD`
- [x] Blockers called out for Q2 ordering
- [ ] Principal accept of classifications (especially aggregate blockers vs SPEC v1)

**Principal decision needed:** confirm Tier A includes grouping/aggregates and
computed/conditional projection (programme text) despite RQL_SPEC v1 exclusion
— labor treats them as **blockers requiring SPEC amendment**, not silent demotion
to Tier C.
