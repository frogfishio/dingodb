# DingoDB

DingoDB is a deterministic relational document engine that lets developers build nested application artefacts directly from relational data, using explicit enrichment semantics instead of hidden joins and ORM hydration.

---

## Production maturity

Not production-ready yet (features are still iterating; see DEF notes in startup output).

Definitive implementation sequence and current starting task:
[MASTER_DELIVERY_PLAN.md](MASTER_DELIVERY_PLAN.md).

---

## Console / CLI

DingoDB includes a small CLI binary named `dingo`.

### Run an interactive console

```bash
dingo console ./path/to/store
```

The console reads DQL commands from **stdin** (one command per line is typical) and executes them against the provided store directory.

#### Example (piped)

```bash
printf '%s\n' \
  'PUT ./tmp/store users/user-1 {"name":"hello"}' \
  'GET ./tmp/store users/user-1' \
| dingo console ./tmp/store
```

### Non-interactive usage

For scripted usage, pipe commands into the console and terminate with `QUIT`.

---

## The problem

Modern applications usually choose between two compromises.

Relational databases provide:
- strong relationships;
- mature indexing;
- transactional semantics.

But application developers often rebuild the final shape through:
```
database rows
    ↓
SQL joins
    ↓
ORM hydration
    ↓
application objects
    ↓
API JSON
```

Document databases provide convenient shapes, but relationships often move into application code.

DingoDB treats relationship formation and document construction as first-class database operations.

---

## See the model

Example:
```text
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

This is not a hidden join.
