# RQL-0 — semantic and implementation inventory

Status: **labor 2026-08-05** · CRITICAL_PATH §4.6 · **Decision 0 amended**  
Board: Query spine Feature `1a8a3e05` · inventory `190a97bd` · Decision 0 `d06c909f`  
**What’s left (short):** [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)  
**Convergence charter:** [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md) ·
[RQL_SPEC.md](../../wip/query/RQL_SPEC.md) ·
[PATH_TO_FULL_RQL.md](./PATH_TO_FULL_RQL.md) ·
[NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md) ·
[DIALECTS.md](../../SDA/DIALECTS.md)

This is the **gap ledger + dependency-ordered package sequence** required by
CRITICAL_PATH §4.6. It is not a second roadmap family. Living package states
remain on the scoreboard; Kanban owns labor workflow only.

**Law:** refuse new RQL syntax until its semantics and execution owner are named
in this ledger (or an amended RQL_SPEC section referenced here).

---

## 0. Decision 0 — one bytecode, one runtime (HARD)

**Class:** architectural violation → convergence. Not scaffolding.

Principal freeze (2026-08-05): parallel **semantic** executors in production are
forbidden. The doctrine end state is:

```text
RQL source ─┐
SQL+ ───────┼─► canonical logical plan ─► one query bytecode ─► one runtime
Rust builder ┘                                      │
                                                    └─► host capabilities
                                                        scan / index / get
```

| Yes | Absolutely not |
|---|---|
| Multiple syntaxes | Multiple semantic executors |
| Multiple compiler stages | Host implementing filter / join / project / order / cardinality / pagination / missing-null / coverage |
| Multiple physical access strategies | A second product runtime |

**In-tree violation (feature-frozen):**

- `crates/residiuum-sdk/src/query_exec_v1.rs` — Core / op-118 page semantics
- `crates/residiuum-sdk/src/rql_full_v1.rs` (`execute_rql_full`) — attach/project façade semantics

**Allowed:** test-only reference interpreter as oracle (not a product path).

**Honesty:** ENR+SDA currently compiles to text and runs through `residiuum-sda`;
that is **not yet** a fully frozen query bytecode. Defining that bytecode and
the host-capability boundary is the next architecture deliverable
([QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) steps 1–8).

**Gate:** until Decision 0 convergence architecture is frozen (bytecode + host
boundary named), **no additional RQL feature work** proceeds (no S1 / D1 /
wire-parity / within-index / new attach surface growth on the frozen executors).

---

## 1. Surfaces (columns)

| Column | Meaning |
|---|---|
| **Parser** | Source accepted/refused with stable diagnostic |
| **Plan** | Canonical `RqlPlanV1` / full compile artefact |
| **Exec** | Host page/attach execution |
| **Index** | Admitted index path (not scan-only oracle) |
| **SDK/wire** | Façade + remote op **118** when claimed |
| **Evidence** | Named corpus / test / dual-pack gate |

Classification: `implemented` · `partial` · `contradictory` · `absent`

Profiles in play (**Decision 0:** the first two are frozen violation surfaces to
converge away from; they are not the intended product architecture):

```text
rql-app-core-v1  → RqlPlanV1 → query_exec_v1 / op 118   (FROZEN — port then delete)
rql-full-v1      → CompiledRqlFull → execute_rql_full    (FROZEN — port then delete)
rql-source-v0.1  → ENR+SDA text → residiuum-sda          (kernel lineage; not frozen bytecode yet)
sql+             → emit/refuse → logical plan            (frontend only; must share one runtime)

TARGET:
  any syntax → canonical logical plan → one query bytecode → one runtime
             → host scan/index/get only
```

---

## 2. Construct ledger (RQL_SPEC §3 / §5)

