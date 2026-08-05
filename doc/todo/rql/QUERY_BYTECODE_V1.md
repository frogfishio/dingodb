# Query bytecode v1 — profile freeze (RQL-X1)

Status: **architecture freeze 2026-08-05** · board RQL-X1 `0f28219d`  
Profile id: **`residiuum-query-bytecode-v1`**  
Charter: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)  
Decision 0: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §0  
Dialect doctrine: [DIALECTS.md](../../SDA/DIALECTS.md)

This document **freezes the architecture boundary**. RQL-X2 foundation
(`query_bytecode_v1`) landed the product entry + `HostCapabilities` + CI gate;
binary ISA + full semantic port remain **RQL-X2b**. Until this profile is
amended, all query work must obey it.

---

## 1. Ordered stack (only legal shape)

```text
syntaxes (RQL | SQL+ | Rust builder | pure ENR+SDA text | …)
    │
    ▼
canonical logical plan          (RqlPlanV1 + admitted full-language extensions)
    │
    ▼
query bytecode                  (residiuum-query-bytecode-v1)
    │
    ▼
ONE runtime                     (bytecode machine)
    │
    ├──► HostCapabilities       (scan / index / get / … data only)
    └──► (explain / budgets / coverage as bytecode effects + evidence)
```

| Layer | Owns | Must not own |
|---|---|---|
| Syntax frontends | Parse / refuse / lower | Execution meaning |
| Logical plan | Canonical query meaning before physical choice | Storage IO |
| Bytecode + runtime | Filter, enrich/cardinality, project, order, page, missing/null, coverage | Collection bytes |
| Host | Key streams, gets, index candidate keys, heap binding | Query algebra |

**Law:** multiple syntaxes and compiler stages are fine. Multiple **semantic**
executors are forbidden.

---

## 2. What “bytecode” means in this freeze

**Target:** a single executable program form for the query runtime, derived
from the logical plan, independent of which syntax produced the plan.

**Lineage:** ENR + SDA (today: text → `residiuum-sda` eval) is the
**mathematical kernel** for value/match/cardinality semantics. The query
bytecode **must lower to that kernel** (or an equivalent frozen binary
encoding of the same AST). It is **not** a second algebra.

**Honesty (current tree):**

| Today | Under this freeze |
|---|---|
| Dialect `rql` → ENR+SDA **text** → `residiuum-sda` | Legal lineage; text is an interim encoding, not a second product runtime |
| Former `query_exec_v1` / `execute_rql_full` | **Removed (X2d)** — semantics under `query_bytecode_v1/` |
| Rust plan/AST as runtime input | Interim views — durable carrier is **`residiuum-query-isa-v1`** ([QUERY_ISA_V1.md](./QUERY_ISA_V1.md)) |
| ENR+SDA text → `residiuum-sda` | Legal lineage; kernel **eval** as product substrate remains **RQL-X4** |

RQL-X3 freezes the durable ISA encoding. **RQL-X4** lowers execution into the
ENR+SDA kernel evaluator.

---

## 3. Host-capability boundary (normative)

The host may expose **only** data-access capabilities. Names are architectural;
Rust traits may match later.

### 3.1 Admitted host ops

| Capability | Meaning |
|---|---|
| `list_keys(collection, after?, limit?)` | Deterministic key stream |
| `get(collection, key)` | Document bytes / JSON value or absence |
| `lookup_index_keys(collection, equality…)` | Optional candidate keys; **not** a semantic filter |
| `heap_id` / collection binding checks | Isolation / refuse cross-Heap |
| Budget / cancel / deadline **signals** | Cooperative stop — host does not invent result rows |

Index lookup returns **candidates**. The bytecode machine must still apply
predicates / cardinality / coverage. A host must never treat “index miss” as
“empty complete answer” without bytecode coverage rules.

### 3.2 Forbidden in the host (semantic)

The host **must not** independently implement:

- filtering / predicate evaluation as the product meaning
- joins / enrich / attach / cardinality (`exactly_one` / `optional` / `many`)
- projection shaping
- ordering / nulls placement / tie-break
- pagination / cursor logic as query meaning (minting may be host crypto; **page selection** is bytecode)
- missing vs null vs value decisions
- coverage / consistency grade decisions
- explain trees of query meaning

Those meanings belong **only** to the bytecode machine (and its ENR/SDA
lowering).

---

## 4. Logical plan role

`RqlPlanV1` (APP-4) remains the **canonical Core logical plan**.

Full-language constructs (`enrich` / `within` / brace `project` / …) extend the
logical plan surface (or a versioned companion plan) that **still lowers to the
same bytecode**. They must not introduce a second executor.

SQL+ and the Rust builder emit logical plan (or RQL source → plan), never a
private runtime.

---

## 5. Runtime singularity

| Path | Rule |
|---|---|
| Embedded SDK query | Same bytecode runtime |
| Op **118** `rql_query` | Same bytecode runtime (wire carries plan/bytecode/source+bindings — physical packing is RQL-X2) |
| Explain | Same compiler pipeline; no executor-private explain |
| Test oracle | Independent **reference interpreter** allowed in tests only; never linked as product path |

---

## 6. Port inventory (from frozen executors)

Useful behavior to **port into** the bytecode machine (RQL-X2), not grow in place:

From `query_exec_v1`:
- multipage field-order + cursor resume
- Core predicate eval + budgets / deadline
- equality-index candidate acceleration with re-eval
- coverage / consistency evidence shapes

From `execute_rql_full`:
- ordered enrich / within pipeline
- cardinality attach
- candidate `where`, nested `within`, brace `project`
- root enrich index candidate path
- structured explain artefacts

Equivalence: old façade results vs new runtime must be proven before delete.

---

## 7. CI architecture gate (RQL-X2 deliverable)

A CI check must fail if a new production semantic executor appears (e.g. another
`execute_*` that evaluates predicates/enrich outside the bytecode crate). Exact
harness is RQL-X2; this freeze **requires** that gate as exit criterion.

---

## 8. Amendment rule

Amending this profile requires an explicit RQL-0 / Decision 0 note and board
card. Silent growth of host semantics or a new Rust executor is a process
violation.

---

## 9. Non-claims

- No claim that bytecode encoding is unimplemented — see [QUERY_ISA_V1.md](./QUERY_ISA_V1.md).
- No claim that ENR+SDA **eval** is the product executor yet (RQL-X4).
- No APP-6 / APP-7 / APB-7 package accept.
- No full RQL product qualification.
- Phase 3 corpora remain **port inventory + evidence**, not a second architecture.
