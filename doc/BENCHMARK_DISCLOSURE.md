# Benchmark disclosure checklist (OVERVIEW §12.2)

Product follow-on: **do not** publish Redis-class or cross-tier latency claims
without this disclosure. Archive and cold paths are a separate performance
class from the hot path (OVERVIEW §12.1).

## Required fields for every published number

| Field | Example / notes |
|-------|-----------------|
| DingoDB version | `VERSION` + git SHA; wire [`WIRE_PROFILE_LABEL`](../crates/dingo-format/src/lib.rs) |
| SDK / cluster labels | `SDK_API_VERSION`, `CLUSTER_PROFILE_VERSION` |
| Durability mode | `memory` / `buffered` / `durable` / `replicated` |
| Verification mode | frame CRC + BLAKE3 body hash on/off for the workload |
| Hardware | CPU, RAM, storage type (NVMe / HDD / object) |
| Storage topology | single node vs cluster node count; tier roots |
| Dataset size | total bytes + live key count |
| Working-set size | bytes/keys expected hot in RAM indexes |
| Payload-size distribution | fixed N B, or histogram |
| Concurrency | client threads / outstanding ops |
| Compression / encryption | none (MVP) or named profile |
| Index freshness | live secondary indexes rebuilt? stale? |
| Replication | RF, ack mode |
| Latency | **p50 / p95 / p99** (not averages alone) |
| Throughput | ops/s and MB/s where relevant |
| Warm-up / recovery state | cold start vs post-rebuild; salvage path separate |

## Path classes (must not be mixed in one claim)

1. **Hot path** — memory indexes, local hot segments, durable or memory ack.
2. **Warm path** — sealed local/remote segments still online.
3. **Archive path** — cold/object/archive media; hierarchical catalogs; high latency expected.

Comparisons with Redis or other systems MUST use equivalent acknowledgement and
durability conditions.

## In-tree bench hooks

| Suite | Path class |
|-------|------------|
| `dingo-store` `stage6_bench_skeleton` | hot-path skeleton |
| `dingo-store` `stage_def_023_write_path` | hot-path write amplification / fsync-ack disclosure (DEF-023) |
| `dingo-store` example `write_latency_breakdown` | hot-path phase split: memory index vs buffered data vs full `persist_index_cache` vs seal |
| `dingo-store` example `write_scale_curve` | buffered scale curve (early vs late windows) |
| `dingo-store` example `read_latency_breakdown` | open-once vs reopen-per-get; PrimaryIndex gets vs Chimera sidecar probe |
| `dingo-store` `stage9_archive_bench` | archive-path class (separate) |

### Write latency: data vs indexing (measured attribution)

**Verdict (DEF-023 after seal / checkpoint decoupling):** the asymptotic
indexing failure is fixed. Steady-state put cost is healthy; remaining
window-level slowdowns come from **synchronous lifecycle work**
(`persist_index_cache`, `seal_active`), not from ordinary in-memory index
insertion.

#### Steady-state buffered puts (no checkpoint, no seal in the sample window)

Illustrative release-mode run on developer hardware (`write_latency_breakdown`,
n=2000 after warmup; absolute µs vary by machine/storage):

| Surface | Mean (approx.) | Notes |
|---------|----------------|-------|
| 4-byte buffered put | **~9 µs** | dual-index + encode/append/`write_all` |
| 8 KiB buffered put | **~20 µs** | data path grows with payload |
| Memory-mode index only (4 B) | **~3 µs** | one `PrimaryIndex` apply, no segment I/O |
| Memory-mode index only (8 KiB) | **~4 µs** | body clone into index entry |

A dual-index model (memory mode ≈ one apply; buffered ≈ two applies + I/O)
attributes roughly **35–46%** of mean put time to dual in-memory index publish
on mid-size payloads, with **data encoding, append, and OS write** dominating
larger values. Tiny payloads lean more index-heavy (often **~50–65%** dual-index
share) because the data path is so short.

#### Lifecycle spikes (still synchronous when they run)

| Surface | Order of cost | Scaling |
|---------|---------------|---------|
| `persist_index_cache` | **tens of ms** once thousands of live keys hold bodies | O(live subjects × body size) serialize + atomic fsync |
| `seal_active` (~4 MiB segment) | **tens of ms** | O(segment) rewrite + BLAKE3 + fsync; **not** a full `primary.idx` rewrite |

These are the remaining bottlenecks on a long pump: rate-limited derived
checkpoints (`DERIVED_CHECKPOINT_EVERY_OPS` = 65_536) and auto-seal at the
segment threshold still run **on the put acknowledgement path** when they fire.
They no longer grow with total retained history the way pre-DEF-023 full
rescans / full-index-on-seal did.

