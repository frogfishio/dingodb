# dingo-store

Single-node authoritative store for DingoDB: filesystem-backed append-only
segments, put/get/delete by subject, durability modes, catalog-independent
recovery, derived state (catalogs, secondary indexes, chunks, history,
compaction, checkpoints), inspect/salvage-to-path helpers, and tiering
(hot/warm/cold/archive) with offline coverage honesty.

Normative sources: repository root [`OVERVIEW.md`](../../OVERVIEW.md) §§5–7, §9, §13;
[`FORMAT_SPEC.md`](../../FORMAT_SPEC.md); [`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md)
Stages 3, 6, 7, and 9; [`doc/RUNBOOK_RETENTION.md`](../../doc/RUNBOOK_RETENTION.md).

## Status

**Shipped** (Stages 3a–3c + 6 + 7 inspect/salvage + 9) —

| Area | What you get |
|------|----------------|
| Core | open/create, put/get/delete, durability modes (`memory`, `buffered`, `durable`) |
| Recovery | rebuildable primary index, salvage after catalog wipe, OVERVIEW §16 suite |
| Meta | framed store descriptor, optional on-disk primary index cache |
| Derived | collection catalog, secondary index files, subject history, checkpoints |
| Chunks | chunked payloads with partial maps; live-state compaction (sources retained) |
| Operator | `open_inspect` (read-only doctor), `salvage_to` |
| Tiering | segment tier move/copy with stable identities, hierarchical segment catalogs |
| Honesty | offline-tier coverage holes, multi-generation format classification |
| Media | `MediaLocator`, `object:local:`, live S3/GCS mirrors via `DINGO_S3_ROOT` / `DINGO_GS_ROOT` |
| Scaffold | `LifecyclePolicy`, `ErasureManifest` (codec not shipped) |

## Layout (OVERVIEW §6.1, §9)

```text
store/
  store-info/     # store_id + meta + descriptor.dingo
  active/         # open append segment (at most one)
  segments/       # sealed hot segments
  tiers/          # warm/cold/archive media + roots.txt
  chunks/         # reserved (payload chunks live in segments)
  catalogs/       # derived only (collections.cat, tier-placement.cat, segments.cat)
  indexes/        # derived only (primary.idx, sec/<coll>/*.six)
  snapshots/      # derived checkpoints
  recovery/       # operator scratch + migrations/ evidence
```

Deleting `catalogs/`, `indexes/`, and `snapshots/` must not prevent recovery:
the store rebuilds current state by scanning `active/`, `segments/`, and online
tier media. Tier **media** files under `tiers/*` are authoritative when segments
live there.

## Surface

| API | Role |
|-----|------|
| `Store::open` / `Store::create` | Create-or-open on a directory path |
| `Store::open_inspect` | Read-only open (no writer, no derived writes) |
| `put` / `get` / `delete` | Subject-keyed current-state operations |
| `get_payload` | Completeness-aware read (`PayloadResult`) |
| `history` | Per-subject event stream |
| `WriteReceipt` | Event identity + acknowledged durability mode |
| `DurabilityMode` | `Memory`, `Buffered`, `Durable` |
| `rebuild_index` / `salvage` | Catalog-free scan of all segment files |
| `salvage_to` | Non-destructive copy of live subjects to a new path |
| `rebuild_catalogs` / `list_collections` | Derived collection catalog |
| `compact_live` | Live projection into a new segment (sources retained) |
| `checkpoint` | Derived snapshot with declared coverage |
| secondary index helpers | Persist/load/delete `*.six` files |
| `examination_sources` | Ordered `(source_name, bytes)` for examination |
| `live_entries` / `live_logical_entries` | Index raw vs reassembled bodies |
| `transfer_segment_to_tier` | Copy/move sealed segment (stable id) |
| `set_tier_available` / `tier_coverage` | Offline tier → coverage hole |
| `get_with_tier_coverage` | Absence only proven when coverage complete |
| `list_segment_summaries` / `rebuild_segment_catalog` | Cold-search hierarchy |
| `classify_segment` | Multi-gen format class; preserve unsupported bytes |
| `open_media` / `CloudMirrorConfig` | Object/media backends incl. S3/GCS mirrors |
| `LifecyclePolicy` | Declarative tier aging rules (`tiers/lifecycle.json`) |
| `ErasureManifest` | Archive shard naming contract (codec not shipped) |

## Out of scope (this crate)

- Native SigV4 HTTP object SDK (use mirror / fuse mount)
- Erasure encode/decode codecs (manifest only)
- Background lifecycle scheduler (policy evaluate only)
- `replicated` durability (cluster path; see [`dingo-cluster`](../dingo-cluster))

## Quick example

```rust
use dingo_store::{DurabilityMode, Store};

# let dir = tempfile::tempdir().unwrap();
# let path = dir.path();
let mut store = Store::create(path)?;
store.put("user-42", b"{\"name\":\"Alice\"}", DurabilityMode::Durable)?;
assert_eq!(store.get("user-42")?.as_deref(), Some(b"{\"name\":\"Alice\"}".as_slice()));
store.delete("user-42", DurabilityMode::Durable)?;
assert!(store.get("user-42")?.is_none());
# Ok::<(), dingo_store::StoreError>(())
```
