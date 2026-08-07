# Directory fixture generators (v1)

## Shared PRNG

Same splitmix64 as [commerce_v1.md](./commerce_v1.md); each generator restarts from its own `seed`.

## `directory.entries_v1`

**Params (defaults):** `n_entries` (40), `n_categories` (8), `n_locations` (12),
`kinds` `["person","org"]`.

| Field | Rule |
|---|---|
| `_key` | `e-{i:04d}` |
| `name` | `"Entry {i}"` |
| `kind` | kinds[i % 2] |
| `category_id` | `cat-{(i % n_categories):04d}` |
| `location_id` | `loc-{(i % n_locations):04d}`; **every 5th** omits field |
| `active` | false every 6th; else true |
| `discovered_at` | `2024-03-01T00:00:00Z` + i×30 minutes (logical clock) |
| `email` | **every 11th** omit; **every 4th** null; else `entry{i}@dir.example.test` |
| `tags` | `[]` every 7th; `["vip","vip"]` every 9th; else `["dir", kind]` |
| `attrs` | `{region, score}` most; **every 8th** missing `attrs` |

## `directory.categories_v1`

**Params:** `n_categories` (8), `parent_slugs` `["people","orgs","places"]`.

| Field | Rule |
|---|---|
| `_key` | `cat-{i:04d}` |
| `slug` | `cat-slug-{i}` |
| `title` | `"Category {i}"` |
| `parent` | parents[i % 3] |
| `depth` | i % 3 |

## `directory.locations_v1`

**Params:** `n_locations` (12), `regions` `["us","eu","apac"]`.

| Field | Rule |
|---|---|
| `_key` | `loc-{i:04d}` |
| `city` | `City{i}` |
| `region` | regions[i % 3] |
| `country` | region uppercased |
| `lat` / `lon` | deterministic offsets from i |