#### Scale curve (asymptotic check)

`write_scale_curve` (~128 MiB buffered 8 KiB puts, 4 MiB seal threshold) now
keeps late-window throughput in the same order as early-window throughput
(typically late/early **≳ 0.7** on quiet hardware). The historic collapse was
**~87%** loss (late/early **~0.13**) when full index rewrites sat on seal or
every few puts.

Run:

```bash
cargo run -p dingo-store --release --example write_latency_breakdown
cargo run -p dingo-store --release --example write_scale_curve
```

#### Maximum point self-check (diminishing returns)

**Question:** Have we reached the maximum useful point on write-path *index*
optimization — beyond which further work is diminishing returns?

| Claim | Verdict |
|-------|---------|
| Asymptotic indexing failure fixed? | **Yes.** Amortized put work no longer grows with retained history; late/early ≳ 0.7. |
| Steady-state put path healthy? | **Yes.** ~µs-class buffered puts; dual-index is a minority share on mid-size values. |
| Further primary-index micro-opts on the put path? | **Diminishing returns.** Shaving dual-index from ~40% → ~20% of a 20 µs put is low leverage vs **tens of ms** lifecycle spikes. |
| Fancy read indexes (PGM++/Hydra foundation)? | **Different axis** — foundation shipped at seal as derived sidecars; **not** yet wired into `Store::get`. Evaluate under dedicated read/rebuild benches, not the write cliff. |
| Write-path performance work finished? | **No.** Next high-leverage move is **async lifecycle**: dual active segments, O(1) rotate, background seal/checkpoint so p99 is not coupled to `seal_active` / `persist_index_cache`. |

**Plain answer:** For *ordinary index insertion on the steady-state write path*,
yes — we are past the cliff and further index-structure thrash is polish. For
*write-path p99 / sustained pump throughput*, no — the maximum point is not
async seal/checkpoint yet; that follow-on still has large returns.

Do **not** spend the next labor tranche on SwissTable/PGM/micro-opts of
`PrimaryIndex` apply unless a new measurement shows index re-dominating after
lifecycle is off the ack path.

### 1 GiB testrig diagnostic snapshot (not a published SLO)

Latest local campaign (`dingo-testrig run`, buffered, 4 KiB payloads, target 1 GiB;
summary under the work dir `testrig-summary.v1.json`). **Diagnostic only.**

| Phase | Signal (order of magnitude) |
|-------|-----------------------------|
| Pump | Target met (~131k keys / ~1.04 GiB on disk); release-mode tens of k ops/s class on quiet developer hardware |
| Baseline gets (128 samples) | All ok; p50/p95/p99 in the **µs** class via hot `PrimaryIndex` path |
| Chaos | Offline punches into sealed segments; salvage reports holes; live subjects retained |
| Post-chaos gets | Sampled keys still ok — integrity path speaks after damage |
| Hydra sidecars | Seal path writes `indexes/seg/*.hdx` (derived); not the get path above |
| Chimera sidecars | Seal path writes `indexes/chimera/*.cmr` (derived); **not** on hot `Store::get` |

### 10 GiB testrig diagnostic snapshot (not a published SLO)

Second local campaign after locator-first PrimaryIndex (**DEF-095**):
`dingo-testrig run --work /var/tmp/dingo-testrig-10g --target-bytes 10G
--payload-size 8192 --durability buffered --chaos-hits 64 --sample-keys 128
--seed 2` (release binary). Summary: `testrig-summary.v1.json`. **Diagnostic only.**

| Phase | Signal (order of magnitude) |
|-------|-----------------------------|
| Pump | Target met (~643k keys / ~10.05 GiB on disk); ~7.4k ops/s / ~119 MB/s class on quiet developer hardware |
| Process RSS (2 s `ps` samples) | Peak ~**0.92 GiB** during pump — **not** O(dataset); pre-fix observation was ~10 GiB → swap |
| Baseline gets (128 samples) | All ok; p50 **18 µs** / p95 **139 µs** / p99 **284 µs** via hot `PrimaryIndex` path |
| Chaos | 64 offline punches; salvage reports 128 holes; live subjects retained |
| Post-chaos gets | All 128 samples ok; p50 **19 µs** / p95 **152 µs** / p99 **279 µs** |
| Result | **PASS** — pump target, baseline healthy, chaos landed, salvage still speaks |
| Concurrency | **1** writer thread / single active segment (not multi-core ingest) |
| Host resource note | On Apple M4 (10 cores), process CPU mostly ~50% with peaks ~97% of **one** core; RSS ~0.92 GiB; SSD not saturated — headroom for DEF-096 parallel ingest |

