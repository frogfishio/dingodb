# Dingo Query Language (DQL)

Status: **Design + implementation v0.1** (official dialect `dql` lowers to ENR1+SDA)  
Audience: language designers, SDK authors, optimizer authors, advanced users  
Normative companions: [SDA_SPEC.md](SDA_SPEC.md), [crates/enr-core/](crates/enr-core/),
[doc/SDA/DIALECTS.md](doc/SDA/DIALECTS.md), [DX_SPEC.md](DX_SPEC.md) §7,
and [DATA_RULES_PROPOSAL.md](DATA_RULES_PROPOSAL.md)

## 1. Why this exists

**ENR + SDA are exact mathematical representations of the product query model.**
Nobody disputes that the pure surface is hard to look at and hard to write for
everyday application work. That is intentional: the algebra fixes value kinds,
carriers, Null vs absence, match bags, cardinality, and failure tags.

Some people will still prefer the pure notation: data scientists,
mathematicians, set theorists, academics. That group is small and very loud.
They get the pure path forever — `Collection::sda`, `Dingo::enr_query`, dialect
id `sda`.

**DQL (Dingo Query Language) is the official human surface on top of ENR+SDA.**
It is designed to be nice to read and write. It *looks* a little like SQL in
places, but it is **not** SQL: cardinality is explicit, joins are enrichment
with named outputs, and projection is nested SDA-shaped shape, not a flat
SELECT list pretending multiplicity does not exist.

DQL is the product’s preferred text dialect. Foreign comfort dialects
(SQL-ish, Mongo-ish, GraphQL-ish, …) and fluent host APIs remain welcome —
every one of them **lowers to the same ENR+SDA IR / bytecode** that the pure
compiler produces. They never redefine the algebra.

```text
  Pure ENR + SDA text ──┐
  DQL (official) ───────┤
  SQL-ish ──────────────┤
  Mongo / JSON filter ──┼── compile ──► [ENR + SDA IR / bytecode] ──► execute
  GraphQL-ish ──────────┤
  Fluent / builder API ─┤
  (your plugin) ────────┘
```

Compile is the expensive, semantic step. Execution is shared.

| Layer | Role | Audience |
|-------|------|----------|
| **ENR + SDA** | Mathematical kernel; truth model; IR | Spec authors, pure-path users, optimizers |
| **DQL** | Official human dialect → same IR | Application developers, product docs, demos |
| **Foreign dialects** | Comfort / familiarity; admit holes | Migration, habits, partial coverage |
| **Fluent API** | Host-native builder; same IR | SDK callers who prefer code over strings |

Rule: **comfort never redefines truth.** If a dialect cannot express a
distinction (Null vs absence, exact cardinality failures, match bags), use
pure ENR+SDA — or extend DQL only when the lowering stays faithful.

## 2. Product position

| Id | Surface | Status |
|----|---------|--------|
| `sda` | Pure SDA / ENR1 source | **implemented** (identity / parse-checked) |
| **`dql`** | **Official Dingo Query Language** | **implemented** v0.1 (`from` / `enrich` / `expect` / `project` → ENR1) |
| `json` / `mongo` | Portable filter objects | implemented (predicate subset) |
| `sql` | Tiny SELECT/WHERE mimicry | partial comfort only |
| `graphql` | Reserved | scaffold |

DQL is **not** another foreign mimicry dialect. It is co-designed with ENR so
that:

1. everyday multi-collection enrichment is readable;
2. every construct has a total lowering into ENR Core IR;
3. the optimizer sees relationships and cardinality, not opaque strings.

See [DIALECTS.md](doc/SDA/DIALECTS.md) for the general dialect contract.
Foreign dialects remain imperfect frontends. DQL aims for **faithful**
coverage of the ENR1 enrichment + SDA projection story (not of full SQL).

## 3. Motivating example

Human surface (DQL):

```text
from orders

enrich customer using customers
  matching customer_id = id
  expect exactly_one

enrich items using items
  matching id = order_id
  expect many

enrich product using products
  matching product_id = id
  expect exactly_one

project {
  order_id,
  customer.name,
  items {
    quantity,
    product.name
  }
}
```

Same program as pure ENR + SDA (illustrative; exact sugar evolves with
`dingo-sda`):

```text
orders
|> enrich {
     customer:
       one!(Match(l, customers,
         getPath(l, Seq["customer_id"]),
         getPath(r, Seq["id"])))
   }
|> enrich {
     items:
       Match(l, items,
         getPath(l, Seq["id"]),
         getPath(r, Seq["order_id"]))
   }
|> /* nest product attach under each item — ForEach(items, …) */
|> project …
```

