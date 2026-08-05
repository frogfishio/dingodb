# Path to full RQL — de-risk expressiveness + performance

Status: **labor map 2026-08-05** (post store P0 / packaging 0.2.2 diversion);  
**Decision 0 (HARD):** [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)  
**Bytecode freeze (RQL-X1):** [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) (`residiuum-query-bytecode-v1`)  
**What’s left (read this first):** [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)  
**RQL-0** gap ledger [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §0 (CRITICAL_PATH §4.6);  
**Phase 1–3** checklists under `doc/todo/rql/` (Phase 3 = **port inventory**)  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md) ·  
[NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md) ·  
[RQL_SPEC.md](../../wip/query/RQL_SPEC.md) ·  
[DIALECTS.md](../../SDA/DIALECTS.md)

**Question answered:** how do we get from *here* to *full RQL support* so we can
run comprehensive tests that decide (1) SQL-class expressiveness and (2) whether
queries perform?

---

## 1. Where we are (honest)

| Layer | State | What it can do today |
|---|---|---|
| **APP-4** predicates + `RqlPlanV1` | **accept** | Canonical plan encoding; builder ↔ fixture hash lock |
| **APP-5** Application Core compiler | **accept** | `from` / multi-`where` / `project` / `order` / coverage / consistency / budgets → plan; **rejects** `enrich` / `within` / `at rank` / access / Core-`after` |
| **APP-6** page executor + cursors | **active** (labor largely in_review historically) | Bounded multipage execution for Core plans |
| **APP-7 / APB-7** `rql_query` op **118** | **active**, **not package accept** | Embedded + remote façade path; dual-pack evidence; **no** product “query qualified” claim |
| **RQL-v1 full language** (`rql-full-v1`) | labor through **T3.10** + **F1/F2** `in_review`; **not package accept** | Enrich/within/project + explain + corpus; scan-attach façade; op 118 **refuses** full-language; `at rank`/index/parity residual |
| **SQL-ish+ → RQL** | design [SQL_TO_RQL_SPEC.md](./SQL_TO_RQL_SPEC.md) | Spec only; not a shipped compiler package |
| **v0.1 dialect** (`dialects/rql`) | guide [USER_GUIDE.md](../../RQL/USER_GUIDE.md) | ENR/SDA `from`/`enrich`/`project` subset — **parallel** to Application Core, not the APB-7 plan runtime |

Two different “RQL” surfaces still coexist:

```text
Application Core (rql-app-core-v1) ──compile──► RqlPlanV1 ──execute──► APB-7 host
      │
      └── product path for M1 query baseline

Legacy dialect rql v0.1 ──► ENR+SDA text ──► older execute paths
      │
      └── useful for enrich experiments; NOT the APB-7 accept surface
```

**Implication:** “Full RQL” for *comprehensive product tests* means the
**plan-encoded** language on the APP-4/5/6 → APB-7 spine — not only expanding
the old ENR dialect.

---

## 2. What “yes” looks like (two bars)

### 2.1 Expressiveness (vs SQL)

Not “implement PostgreSQL.” Residiuum deliberately **does not** flatten joins,
offsets, arbitrary aggregates, or DDL in RQL v1 ([RQL_SPEC §3](../../wip/query/RQL_SPEC.md)).

The honest SQL-class bar:

| SQL-ish need | Residiuum answer | When testable |
|---|---|---|
| Filter + project + order + page | Application Core | **Now** (APP-5/6/APB-7) |
| Equijoin / attach related docs | `enrich` + cardinality (`exactly_one`/`optional`/`many`) | After **RQL-v1 full language** package |
| Nested attach / bag-scoped filter | `within` / nested enrich | Same full-language package |
| Ranked “top-k” without offset scan | `at rank` / direct access | After DDA packages (later than v1 Core) |
| Aggregates / GROUP BY | **Out of RQL v1** → APB-8 aggregates lane | Separate package |
| Refuse illegal SQL silently | SQL-ish+ compiler: emit / conditional / **refuse** | After SQL→RQL compiler package |

**Pass criterion for expressiveness de-risk:** a locked corpus of
*supported* shapes + *refuse* shapes (SQL and RQL) with deterministic plan hashes
and execution oracles — not anecdote.

### 2.2 Performance

Store P0 / CompactShadow work improved the **byte plane**. Query perf is a
**separate** claim chain (plan → scan/index choice → page cost → budget).

**Pass criterion for query perf de-risk:**