### 10 GiB sharded testrig diagnostic snapshot (Axis B, not a published SLO)

Third local campaign after Axis A+B + testrig harness (2026-07-27):
`dingo-testrig run --work /var/tmp/dingo-testrig-10g-shards --target-bytes 10G
--payload-size 8192 --durability buffered --chaos-hits 64 --sample-keys 128
--seed 2 --writer-shards 4` (release binary). Summary:
`testrig-summary.v1.json`. **Diagnostic only.**

| Phase | Signal (order of magnitude) |
|-------|-----------------------------|
| Pump | Target met (~680k keys / ~10.57 GiB on disk); **~8.1k ops/s / ~129 MB/s** wall average; early window ~15–28k ops/s class before late-run lifecycle pressure |
| Process samples (`ps`, 2 s) | Peak RSS ~**1.05 GiB** (still O(keys), not O(dataset)); peak process CPU% ~**95** (macOS: 100% ≈ one core — serial index publish after `put_many` still limits multi-core %) |
| Baseline gets (128 samples) | All ok; p50 **24 µs** / p95 **426 µs** / p99 **563 µs** |
| Chaos | 64 offline punches; salvage reports 128 holes; live subjects retained |
| Post-chaos gets | All 128 samples ok; p50 **40 µs** / p95 **495 µs** / p99 **808 µs** |
| Result | **PASS** |
| Concurrency | **4** (`writer_shards: 4`, `writer_model: sharded_active_segments`, pump via `put_many`) |

Compare to single-shard 10 GiB snapshot above (~7.4k ops/s, concurrency 1, peak RSS ~0.92 GiB). Axis B raises disclosed concurrency and early-window throughput; wall-average lift is modest while serial PrimaryIndex publish + seal pipeline still bound multi-core CPU%. **Do not** claim “maxed out the M4.” Axis C (multi-process / cluster) remains the capacity path — see [`PARALLEL_INGEST.md`](PARALLEL_INGEST.md).

#### Multi-core write path status

After DEF-095 + DEF-096 Axis A+B:

1. Async lifecycle — **shipped**.
2. Sharded writers + testrig `--writer-shards N` disclosure — **shipped** (measured above).
3. Multi-store / cluster — **Axis C remains** (horizontal capacity).

#### Read-path failure that was fixed (Chimera-before-index)

A review of an intermediate testrig report saw **~245–259 ms get p50** with healthy
writes (~15k ops/s, 4–6 µs puts). Root cause was **not** salvage, reopen-per-get in
the harness, or NVMe cold reads: `Store::get_payload` preferred
`try_get_via_chimera`, which **fs::read + decode of the full per-segment `.cmr`**
(containers + value log) on **every** get, even when `PrimaryIndex` already held
the resident body.

| Surface | Expected after fix |
|---------|-------------------|
| Open-once `Store::get` / `open_inspect` get | **µs** class (resident PrimaryIndex) |
| Reopen inspect per get | open+rebuild cost (anti-pattern; measured separately) |
| `Store::get_via_chimera` | full sidecar load — diagnostic / future body-less path only |

Measure with:

```bash
cargo run -p dingo-store --release --example read_latency_breakdown
```

Do **not** quote testrig or example numbers as Redis comparisons, multi-node SLOs,
or “Hydra/Chimera latency” without a dedicated probe that actually uses those
indexes and full disclosure fields in the table at the top of this file.

### “Big flex” honesty line

Hydra + testrig + write-cliff fix is a **legitimate engineering flex** when
scoped as: adaptive derived segment indexes, scale-ladder integrity evidence,
and fixed asymptotic write indexing. It is **not** a production-maturity or
cross-engine performance claim. Full scope table:
[WORK_HORIZON.md](WORK_HORIZON.md) (“Is this a big flex?” self-check).

## Marketing language

- Prefer “hot-path p99 under durable ack on NVMe” over unqualified “Redis-fast”.
- Never claim archive reads have memory latency.
- Never attribute hot-path get latency to Hydra until `Store::get` (or a
  documented sealed-segment probe) actually uses `HydraIndex`.
- Never attribute hot-path get latency to Chimera until product get uses a
  **cached** locator path; full `.cmr` reload is not a point-read.
- Incomplete coverage (offline tier) is not “empty success” and must not appear
  as a zero-latency miss in charts without labeling.
