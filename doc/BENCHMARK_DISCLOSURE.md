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
| `dingo-store` `stage9_archive_bench` | archive-path class (separate) |

## Marketing language

- Prefer “hot-path p99 under durable ack on NVMe” over unqualified “Redis-fast”.
- Never claim archive reads have memory latency.
- Incomplete coverage (offline tier) is not “empty success” and must not appear
  as a zero-latency miss in charts without labeling.
