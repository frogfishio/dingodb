# Query Dialects

Status: Draft v0.4

Audience: SDK authors, host integrators, advanced query users
Normative companions: [SDA_SPEC.md](../reference/query/SDA_SPEC.md), [DX_SPEC.md](../reference/product/DX_SPEC.md) §7

## Thesis

**SDA (and ENR1 on top of it) is the mathematical language.** Pure ENR + SDA
notation is exact and often atrocious to write. Some people will love it
(mathematicians, data scientists, set theorists, academics). Everyone else
gets dialects.

Dialects are frontends that *compile into* the same ENR+SDA IR / bytecode.
They never redefine SDA meaning. Compile is the expensive semantic step;
execution is shared.

**Decision 0 / RQL-X* (2026-08-05):** parallel product semantic executors are
forbidden. Architecture freeze:
[QUERY_BYTECODE_V1.md](../todo/rql/QUERY_BYTECODE_V1.md)
(`residiuum-query-bytecode-v1`). Host = scan/index/get only. Former Rust
executors (`query_exec_v1` / `execute_rql_full`) were removed in X2d; product
path is `query_bytecode_v1/`. Durable binary ISA remains **RQL-X3** — not a
second doctrine.

```text
  Pure ENR+SDA ──┐
  RQL (official) ┤
  JSON/Mongo ────┼── compile ──► [ENR + SDA IR / bytecode] ──► one runtime
  SQL mimicry ───┤                         │
  GraphQL / … ───┤                         └─► host: scan / index / get
  Fluent API ────┘
```

| Kind | Role |
|------|------|
| **Pure ENR + SDA** | Mathematical kernel; always available; only lossless path for full algebra |
| **RQL** | **Official** human dialect of Residiuum — co-designed with ENR; faithful lowering ([RQL_SPEC.md](../wip/query/RQL_SPEC.md)) |
| **Foreign dialects** | Comfort / familiarity (SQL-ish, Mongo-ish, …); admit holes and notes |
| **Fluent / builder API** | Host-native code surface; same IR target |

Rule:

- **Pure SDA / ENR1** is always available and always correct relative to the corpus.
- **RQL** is the preferred product text surface for enrichment + projection once implemented.
- **A dialect** MUST compile to pure SDA / shared IR (or refuse with a clear error).
- **No foreign dialect** may claim to be a complete encoding of SDA, SQL, MongoDB, or
  GraphQL. Mimicry is intentional and documented.
- Hosts MAY register additional dialects. Builtin dialects are conveniences,
  not a closed set.
- **If the distinction matters and a dialect cannot say it, use pure SDA (or RQL when it covers that construct).**

## Why dialects, not hybrid

A **hybrid** model would treat SQL / Mongo / GraphQL and SDA as co-equal query
languages (or try to smuggle algebra into a “familiar enough” surface until the
pure notation is optional). That fails as soon as the job needs an exact
meaning the foreign surface does not have.

| Model | What it claims | Why we reject / accept it |
|-------|----------------|---------------------------|
| **Hybrid** | Pick SQL *or* Mongo *or* SDA as full peers; grow each until they “cover” the product | Impossible without lying: foreign null/cardinality models cannot encode SDA laws |
| **Dialects (this product)** | One mathematical kernel (SDA + ENR); dialects compile in and admit limits | Pure path remains available for exact work; comfort never redefines truth |

Ugly pure notation is still preferable to a pretty surface that cannot mean what
you need. Dialects exist so people are not forced to learn the algebra for
everyday filters — not so the algebra can be abandoned.

