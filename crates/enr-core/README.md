# enr-core — Enrichment algebra (ENR1 in SDA; ENR2 design only)

Status: **ENR1 kernel implemented inside `residuum-sda`** (same parser/evaluator).  
**ENR2** remains design notes only — do not implement yet.

Companion specs: [ENR1.md](ENR1.md), [ENR2.md](ENR2.md), [SDA_SPEC.md](../../SDA_SPEC.md), [doc/SDA/DOCTRINE.md](../../doc/SDA/DOCTRINE.md).

## Decision: one compile path (SDA + ENR1)

SDA, ENR1, and ENR2 were **designed together**. Shipping ENR1 as a separate
parser would mean two languages and two compiles. Instead:

| Choice | Rationale |
|--------|-----------|
| **Add ENR1 to SDA** (`residuum-sda` / `crates/sda-core`) | One `Program::parse`, one AST, pure eval |
| Match bag = SDA comprehension + `asBag` / `matchBag` when carrier must be Bag | Reuse existing `{ r \| r in R \| pred }` form |
| Cardinality / merge as stdlib + sugar | `one?` / `one!` parse sugar; `only`, `first`, `last`, `merge`, `+` attach |
| ENR2 stays out | Candidate ranking / multi-source / explain = cottage industry |

Profile tag: `residuum_sda::ENR1_PROFILE_TAG` (`sda-enr1-v0.1`).  
Tests: `crates/sda-core/tests/enr1_kernel.rs`.

Standalone SDA conformance (`sda-standalone-v1.0`) is **unchanged**; ENR1 is
additive (new names + `+` on Prod/Map).

## Tandem design (SDA · ENR1 · ENR2)

| Layer | Primitive | Job | Code today |
|-------|-----------|-----|------------|
| **SDA** | Values, carriers, pure transforms | Tree/algebra on values the host supplies | `residuum-sda` |
| **ENR1** | Match bag | Relate left value ↔ right dataset; explicit cardinality + attach/merge/expand | **in `residuum-sda`** |
| **ENR2** | Candidate bag → resolve → combine → explain | Multi-source, refine, rank, ambiguity, provenance | specs only |

Shared laws (do not break when implementing any layer):

- Pure: no acquisition, HTTP, retries, auth, or file IO (Axiom / ResiduumDB host).
- Null ≠ absence; no match means empty bag / `NoMatch`, not `Null`.
- No implicit uniqueness or silent row drop.
- Duplicates preserved until an explicit operator resolves them.
- Failure tags are stable (`t_enr_*`).

SDA_SPEC parks enrichment outside the SDA core algebra narrative; ENR fills
that middle band so host join code is not the only place match policy lives.
Implementation still reuses the SDA surface so hosts compile once.

## ENR1 surface (shipped)

| Form | Meaning |
|------|---------|
| `{ r \| r in R \| pred }` | Match multiset (carrier follows `R`; use `asBag`/`matchBag` for Bag) |
| `Match(l, R, kL, kR)` | Special form: `{ r ∈ R \| kR = kL }` as **Bag** (`l`/`r` bound while keys eval) |
| `R[kR = kL]` | Keyed sugar → same match bag (`r` bound in `kR`/`kL`) |
| `enrich { field: E, … }` | Pipe sugar: for each left row in `_`, bind `l`, attach fields via Map `+` |
| `refine { … }` | Verb sugar for a bare SDA comprehension over pipe `_` (same as `{ … }`) |
| `one?(B)` / `oneOpt(B)` | 0 → `None`, 1 → `Some(v)`, >1 → `Fail(t_enr_duplicate)` |
| `one!(B)` / `oneReq(B)` | 0 → `Fail(t_enr_missing)`, 1 → `v`, >1 → `Fail(t_enr_duplicate)` |
| `only(B)` | Exact uniqueness (same empty/multi outcomes as `one!`) |
| `first(B)` / `last(B)` | Ordered policy on **Seq** only; Bag/Set → `t_enr_unordered_policy` |
| `merge` / `mergeFail` / `l + r` | mergeFail on Prod/Map; collision → `t_enr_field_collision` |
| `mergeLeft` / `mergeRight` | Explicit collision policies (keep left / right) |
| `source name : Index[…]` | Semantic source declaration (eval no-op; host still binds data) |
| `asBag` / `matchBag` | Force Bag carrier for ENR match-bag law |
| `t_enr_invalid_key` | Match key expr not comparable |

Example (attach required customer):

```sda
{
  yield o + Map{
    "customer" -> one!({
      c | c in customers
        | getPath(c, Seq["id"]) = getPath(o, Seq["customer_id"])
    })
  }
  | o in orders
}
```

Hosts that apply one program to many bags should `Program::parse` once and
`run_json` / `eval` per document — ENR operators ride the same path.

## Staging rule

| Now | Later |
|-----|--------|
| **ENR1 in `residuum-sda`** (this cut) | **ENR2** candidates / ranking / explain — not yet |
| Host engines (`Residuum::query`, hash Index) remain valid stand-ins | Optionally compile host joins from ENR1 programs |
| **Text path** — `Collection::sda` / `Residuum::sda_query` run ENR1+SDA source | Pushdown / plan compile of ENR1 text (optional) |

