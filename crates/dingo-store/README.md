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
| Ownership | exclusive writer lock (`store-info/writer.lock`, DEF-020); inspect is lock-free |
| Control docs | atomic durable replace via `atomic_file` (DEF-021); `*.prev` for non-trivial meta |
| Recovery | rebuildable primary index, salvage after catalog wipe, OVERVIEW §16 suite |
| Meta | framed store descriptor, optional on-disk primary index cache (DEF-023 frontier v2) |
| Derived | collection catalog, secondary index files, subject history, checkpoints |
| Chunks | chunked payloads with partial maps; live-state compaction (phased DEF-024 jobs; sources retained by default) |
| Operator | `open_inspect` (read-only doctor), evidence `salvage_to` + `export_live_state` |
| Tiering | segment tier move/copy with stable identities, hierarchical segment catalogs |
| Honesty | offline-tier coverage holes; fail-closed logical scans (DEF-012); durable-frontier catalogs (DEF-013); write dedup table (DEF-010); multi-gen format |
| Crash matrix | DEF-022 hardened: matrix + failpoints (`Abort`, ENOSPC, short-write) + multi-process child; CI subset always, full matrix nightly (`DINGO_CRASH_MATRIX_FULL=1`) |
| Write path | DEF-023: durable projection + rate-limited frontier checkpoints; no full-store rescan on ack |
| Compaction | DEF-024: durable phases + job records; reclaim only with `allow_history_loss` |
| Identifiers | DEF-025: `dingo-id-v1` — OS CSPRNG (`getrandom`) for random ids; sortable segment seq; fail closed |
| Media | `MediaLocator`, `object:local:`, live S3/GCS mirrors via `DINGO_S3_ROOT` / `DINGO_GS_ROOT` |
| Scaffold | `LifecyclePolicy`, `ErasureManifest` (codec not shipped) |

## Layout (OVERVIEW §6.1, §9)

```text
store/
  store-info/     # store_id + meta + descriptor.dingo + writer.lock + write_dedup.v1
  active/         # open append segment (at most one)
  segments/       # sealed hot segments
  tiers/          # warm/cold/archive media + roots.txt
  chunks/         # reserved (payload chunks live in segments)
  catalogs/       # derived only (collections.cat, tier-placement.cat, segments.cat)
  indexes/        # derived only (primary.idx, sec/<coll>/*.six)
  snapshots/      # derived checkpoints
  recovery/       # operator scratch + migrations/ + compaction/ jobs (DEF-024)
```

Deleting `catalogs/`, `indexes/`, and `snapshots/` must not prevent recovery:
the store rebuilds current state by scanning `active/`, `segments/`, and online
tier media. Tier **media** files under `tiers/*` are authoritative when segments
live there.

## Surface

| API | Role |
|-----|------|
| `Store::open` / `Store::create` | Create-or-open; takes exclusive writer lock |
| `Store::open_inspect` | Read-only open (no writer lock, no derived writes) |
| `put` / `get` / `delete` | Subject-keyed current-state operations |
| `get_payload` | Completeness-aware read (`PayloadResult`) |
| `history` | Per-subject event stream |
| `WriteReceipt` | Event identity + acknowledged durability mode |
| `DurabilityMode` | `Memory`, `Buffered`, `Durable` |
| `rebuild_index` / `salvage` | Catalog-free scan of all segment files |
| `salvage_to` | Evidence-preserving frame copy + recovery manifest (DEF-011) |
| `export_live_state` | Live-only re-put materialization (new lineage) |
| `rebuild_catalogs` / `list_collections` | Derived collection catalog |
| `compact_live` / `compact_live_with` | Live projection via phased job (activate; optional reclaim) |
| `reclaim_compact_job` / `cancel_compact_job` | Explicit reclaim or pre-activate cancel (DEF-024) |
| `list_compact_jobs` / `load_compact_job` | Inspect durable compaction job records |
| `checkpoint` | Derived snapshot with declared coverage |
| secondary index helpers | Persist/load/delete `*.six` files |
| `examination_sources` | Ordered `(source_name, bytes)` for examination |
| `live_entries` | Index raw bodies (manifests for chunked subjects) |
| `live_logical_entries` | Fail-closed complete reassembly only |
| `scan_live_logical` | Opt-in envelope with incomplete subjects listed |
| `transfer_segment_to_tier` | Copy/move sealed segment (stable id) |
| `set_tier_available` / `tier_coverage` | Offline tier → coverage hole |
| `get_with_tier_coverage` | Absence only proven when coverage complete |
| `list_segment_summaries` / `rebuild_segment_catalog` | Cold-search hierarchy |
| `classify_segment` | Multi-gen format class; preserve unsupported bytes |
| `open_media` / `CloudMirrorConfig` | Object/media backends incl. S3/GCS mirrors |
| `LifecyclePolicy` | Declarative tier aging rules (`tiers/lifecycle.json`) |
| `ErasureManifest` | Archive shard naming contract (codec not shipped) |
| `failpoint` / crash matrix | DEF-022 injection points + embedded `crash_matrix.v1.json` |
| `IndexFrontier` / `persist_index_cache` | DEF-023 sealed fingerprint + active covered length; no write-path rescan |
| `random_id` / `mint_sortable_segment_id` / `ID_PROFILE` | DEF-025 CSPRNG random identities vs sortable segment ids |

Crash-boundary narrative: [`doc/CRASH_CONSISTENCY.md`](../../doc/CRASH_CONSISTENCY.md).
Write-path derived state: [CAPABILITY_MATRIX.md](../../doc/CAPABILITY_MATRIX.md) (DEF-023).
Compaction reclaim: same matrix (DEF-024).
Identifiers: same matrix (DEF-025).

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
