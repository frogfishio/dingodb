# Performance strategies (canonical)

Status: **written down 2026-07-27** after Axis A–C measurement and the
“things can go faster / numbers not ideal” and “next steps toward maximum
performance” self-checks.

This is the single place to re-read before spending labor on performance.
Narrative evidence lives in [`WORK_HORIZON.md`](WORK_HORIZON.md),
[`PARALLEL_INGEST.md`](PARALLEL_INGEST.md), and
[`BENCHMARK_DISCLOSURE.md`](BENCHMARK_DISCLOSURE.md).

---

## Principles (always)

1. **Fix the cliff, then measure, then choose the next bottleneck.** Do not
   guess the next optimization from vibe.
2. **Parallelism only pays when the shared serial section shrinks.**
3. **Capacity ≠ efficiency.** Multi-process harness proves media headroom;
   product scale is cluster partitions / multi-node.
4. **Free disk and path class are part of the experiment.** Near-full volume
   poisons numbers; hot / warm / archive claims must not mix.
5. **Diminishing returns are local, not global.** Write-index micro-opts are
   done; production readiness and other path classes are not.

---

## Write-path strategies (S1–S6)

| # | Strategy | Do | Do not |
|---|----------|----|--------|
| **S1** | **Declare write cliffs closed** | Treat DEF-023 / DEF-095 / DEF-096 A–C harness as closed cuts with residual only | Reopen PrimaryIndex structure thrash or “one more seal threshold tweak” as the main program |
| **S2** | **One optional Axis B efficiency cut** (only if labor stays on single-node write) | Shrink serial work after `put_many` (index publish / dual-apply tax); seal pipeline scales with shards | Rayon over one `Store::put`; multi-thread one active segment |
| **S3** | **Productize capacity on the cluster path** | Independent partition leaders (residiuum-cluster), honest RF/ack, network multi-process proof | Treat testrig `--stores N` as multi-tenant product sharding |
| **S4** | **Gate-driven program labor (default next)** | Jepsen/soak, fuzz, wire freeze, security review, CI quality bar | Polish already-landed cut follow-ons as if they move maturity |
| **S5** | **Measurement hygiene** | Disclose concurrency / writer_model / free disk / durability; p50/p95/p99; path class | Publish averages alone; claim multi-core from ~100% CPU |
| **S6** | **Read path only under dedicated benches** | Hydra/Chimera on get only after body-less / locator-cache design + `read_latency_breakdown` | “Parallelize gets” by loading full `.cmr` |

### Labor split (next few tranches)

```text
~70%  gate-driven readiness   (S4)
~20%  product capacity path   (S3)
~10%  optional Axis B residual (S2) — only with before/after numbers
  0%  PrimaryIndex micro-opts / Chimera-as-hot-get / put authority flip
```

---

## Maximum-performance ordered residuals

When labor is intentionally on performance (otherwise prefer S4):

| Priority | Step | Acceptance signal | Anti-goal |
|----------|------|-------------------|-----------|
| **1** | Axis B residual — serial index publish after `put_many` | Wall ops/s **and** process CPU% rise together at shards≥4; late/early ≳ 0.7 | More shards with same serial publish |
| **2** | Product capacity path (cluster) | Multi-process / multi-node with honest RF/ack | Harness multi-store as product |
| **3** | Durable / replicated ingest disclosure | `durable` (and replicated) benches with p50/p95/p99 | Claiming durable ≈ buffered |
| **4** | Hot sealed-segment reads (Hydra wire-up) | Cached Hydra → frame pread; attributed in `read_latency_breakdown` | Full `.cmr` load per get |
| **5** | Chimera compiler worker | Relocate/GC/recluster on derived layouts; frames stay authority | Put-path authority flip; dual-rep/ZNS |
| **6** | DEF-093 reproducible suite | Commands + raw results; CI catastrophic guard | Marketing averages without disclosure |
| **7** | Archive path (later) | Separate class after Milestone B/C | Cold under hot SLOs |

Scoreboard (diagnostic, buffered 8 KiB, M4-class):

| Config | Wall ops/s (10 GiB class) | Multi-core? |
|--------|---------------------------|-------------|
| Single | ~7.4k | no (~1 core) |
| Axis B shards=4 | ~8.1k (~+10%) | no (CPU% ~95) |
| Axis C stores=4 | ~17.7k (~2.4×) | yes (CPU% sum ~376) |

---

## SDA performance strategies

SDA (Structured Data Algebra) is a **separate performance class** from store
ingest/hot-get. Pure CPU, no disk; claims must not be mixed with store SLOs.

### Goals

| Layer | Product target | Current state |
|-------|----------------|---------------|
| **Standalone eval** | Fast parse-once / eval-many for host filters and examination | `run` re-parses every call; compile-once API + harness landed |
| **Filter parity path** | `matches_sda` usable for pushdown without recompile-per-doc | `compile_sda` + `matches_compiled_sda`; native `matches` remains default find path |
| **Distributed examination** | Push pure programs; workers share compiled semantics | CLUSTER_SPEC §17; not a store-bench problem |

### Strategies (apply in order)