**Do not implement ENR2 yet.** Read it only to understand that ENR1 is the
*kernel* of a larger, co-designed surface—not a dead-end sketch.

Evidence that ENR1 is the right *first* cut: multi-collection join measurements
(nested pure SDA vs host hash equijoin) show the semantic gap is match +
explicit cardinality, not candidate ranking. See
`doc/PERFORMANCE_STRATEGIES.md` and the multi-collection SDA join tests under
`residuum-sdk`.

## Host text path (people still write ENR + SDA)

ENR + SDA are the **exact mathematical surface** — often atrocious DX, loved by
a small loud technical audience. Everyday product text is intended to be
**RQL** (Residuum Query Language): the official human dialect that lowers into the
same ENR+SDA IR. User guide: [doc/RQL/USER_GUIDE.md](../../doc/RQL/USER_GUIDE.md).
Design: [RQL_SPEC.md](../../RQL_SPEC.md).

**RQL** (`dialect "dql"`) is the official human surface and lowers into the same
ENR1+SDA programs (`Match` / `enrich` / cardinality). Fluent filters, equijoins,
pure ENR text, and **foreign query dialects** (`json` / `mongo` / `sql` → pure
SDA) remain everyday DX frontends, not hybrid peer languages: pure notation
remains the only lossless path for distinctions foreign surfaces cannot express
(notably **Null vs absence**). See [doc/SDA/DIALECTS.md](../../doc/SDA/DIALECTS.md)
and [RQL_SPEC.md](../../RQL_SPEC.md).

Users who prefer the algebra write programs as **text**:

| API | Role |
|-----|------|
| `Collection::sda(program)` | Scan one collection → `input` = doc array → pure SDA/ENR1 (DX §7.6) |
| `Collection::filter_sda(pred)` | Per-doc boolean SDA/ENR text predicate; keeps keys |
| `Residuum::enr_query().bind(..).run(program)` | Multi-collection ENR surface (`Match` / `enrich`); aliases free |
| `Residuum::sda_query()` | Same builder (alias of `enr_query`) |
| `Residuum::sda(&["orders","customers"], program)` | Convenience multi-bind + run |
| `eval_sda_program(program, input)` | Host already has the JSON value (object keys → free names) |

Example (preferred `Match` + `enrich` pipe against live collections):

```rust
let out = db.enr_query()
    .bind("orders")
    .bind("customers")
    .run(r#"
      orders
      |> enrich {
          customer:
            one!(
              Match(
                l,
                customers,
                getPath(l, Seq["customer_id"]),
                getPath(r, Seq["id"])
              )
            )
        }
      |> refine {
          yield o + Map{
            "customer_name" -> getPath(o, Seq["customer", "name"])
          }
          | o in _
        }
    "#)?;
```

Host join engines remain optimisations; they do not replace the portable text
surface.

## Why ENR1 “limits” are not design failures

| ENR1-only concern | How the tandem answers it (ENR2 / SDA, without implementing ENR2) |
|-------------------|---------------------------------------------------------------------|
| Equijoin-only feel | ENR2 `candidates(l, R, pred)` is predicate form; keyed match is sugar. Ranking / fuzzy is ENR2, not a missing ENR1 bug. |
| `one?` / `one!` binary outcomes | ENR2 `resolveOptional` / `resolveRequired` add `NoMatch` / `Unique` / `Ambiguous` / `Rejected` with decision provenance. ENR1 keeps the minimal interpretation. |
| `only` vs `one!` | Both reserved on the ENR2 surface; ENR1 minimal subset prefers `one!` / `one?`. |
| `first` / `last` on Bag | ENR2 `resolveFirst` / `resolveBest` require defined order (or fail `t_enr_unordered_policy`) and record rejected alternatives. ENR1 states the same order rule more thinly (Seq only). |
| Multi-generator expand vs SDA v1 (one generator) | Enrichment owns expand semantics; nest match bags inside yield for one-to-many until multi-generator lands. |
| Attach/merge want `Prod`; ResiduumDB JSON is `Map` | Both `Map` and `Prod` work with `merge` / `+`. |
| Multi-source / fallback / explain | ENR2. Explicitly out of ENR1 minimal subset. |
| Relation to `Residuum::query` | Host join is an **engine**. ENR1 is the portable program surface those engines can evaluate or compile from. |

## How this language is designed (not “Bob called Oracle”)

**Languages are not grown by waiting for application developers to request
features.** The pure surface (SDA + ENR) is designed **top-down** from a
closed algebraic model. Dialects (SQL/Mongo/…) are the comfort layer for
people who already speak something else. That split is intentional.

### PL/SQL was not a user wishlist

Oracle’s PL/SQL did **not** happen because “Bob from across the street”
called and asked SQL to grow loops. Primary historical account (Peter Clare /
Kendall Jensen et al., summarized in public Oracle-internals history):

- Competitive product strategy: vendors had weak triggers / external
  procedural hooks; Oracle wanted a **complete procedural language inside
  the database**, shared across tools — leapfrog, not react to one customer
  ticket.
