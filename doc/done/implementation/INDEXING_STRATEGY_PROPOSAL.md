# Hydra indexing strategy

Query-order companion:
[ORDER_WAVELET_SPEC.md](../../todo/order-wavelets/ORDER_WAVELET_SPEC.md). Hydra locates immutable
records; Residiuum Order Wavelets are a separate derived structure for exact
filter-conditioned ranked ordering. Neither is required to impersonate the
other.

Status: **implemented (foundation)** in `residiuum-store::hydra`  
Companion: seal path writes `indexes/seg/{segment_hex}.hdx` sidecars (derived only).  
Honesty: hot `Store::get` uses resident `PrimaryIndex` only. Hydra is seal/rebuild/load
API today; Chimera layouts are seal/compact derived sidecars probed via
`Store::get_via_chimera` (see `doc/done/programs/WORK_HORIZON.md` and `doc/reference/operations/BENCHMARK_DISCLOSURE.md`).
**Do not** load full `.cmr` files on the product get path — that regression produced
~250 ms sample gets on 1 GiB testrig runs.

**Chimera** (FINAL DESIGN below): **foundation + seal/compaction wire-up** in
`residiuum-store::chimera` — locator types, value-class selection, micro-page
containers, large-value log codec, adaptive I/O path selection, background-compiler
plans, and `indexes/chimera/*.cmr` layouts written at seal/compact. Put still writes
segment frames (authoritative). Sequencing: do **not** flip put to omit frame bodies
yet; dual-rep/ZNS stay deferred; next Chimera cut is a compiler **worker** plus
**cached** locator resolve (see sequencing table below).

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
3. **Derived-only** sidecar codec (`RHYDRA01`); loss never blocks salvage.

Still proposed (not required for this foundation cut):

- Global latest-version SwissTable sharded by fingerprint (write-path already has `PrimaryIndex`).
- Binary-fuse filter on every immutable segment.
- Hot-key TinyLFU accelerator in front of Hydra.

## API surface (`residiuum-store`)

```rust
// Build one segment
let idx = residiuum_store::build_hydra_index(&records, &HydraBuildOptions::default());
assert!(idx.get(b"key").is_some());

// Parallel build of many segments
let many = residiuum_store::build_hydra_indexes(&batches, &HydraBuildOptions { threads: 4, ..Default::default() });

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
crates/residiuum-store/src/hydra/
  mod.rs       # HydraIndex enum, build / build_many, codec, seal helpers
  select.rs    # KeyShape + IndexKind selection
  eytzinger.rs # tiny
  pgm.rs       # PGM++ + RadixSpline
  radix.rs     # compressed ART/radix
  mphf.rs      # MPHF + fingerprint
```

Tests: unit tests under `hydra::*` and integration `tests/hydra_segment_index.rs`.



# FINAL DESIGN

Yes: build a **workload-compiled storage layer**, not one universal SSTable format.

## “Chimera” data storage

### 1. Three physical value classes

At write time, classify by size and access pattern:

```text
tiny values       → inline with Hydra entry
medium values     → packed into immutable micro-pages
large values      → append-only extent/value log
```

Do not send every value through the same layout. Key/value separation reduces compaction write amplification, but indiscriminate separation hurts tiny-value locality. WiscKey validates the basic SSD-oriented separation model. 

### 2. Direct-addressable micro-pages

Replace traditional 4–16 KiB SSTable blocks with sealed **64–256 KiB containers** containing independently readable records:

```text
container header
dictionary
offset table
individually compressed values
checksums
```

Hydra stores:

```text
container_id + slot + generation
```

A point read fetches only the record’s required aligned region—not a whole compressed block.

### 3. Temperature-and-lifetime placement

Compile sealed data into physical classes:

```text
hot/random       → replicated or cache-aligned NVMe extents
warm/mixed       → packed micro-pages
cold/range-heavy → key-ordered compressed runs
short-lived      → dedicated append zones
long-lived       → separate zones/extents
```

Grouping objects with similar lifetimes reduces garbage-collection copying. This becomes even more powerful on ZNS or FDP-capable SSDs, where the host controls placement instead of fighting opaque device garbage collection. Recent ZNS research reports large reductions in device-level write amplification when placement and zone management are handled carefully, although results remain hardware- and prototype-specific. 

