# Parallel ingest: multi-core write path

Status: **Axis A + Axis B + Axis C harness implemented; testrig `--writer-shards` / `--stores` + 10 GiB Axis B re-measure done (2026-07-27)**  
Date: 2026-07-27  
Trigger: 10 GiB `dingo-testrig` on MacBook Air M4 after DEF-095  
Companion: DEF-023 follow-on, DEF-096, `doc/BENCHMARK_DISCLOSURE.md`, OVERVIEW §12

## 1. Observation (what you saw)

Hardware: **Apple M4**, 10 logical cores, fast internal SSD, plenty of RAM.

During the second 10 GiB pump (locator-first index, buffered, 8 KiB payloads):

| Resource | Observation | Interpretation |
|----------|-------------|----------------|
| **CPU** | Mostly ~50%, peak ~97% | One writer thread doing useful work part of the time; Activity Monitor process % is relative to **one core** (max ≈100% for single-threaded). Not “half of all cores.” |
| **Memory** | Peak RSS ~0.92 GiB on a 10 GiB store | DEF-095 worked. Not the limiter. |
| **Disk** | SSD not saturated | We are not I/O-bound at the media limit. |
| **Throughput** | ~7.4k ops/s, ~119 MB/s | Healthy for a serial put loop; far below what N cores × NVMe can do. |

Net: **CPU headroom and disk headroom both exist.** The pump is a single-CPU game because the store write path is a single exclusive writer over one active segment. Parallelizing is the right next scale question — but the shape matters.

## 2. What is serial today (code truth)

```text
dingo-testrig pump
  └── for each key: Store::put  (one thread, one Store handle)
        └── exclusive WriterLock (DEF-020)
              └── one active segment file
                    ├── encode frame
                    ├── append + write_all (buffered)
                    ├── dual PrimaryIndex apply
                    ├── every ~65k ops: persist_index_cache  (tens of ms, ON ack path)
                    └── when segment ≥ threshold: seal_active  (ON ack path)
                          ├── fsync rewrite sealed image
                          ├── BLAKE3 full segment
                          ├── Hydra compile  (single-threaded on seal)
                          ├── Chimera layout (single-threaded on seal)
                          └── start next active
```

Hard serial constraints:

1. **Exclusive writer lock** — one process owns the store for mutation.
2. **Single active segment** — appends are ordered into one file; offset assignment is sequential.
3. **Lifecycle on the put ack path** — seal and rate-limited checkpoint block the next put.
4. **Pump harness** — deliberately single-threaded; no client-side fan-out.

Already parallel elsewhere (not on the hot put path):

- Hydra `build_many` / `rebuild_hydra_indexes` (worker pool across segments).
- Server accept path: thread-per-connection (mutations still serialize on store mutex).
- Cluster: sharded partitions / leaders (horizontal, multi-process).

Product doctrine already names the target: OVERVIEW (“append-oriented, **parallel**, minimally coordinated”), USP (“**sharded writers**”), ingest path (“**sharded append** with optional asynchronous indexing”).

## 3. Wrong moves (do not do these first)

| Temptation | Why it fails |
|------------|--------------|
| Spawn N pump threads against one `Store` | Exclusive lock + single active segment → thrash or require `&mut` redesign; no free win. |
| “Just use rayon on put” | Put is not embarrassingly parallel; frame offsets and writer_seq need ordering. |
| Multi-thread PrimaryIndex micro-opts | Past the cliff (DEF-023 maximum-point self-check). Dual-index is a minority of a ~20 µs put. |
| Bigger seal threshold only | Reduces seal frequency; does not use other cores; makes crash windows larger. |
| Parallelize disk with more fsyncs | SSD is not the problem; more fsyncs make durable mode worse. |

## 4. Layered strategy (use cores in the right order)

Think of three independent axes. Each multiplies throughput without requiring the next.

```text
Axis A — Async lifecycle     (same logical writer, free other cores for seal work)
Axis B — Sharded append      (N active segments / writers in one process)
Axis C — Horizontal scale    (N stores / partitions / nodes — cluster path)
```

### Axis A — Async lifecycle (first; highest leverage per complexity)

**Status: landed** in `dingo-store` (`seal_pipeline` + `Store::rotate_active_async`).

Implementation shape:

1. **Dual-slot bound** — `DEFAULT_MAX_PENDING_SEALS = 2` (live active + pending finalize).
2. **O(1) rotate** — on auto-seal threshold: durable flush, `rename` `active/active.dingo` → `active/pending/{hex}.dingo`, start new active. Put does **not** wait for BLAKE3/Hydra/Chimera.
3. **Background worker** (`dingo-seal-pipeline` thread) finalizes: offset-preserving summary seal, sealed image write, BLAKE3, Hydra, Chimera, then catalog apply on the writer thread via `poll`/`drain`.
4. **Background checkpoint** — rate-limited `persist_index_cache` clones locator-first durable index and writes on the worker.
5. **Backpressure** when `inflight_seals >= max` — put waits for one completion.
6. **Explicit `seal_active`** remains synchronous (drains pipeline first) for tests/failpoints.
7. **Crash recovery** — `Store::open` runs `recover_all_pending` before index rebuild.

Tests: `crates/dingo-store/tests/stage_def_096_async_lifecycle.rs`.

Why this matches the M4 observation:

- Steady-state put is already µs-class; the long-run 1 GiB → 10 GiB gap is lifecycle + cache pressure, not index asymptotics.
- Seal work is **CPU-heavy** (BLAKE3, Hydra, Chimera) on **immutable** bytes → perfect for other cores.
- Single-writer correctness stays intact (one owner process, ordered acks still possible).
- Expected: put thread stays near 100% of one core on pure append; remaining cores absorb seals; process CPU% can climb toward several hundred % on Activity Monitor; wall-clock pump time drops without multi-writer races.

**Acceptance sketch:**

- Buffered pump p99 not coupled to `seal_active` / `persist_index_cache` duration.
- Background seal preserves crash matrix (failpoints still meaningful).
- Derived indexes remain non-authoritative; loss → rebuild.
- 10 GiB pump: higher ops/s and/or multi-core process CPU% while RSS stays O(keys).

### Axis B — Sharded writers (second; true multi-core append)

**Status: landed** in `dingo-store` (`Store::create_with_shards`, subject-hash routing, `put_many`).

Implementation shape:

1. **Shard by subject hash** (`subject_writer_shard` / first 8 LE bytes of `subject_item_id`) into **N active segments** (`1..=MAX_WRITER_SHARDS`).
2. Each shard: own file handle + append offset under `active/shard-NN/active.dingo` (N=1 keeps legacy `active/active.dingo`).
3. Shared process-wide writer lock (DEF-020); seal lifecycle is per-shard auto-rotate with a shared pending dir + worker pool (backpressure scales with N).
4. Single primary index (latest-wins); subject → one home shard so concurrent multi-subject puts never race the same key across writers.
5. **`put_many`** partitions by shard and runs shard appends in parallel (`std::thread::scope`), then publishes the index serially.
6. Count persisted in `store-info/writer_shards`; recovered on open. Tests: `stage_def_096_sharded_writers`.

**Correctness rules (held):**

- Latest-wins per subject stays well-defined (subject → one home shard).
- History / salvage still segment-local; rebuild/open scan all active shards + pending + sealed.
- Process-wide exclusive writer lock retained; shard sub-ownership only for the parallel batch path.
- Multi-shard index frontier is sealed-only; open re-applies all active shard files.

**When to use:** create with `Store::create_with_shards(path, n)` when one append core saturates and media still has headroom. Default `create` remains single-shard for compatibility.

### Axis C — Horizontal / multi-process (harness landed)

**Status: testrig multi-store harness landed** (`--stores N` → N child processes).

- **Harness:** `dingo-testrig pump|run --stores N` (N>1) creates `store-00`… under the parent/work dir, spawns N child `pump` processes (true multi-process), splits `target-bytes` across stores, aggregates wall-clock ops/s + summed child RSS/CPU%, discloses `store_count` / `concurrency = stores × writer_shards` / `writer_model: multi_process_stores[_sharded]`.
- Multiple independent stores (manual N pumps under N dirs still works).
- Cluster partitions with independent leaders (dingo-cluster) remain the **product** multi-node path.
- This is **capacity**, not single-node efficiency. The multi-store pump is a media upper-bound bench — **not** product sharding.

## 5. Where the cores go (target pipeline)