- Language design by language people: modeled after **Ada** (strong typing,
  modularity; Ada/DIANA IR; PL/I lineage shared with SQL), implemented as a
  real compiler/VM project (1987+), later embedded in Oracle 7.
- Customer *use* later drove **performance and tooling**; it did not invent
  the language’s shape. Bob Miner’s team and PL/SQL authors owned the design.

Same pattern elsewhere: C was Ritchie/Thompson; Go was Griesemer/Pike/Thompson
with deliberate restraint; SQL’s relational core was Codd/Chamberlin–Boyce,
then vendor dialects and ISO/ANSI **committees** (not “change SQL because
one app shop asked”). Community processes (Python PEPs, C++ WG21 papers,
Rust RFCs, JS TC39) let users and implementers **propose**; they do not hand
the grammar to whoever files a ticket. Acceptance stays with language
authority. Where standards “begrudgingly” absorb popular impurities, that is
usually vendors shipping extensions first and committees codifying later —
still top-down ratification, not crowdsourced syntax.

**Users change *implementations* and *ecosystems* constantly.** Cases of
users *redesigning a language kernel* are rare: forks (e.g. community
taking a dead product), research languages that absorb lab practice, or
macro systems where the language *already* licenses user-defined surface
(Lisp). That is not the normal path for SQL, Ada, or a mathematical query
kernel.

### What that means for ENR1 / ResiduumDB

| Layer | Design authority | How it evolves |
|-------|------------------|----------------|
| **Pure SDA + ENR algebra** | Spec / doctrine (closed laws) | Top-down. Completeness is algebraic, not “someone asked.” |
| **Host engines** (hash join, indexes) | Product + storage | Optimize evaluation; do not redefine meaning. |
| **Dialects** (SQL, Mongo, …) | Comfort / familiarity | Mimicry with holes; compile into pure SDA or refuse. |

Do **not** treat missing ENR1 sketch items as “waiting for developers to say
what would be cool.” Those items were **already designed** with SDA. Leaving
them uncoded is an **implementation backlog** of a designed language, or a
hard **dependency** on SDA (e.g. multi-generator), not a user-research gap.

Dialects are where user impurities belong. The pure language does not grow
by absorbing SQL habits.

## Implementation cut of ENR1 (what shipped vs backlog)

**Shipped** (`sda-enr1-v0.1`): the **match-bag kernel** the join evidence
required — match formation, explicit cardinality (`one?` / `one!` / …),
attach/merge, `enrich` / `refine`, Map/`+`, `asBag` / `matchBag`. That is
what pure SDA alone could not productise and host hash joins hard-coded in
control flow.

**Not yet coded** (still part of the ENR1 *design*, not rejected, not
“waiting for Bob”):

| ENR1 sketch piece | In `sda-enr1-v0.1`? | Status |
|-------------------|---------------------|--------|
| Match bag comprehension + `Match(…)` | **Yes** | Kernel |
| `one?` / `one!` / `only` / `first` / `last` | **Yes** | Kernel |
| `merge` / `+` attach, `enrich` pipe | **Yes** | Kernel |
| `asBag` / `matchBag` | **Yes** | Kernel |
| **Source declarations** (`source X : Index[K,V]`) | **Yes** | Surface annotation; host `.bind` still supplies values |
| **Keyed sugar** `R[kR = kL]` | **Yes** | Desugars to match-bag comprehension |
| **`mergeLeft` / `mergeRight`** | **Yes** | Explicit collision policies |
| **`t_enr_invalid_key`** | **Yes** | Match key not comparable |
| **Multi-generator expand** | **No** | Designed; blocked on SDA multi-gen. Nest match bags until then. |
| **Provenance annotations** | **No** | Orthogonal; serious decision provenance is **ENR2**. |

### Framing (do not re-litigate)

1. **Design is top-down.** ENR1 was co-designed with SDA. Completing the
   designed surface is language work, not feature-request triage.
2. **Implementation may still be staged** (kernel first, sugar next, SDA
   dependencies when the algebra layer is ready). Staging is engineering
   order — **not** “languages wait for developers to invent them.”
3. **ENR2** is a larger designed layer (candidates / rank / multi-source /
   explain). Same rule: open when the *model* requires it for product
   completeness, not because a user asked for SQL-shaped enrichment.
4. **Dialects** absorb familiarity. Pure notation stays mathematical even
   when ugly.

When coding any remaining ENR1 piece: keep one compile path in `residuum-sda`,
preserve match-bag reduction law, do not invent a second parser.

## Reading order

1. [ENR1.md](ENR1.md) — match-bag kernel (normative intent).
2. Code: `residuum-sda` stdlib ENR ops + `tests/enr1_kernel.rs`.
3. [ENR2.md](ENR2.md) — **read for orientation only until needed**.
4. [SDA_SPEC.md](../../SDA_SPEC.md) + [DOCTRINE.md](../../doc/SDA/DOCTRINE.md) — value laws ENR must not weaken.

## Crate intent

This directory holds **specs**. Runtime enrichment is in **`residuum-sda`** so SDA
and ENR1 compile once. A future split into a thin `enr-lib` wrapper is optional
and must not fork the parser.
