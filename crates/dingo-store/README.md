# dingo-store

Stage 3 single-node authoritative store for DingoDB: filesystem-backed
append-only segments, put/get/delete by subject, durability modes, and
catalog-independent recovery via `dingo-format` salvage scanning.

Normative sources: repository root [`OVERVIEW.md`](../../OVERVIEW.md) §§5–7,
[`FORMAT_SPEC.md`](../../FORMAT_SPEC.md), [`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md)
Stage 3.

## Status

**Stage 3a–3c** — open/create, put/get/delete, durability modes (`memory`,
`buffered`, `durable`), rebuildable primary index, salvage after catalog wipe,
OVERVIEW §16 store-level destructive suite, framed store descriptor, optional
on-disk primary index cache.

Envelope bytes use a **draft fixed layout** (not yet deterministic CBOR).
Payloads are opaque bytes.

## Layout (OVERVIEW §6.1)

```text
store/
  store-info/     # store_id + draft meta + descriptor.dingo (Stage 3c)
  active/         # open append segment (at most one)
  segments/       # sealed immutable segments
  chunks/         # reserved
  catalogs/       # derived only
  indexes/        # derived only (primary.idx cache, Stage 3c)
  snapshots/      # derived only
  recovery/       # operator scratch
```

Deleting `catalogs/`, `indexes/`, and `snapshots/` must not prevent recovery:
the store rebuilds current state by scanning `active/` + `segments/`. The
optional `indexes/primary.idx` cache is fingerprint-bound to segment files and
never the sole map of surviving data.

## Surface

| API | Role |
|-----|------|
| `Store::open` / `Store::create` | Create-or-open on a directory path |
| `put` / `get` / `delete` | Subject-keyed current-state operations |
| `WriteReceipt` | Event identity + acknowledged durability mode |
| `DurabilityMode` | `Memory`, `Buffered`, `Durable` |
| `rebuild_index` / `salvage` | Catalog-free scan of all segment files |
| `examination_sources` | Ordered `(source_name, bytes)` for Stage 5 examination |
| `live_entries` | Iterate live subjects + bodies (for SDK scans) |
| `persist_index_cache` | Write optional derived primary index (Stage 3c) |
| `store_descriptor_path` | Path of framed store descriptor (Stage 3c) |

## Non-goals (yet)

- Collection SDK / JSON DX (Stage 4 — see `dingo-sdk`)
- SDA examination projection (Stage 5 — see `dingo-examine`)
- Secondary indexes, query language, network
- Full deterministic-CBOR envelopes
- Chunked payloads on the write path
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
