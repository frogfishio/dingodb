# RQL User Guide

**Residiuum Query Language (RQL)** is the official human way to write multi-collection
queries in Residiuum. You write readable `from` / `enrich` / `project` text; the
engine compiles it into the same ENR + SDA programs that pure algebra users write
by hand.

This guide is for application developers. It teaches what to type, how cardinality
works, and how to run a program from the Rust SDK. It is **not** the design
authority — that is [RQL_SPEC.md](../../RQL_SPEC.md).

| Document | Role |
|----------|------|
| **This guide** | How to express and run RQL |
| [RQL_SPEC.md](../../RQL_SPEC.md) | Normative design, lowering, IR |
| [DIALECTS.md](../SDA/DIALECTS.md) | How dialects sit on pure SDA |
| [SDA USER_MANUAL.md](../SDA/USER_MANUAL.md) | Pure SDA when you need full control |

Status: **v0.1** (dialect id `dql`). Nested bag-scoped enrich is still partial;
see [Limitations](#12-limitations-v01).

---

## 1. When to use RQL

Use RQL when you want to:

- start from one collection of documents
- **attach** related data from other collections under named fields
- state **how many** matches you expect (`exactly_one`, `optional`, or `many`)
- shape a nested result with `project`

Prefer something else when:

| Job | Prefer |
|-----|--------|
| Filter one collection (`status == "active"`) | `Filter` builder, or dialect `json` / `mongo` / `sql` via `find_dialect` |
| Exact Null vs missing-key distinctions | Pure SDA ([USER_MANUAL.md](../SDA/USER_MANUAL.md)) |
| Fluent host equijoin without text | `db.query()` multi-collection API |
| Full algebraic control | Pure ENR1 text via `db.enr_query()` |

RQL is **not SQL**. It looks a little like SQL in places, but joins do not flatten
rows — they **attach** named fields — and multiplicity is always explicit.

---

## 2. Ten-minute mental model

A RQL program has three kinds of steps, in order:

```text
from <root_collection>          -- required in v0.1

enrich <field> using <source>   -- zero or more
  matching <left_key> = <right_key>
  expect <cardinality>

project { … }                   -- optional; shapes the output
```

Think of each document in the root collection as an **artefact**. Each `enrich`
looks up related rows and **attaches** them under a field name. `project` picks
what you want to keep, including nested shapes under bag fields.

```text
  orders
    │
    ├─ enrich customer  →  Order { …, customer: Customer }
    ├─ enrich items     →  Order { …, customer, items: [ Item, … ] }
    └─ project { … }    →  smaller nested JSON
```

That is different from a SQL `JOIN`, which multiplies rows into a flat table.

---

## 3. A first complete example

### Data (three collections)

```json
// orders
[
  { "id": "o1", "customer_id": "c1", "qty": 2 },
  { "id": "o2", "customer_id": "c2", "qty": 1 }
]

// customers
[
  { "id": "c1", "name": "Ada" },
  { "id": "c2", "name": "Bob" }
]

// items
[
  { "order_id": "o1", "sku": "A", "quantity": 2 },
  { "order_id": "o1", "sku": "B", "quantity": 1 },
  { "order_id": "o2", "sku": "A", "quantity": 1 }
]
```

### RQL

```text
from orders

-- attach exactly one customer per order
enrich customer using customers
  matching customer_id = id
  expect exactly_one

-- attach every line item for this order
enrich items using items
  matching id = order_id
  expect many

project {
  id,
  customer.name,
  items {
    sku,
    quantity
  }
}
```

### What you get (conceptually)

```json
[
  {
    "id": "o1",
    "customer": { "name": "Ada" },
    "items": [
      { "sku": "A", "quantity": 2 },
      { "sku": "B", "quantity": 1 }
    ]
  },
  {
    "id": "o2",
    "customer": { "name": "Bob" },
    "items": [
      { "sku": "A", "quantity": 1 }
    ]
  }
]
```

Line comments start with `--` and run to end of line.

---

## 4. How to run RQL (Rust SDK)

RQL compiles to pure ENR1/SDA text. You then bind the named collections and
execute that program.

```rust
use residiuum_sdk::{compile_dialect, json, Residiuum};

fn main() -> Result<(), residiuum_sdk::Error> {
    let mut db = Residiuum::open("./app.dingo")?;

    // Load sample data (abbreviated).
    {
        let mut orders = db.collection("orders")?;
        orders.put("o1", &json!({"id": "o1", "customer_id": "c1", "qty": 2}))?;
        let mut customers = db.collection("customers")?;
        customers.put("c1", &json!({"id": "c1", "name": "Ada"}))?;
    }

    let dql = r#"
        from orders
        enrich customer using customers
          matching customer_id = id
          expect exactly_one
        project {
          id,
          customer.name
        }
    "#;

    // 1) Compile RQL → pure SDA program text
    let compiled = compile_dialect("dql", dql)?;
    // compiled.dialect == "dql"
    // compiled.sda     == ENR1/SDA source the engine actually runs

    // 2) Bind every free name the program uses, then run
    let result = db
        .enr_query()
        .bind("orders")
        .bind("customers")
        .run(&compiled.sda)?;

    println!("{result}");
    Ok(())
}
```

### Rules of thumb

1. **`from` and every `using` name must be bound** (same string as the collection
   name, or use `.bind_as("real_name", "alias")` and put the alias in RQL).
2. Binding materialises live JSON documents for each source; optional
   `.filter(...)` / `.source_limit(n)` apply **per source before** the program runs.
3. `compile_dialect("dql", …)` and `BuiltinDialect::Dql.compile(…)` are equivalent.
   Alias id `dingo-ql` also selects RQL.

You can inspect the lowered program anytime:

```rust
let compiled = compile_dialect("dql", dql)?;
eprintln!("{}", compiled.sda);
```

Example lowering for a single enrich:

```text
orders
|> enrich {
    customer: one!(Match(
      l,
      customers,
      getPath(l, Seq["customer_id"]),
      getPath(r, Seq["id"])
    ))
  }
```

Conformance tests require that this result matches hand-written ENR1 on the same
bindings — RQL does not invent a second evaluator.

---

## 5. Program shape (reference)

```text
Program
 ├── from <ident>                 required in v0.1
 ├── enrich …                     zero or more
 └── project { … }               at most one
```

Keywords are **case-insensitive** (`FROM`, `Enrich`, `project` all work).
Identifiers (collection names, field names) are case-sensitive as written.

### Syntax sketch

```text
from <root>

enrich <output_field> using <source_collection>
  matching <left_path> = <right_path>
  expect exactly_one | optional | many

project {
  field,
  path.to.field,
  bag_field {
    nested_field,
    other.path
  },
  …
}
```

Paths use dots: `customer_id`, `meta.region`, `a.b.c`.

---

## 6. `from` — the root collection

```text
from orders
```

- Names the root document sequence.
- Every enrich is relative to the **current** artefact flowing through the
  pipeline (starting as each order).
- In v0.1, `from` is **required** for lowering. A program that is only `enrich`
  clauses without `from` is rejected.

The free name `orders` must match a binding when you run the program.

---

## 7. `enrich` — attach related data

```text
enrich <output> using <source>
  matching <left_key> = <right_key>
  expect <cardinality>
```

| Part | Namespace | Meaning |
|------|-----------|---------|
| `output` | **field on the left row** | Where the match result is stored |
| `source` | **right-hand collection** | Dataset to search |
| `left_key` | path on the **current** row | e.g. `customer_id` on an order |
| `right_key` | path on each **right** row | e.g. `id` on a customer |
| `expect` | cardinality | How many right rows are allowed |

### Namespaces (do not mix them up)

```text
enrich customer using customers
  matching customer_id = id
  expect exactly_one
```

- `customer` → new field on the order  
- `customers` → collection binding  
- `customer_id` → field on the order (left)  
- `id` → field on each customer (right)

`customer` and `customers` are different names on purpose.

### Matching direction

```text
matching <left> = <right>
```

- **Left** is always evaluated on the current artefact.
- **Right** is always evaluated on each candidate from `using`.

Examples:

```text
-- order.customer_id  ==  customer.id
matching customer_id = id

-- order.id  ==  item.order_id
matching id = order_id

-- nested left path
matching meta.region_id = id
```

### What enrich does *not* do

- It does not flatten into a Cartesian product of SQL-style join rows.
- It does not silently pick the “first” match when you asked for many, or invent
  a row when you asked for exactly one.
- It does not open collections by itself — the **host** binds data under free
  names before execution.

---

## 8. `expect` — cardinality (required)

Every enrich must say how many right-hand matches are acceptable. This is not
documentation; it chooses the ENR cardinality operator.

| Keyword | Aliases | Match bag size | Result |
|---------|---------|----------------|--------|
| `exactly_one` | `exactlyone`, `one` | 0 | **Fail** (`t_enr_missing`) |
| | | 1 | The single value |
| | | >1 | **Fail** (`t_enr_duplicate`) |
| `optional` | `opt` | 0 | Optional empty (`None`) |
| | | 1 | Optional some(value) |
| | | >1 | **Fail** (`t_enr_duplicate`) |
| `many` | — | any | Always a **bag** (list); duplicates kept |

### Choosing cardinality

| Situation | Use |
|-----------|-----|
| Foreign key that must resolve | `exactly_one` |
| Foreign key that may be missing | `optional` |
| Children / line items / tags | `many` |

```text
-- required parent
enrich customer using customers
  matching customer_id = id
  expect exactly_one

-- optional shipping address
enrich ship_to using addresses
  matching shipping_address_id = id
  expect optional

-- zero or more line items
enrich lines using order_lines
  matching id = order_id
  expect many
```

### Failure style

RQL inherits ENR failure tags. When `exactly_one` finds zero or two+ matches,
the program fails with a stable tag rather than inventing a row or silently
picking one. That is intentional: **honest cardinality over convenient guesses**.

---

## 9. `project` — shape the result

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

Rules:

- At most **one** `project` per program.
- A bare name keeps that field: `order_id`.
- Dotted paths nest into maps: `customer.name` → `{ "customer": { "name": … } }`.
- A block `name { … }` maps over a bag/sequence field and projects each element.

Project runs **after** enrich. It does not change relationship semantics; it only
reduces the enriched artefact (SDA-style nested shape).

### Project-only vs enrich-only

```text
-- enrich only: full enriched documents
from orders
enrich customer using customers
  matching customer_id = id
  expect exactly_one

-- enrich + project: slim nested view
from orders
enrich customer using customers
  matching customer_id = id
  expect exactly_one
project {
  id,
  customer.name
}
```

---

## 10. Recipes

### A. Single enrich (lookup)

```text
from invoices
enrich account using accounts
  matching account_id = id
  expect exactly_one
```

### B. Optional parent

```text
from events
enrich actor using users
  matching actor_id = id
  expect optional
```

If there is no user for `actor_id`, the attach is empty rather than failing the
whole row’s program the way `exactly_one` would.

### C. One-to-many children

```text
from posts
enrich comments using comments
  matching id = post_id
  expect many
project {
  id,
  title,
  comments {
    body,
    author_id
  }
}
```

### D. Chain of top-level enriches

```text
from orders

enrich customer using customers
  matching customer_id = id
  expect exactly_one

enrich warehouse using warehouses
  matching warehouse_id = id
  expect exactly_one

project {
  id,
  customer.name,
  warehouse.code
}
```

Each enrich attaches to the **order** (root), not under another enriched field.
See [Limitations](#12-limitations-v01) for nested-under-bag enrich.

### E. Comments and layout

```text
from orders

-- customers must exist for every order we ship
enrich customer using customers
  matching customer_id = id
  expect exactly_one

project {
  id,
  customer.name,   -- trailing commas allowed
}
```

Whitespace and newlines are free. Prefer one enrich block per attachment so
cardinality stays visible.

---

## 11. RQL vs SQL (quick contrast)

| Concern | SQL habit | RQL |
|---------|-----------|-----|
| Join | `JOIN t ON …` flattens / multiplies rows | `enrich` **attaches** a named field |
| Multiplicity | Often implicit | `expect exactly_one \| optional \| many` **required** |
| Nested JSON | Awkward / vendor functions | Nested `project { items { … } }` |
| Missing vs null | SQL `NULL` muddle | ENR/SDA carriers (use pure SDA for exact Null≠absence) |
| Wrong match count | Wrong row counts / runtime surprise | Stable ENR fail tags |

The foreign dialect id `sql` is a **comfort** frontend for simple `WHERE` filters
on one collection. It is **not** RQL and must not be used as the product’s
official multi-collection language.

---

## 12. Limitations (v0.1)

Honest list of what v0.1 does and does not cover:

| Feature | Status |
|---------|--------|
| `from` + `enrich` + `expect` + `project` | **Supported** |
| Cardinality `exactly_one` / `optional` / `many` | **Supported** |
| Dotted key paths and nested `project` | **Supported** |
| Line comments `--` | **Supported** |
| Nested enrich **under** an `expect many` bag (ForEach scope) | **Partial / next** — prefer top-level enriches or pure ENR for now |
| Fluent enrich builder → same IR | Design only |
| Filters / `where` inside RQL text | **Not in RQL** — filter on the host binding (`.filter` / `.where_eq`) |
| Aggregates, `ORDER BY`, pagination | **Host / other APIs** — not RQL meaning |
| DDL / writes | Out of scope (use collection put/delete) |

When RQL cannot say what you need without weakening meaning, write pure ENR1/SDA
and run it with `enr_query` — same engine, full kernel.

---

## 13. Errors you will see

Compilation fails closed with messages prefixed `dialect 'dql': …`.

| Situation | Typical message theme |
|-----------|------------------------|
| Empty source | empty program |
| No `from` and no enrich | needs `from` or enrich |
| Missing `from` at lower time | `` `from <collection>` is required for v0.1 `` |
| Bad cardinality keyword | use `exactly_one`, `optional`, or `many` |
| Two `project` clauses | only one project allowed |
| Unexpected character / token | lexer/parser error with token dump |
| Project path conflict | same key used as leaf and group |

Runtime (after compile) uses ENR/SDA evaluation errors — e.g. missing match for
`exactly_one`. Inspect `compiled.sda` when debugging.

---

## 14. Cheat sheet

```text
from <root>

enrich <field> using <collection>
  matching <left_path> = <right_path>
  expect exactly_one | optional | many

project {
  field,
  a.b,
  bag { nested, path.to }
}

-- comment
```

```rust
let c = compile_dialect("dql", dql)?;
let v = db.enr_query()
    .bind("orders")
    .bind("customers")
    .run(&c.sda)?;
```

| expect | ENR lower |
|--------|-----------|
| `exactly_one` | `one!(Match(…))` |
| `optional` | `one?(Match(…))` |
| `many` | bare `Match(…)` bag |

---

## 15. See also

- [RQL_SPEC.md](../../RQL_SPEC.md) — full design and lowering contract  
- [doc/SDA/DIALECTS.md](../SDA/DIALECTS.md) — dialect stack and foreign comfort rules  
- [doc/SDA/USER_MANUAL.md](../SDA/USER_MANUAL.md) — pure SDA everyday use  
- [DX_SPEC.md](../../DX_SPEC.md) §7 — everyday query experience  
- [crates/enr-core/README.md](../../crates/enr-core/README.md) — ENR1 kernel surface  
- Implementation: `crates/residiuum-sdk/src/dialects/dql.rs`  
- Proof tests: `crates/residiuum-sdk/tests/dialects_query.rs`