Same semantics via a fluent host API (optional parallel surface; not DQL text):

```ts
const result = await dingo
  .from(orders)
  .enrich("customer")
    .with(customers)
    .match("customer_id", "id")
    .expect("exactly_one")
  .enrich("items")
    .with(items)
    .match("id", "order_id")
    .expect("many")
  .enrich("product")
    .with(products)
    .match("items.product_id", "id")
    .expect("exactly_one")
  .project({
    order_id: true,
    customer: { name: true },
    items: {
      quantity: true,
      product: { name: true },
    },
  });
```

All three paths MUST lower to the same IR primitives (§5–§7).

## 4. Program shape

```text
Program
 ├── FromStep?          // root artefact / collection binding
 ├── EnrichStep*
 └── ProjectStep?
```

Normative sketch:

- A program starts from a bound collection or free name (`from orders`, or a
  host-supplied root).
- Zero or more `enrich` steps attach related artefacts under **output field
  names**.
- An optional `project` shapes the final artefact (SDA-style nested shape).

### 4.1 Namespaces

In:

```text
enrich customer using customers
  matching customer_id = id
  expect exactly_one
```

| Name | Namespace | Meaning |
|------|-----------|---------|
| `customer` | **output field** | Field attached on the current artefact |
| `customers` | **source artefact** | Right-hand dataset / collection binding |
| `customer_id` | left key path | On the current (left) row |
| `id` | right key path | On each right-hand row |

`customer` and `customers` are different namespaces. Confusing them is a
static error when both are in scope without qualification.

## 5. Enrich AST → Match + Cardinality + Attach

### 5.1 Surface

```text
enrich <output_name> using <source>
  matching <left_key> = <right_key>
  expect <cardinality>
```

Cardinality keywords (v0.1):

| Surface | IR constructor | Semantics |
|---------|----------------|-----------|
| `exactly_one` | `One(Match(…))` | 0 → `Fail(t_enr_missing)`; 1 → value; >1 → `Fail(t_enr_duplicate)` |
| `optional` | `Optional(Match(…))` | 0 → `None`; 1 → `Some(value)`; >1 → `Fail(t_enr_duplicate)` |
| `many` | `Many(Match(…))` | Always `Bag` (duplicates preserved) |

`expect` is not documentation-only. It is a **lowering instruction**: it
chooses the cardinality operator over the match bag.

### 5.2 Match algebra IR

Every enrich lowers through the same primitive:

```text
Match(left, right, left_key, right_key)
```

Result of bare match is always a **bag** of right rows (never a silent single
value). Cardinality operators interpret the bag.

Example:

```text
enrich items using items
  matching id = order_id
  expect many
```

becomes:

```text
Many(
  Match(current_order, items, order.id, item.order_id)
)
```

### 5.3 Attach IR

The enrich step itself becomes:

```text
Attach {
  field: customer,
  value: One(Match(...))
}
```

Before:

```text
Order
```

After:

```text
Order {
  customer: Customer   // or Bag / Optional per expect
}
```

Nested enrichment under a bag field (e.g. product under items) lowers with an
explicit `ForEach` over that field so left keys evaluate in the child row’s
scope.

## 6. Project AST

Surface:

```text
project {
  order_id,
  customer.name,
  items {
    quantity,
    product.name
  }
}
```

Lowers to nested projection IR (SDA-shaped):

```text
Project {
  fields: [
    order_id,
    Path(customer, name),
    Nested(items, [
      quantity,
      Path(product, name)
    ])
  ]
}
```

Project does not invent new relation semantics; it reduces the enriched
artefact. That reduction is **SDA’s job** after ENR expansion.

## 7. Complete lowering example

Program:

```text
from orders

enrich customer using customers
  matching customer_id = id
  expect exactly_one

enrich items using items
  matching id = order_id
  expect many

enrich product using products
  matching product_id = id
  expect exactly_one

project {
  order_id,
  customer.name,
  items {
    quantity,
    product.name
  }
}
```

Target IR sketch:

```text
Pipeline {
  root: Orders,

  Attach(
    customer,
    One(Match(Order, Customers, Order.customer_id, Customer.id))
  ),

  Attach(
    items,
    Many(Match(Order, Items, Order.id, Item.order_id))
  ),

  ForEach(items,
    Attach(
      product,
      One(Match(Item, Products, Item.product_id, Product.id))
    )
  ),

  Project(...)
}
```

## 8. Why the IR matters (optimizer)

Once lowered, the engine does not see “run a query string.” It sees:

```text
Relationship:  Order.customer_id → Customer.id
Cardinality:   exactly one
Physical options:
  - hash lookup
  - Hydra index
  - remote index
  - cached pointer
  - batch probe
```

For `expect many`:

```text
Need:     Bag<T>
Allowed:  multi-index, range lookup, batch gather
```

The surface stays readable. The IR stays powerful. Storage and distribution
get explicit cardinality and key relationships.

## 9. ENR Core IR freeze (target)

DQL (and pure ENR text, and fluent builders) SHOULD lower into these
primitives — and no more — for the enrichment kernel:

```text
Match
Cardinality   // One / Optional / Many (and ordered policies later)
Attach
Merge
Project
ForEach       // nested attach under bag fields
```

Everything else is sugar or host binding. That keeps ENR from becoming another
giant query language. Detailed pure algebra: [ENR_CORE.md](crates/enr-core/ENR_CORE.md),
[ENR1.md](crates/enr-core/ENR1.md).

## 10. DQL is not SQL

They can look similar at a glance. They are not the same language.

| Concern | Typical SQL | DQL |
|---------|-------------|-----|
| Join shape | `JOIN … ON` flattens or multiplies rows | `enrich` **attaches** a named field |
| Multiplicity | Often implicit / DISTINCT later | `expect exactly_one \| optional \| many` is required |
| Nested result | Awkward / JSON functions / ORM | Nested `project { items { … } }` is first-class |
| Null model | Three-valued SQL NULL | SDA Null **vs** absence (carriers) |
| Failure | Runtime errors / wrong row counts | Stable ENR tags (`t_enr_missing`, `t_enr_duplicate`, …) |
| Truth model | SQL semantics | ENR + SDA algebra |

SQL mimicry (`dialect "sql"`) remains a **foreign comfort** dialect with known
holes. It is not DQL and must not be documented as the official language.

## 11. Compilation contract

A conforming DQL implementation MUST:

1. Parse DQL source into the Program AST (§4).
2. Lower totally into ENR Core IR + SDA project forms (§5–§7).
3. Emit the **same IR class** that pure ENR+SDA compilation produces (shared
   bytecode / interpreter / plan entry).
4. Refuse unmappable constructs with a clear static error (no silent weakening
   of cardinality or Null/absence).
5. Preserve ENR failure tags when cardinality operators fail.

Host concerns (collection binding, indexes, budgets, coverage, pages) stay
**outside** DQL meaning — same split as pure SDA hosts ([SDA_PROFILE.md](SDA_PROFILE.md)).

## 12. Implementation status

| Piece | Status |
|-------|--------|
| Pure SDA + ENR1 in `dingo-sda` | **shipped** |
| Host text path (`sda` / `enr_query`) | **shipped** |
| Foreign dialects `json` / `mongo` / `sql` | **shipped** (partial for sql) |
| DQL grammar + compiler | **shipped** v0.1 — `dingo_sdk::dialects` id `dql` → pure ENR1/SDA text |
| Conformance: DQL ≡ pure ENR | **shipped** — `dialects_query::dql_equals_pure_enr1_on_enrich` + unit tests |
| Fluent enrich builder → same IR | **design** (API sketch only) |
| Nested bag-scoped enrich (ForEach) | **partial** — top-level enrich chain in v0.1; nested scope next |
| Shared bytecode / plan IR freeze | **design** — align with ENR Core IR v0.1 |

Code entry points:

- Compiler: `crates/dingo-sdk/src/dialects/dql.rs` (`compile_dialect("dql", …)`)
- Proof tests: `crates/dingo-sdk/tests/dialects_query.rs`, `dialects::dql` unit tests

Next concrete work:

1. Nested enrich under `expect many` fields (ForEach attach).
2. Richer `project` sugar and path ergonomics.
3. Freeze ENR Core IR v0.1 primitives for optimizer/plan compile.

## 13. See also

- [doc/DQL/USER_GUIDE.md](doc/DQL/USER_GUIDE.md) — **how to write and run DQL** (application developers)  
- [DATA_RULES_PROPOSAL.md](DATA_RULES_PROPOSAL.md) — the visually compatible declaration language for enforceable invariants
- [doc/SDA/DIALECTS.md](doc/SDA/DIALECTS.md) — dialect stack and foreign comfort rules  
- [doc/SDA/DOCTRINE.md](doc/SDA/DOCTRINE.md) — certainty kernel vs comfort  
- [DX_SPEC.md](DX_SPEC.md) §7 — everyday query experience  
- [crates/enr-core/README.md](crates/enr-core/README.md) — ENR1 surface and design authority  
- [SDA_SPEC.md](SDA_SPEC.md) — pure algebra  
