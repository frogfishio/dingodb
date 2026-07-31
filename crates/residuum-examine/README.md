# residuum-examine

**SDA examination host** for ResiduumDB: map recovered frames, items, and holes to
normative ExaminationUnit values, stream them deterministically from store
salvage, and evaluate pure SDA programs over them.

Storage damage remains examination **data** (status tags, holes, uncertainty).
SDA language errors remain `Fail`. Pure SDA evaluation lives in
[`residuum-sda`](https://crates.io/crates/residuum-sda); this crate is the host that
projects store evidence into SDA values.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Pure SDA over arbitrary JSON | [`residuum-sda`](https://crates.io/crates/residuum-sda) |
| Examine a ResiduumDB store / salvage stream | **`residuum-examine`** (this crate) |
| CLI health report | `residuum doctor` ([`residuum-cli`](https://crates.io/crates/residuum-cli)) |

## Install

```toml
[dependencies]
residuum-examine = "0.1"
residuum-store = "0.1"
```

Or: `cargo add residuum-examine`

## Status

**Shipped** (Stage 5). ExaminationUnit projection, store-wide salvage stream,
SDA filter/map over units, and bounded pages with explicit incomplete results.

Rule:

> If ResiduumDB can recover it, SDA can examine it.

SDA remains pure. Storage access, decoding, and resource control happen in this
host before evaluation. The `residuum doctor` CLI uses this crate for recovery
unit summaries.

## Quick example

```rust
use residuum_examine::{examine_store, filter_units, ExamineLimits};
use residuum_store::{DurabilityMode, Store};

# let dir = tempfile::tempdir().unwrap();
let mut store = Store::create(dir.path())?;
store.put("k", b"alive", DurabilityMode::Durable)?;

let page = examine_store(&store, ExamineLimits::default())?;
assert!(page.units.iter().any(|u| u.status == "verified-complete"));

// Keep only verified complete units via SDA.
let islands = filter_units(
    &page.units,
    r#"input<status> = "verified-complete""#,
)?;
assert!(!islands.is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## API surface

| API | Role |
|-----|------|
| `examine_store` / `examine_bytes` / `examine_sources` | Project salvage scan → ordered units |
| `ExaminationUnit` | Profile product shape (status, integrity, payload, provenance) |
| `filter_units` / `map_units` / `eval_unit` / `eval_page` | Run SDA programs over units |
| `filter_holes` / `filter_status` / `filter_verified_complete` | Common filters |
| `ExamineLimits` / `ExaminePage` | Bounds + incomplete / resource-limited pages |
| `project_bytes` / `project_region` | Lower-level projection helpers |

Unit status tags include `verified-complete`, holes, and uncertainty markers —
they are **not** collapsed into a single error type.

## Ordering

Units follow a deterministic order: `segment_id` → physical `source` →
`offset` → `event_id` → canonical unit encoding. Filesystem enumeration order
is never exposed as SDA sequence order.

## Limits

When `max_units` or `max_bytes_read` is exceeded, the page reports
`complete = false` and uncertainty tag `resource-limited`. Limits never become
a silent empty success.

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`residuum-sda`](https://crates.io/crates/residuum-sda) | MIT | Pure SDA language runtime |
| [`residuum-store`](https://crates.io/crates/residuum-store) | MPL-2.0 | Store salvage sources |
| [`residuum-format`](https://crates.io/crates/residuum-format) | MIT | Frame verification under salvage |
| [`residuum-cli`](https://crates.io/crates/residuum-cli) | AGPL-3.0-or-later | `residuum doctor` |

## Documentation

- Examination profile: [SDA_PROFILE.md](https://github.com/frogfishio/dingodb/blob/main/SDA_PROFILE.md)
- Architecture: [OVERVIEW.md](https://github.com/frogfishio/dingodb/blob/main/OVERVIEW.md) §11
- SDA language: [SDA_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/SDA_SPEC.md)

## License

MPL-2.0.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb). Multi-tier license map:
[doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md).
