# dingo-examine

Stage 5 SDA examination host for DingoDB: map recovered frames, items, and
holes to normative [`ExaminationUnit`](../../SDA_PROFILE.md) values, stream them
deterministically from store salvage, and evaluate pure SDA programs over them.

Normative sources: [`SDA_PROFILE.md`](../../SDA_PROFILE.md),
[`OVERVIEW.md`](../../OVERVIEW.md) §11, [`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md)
Stage 5.

## Status

**Stage 5** — ExaminationUnit projection, store-wide salvage stream, SDA filter
/ map over units, bounded pages with explicit incomplete results.

Rule:

> If DingoDB can recover it, SDA can examine it.

SDA remains pure. Storage access, decoding, and resource control happen in this
host before evaluation.

## Surface

| API | Role |
|-----|------|
| `examine_store` / `examine_bytes` | Project salvage scan → ordered units |
| `ExaminationUnit` / `to_sda_value` | Profile product shape as SDA `Prod` |
| `filter_units` / `map_units` | Run an SDA program per unit |
| `ExamineLimits` / `ExaminePage` | Bounds + incomplete / resource-limited pages |
| `unit_status` tags | `verified-complete`, holes, etc. (not collapsed to one error) |

## Quick example

```rust
use dingo_examine::{examine_store, filter_units, ExamineLimits};
use dingo_store::{DurabilityMode, Store};

# let dir = tempfile::tempdir().unwrap();
let mut store = Store::create(dir.path())?;
store.put("k", b"alive", DurabilityMode::Durable)?;

let page = examine_store(&store, ExamineLimits::default())?;
assert!(page.units.iter().any(|u| u.status == "verified-complete"));

// Keep only verified complete units via SDA (status field total selector).
let islands = filter_units(&page.units, r#"input<status> = "verified-complete""#)?;
assert!(!islands.is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Ordering

Units follow [SDA_PROFILE §12](../../SDA_PROFILE.md): `segment_id` → physical
`source` → `offset` → `event_id` → canonical unit encoding. Filesystem
enumeration order is never exposed as SDA sequence order.

## Limits

When `max_units` or `max_bytes_read` is exceeded, the page reports
`complete = false` and uncertainty tag `resource-limited`. Limits never become
a silent empty success.