| Construct | Parser | Plan | Exec | Index | SDK/wire | Evidence | Class | Notes |
|---|---|---|---|---|---|---|---|---|
| `from` root collection | Core+full | Core | Core | scan / point | Core + op 118 | app5 corpus; apb7 dual | **implemented** | Product Core path |
| Root `where` (pre-attach) | Core+full | Core | Core | pushdown partial | Core + op 118 | app5; apb7_index_pushdown | **partial** | Range/multi-field multipage residual |
| Named parameters `$` | Core+full | Core | Core | — | Core + op 118 | app5; cursor param MAC | **implemented** | |
| Flat `project a, b` | Core | Core | Core | — | Core + op 118 | app5 | **implemented** | Core-owned when no braces |
| Brace `project { … }` nested | full | full strip | full post-pipe | — | **local façade only** | rql_full_project; corpus | **partial** | Not on op 118 wire |
| `order by` + key tie-break | Core | Core | Core | — | Core + op 118 | app6 field-order | **implemented** | |
| `limit` / `page size` | Core | Core | Core | — | Core + op 118 | app6 page executor | **implemented** | |
| Authenticated `after` / cursor | Core reject → APP-6 | cursor-v1 | APP-6 | — | Core + op 118 | app6_cursor; apb7_cursor_secrets | **partial** | Heap-confined secrets residual |
| `coverage` / `consistency` | Core | Core | Core | — | Core + op 118 | apb7_coverage_grade | **implemented** | Incomplete fail-closed |
| `budget` docs/bytes/result | Core | Core | Core | — | Core + op 118 | app5; apb7 deadline | **implemented** | |
| `explain` (Core) | Core | Core | n/a | — | Core + op 118 explain | explain_rql_source | **implemented** | Plan tree + hash |
| `explain` (full) | full | full tree | n/a | — | local explain API | rql_full_explain | **implemented** | RQL-F1; not on op 118 |
| `enrich` + cardinality | full; Core **refuse** | full EnrichStep | scan attach | **root eq-index** | local; **wire refuse** | rql_full_*; I1 | **partial** | root index+scan fallback; within still scan; op 118 refuse |
| Enrich candidate `where` | full | full | scan filter | absent | local; wire refuse | rql_full_candidate_where | **partial** | Same attach residual |
| `within` nested carrier | full; Core refuse | full Within | bag map | absent | local; wire refuse | rql_full_within/nested | **partial** | MAX_WITHIN_DEPTH host bound |
| Nested `where` in `within` | full | full Filter | bag filter | — | local; wire refuse | rql_full_nested_where | **partial** | |
| Root post-attach `where` | full | pipeline Filter | page-then-attach | — | local; wire refuse | rql_full_root_where | **partial** | Not global re-page/re-limit |
| Ordered multi enrich/within | full | pipeline | ordered attach | absent | local; wire refuse | rql_full_multi/nested | **partial** | |
| Brace `project { … }` | full | full | post-pipe | — | local; wire refuse | rql_full_project | **partial** | Flat Core project still on wire |
| `at rank` / ranked access | refuse both | — | — | — | — | Core+full refuse | **absent** | DDA / DIRECT_ACCESS owner |
| Access policies (`sequential`/`direct`/`build`) | Core refuse | — | — | — | — | app5 reject | **absent** | DDA-linked |
| Aggregates / GROUP BY | refuse (out of v1) | — | — | — | — | sql+ refuse | **absent** | APB-8 lane — not RQL v1 syntax |
| SQL offset / silent discard | refuse | — | — | — | — | design | **absent** | Deliberate v1 exclusion |

Legacy `rql-source-v0.1` (ENR dialect) remains a **parallel** surface — not an
APB-7 accept owner. Do not treat its enrich demos as product wire evidence.

---

## 3. SQL-use-case corpus (adequacy bar)

| Corpus | Path | Counts | Role |
|---|---|---|---|
| Application Core | `spec/app/v1/rql_app_core_corpus_v1.json` | accept **15** / reject **17** | Core compile lock |
| Core execute | `tests/app_core_execute_corpus.rs` | Phase 1 labor | Compile→page oracle |
| Full attach | `spec/app/v1/rql_full_v1_corpus_v1.json` | accept **8** / refuse **5** / execute **3** | Attach-class lock |
| SQL-ish+ | `spec/app/v1/sql_plus_corpus_v1.json` | emit **4** / refuse **8** | Emit/refuse scaffold → Core |
| Plan vectors | `spec/app/v1/plan_vectors_v1.json` | hash lock | APP-4 encoding |

**Adequacy gaps (not yet a Gate-1 exit corpus):**

1. SQL JOIN → enrich/`within` emit vectors (joins currently **refuse** only).
2. Independent semantic oracle separate from the optimiser/executor (CRITICAL_PATH §4.3).
3. Damage/incomplete-coverage query cases beyond Core coverage grade.
4. Cross-engine comparison harness (MongoDB / Couchbase Lite) — Gate-1 read qual.

---

## 4. Reference-oracle boundary (frozen for next packages)

```text
Core page oracle     = list_keys + get (+ field-order multipage) vs executor page
Attach oracle        = complete foreign list_keys + get + cardinality attach
Index differential   = admitted only where APB-7 index pushdown already claims
                       (Core predicates); enrich match keys = scan-only until RQL-I*
Wire oracle          = op 118 response shape vs embedded Collection.rql for Core only
```

