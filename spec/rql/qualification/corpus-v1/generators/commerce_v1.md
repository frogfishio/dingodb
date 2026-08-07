# Commerce fixture generators (v1)

## Shared PRNG

```text
u64 state = seed (as u64)
next_u64():
  state += 0x9E3779B97F4A7C15
  z = state
  z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
  z = (z ^ (z >> 27)) * 0x94D049BB133111EB
  return z ^ (z >> 31)
next_u32() = next_u64() & 0xffffffff
pick(list) = list[next_u32() % len(list)]
```

Document keys: string `"o-{i:04d}"`, `"p-{i:04d}"`, `"c-{i:04d}"`, `"li-{i:04d}"`, `"inv-{i:04d}"`.

## `commerce.orders_v1`

**Params (defaults):** `n_orders` (32), `n_customers` (16), `regions` `["us","eu","apac"]`,
`statuses` `["paid","open","cancelled","shipped"]`.

For `i in 0..n_orders-1`:

| Field | Rule |
|---|---|
| `_key` | `o-{i:04d}` |
| `status` | statuses[i % 4] |
| `region` | regions[i % 3] |
| `amount` | 10 + (next_u32() % 990) as number; **every 11th** order sets amount to string `"NaN"` (wrong_type cell) |
| `created_at` | ISO-8601 `2024-01-01T00:00:00Z` + i minutes |
| `customer.id` | `c-{(i % n_customers):04d}`; **every 7th** omits `customer` entirely (missing nested) |
| `deleted_at` | **every 13th** sets null; **every 17th** omits field; else absent meaning live |
| `notes` | null every 5th; else `"note-{i}"` |
| `tags` | `[]` every 9th; `["a","a"]` every 10th; else `["sku", region]` |
| `name` | `"Order {i}"` |

## `commerce.products_v1`

**Params:** `n_products` (48), `categories` `["widget","gadget","supply"]`.

| Field | Rule |
|---|---|
| `_key` | `p-{i:04d}` |
| `sku` | `SKU-{i:04d}` |
| `category` | categories[i % 3] |
| `price_cents` | 100 + (next_u32() % 9900) |
| `active` | i % 5 != 0 |
| `attrs.color` | pick red/blue/green; every 8th missing `attrs` |

## `commerce.customers_v1`

**Params:** `n_customers` (16).

| Field | Rule |
|---|---|
| `_key` | `c-{i:04d}` |
| `email` | `user{i}@example.test` |
| `tier` | bronze/silver/gold by i%3 |
| `region` | us/eu/apac by i%3 |

## `commerce.line_items_v1`

**Params:** `n_orders` (32), `items_per_order` (1..3 by i%3+1).

| Field | Rule |
|---|---|
| `_key` | `li-{i:04d}` |
| `order_id` | parent order key |
| `product_id` | `p-{(i % n_products):04d}` |
| `qty` | 1 + i%5 |
| `unit_price_cents` | 100 + i*7 |

## `commerce.inventory_v1`

**Params:** `n_products` (48), warehouses `["w1","w2"]`.

| Field | Rule |
|---|---|
| `_key` | `inv-{product}-{wh}` |
| `product_id` | product key |
| `warehouse` | w1/w2 |
| `qty_on_hand` | next_u32() % 500 |
| `qty_reserved` | min(on_hand, next_u32()%50) |
