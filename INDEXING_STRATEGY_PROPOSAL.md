# Hydra indexing strategy

Status: **implemented (foundation)** in `dingo-store::hydra`  
Companion: seal path writes `indexes/seg/{segment_hex}.hdx` sidecars (derived only).  
Honesty: hot `Store::get` still uses `PrimaryIndex`; Hydra is seal/rebuild/load API today
(see `doc/WORK_HORIZON.md` “big flex” self-check and `doc/BENCHMARK_DISCLOSURE.md`).

## Recipe: hydra + multithread

Most engines pick one index structure globally. Hydra **compiles the optimal
index independently for every immutable segment**, using key distribution and
workload hints, and builds many segments in parallel.

| Segment shape | Physical structure | Module |
|---------------|-------------------|--------|
| Tiny (≤ 64 keys, default) | Sorted **Eytzinger** array | `hydra::eytzinger` |
| Ordered numeric, sparse | **PGM++**-style piecewise linear + bounded last mile | `hydra::pgm::PgmIndex` |
| Ordered numeric, dense | **RadixSpline** (radix table + spline knots) | `hydra::pgm::RadixSplineIndex` |
| Ordered strings / irregular | **Compressed ART / radix** (path-compressed) | `hydra::radix` |
| Point-only immutable set | **MPHF + fingerprint** (CHD hash-and-displace) | `hydra::mphf` |

Selection is automatic via `select_index_kind` / `HydraBuildOptions`
(`point_only`, `tiny_threshold`, `force_kind`, `threads`).

## Read path (target architecture)

```text
hot cache (future TinyLFU)
→ latest-version hash (PrimaryIndex / frontier cache)
→ binary-fuse filter (planned per-segment)
→ adaptive per-segment Hydra index
→ one bounded last-mile search
→ segment frame offset
```

Shipped today:

1. **Adaptive per-segment Hydra** at seal time (and `Store::rebuild_hydra_indexes`).
2. **Multithreaded** multi-segment build (`build_many` / rebuild API).
3. **Derived-only** sidecar codec (`DHYDRA01`); loss never blocks salvage.

Still proposed (not required for this foundation cut):

- Global latest-version SwissTable sharded by fingerprint (write-path already has `PrimaryIndex`).
- Binary-fuse filter on every immutable segment.
- Hot-key TinyLFU accelerator in front of Hydra.

## API surface (`dingo-store`)

```rust
// Build one segment
let idx = dingo_store::build_hydra_index(&records, &HydraBuildOptions::default());
assert!(idx.get(b"key").is_some());

// Parallel build of many segments
let many = dingo_store::build_hydra_indexes(&batches, &HydraBuildOptions { threads: 4, ..Default::default() });

// Store integration
store.seal_active()?;                    // writes indexes/seg/{id}.hdx
store.rebuild_hydra_indexes(&opts)?;     // multithread rebuild
store.load_hydra_index(segment_id)?;     // Option<HydraIndex>
```

## Why this beats a single global B-tree/LSM fan-out

- Seal is already O(segment); compiling Hydra there does not re-touch retained history.
- Point reads on sealed data become one adaptive probe instead of multi-segment merge.
- Tiny segments skip learned models; string keys skip numeric rank models; point-only
  workloads skip ordered structure entirely (MPHF).
- Multithread rebuild keeps recovery/compaction-side index compile off the hot write path.

## Correctness rules

- Hydra files are **derived only** (OVERVIEW §5.5). Missing/corrupt → rebuild or ignore.
- MPHF path does not support ordered scan (`scan_after` returns empty).
- Duplicate keys within a segment: **last offset wins** (matches seal-time latest).
- Fingerprint verification on MPHF rejects wrong keys at the perfect slot (false
  positive probability ~2⁻⁶⁴ per probe under the 64-bit fingerprint).

## Implementation map

```text
crates/dingo-store/src/hydra/
  mod.rs       # HydraIndex enum, build / build_many, codec, seal helpers
  select.rs    # KeyShape + IndexKind selection
  eytzinger.rs # tiny
  pgm.rs       # PGM++ + RadixSpline
  radix.rs     # compressed ART/radix
  mphf.rs      # MPHF + fingerprint
```

Tests: unit tests under `hydra::*` and integration `tests/hydra_segment_index.rs`.