1. Correctness corpora green on dual backend (embedded + remote).
2. Bounded campaigns (disk-safe TMPDIR; no accidental multi‑GiB dumps).
3. Oracles: scan vs index pushdown parity (APB-7 T4 already started this).
4. Explicit budgets / deadlines fail closed (APB-7 T8/T9 evidence).
5. Only then: publish numbers with profile labels — never as store-suite
   “known tip” waivers.

---

## 3. Sequence (smallest path to comprehensive tests)

Do **not** jump to full-language enrich before Core execution is package-honest.

```text
Phase 0  PATH map (this doc) + board staging
    │
Phase 1  CLOSE M1 query baseline (Application Core product)
    │      • APB-7 + APP-6 residual → principal package accept
    │      • Comprehensive Core corpus: compile + execute + multipage oracle
    │      • Dual-pack embedded/remote on op 118
    │      • Scoreboard: APB-7 / APP-6 / APP-7 → accept (principal)
    │
Phase 2  EXPRESSIVENESS matrix on Core (+ SQL-ish refuse edges)
    │      • Expand rql_app_core_corpus + plan_vectors
    │      • Adversarial “gotcha” cases (absent≠null, coverage holes, budgets)
    │      • Optional: start SQL_TO_RQL pure compiler (emit/refuse only)
    │      → Answer: “Core is SQL-filter-class expressive; joins still pending”
    │
Phase 3  FULL RQL-v1 language (enrich / within / …)
    │      • Pull board card 89a80e77 only AFTER Phase 1 accept
    │      • Spec-first grammar amend if needed; APP-5 profile bump or APP-5b
    │      • Executor enrichment attach path + cardinality oracles
    │      • Corpus: multi-collection attach vs independent get-oracle
    │      → Answer: “attach-class SQL joins expressed honestly”
    │
Phase 4  QUERY PERF campaign (disk-safe)
         • After Phase 1 (Core) minimum; Phase 3 for enrich costs
         • Index vs scan, page sizes, remote RTT, budget ceilings
         → Answer: “performing well” with evidence paths
```

### What to run for “comprehensive tests” at each phase

| Phase | Compile tests | Execute tests | Perf |
|---|---|---|---|
| 1 | `rql_app_core` + plan_vectors | APB-7 dual-pack + multipage oracles | smoke only |
| 2 | expanded Core corpus + refuse matrix | same executors | optional micro |
| 3 | full-language corpus | enrich attach oracles | after green |
| 4 | — | — | bounded campaigns |

Suggested crate commands (disk-safe):

```text
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --lib rql_app_core -- --test-threads=1
cargo test -p residiuum-sdk --test app5_rql_app_core -- --test-threads=1
cargo test -p residiuum-sdk --test app4_predicate_plan -- --test-threads=1
# APB-7 / APP-6 / HAR-4 query gates as named in scoreboard evidence docs
```

---

## 4. Board binding (labor)

| Priority | Card intent | Stage rule |
|---:|---|---|
| 0 | This PATH map | `todo`→`doing`→`in_review` (this turn) |
| 1 | Close APB-7 / APP-6 residual for Core comprehensive suite | claim from **todo** only |
| 2 | Expressiveness + refusal corpus (Core + SQL-ish edges) | after Phase 1 labor ready |
| 3 | Full RQL-v1 language (`89a80e77`) | keep **backlog** until Phase 1 accept; then promote |
| 4 | Query perf matrix (disk-safe) | after Phase 1 correctness; enrich costs after Phase 3 |

Do **not** treat store Step 9 / CSE perf campaigns as substitutes for query
de-risk.

---

## 5. Explicit non-claims

- Packaging **0.2.2** closed segment-identity P0; it does **not** qualify RQL.
- Application Core ≠ full RQL v1.
- Op 118 active ≠ package accept ≠ “as expressive as SQL.”
- Aggregates / windows / SQL offset remain out of RQL v1 by design.

---

## 6. Next concrete pull

**Canonical answer:** [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

```text
NEXT labor  = RQL-X2 implement shared bytecode runtime + delete frozen executors
FROZEN      = query_exec_v1 + execute_rql_full (Decision 0)
BYTECODE    = residiuum-query-bytecode-v1 (RQL-X1 architecture freeze)
NOT next    = S1, D1, wire parity, façade features, premature “query qualified”
```

Prior Phase 3 / F1/F2/I1 labor is **port inventory** under Decision 0 — do not grow those executors.