### 4. Data recompilation during GC

Garbage collection becomes an optimizer:

```text
point-hot records     → dense random-read containers
scan-hot key ranges   → key-ordered extents
compressible families → shared trained dictionary
cold records          → larger compressed containers
```

Hydra locator updates occur through generation-safe atomic relocation. Old extents remain readable until epoch reclamation completes.

### 5. Dual physical representations

For truly hot mixed workloads, allow selected values to exist twice:

```text
point-optimized copy + scan-optimized copy
```

Hydra chooses the representation based on operation type. This trades controlled space amplification for lower read amplification—the same type of explicit trade-off conventional engines already make, but at record or range granularity.

### 6. Hardware-native I/O

Use:

- per-core registered-buffer pools;
- batched `io_uring`;
- fixed file descriptors;
- direct I/O for controlled large reads/writes;
- buffered I/O for tiny or highly cached access;
- multiple independent queues per NVMe device;
- NUMA-local completion processing.

Do not assume `io_uring` or direct I/O is inherently faster. Select the path by size and locality.

## Final architecture

```text
Hydra locator
    ↓
resident value
OR inline value
OR point container
OR scan extent
OR large-value log
    ↓
adaptive buffered/direct async I/O
    ↓
record-level decompression

Background compiler:
GC + relocation + reclustering
+ dictionary training
+ hot/cold migration
+ lifetime-aware placement
+ optional representation replication
```

The genuinely insane differentiator is:

> **Indexes and data are both compiled independently per segment or key range from measured workload characteristics.**

Traditional engines ask, “Which global storage format should we use?” Yours asks, “What physical representation is fastest for this particular data partition right now?” That could produce exceptional results—but begin with inline/value-log hybrid, micro-pages, and temperature-aware GC before adding duplicate representations or ZNS-specific placement.

## Implementation map (Chimera foundation)

```text
crates/residiuum-store/src/chimera/
  mod.rs        # ValueLocator, resolve API, module root
  classify.rs   # ValueClass + temperature/lifetime selection
  container.rs  # point micro-page containers (independent slots)
  value_log.rs  # large-value append log codec
  io_path.rs    # adaptive buffered vs direct/async selection
  compiler.rs   # GC / relocation / reclustering / dictionary / hot-cold plans
```

Tests: unit tests under `chimera::*` + store seal/get/compact wire-up tests.

**Seal/compaction wire-up (landed):** default `build_compact_layout` →
`indexes/chimera/{hex}.cmr` **layout version 2** with `SegmentFrame` locators
(segment id + frame offset/len; empty containers/value-log). Payloads remain in
authoritative segments. Legacy version-1 full-payload embeds remain readable;
`build_materialized_layout` is explicit/obsolete. Hot `Store::get` uses
PrimaryIndex; `get_via_chimera` resolves compact locators via segment pread.
Put path still writes full segment frames (Chimera is **derived**, never
authority). Dual-representation and ZNS placement stay deferred.

### Sequencing decision (do we implement put-compile / dual-rep·ZNS·worker?)

| Candidate | Decision | Rationale (short) |
|-----------|----------|-------------------|
| Put workload-compiled (omit full frame bodies) | **Not yet** | Would make `.cmr` authoritative and break “wipe derived → still recover” (OVERVIEW). Needs FORMAT/profile + crash-matrix redesign first. |
| Compiler **worker** (execute `plan_compile` ops) | **Next Chimera cut** when prioritized | Planner exists; without execute, layouts never recompile after seal. Operate on derived `.cmr` only; frames remain salvage truth. |
| Dual physical representation | **Deferred** | After temperature GC + measured mixed-hot telemetry; opt-in already stubbed (`enable_dual_representation`). |
| ZNS / FDP host placement | **Deferred** | After portable lifetime/temperature zones; device-specific evidence required. |

See `doc/done/programs/WORK_HORIZON.md` “Decision: implement put-path Chimera / dual-rep·ZNS·worker?”
for the full authority and program-leverage argument. Do not claim “primary storage
is workload-compiled” until put classification and payload omission are explicit
format work — not a silent put-path change.
