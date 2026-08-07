# RQL-Q1 fixture generators

Deterministic, seed-parameterised fixture generators for the practical query corpus.

**Authority:** programme §4.1–4.2 · cases bind `fixture.generator_id` + `fixture.seed`.

## Contract

1. Same `(generator_id, seed, params)` ⇒ identical collection contents (byte-stable JSON).
2. Generators produce **logical fixtures** only — no Residiuum store I/O.
3. Expected results / oracle rules must be evaluable against the materialised fixture without the product optimiser.
4. Algorithm: splitmix64-style PRNG from `seed` (see each generator); no `random` module nondeterminism.

## Catalog (Q1.2)

| generator_id | Domain | Collections | Spec |
|---|---|---|---|
| `commerce.orders_v1` | commerce | `orders` | [commerce_v1.md](./commerce_v1.md) |
| `commerce.products_v1` | commerce | `products` | [commerce_v1.md](./commerce_v1.md) |
| `commerce.customers_v1` | commerce | `customers` | [commerce_v1.md](./commerce_v1.md) |
| `commerce.line_items_v1` | commerce | `line_items` (+ orders/products refs) | [commerce_v1.md](./commerce_v1.md) |
| `commerce.inventory_v1` | commerce | `inventory` | [commerce_v1.md](./commerce_v1.md) |
| `messaging.conversations_v1` | messaging | `conversations` | [messaging_v1.md](./messaging_v1.md) |
| `messaging.messages_v1` | messaging | `messages` | [messaging_v1.md](./messaging_v1.md) |
| `messaging.participants_v1` | messaging | `participants` | [messaging_v1.md](./messaging_v1.md) |

## Materialiser

```sh
python3 tools/rql_q1/materialise_fixture.py --generator commerce.orders_v1 --seed 1 --params '{"n_orders":32}'
```

Exit 0 prints JSON `{ "collections": { ... } }`. Used by later Q3/Q4 harnesses; Q1 cases only **name** the generator.

## Dogfood honesty

No in-tree Residiuum dogfood datasets for commerce/messaging were found (2026-08-07).
All Q1.2 cases are tagged `dogfood.origin = invented_honest_label` with shapes aligned to
programme domain table + existing app-core `orders` dialect. Real dogfood can replace
generators under a versioned amendment without redefining frozen `case_id`s (archive + replace).