**Design authority:** the pure language is top-down (algebra first). Dialects
are the sanctioned place for user familiarity and “impurities.” We do not
wait for developers to invent SDA/ENR by feature request, and we do not bend
the kernel toward SQL because Bob asked. See
[DOCTRINE.md § Language Design Authority](./DOCTRINE.md#language-design-authority-top-down-kernel)
and [enr-core/README.md](../../crates/enr-core/README.md#how-this-language-is-designed-not-bob-called-oracle).

## The clincher: Null vs absence (no value)

SDA_SPEC §4.0.1:

- **Absence** — no binding for the key (`None` / missing).
- **Null** — a binding exists and its value is `Null` (`Some(Null)`).
- Optional (`?`) and required (`!`) extraction report absence from bindings,
  not from whether the stored value is null.

No amount of SQL mimicry or Mongo filter vocabulary recovers that distinction
losslessly:

| Surface | Typical collapse |
|---------|------------------|
| SQL `IS NULL` | Three-valued logic; document dialects often OR together “missing” and “JSON null” (this repo’s SQL mimicry does exactly that — with notes) |
| Mongo / JSON filters | `$eq: null` and `$exists` are separate operators, but the familiar object filter is still not the full SDA carrier algebra (no `Some`/`None`/`Fail` story, no ENR match bags) |
| GraphQL | Nullability in the schema is not SDA absence vs stored `null` |

**Whole point:** if you must differentiate **null** from **no value** (or other
exact carrier / failure outcomes), you are stuck with pure SDA. Dialects may
approximate; they must not pretend the approximation is the algebra.

Concrete pure SDA (distinguishes the two cases):

```text
// Stored null: Some(Null)
getPath(input, Seq["nickname"]) = Some(null)

// Absent key: None
getPath(input, Seq["nickname"]) = None
```

SQL dialect mimicry (`nickname IS NULL`) matches **both** missing and stored
null — useful comfort, not exact meaning. See builtin SQL notes and tests.

## Why not make SDA look like SQL?

SDA is **not** a database query language (SDA_SPEC opening). Relational SQL,
document query objects, and GraphQL selection sets each smuggle different
cardinality, null, and failure models. Translating them losslessly is
impossible without changing one of the models.

So we do not bend the algebra toward familiarity. We provide:

1. the **pure language** (hard, complete, mathematical — including ENR1);
2. **RQL** — the official human dialect co-designed to lower faithfully into ENR IR;
3. a **bunch of foreign comfortable options** that cover common journeys and admit
   their limits.

Example: SQL `NULL` three-valued logic is not SDA absence/`Null`/`Fail`. The
SQL dialect maps a useful subset and emits notes when the mapping is shallow.
RQL does not adopt SQL’s null model; it inherits ENR+SDA carriers.

## Builtin dialects

| Id | Surface | Compiles to | Maturity |
|----|---------|-------------|----------|
| `sda` | Pure SDA / ENR1 source | identity (parse-checked) | **complete** for standalone + ENR1 |
| **`rql`** | **Residiuum Query Language** (official human surface) | pure ENR1/SDA program (`Match` / `enrich` / cardinality) | **v0.1 implemented** — [RQL_SPEC.md](../wip/query/RQL_SPEC.md) |
| `json` | DX/Mongo-style filter object | document predicate via [`Filter::to_sda`](../../crates/residiuum-sdk/src/filter.rs) | **complete** for the portable vocabulary (DX §7.1) |
| `mongo` | Alias of `json` | same | same (name for Mongo-familiar callers) |
| `sql` | Tiny legacy `SELECT` / `WHERE` mimicry | document predicate or projection program | **implemented, deprecated when SQL-ish+ ships** |
| `sql+` / `sql-plus` | **SQL-ish+** executable compatibility surface | canonical RQL v1 plan | **specified, not implemented** — [SQL_TO_RQL_SPEC.md](../todo/rql/SQL_TO_RQL_SPEC.md) |
| `graphql` | Reserved id | — | **scaffold** — not implemented |

### RQL (official)

RQL is the product’s preferred text dialect for multi-collection enrichment and
nested projection. It looks a little like SQL in places but is **not** SQL:
`enrich` attaches named fields, `expect` states cardinality, and `project` is
nested. How to write and run it: [USER_GUIDE.md](../RQL/USER_GUIDE.md). Full
design: [RQL_SPEC.md](../wip/query/RQL_SPEC.md).

**v0.1:** `compile_dialect("rql", …)` / `BuiltinDialect::Rql` lowers
`from` + `enrich … matching … expect …` (+ optional `project`) into the same
ENR1 programs pure text uses. Conformance tests prove RQL ≡ pure ENR on shared
bindings. Nested bag-scoped enrich (ForEach) is next.

### JSON / Mongo filter (`json`, `mongo`)

Already the everyday portable filter (DX_SPEC §7.1):

```json
{ "status": "active", "age": { "$gte": 18 } }
```

Compiles to a boolean SDA predicate over document binding `input`, e.g.:

```text
(getPath(input, Seq["status"]) = Some("active")) and (mapOpt(getPath(input, Seq["age"]), x => x >= 18) = Some(true))
```

Native evaluation and SDA evaluation agree on the portable vocabulary
(DEF-028).

### SQL mimicry (`sql`)

Supported sketch (not full SQL):

```sql
SELECT * WHERE status = 'active' AND age >= 18
SELECT name, city WHERE status = 'active'
SELECT * FROM users WHERE country IN ('TH', 'SG')
```

- `FROM` names are ignored by compilation (the host chooses the collection).
- `SELECT *` → document predicate (for `find` / filter paths).
- `SELECT a, b, …` → full program projecting a `Map` over matching rows of a
  sequence bound as `input`.
- Operators: `=`, `!=` / `<>`, `<`, `<=`, `>`, `>=`, `AND`, `OR`, `NOT`,
  `IN (…)`, `IS NULL`, `IS NOT NULL`, dotted field paths, string/number/bool
  literals.

**`IS NULL` is mimicry, not SDA law.** Compilation maps `col IS NULL` to
“missing **or** stored JSON null” and attaches a mapping note. That is the
comfort choice SQL users expect; it is **not** the pure algebra. To match only
stored null or only absence, use pure SDA (or the `json` dialect’s
`$eq: null` / `$exists` separately — still not full carrier algebra).

Out of scope (and refused or not recognized): joins, subqueries, aggregates,
`ORDER BY` / `LIMIT` (use SDK `QueryOptions`), functions, `LIKE`, DDL/DML.

This is not SQL-ish+. The separately specified `sql+` / `sql-plus` profile
translates a richer SQL subset into RQL, models SQL three-valued predicates
explicitly, and requires proof for hidden join cardinality before direct
execution. It is the intended successor to this legacy path. See
[SQL_TO_RQL_SPEC.md](../todo/rql/SQL_TO_RQL_SPEC.md).

### GraphQL (`graphql`)

Id reserved so hosts and docs can talk about `graphql → sda` in the same
pluggable table. Compilation currently fails closed with a clear error until a
deliberate selection-set subset is designed.

### Pure SDA (`sda`)

Pass-through after `Program::parse`. Prefer this when you need certainty rather
than comfort.

## Pluggable model (SDK)

Rust surface lives in `residiuum-sdk::dialects`:

- [`QueryDialect`] — trait for a frontend that compiles source → pure SDA.
- [`BuiltinDialect`] — builtin ids above.
- [`compile_dialect`] — dispatch by id (builtin registry).
- [`DialectRegistry`] — builtin + caller-registered custom dialects.
- [`CompiledSda`] — `sda` source, shape (predicate vs program), and mapping notes.

Custom dialects implement `QueryDialect` and register by id. They SHOULD:

1. refuse unmappable constructs (do not silently weaken SDA);
2. attach human-readable `notes` when the mapping is approximate;
3. target either a **document predicate** (`input` = one JSON doc) or a
   **program** (`input` = host value, often a sequence).

## Layering

```text
Application
    │
    ├─ fluent Filter / QueryBuilder / enrich builder   (no dialect string)
    ├─ dialect "rql"                    (official human surface — design)
    ├─ find_json / dialect "json"       (Mongo-style object)
    ├─ dialect "sql+"                   (SQL-ish+ → canonical RQL)
    ├─ dialect "sql"                    (legacy SQL mimicry)
    ├─ Collection::sda / sda_query      (pure ENR+SDA text)
    └─ dialect "sda"                    (same pure path, explicit id)
    │
    ▼
Compiled pure SDA / shared IR  →  residiuum-sda  →  value / Fail
```

Storage, indexes, budgets, and coverage remain **host** concerns. Dialects do
not open collections and do not change examination profiles.

## Non-goals

- Full SQL, Mongo aggregation pipeline, or GraphQL schema execution.
- Replacing SDA notation in the normative corpus.
- A **hybrid** product language where dialects are co-equal with SDA.
- Silent coercion of dialect null/missing models into “truthy” SDA results
  without notes (approximate maps like SQL `IS NULL` MUST remain documented).

## See also

- [RQL USER_GUIDE.md](../RQL/USER_GUIDE.md) — how to express and run RQL
- [RQL_SPEC.md](../wip/query/RQL_SPEC.md) — official Residiuum Query Language (design authority)
- [JSON_FILTER_DEMO.md](./JSON_FILTER_DEMO.md) — SDA as a jq-like filter
- [FOR_JQ_USERS.md](./FOR_JQ_USERS.md) — notation contrast for jq users
- [DOCTRINE.md](./DOCTRINE.md) — why certainty beats convenience
- [ENR README](../../crates/enr-core/README.md) — ENR1 text path (same pure kernel)
- DX_SPEC §7 — everyday query experience