| # | Strategy | Do | Do not |
|---|----------|----|--------|
| **A1** | **Measure phases first** | Attribute lex / parse / from_json / eval / to_json with `sda_latency_breakdown` | Optimize “SDA is slow” without a phase winner |
| **A2** | **Compile once, eval many** | Hosts that apply one program to N docs use `Program::parse` then `run_json` / `eval` | Re-parse source on every document (old `matches_sda` shape) |
| **A3** | **Keep native filter for scan find** | Default collection scan stays on `Filter::matches` until pushdown needs SDA | Replace every find with SDA eval without a measured reason |
| **A4** | **Preserve conformance** | Any polish must keep `sda-standalone-v1.0` + DEF-028 parity green | Speed by relaxing Fail/None/Null semantics |
| **A5** | **Disclosure** | Report program class (literal / projection / filter / map-comprehension), input size, iterations, p50/p95/p99 | Cross-compare to jq/Redis without equivalent work |

### Harness

```bash
# Diagnostic phase breakdown (release)
cargo run -p residiuum-sda --release --example sda_latency_breakdown

# CI skeleton (no performance gate — absurdity bounds only)
cargo test -p residiuum-sda --test sda_bench_skeleton
```

Numbers are **diagnostic only**. See
[`BENCHMARK_DISCLOSURE.md`](BENCHMARK_DISCLOSURE.md) § SDA.

### First measurement read (2026-07-27, diagnostic)

| Finding | Implication |
|---------|-------------|
| Filter re-parse wastes ~40–60% wall on DEF-028 predicates | **A2 is mandatory** for multi-doc SDA; `compile_sda` landed |
| Tiny projection/eval already ~0.5–1 µs p50 | Do not thrash micro-opts on filter eval next |
| `seq_comp_1k` ~6.5 ms eval p50 | Next **optional** residual is bulk comprehension / ExactNum path — only if a product host needs it |
| Native find still correct default | Keep **A3** |

### Multi-collection join (2026-07-28 re-measure, diagnostic)

Two client-side patterns at the same (customers, products, orders) scales:

| Pattern | Where join lives | Harness |
|---------|------------------|---------|
| **Nested SDA** | Pure SDA nested yields over combined bag | `multi_collection_sda_join_perf` |
| **Host hash equijoin + SDA normalise** | `Residiuum::query` progressive hash join; SDA projects | `multi_query_join_perf` |

```bash
# Nested SDA join (release)
cargo test -p residiuum-sdk --release --test multi_collection_sda_join \
  multi_collection_sda_join_perf -- --nocapture

# Host join + SDA normalise (release); MULTI_JOIN_BENCH_STRESS=1 for mod scale
cargo test -p residiuum-sdk --release --test multi_query_join_sda \
  multi_query_join_perf -- --nocapture
```

Illustrative release means on developer hardware (absolute ms vary):

| Scale | Nested SDA join mean | Host collect mean | Host map_sda mean | Ratio (nested / map_sda) |
|-------|----------------------|-------------------|-------------------|--------------------------|
| demo 3/3/5 | ~0.26 ms | ~0.10 ms | ~0.16 ms | ~1.6× |
| small 30/20/100 | **~143 ms** | **~1.4 ms** | **~1.9 ms** | **~75×** |
| mod 100/50/500 | ~8 s (stress note) | **~2.3 ms** | **~29 ms** | **~280×** (vs map_sda) |

**Read:** Host equijoin is the right product shape. Nested SDA is a viability
demo only — O(|orders| × lookups) pure eval cliffs at small product sizes.
At mod scale, host **collect** stays ~ms; residual wall is **SDA normalise**
over the joined bag (and `map_joined_sda` still re-parses the program each
call — optional A2 residual for the multi-query helper).

| Finding | Implication |
|---------|-------------|
| Host join ~75× faster than nested SDA at small | Prefer `Residiuum::query` + optional `map_sda`; do not nest joins in SDA |
| Collect ~ms through mod; SDA normalise grows with bag | Next optional residual: compile-once `map_joined_sda` + strip fat fields before normalise |
| Seed still dominates wall on tiny stores | Perf claims must separate seed I/O from join CPU |

### Anti-strategies

- Micro-optimizing ExactNum for programs that never allocate numbers.
- Spawning rayon inside pure eval without proving sequential eval is the bottleneck.
- Claiming examination throughput from standalone microbenches alone (missing unit projection / store read cost).
- Replacing default `Filter::matches` find with SDA without a pushdown plan.
- Nesting equijoins inside pure SDA when a host hash join is available.

---

## Verdict one-liners

| Question | Answer |
|----------|--------|
| Things can go faster? | **Yes** — Axis C and early windows show headroom. |
| Numbers ideal? | **No** — Axis B modest; single-process ~1-core. |
| Making movement? | **Yes** — cliffs closed; capacity evidence exists. |
| Next main program labor? | **S4 gates by default.** |
| Next performance residual? | Serial `put_many` index publish (ingest) or Hydra get (hot), only with measurement. |
| SDA next? | Phase-attributed harness + compile-once host path; keep native filter default. |
| Stop performance work forever? | **No** — stop **unmeasured** cliff-hunting and path-class mixing. |
