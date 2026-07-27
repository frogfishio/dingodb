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
| Fancy read indexes (PGM++/Hydra proposal)? | **Different axis** — not a write-path steady-state win; evaluate under read benchmarks, not this cliff. |
| Write-path performance work finished? | **No.** Next high-leverage move is **async lifecycle**: dual active segments, O(1) rotate, background seal/checkpoint so p99 is not coupled to `seal_active` / `persist_index_cache`. |

**Plain answer:** For *ordinary index insertion on the steady-state write path*,
yes — we are past the cliff and further index-structure thrash is polish. For
*write-path p99 / sustained pump throughput*, no — the maximum point is not
async seal/checkpoint yet; that follow-on still has large returns.

Do **not** spend the next labor tranche on SwissTable/PGM/micro-opts of
`PrimaryIndex` apply unless a new measurement shows index re-dominating after
lifecycle is off the ack path.

## Marketing language

- Prefer “hot-path p99 under durable ack on NVMe” over unqualified “Redis-fast”.
- Never claim archive reads have memory latency.
- Incomplete coverage (offline tier) is not “empty success” and must not appear
  as a zero-latency miss in charts without labeling.