```text
                    ┌──────────── client / pump threads ────────────┐
                    │  (1 after A; N after B)                         │
                    └───────────────┬────────────────────────────────┘
                                    │ put(subject, body)
                    ┌───────────────▼────────────────────────────────┐
                    │  shard select (B) → append encode + write_all   │  few cores
                    │  publish locator-first PrimaryIndex             │
                    └───────────────┬────────────────────────────────┘
                                    │ rotate (O(1))
              ┌─────────────────────┼─────────────────────┐
              ▼                     ▼                     ▼
         seal worker           seal worker           checkpoint worker
         BLAKE3                BLAKE3                primary.idx frontier
         Hydra compile         Hydra compile         catalog snapshot
         Chimera layout        Chimera layout
              (many cores; already-safe immutable segment bytes)
```

Hydra `build_many` already knows how to fan out; seal workers should call the same builders off the put thread.

## 6. Testrig / measurement changes

| Change | Status |
|--------|--------|
| Report `concurrency` / `writer_model` / `writer_shards` in pump + run JSON | **Done** — honest disclosure (BENCHMARK_DISCLOSURE “Concurrency”). |
| Per-interval ops/s + peak RSS / process CPU% via `ps` | **Done** in pump progress + summary. |
| `--writer-shards N` on `pump` / `run` | **Done** — creates with `create_with_shards(N)`; N>1 pumps via `put_many`. |
| Multi-store pump mode (`--stores N` → N store roots) | **Done** — multi-process child pumps; Axis C capacity harness. |

```sh
# Multi-core Axis B campaign (diagnostic only)
dingo-testrig run \
  --work /var/tmp/dingo-testrig-10g-shards \
  --target-bytes 10G \
  --payload-size 8192 \
  --writer-shards 4 \
  --seed 2

# Multi-process Axis C campaign (diagnostic only; N independent stores)
dingo-testrig run \
  --work /var/tmp/dingo-testrig-10g-stores \
  --target-bytes 10G \
  --payload-size 8192 \
  --stores 4 \
  --seed 3
```

Multi-store pump is a **harness** parallelization for Axis C upper-bound media benches — not product sharding.

## 7. Sequencing and anti-goals

**Done (DEF-096):**

1. Axis A (async lifecycle, dual slots) — landed.
2. Axis B sharded writers (`create_with_shards`, `put_many`) — landed.

**Do next:**

3. ~~Wire testrig `--writer-shards N` / multi-core pump disclosure fields.~~ **Done** (2026-07-27).
4. ~~Re-run 10 GiB testrig with writer shards; compare ops/s, p99, process CPU%, RSS.~~ See `doc/BENCHMARK_DISCLOSURE.md` (sharded 10 GiB snapshot).
5. ~~Multi-store harness mode (`--stores N`).~~ **Done** (2026-07-27) — multi-process child pumps + disclosure.
6. Optional: large multi-store 10 GiB re-measure + disclose in `BENCHMARK_DISCLOSURE.md` (diagnostic only).
7. Product cluster capacity remains dingo-cluster partitions / multi-node (not this harness).

**Do not:**

- Rewrite PrimaryIndex for write throughput.
- Put Chimera/Hydra on the hot get path as a “parallelization” project.
- Claim multi-core ingest until disclosure fields show concurrency > 1 or multi-core CPU samples.
- Treat `--stores N` as product multi-tenant sharding.

## 8. Expected outcomes (honest)

| Stage | Cores used | Rough hope on M4-class machine |
|-------|------------|--------------------------------|
| Today | ~1 (part-time) | ~7–20k ops/s depending on size / seal rate |
| After Axis A | 1 append + K seal workers | Higher sustained ops/s; p99 flatter; process CPU% multi-core |
| After Axis B | N append + K seals | Near-linear until media or memory bandwidth |
| After Axis C harness | multi-process N stores | Media upper-bound capacity (testrig) |
| Product multi-node | cluster partitions / leaders | Deployment capacity |

No SLO claims until measured with full `doc/BENCHMARK_DISCLOSURE.md` fields.

## 9. Decision (2026-07-27 self-check)

**Question:** Given free CPU and free SSD after DEF-095, should we parallelize?

**Answer: Yes — but via async lifecycle first, then sharded writers, not by multi-threading a single `Store::put`.**

Your instinct (“single-CPU game; max out all the cores”) is correct. The product specs already demand it. The implementation debt is **lifecycle on the ack path + single active segment**, not missing rayon on the pump loop.