**Rules:**

- A hole must never become an empty complete page (coverage honesty).
- Scan attach must not be labeled “index pushdown.”
- `execute_rql_full` must not be labeled “product op 118 enrich.”
- Refuse diagnostics are successful safety outcomes (sql+ / Core non-features).

---

## 5. Dependency-ordered next packages

Pull **only** in this order unless principal amends CRITICAL_PATH. Each package
owns one concern; inventing parallel “PATH T*” without updating this sequence
is forbidden.

| ID | Package | Depends on | Exit (labor → principal) |
|---|---|---|---|
| **RQL-0** | This ledger | CRITICAL_PATH | Ledger + sequence accepted (this card) |
| **RQL-0D** | Decision 0 charter + feature freeze | RQL-0 | **this amend** — [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md); freeze `query_exec_v1` / `execute_rql_full` |
| **RQL-X1** | Freeze query **bytecode** + host-capability boundary | RQL-0D | Normative bytecode profile; host = scan/index/get only |
| **RQL-X2** | Lower all syntaxes + route emb/op118; port; equivalence; delete Rust executors; CI anti-executor | RQL-X1 | One runtime; CI gate; production semantic executors gone |
| **RQL-C1** | Core product accept residuals | RQL-X2 (or principal waiver) | Scoreboard APP-6/APP-7/APB-7 → `accept` (principal) |
| **RQL-F1** | Full explain artefact for `rql-full-v1` | RQL-0, Phase 3 surface | **labor closed** — port inventory only under Decision 0 |
| **RQL-F2** | Op-118 enrich/within/project wire **or** explicit wire refuse | RQL-F1 | **labor closed (refuse path)** — parity deferred to shared runtime |
| **RQL-I1** | Index pushdown for enrich match keys | RQL-F2 decision, Core index path | **labor closed** — port inventory; do not grow on frozen façade |
| **RQL-S1** | SQL+ → enrich/`within` emit (JOIN class) | **RQL-X1** minimum | Emit into logical plan → shared bytecode only |
| **RQL-D1** | `at rank` / access policies | DDA specs + **RQL-X1** | Spec-first; only after DIRECT_ACCESS + bytecode owner frozen |
| **RQL-Q1** | Query perf / read qualification campaign | RQL-C1 + shared runtime | CRITICAL_PATH §4.4 evidence law |

**Explicitly not next:** further features on `query_exec_v1` / `execute_rql_full`,
Studio/UI, search dialects, APB-8 aggregates syntax, store perf as RQL
substitutes, Embedded product build-out.

---

## 6. Named owners (syntax admission gate)

| Construct / change | Semantic owner | Execution owner | May add syntax? |
|---|---|---|---|
| All query meaning | RQL_SPEC + ENR/SDA | **one bytecode runtime** (RQL-X*) | No until RQL-X1 freezes bytecode |
| Core surface (interim) | RQL_SPEC §3.1 + APP-5 | `query_exec_v1` (**frozen**) | No |
| enrich / within / brace project (interim) | RQL_SPEC §6–9 | `execute_rql_full` (**frozen**) | No — port via RQL-X2 |
| `at rank` / access | DIRECT_ACCESS_SPEC | DDA + shared bytecode (RQL-D1) | **No** until RQL-D1 + RQL-X1 |
| Aggregates | APB-8 / future spec | APB-8 | Out of RQL v1 |
| SQL+ emit extensions | SQL_TO_RQL_SPEC | sql+ → logical plan → bytecode | Only via RQL-S1 after RQL-X1 |

---

## 7. Evidence commands (disk-safe)

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --lib rql_app_core -- --test-threads=1
cargo test -p residiuum-sdk --test app5_rql_app_core -- --test-threads=1
cargo test -p residiuum-sdk --test rql_full_corpus -- --test-threads=1
cargo test -p residiuum-sdk --test app_core_expressiveness -- --test-threads=1
```

---

## 8. Non-claims

- This ledger does **not** accept APB-7 / APP-6 / APP-7 or full RQL-v1.
- Phase 3 attach labor ≠ Gate-1 RQL exit (CRITICAL_PATH §4.5).
- Packaging 0.2.2 / CompactShadow campaigns ≠ query qualification.
- Board `in_review` ≠ package `accept`.
- Decision 0: `query_exec_v1` / `execute_rql_full` are **not** the product
  architecture; they are frozen violation surfaces pending RQL-X*.
- ENR+SDA text evaluation is **not** yet the frozen query bytecode.
