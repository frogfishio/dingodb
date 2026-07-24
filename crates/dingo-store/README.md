# dingo-store

Single-node authoritative store for DingoDB: filesystem-backed append-only
segments, put/get/delete by subject, durability modes, catalog-independent
recovery, and Stage 6 derived state (catalogs, secondary indexes, chunks,
history, compaction, checkpoints).

Normative sources: repository root [`OVERVIEW.md`](../../OVERVIEW.md) §§5–7, §13;
[`FORMAT_SPEC.md`](../../FORMAT_SPEC.md); [`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md)
Stages 3 and 6.

## Status

**Stages 3a–3c + 6** — open/create, put/get/delete, durability modes (`memory`,
`buffered`, `durable`), rebuildable primary index, salvage after catalog wipe,
OVERVIEW §16 store-level destructive suite, framed store descriptor, optional
on-disk primary index cache, collection catalog, secondary index files, subject
history, chunked payloads with partial maps, live-state compaction (sources
retained), derived checkpoints, benchmark skeleton.

Envelope bytes use a **draft fixed layout** (not yet deterministic CBOR).

## Layout (OVERVIEW §6.1)

```text
store/
  store-info/     # store_id + meta + descriptor.dingo
  active/         # open append segment (at most one)
  segments/       # sealed immutable segments
  chunks/         # reserved (payload chunks live in segments for Stage 6)
  catalogs/       # derived only (collections.cat)
  indexes/        # derived only (primary.idx, sec/<coll>/*.six)
  snapshots/      # derived checkpoints
  recovery/       # operator scratch
```

Deleting `catalogs/`, `indexes/`, and `snapshots/` must not prevent recovery:
the store rebuilds current state by scanning `active/` + `segments/`.

## Surface

| API | Role |
|-----|------|
| `Store::open` / `Store::create` | Create-or-open on a directory path |
| `put` / `get` / `delete` | Subject-keyed current-state operations |
| `get_payload` | Completeness-aware read (`PayloadResult`) |
| `history` | Per-subject event stream |
| `WriteReceipt` | Event identity + acknowledged durability mode |
| `DurabilityMode` | `Memory`, `Buffered`, `Durable` |
| `rebuild_index` / `salvage` | Catalog-free scan of all segment files |
| `rebuild_catalogs` / `list_collections` | Derived collection catalog |
| `compact_live` | Live projection into a new segment (sources retained) |
| `checkpoint` | Derived snapshot with declared coverage |
| secondary index helpers | Persist/load/delete `*.six` files |
| `examination_sources` | Ordered `(source_name, bytes)` for Stage 5 examination |
| `live_entries` / `live_logical_entries` | Index raw vs reassembled bodies |

## Non-goals (yet)

- Collection JSON DX (Stage 4/6 — see `dingo-sdk`)
- SDA examination projection (Stage 5 — see `dingo-examine`)
- Network / CLI doctor (Stage 7)
- Full deterministic-CBOR envelopes
- `replicated` durability

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
