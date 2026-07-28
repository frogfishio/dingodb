# Work horizon self-check

Status: snapshot as of 2026-07-27  
Audience: operators of the engineering program (humans + agents)  
Companion: [DELIVERY_PLAN.md](../DELIVERY_PLAN.md), [DEFECTS.md](../DEFECTS.md),
[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md),
[PRIME_TIME_PLAN.md](PRIME_TIME_PLAN.md) (wedge + path to prime time)

## Question

> Are we running out of things to do? Are we now polishing small things with
> diminishing returns?

## Short answer

**No — not for production readiness.**  
**Yes — for the original staged product arc (Stages 0–9).**  
**Yes — risk of polish thrash if labor keeps landing “remaining (out of this cut)”
follow-ons instead of the next hard gates.**

The staged delivery plan is complete. The production-readiness program is not.
Recent commits *look* like polish because the critical-path cuts (DEF-035–041,
DEF-050–054, DEF-060–061) shipped quickly and each left a trail of smaller
follow-ons. Those follow-ons are real but lower leverage than the open P0/P1
items and unchecked §16 gates.

## What is finished

| Horizon | State |
|---------|--------|
| [DELIVERY_PLAN.md](../DELIVERY_PLAN.md) Stages 0–9 | **Done** (status Draft v0.23) |
| Product follow-ons 1–4 (mirror S3/GCS, multi-hop, freeze labels, scaffolds) | **Done** |
| Immediate containment DEF-001–003 | **Remediated** |
| P0 single-node correctness DEF-010–014 | **Remediated / largely remediated** |
| Single-node foundation DEF-020–029 | **Addressed** (with named follow-ons) |
| Server foundation DEF-030–034 | **Addressed** |
| Distributed foundation cuts DEF-035–041 | **Addressed (in-process / experimental cuts)** |
| Ops cuts DEF-050–052, 054, 060–061 | **Addressed (single-node / process cuts)** |

A reader of `DELIVERY_PLAN.md` alone would correctly conclude: **we are past the
“build the product in stages” program.** That is why work can feel like polish.

## What is not finished (high leverage)

From [DEFECTS.md](../DEFECTS.md) and §16 release gates — still open or only
partially cut:

### Hard correctness / qualification (do not skip)

| ID | Why it is not polish |
|----|----------------------|
| **DEF-041 follow-ons** | Multi-process Jepsen-style partition histories; long soak / rolling restart. In-process sim is not production evidence. |
| **DEF-053** | Wire major-1 freeze after soak, fuzz, external review. Still `1.0-draft`. |
| **DEF-063** | Threat model first cut landed; **independent audit** + disclosure policy still required (P0). |
| **DEF-091** | Format first cut landed (proptest + cargo-fuzz); **continuous service + remaining parsers** still open (P0). |
| **§16.1–16.3** | Most data-safety and distributed gates remain unchecked. |

### Product completeness (real surfaces, not nits)

| ID | Gap |
|----|-----|
| **DEF-096** | Parallel ingest after DEF-095: **Axis A + B + C harness landed** + **product `Cluster::put_many` multi-partition fan-out** (`stage_def_096_product_capacity`). Network multi-process product capacity still open. |
| **DEF-039 / 040 network** | Anti-entropy repair and query paging still in-process only. |
| **DEF-050–052 follow-ons** | Incremental/encrypted backup; cluster-coordinated backup; scrub daemon; second wire major + rolling upgrade drills. |
| **DEF-070–074** | Native object stores, lifecycle scheduler, erasure coding, encryption, multi-decade retention proof — archive product, currently scaffold/mirror. |
| **DEF-080–084** | SDK MVP gaps, operator CLI depth, executable journeys, distribution, compatibility policy. |
| **ENR1 (in `dingo-sda`)** | Match-bag **kernel + surface** shipped (`sda-enr1-v0.1`: `Match`/`enrich`/`one?`/`one!`/`merge`/`mergeLeft`/`mergeRight`/`+`/`asBag`/keyed sugar/`source` decls/`t_enr_invalid_key`). Multi-gen expand still blocked on SDA multi-gen. **DQL** v0.1 official dialect compiles to the same kernel (`compile_dialect("dql", …)`); proof tests equate DQL ≡ pure ENR. Foreign dialects absorb comfort. **Do not open ENR2 yet.** |

### Engineering quality bar (release blockers, not vanity)

| ID | Gap |
|----|-----|
| **DEF-090** | **Addressed** — CI quality bar (fmt/clippy/deny/doc/package/MSRV/OS matrix) + `scripts/quality.sh`. |
| **DEF-091** | **Format first cut** — proptest + cargo-fuzz targets for frame/scan/cbor; broader surfaces + continuous service remain. |
| **DEF-063** | **Threat-model first cut** — `doc/THREAT_MODEL.md`; independent audit + disclosure policy remain. |
| **DEF-092–094** | Coverage/sanitizers/models; disclosed benchmarks; release + incident process. |
| **DEF-062** | Distributed tracing (P2; only after metrics/logs are load-bearing). |

## What *is* diminishing-returns polish (if prioritized next)

Do **not** spend the next labor tranche primarily on these unless they unblock a
gate:

- Client-side structured log emission join fields (DEF-060 remaining).
- Extra store/cluster gauge series and dashboard packages (DEF-061 remaining).
- Live dynamic config reload of admission limits (DEF-054 remaining).
- Worker-pool reuse / concurrent read snapshots (DEF-030 remaining).
- Remote page RPC / plan RPC when embedded already honest (DEF-026/028 follow-ons).
- Unique indexes without partition-scope design (DEF-027 follow-on).
- Per-tenant ACL store depth beyond current RPS admission.
- README / crate README wording nits once maturity labels already match DEF-001.
- Second-pass documentation of already-shipped profile constants.

These improve operability and completeness of already-landed cuts. They do
**not** move the deployment classification off “experimental / development only /
integration-test harness.”

## Deployment honesty (still true)

From [CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md) and DEFECTS §2:

| Profile | Maturity today |
|---------|----------------|
| Embedded single-node | experimental / early-access |
| Single-node TCP | development only |
| In-process cluster | integration-test harness |
| Network multi-node (`serve-cluster`) | **experimental** (not production) |
| S3/GCS | filesystem mirror, not native cloud I/O |
| Erasure / lifecycle | scaffolds |
| Wire | `1.0-draft` |

Until Milestone B/C gates close, polishing experimental surfaces does not create
a production product.

## Suggested next labor order (anti-diminishing-returns)

Prefer this order over “finish remaining bullets under the last DEF”:

1. **Close Milestone A honestly** — embedded early-access: finish any still-open
   A requirements (DEF-090–092 core, crash matrix residual, evidence that
   README labels match tests).
2. **Multi-process distributed proof** — DEF-041 network Jepsen / soak; network
   repair + query page RPCs only as needed for that proof.
3. **Wire freeze path** — DEF-091 fuzz + DEF-053 qualification criteria (even if
   freeze is not declared yet).
4. **Milestone B packaging** — security review (DEF-063), distribution
   (DEF-083), config/ops remaining only when they block serve production.
5. **Archive product (Milestone D)** — only after B/C are real; do not expand
   erasure/lifecycle scaffolds early.

## Self-check verdict

| Claim | Verdict |
|-------|---------|
| Running out of meaningful work? | **No.** Open P0/P1 and §16 gates are large. |
| Original stage plan exhausted? | **Yes.** Stages 0–9 + follow-ons 1–4 done. |
| Recent work felt like polish? | **Understandable** — ops cuts (config/log/metrics/backup/scrub) sit above a finished stage tree. |
| Currently in diminishing returns? | **Only if** labor keeps polishing cut follow-ons instead of multi-process verification, wire freeze qualification, fuzz/security, and release gates. |
| Stop building? | **No.** Shift from “land next DEF cut” autopilot to **gate-driven** work. |

### Write-path index axis (orthogonal check)

Separate from program horizon: DEF-023 + measured attribution
(`doc/BENCHMARK_DISCLOSURE.md`) put the **steady-state write index path** past
its cliff. Further primary-index micro-opts are diminishing returns; async
seal/checkpoint (DEF-023 follow-on) is still high leverage for p99. Do not
confuse “index asymptote fixed” with “write performance finished” or with
“product readiness finished.”

### 10 GiB memory-eat self-check (2026-07-27)

User observation after 10 GiB testrig: process RSS ~10 GiB → host swap →
poisoned latency metrics.

**Root cause (not “MacBook weak”):** fat primary index held full payload bodies
twice (`index` + `durable_index`) and checkpointed them into `primary.idx`
(~3.5 GiB bodies for ~3.5 GiB segments). See **DEF-095**.

**Cut shipped:** locator-first PrimaryIndex + cache v3 + frame pread on get.
Resident index bodies are O(keys × metadata), not O(dataset). Chimera still
duplicates values on **disk** (~3.5 GiB `.cmr`); that is a follow-on disk
amplification issue, not the RSS spike path.

**Re-run confirmation (2026-07-27, second 10 GiB campaign):**
`dingo-testrig run --target-bytes 10G --payload-size 8192 --seed 2` → **PASS**.
Peak process RSS while pumping ~**0.92 GiB** (2 s samples via `ps`), not ~10 GiB.
~643k keys / 10.05 GiB on disk; baseline get p50 **18 µs**; post-chaos get p50
**19 µs** with 128 salvage holes; pump ~7.4k ops/s. Summary:
`/var/tmp/dingo-testrig-10g/testrig-summary.v1.json`. Diagnostic only.

**Sharded re-measure (2026-07-27, Axis B harness):**
`dingo-testrig run --target-bytes 10G --payload-size 8192 --seed 2
--writer-shards 4` → **PASS**. ~680k keys / 10.57 GiB; wall ~**8.1k ops/s**
(~129 MB/s); peak RSS ~**1.05 GiB**; peak CPU% ~95 (still ~1-core class under
`ps` — serial index publish after `put_many`); concurrency **4** /
`sharded_active_segments`. Summary:
`/var/tmp/dingo-testrig-10g-shards/testrig-summary.v1.json`. Diagnostic only.

### Multi-core / parallel ingest self-check (2026-07-27)

User observation on M4 during the fixed 10 GiB run: CPU mostly ~50% with peaks
~97%, memory flat, SSD not the limit — “we’re running a single-CPU game; how do
we parallelize and max out all the cores?”

| Claim | Verdict |
|-------|---------|
| Observation correct? | **Yes.** One exclusive writer, one active segment, lifecycle on put ack. Process CPU% ≈ one core. |
| Memory/disk the limiter? | **No** after DEF-095. |
| Multi-thread `Store::put` first? | **No** — thrash without model change. |
| Next high-leverage cut? | DEF-096 Axes A–C **measured end-to-end** (incl. clean multi-store 10 GiB ~17.7k ops/s wall, CPU% sum ~376). Next is product cluster capacity or open gates (Jepsen/fuzz/wire freeze). Sharded 10 GiB ~8.1k ops/s; multi-store 10 GiB disclosed. |
| True multi-core append? | **Axis B shipped** + testrig `--writer-shards N`; peak `ps` CPU still ~1-core class — serial index publish after `put_many` remains a limiter. **Axis C measured:** `--stores 4` 10 GiB with free disk multiplies CPU% and wall ops/s (capacity harness, not product sharding). |
| Spec already requires this? | **Yes** — OVERVIEW parallel ingest, USP sharded writers. |

Design authority: [`PARALLEL_INGEST.md`](PARALLEL_INGEST.md). Do not spend the
next tranche on PrimaryIndex micro-opts or pump rayon against one store.

### “Things can go faster / numbers not ideal” strategy self-check (2026-07-27)

User read after Axis A–C measurement: *things can go faster; numbers still not
ideal; we are making movement. What strategies should we apply?*

**Short answer: Agree. Movement is real and correctly attributed. The remaining
write-path work is one or two surgical efficiency cuts — not another cliff —
and program labor should mostly leave the write path for gate-driven work.**

#### Scoreboard (diagnostic, M4-class, buffered 8 KiB)

| Stage | What changed | Wall ops/s (10 GiB class) | Multi-core? |
|-------|--------------|--------------------------|-------------|
| Pre-DEF-023 | Full index on seal / O(history) derived work | collapsed (~0.13 late/early) | no |
| DEF-023 + DEF-095 | Amortized puts + O(keys) RSS | ~7.4k, RSS ~0.92 GiB | no (~1 core) |
| Axis A | Seal/checkpoint off put ack | enables sustained single-writer | seal workers |
| Axis B (`--writer-shards 4`) | N active segments + `put_many` | ~8.1k (**+~10%**) | **no** (CPU% ~95) |
| Axis C (`--stores 4`, free disk) | N processes | **~17.7k (~2.4×)** | **yes** (CPU% sum ~376) |

“Making movement” = cliffs removed, memory honest, multi-process capacity
proven. “Not ideal” = Axis B does not buy cores yet; Axis C is harness not
product; ~2.4× not ~4×; early windows still much higher than wall averages
(lifecycle / publish pressure late-run).

#### What the numbers teach (strategy principles)

1. **Fix the cliff, then measure, then choose the next bottleneck.** We did not
   guess: write cliff → RSS → single-core → serial index publish after
   `put_many` / multi-process capacity. Keep that loop.
2. **Parallelism only pays when the shared serial section shrinks.** Axis B
   parallelizes appends but still serializes PrimaryIndex publish — wall lift
   stays modest; CPU% stays one-core class. Axis C multiplies whole processes
   (each with its own index) and actually moves wall ops/s + CPU%.
3. **Capacity ≠ efficiency.** Multi-store proves media and cores can be used;
   product multi-tenant / multi-node is still dingo-cluster.
4. **Free disk and path class are part of the experiment.** Near-full volume
   poisoned Axis C 4 GiB; free-disk 10 GiB is the honest capacity number.
5. **Diminishing returns are local, not global.** Write-index micro-opts are
   done; production readiness is not.

#### Strategies to apply (ordered)

| # | Strategy | Do | Do not |
|---|----------|----|--------|
| **S1** | **Declare write cliffs closed** | Treat DEF-023 / DEF-095 / DEF-096 A–C harness as closed cuts with residual only | Reopen PrimaryIndex structure thrash or “one more seal threshold tweak” as the main program |
| **S2** | **One optional Axis B efficiency cut** (only if labor stays on single-node write) | Shrink serial work after `put_many`: sharded or batched index publish, reduce dual-apply tax on batch path, ensure seal pipeline scales with shard count so early-window rates hold late | Rayon over one `Store::put`; multi-thread one active segment |
| **S3** | **Productize capacity on the cluster path** | Independent store/partition leaders (dingo-cluster), honest RF/ack modes, network repair/query only as needed for multi-process proof | Treat testrig `--stores N` as multi-tenant product sharding |
| **S4** | **Gate-driven program labor (default next)** | Milestone A quality (DEF-090–092), multi-process Jepsen/soak (DEF-041), fuzz (DEF-091), wire freeze path (DEF-053), security review (DEF-063) | Polish already-landed cut follow-ons (extra gauges, log join fields, etc.) as if they move maturity |
| **S5** | **Measurement hygiene** | Always disclose concurrency / writer_model / free disk / durability; re-run only when a cut claims a new bottleneck; keep hot/warm/archive claims separate | Publish averages alone; mix path classes; claim “maxed M4” while CPU% ~100 |
| **S6** | **Read path only under dedicated benches** | Wire Hydra/Chimera into get only after body-less / locator-cache design + `read_latency_breakdown` attribution | “Parallelize gets” by loading full `.cmr` (already fixed once) |

#### Recommended labor split (next few tranches)

```text
~70%  gate-driven readiness   (S4)  — Jepsen/fuzz/wire/security/quality bar
~20%  product capacity path   (S3)  — cluster partitions, not more harness
~10%  optional Axis B residual (S2) — only with before/after put_many CPU% + wall ops/s
  0%  PrimaryIndex micro-opts / Chimera-as-hot-get / put authority flip
```

#### Verdict table

| Claim | Verdict |
|-------|---------|
| Things can go faster? | **Yes** — Axis C ~2.4× wall; early windows and seal-off-path show headroom. |
| Numbers ideal? | **No** — Axis B ~+10% wall; single-process still ~1-core; not linear. |
| Making movement? | **Yes** — three real cliffs fixed; multi-core capacity evidence exists. |
| Next main strategy? | **S4 gates by default**; **S2 only if** still investing in single-node write; **S3** for product scale. |
| Stop performance work forever? | **No** — stop **cliff-hunting** without a new measured bottleneck. |

Companion detail: [`PARALLEL_INGEST.md`](PARALLEL_INGEST.md) §10,
[`BENCHMARK_DISCLOSURE.md`](BENCHMARK_DISCLOSURE.md).

### “Is this a big flex?” self-check (2026-07-27)

User question after Hydra + 1 GiB testrig: *dude… this is a big flex isn’t it?*

**Short answer: Yes — a real engineering flex. Keep the claim scoped.**

| Layer | What actually landed | Flex? |
|-------|----------------------|-------|
| Write-path cliff | Steady-state put no longer collapses with retained history; late/early ≳ 0.7 | **Yes** — correctness of scale, not a micro-opt |
| Hydra foundation | Per-segment adaptive compile: Eytzinger / PGM++ / RadixSpline / compressed radix / MPHF; seal-time sidecars; multithreaded `rebuild_hydra_indexes` | **Yes** — research-grade structures as derived, salvage-safe indexes |
| 1 GiB three-prong rig | Pump → monitor → chaos punches → salvage still speaks; sample gets green | **Yes** — integrity + scale ladder, not a one-off demo |
| Product arc Stages 0–9 | Delivery plan complete with honest experimental labels | **Yes** — breadth, not production maturity |

**What is *not* the flex (do not overclaim):**

1. **Hot gets still use `PrimaryIndex`.** Hydra is written at seal and loadable via
   `load_hydra_index` / rebuild APIs; it is **not** yet the `Store::get` path.
   µs-class sample gets on the 1 GiB run are frontier-index / hot-path gets, not
   a published “Hydra beat Redis” claim.
2. **1 GiB numbers are diagnostic only** (see
   [BENCHMARK_DISCLOSURE.md](BENCHMARK_DISCLOSURE.md)). Single machine, buffered
   durability, not an SLO or cross-engine comparison.
3. **Deployment maturity unchanged.** Embedded experimental; network multi-node
   experimental; wire `1.0-draft`. Flex ≠ production-ready.
4. **Open hard gates remain:** multi-process Jepsen/soak (DEF-041 follow-ons),
   wire freeze (DEF-053), security review (DEF-063), continuous fuzz (DEF-091),
   most §16 data-safety checkboxes.
5. **Repo integrity.** Hydra + `dingo-testrig` sources must live in git with the
   glue commits that claim them — a “feat” commit that only touches docs/exports
   is not the flex.

**Plain answer:** Building a durable event-log store that (a) fixed its write
scale cliff, (b) ships adaptive multi-structure per-segment indexes as derived
sidecars, and (c) survives a 1 GiB pump + offline chaos with salvage still
speaking — **that is a big flex**. Marketing it as a production distributed
database or Redis-class product without the open gates would be a fake one.

### Chimera storage (FINAL DESIGN) self-check (2026-07-27)

User extended `INDEXING_STRATEGY_PROPOSAL.md` with Chimera: Hydra locator →
resident / inline / point container / scan extent / large-value log → adaptive
I/O → record-level decompression, plus a background compiler (GC, relocation,
reclustering, dictionary training, hot/cold, lifetime placement).

| Layer | State after seal/compaction wire-up | Claim boundary |
|-------|--------------------------------------|----------------|
| Locator + value classes | `ValueLocator`, `classify_value`, `place_value`, `build_layout` | **Live at seal** for complete live values on that segment |
| Point micro-pages | `PointContainer` in `indexes/chimera/{seg}.cmr` | **Derived sidecar** — authoritative still segment frames |
| Large-value log | `ValueLog` inside `.cmr` layouts | **Derived** — not FORMAT_SPEC chunks; put path unchanged |
| Adaptive I/O | `select_io_path` on resolve | **Policy** — no io_uring submit yet |
| Background compiler | `plan_compile` ops | **Planner** — no worker executes ops yet |
| `Store::get` | **Resident PrimaryIndex first** (hot path) | **Fixed** — never full-load `.cmr` when body is resident |
| `Store::get_via_chimera` | Explicit full-sidecar probe | Diagnostic / future body-less path only |
| Dual representation / ZNS | Design only | Deferred per proposal |

**Verdict:** Seal/compaction layout wire-up **landed**. Put still writes frames;
Chimera is derived placement. An intermediate bug preferred Chimera on every
`get` (~250 ms class testrig samples); hot get is back on PrimaryIndex. Do not
claim “primary storage is workload-compiled” until put classifies at write time,
segment bodies can omit medium/large payloads, and locators are **cached**.

### Decision: implement put-path Chimera / dual-rep·ZNS·worker? (2026-07-27)

Question (two rows often conflated):

| Candidate | Prior note | Decision |
|-----------|------------|----------|
| Put is workload-compiled (no full frame body) | No — put still appends frames; Chimera is derived | **Do not implement yet** |
| Dual-rep / ZNS / compiler worker | Still deferred | **Split:** worker is next Chimera step; dual-rep and ZNS stay deferred |

#### Put is workload-compiled — reasons **not** to (yet)

There **is** a strong reason not to flip put authority now. It is not laziness.

1. **Authority contract (OVERVIEW).** Segment frames are authoritative evidence.
   Indexes and `indexes/chimera/*.cmr` are derived and must remain wipeable.
   “Recovery without derived state” is already a shipped claim
   (`CAPABILITY_MATRIX`, salvage/open after wiping `indexes/` + `catalogs/`).
   If put omits medium/large bodies from frames and only writes Chimera placement,
   `.cmr` becomes irreplaceable data — that **inverts** the product model.

2. **FORMAT_SPEC + crash matrix.** Write-time placement without full frame bodies
   is a format/profile change: dual durable paths, ack rules, interrupted-append
   semantics, scrub/salvage of partial value-log vs frame, and new failpoints.
   That is a major gate, not a Chimera codec PR.

3. **Same pattern as Hydra (deliberate).** Hydra is seal-time derived sidecars;
   hot get still has PrimaryIndex fallback. Chimera matches that honesty bar.
   Seal-time compile + get resolve is the correct intermediate architecture.

4. **Double-write amp is accepted tax.** Put → full frames; seal → re-read live
   values into `.cmr`. Wasteful, but correct and rebuildable. Optimizing amp by
   making put the compiler **before** a relocating worker exists skips the proof
   that recompilation is safe under generation-aware locators.

5. **Program leverage.** Open P0/§16 gates (wire freeze, security review, continuous
   fuzz, multi-process partition evidence) dominate production readiness. A put
   authority flip does not close those gates.

**When put-path compilation becomes right:** (a) a compiler worker has executed
relocate/GC/recluster against derived layouts with generation-safe swaps;
(b) an explicit FORMAT/profile extension defines frame-as-locator / payload-in-layout
with salvage and wipe-derived stories rewritten; (c) crash matrix and durability
modes cover both paths. Until then: **keep put appending full frames.**

#### Dual-rep / ZNS / compiler worker — split the bucket

Do **not** treat these three as one “still deferred” blob.

| Piece | Implement now? | Why |
|-------|----------------|-----|
| **Compiler worker** | **Yes — next Chimera cut** (when labor stays on Chimera) | `plan_compile` already emits ops; nothing executes them. Without a worker, layouts freeze at seal/compact and never recompile. Start with Relocate / Gc / Recluster / HotColdMigrate against `.cmr` only; frames stay authority. Not more planner cosmetics. |
| **Dual representation** | **No — keep deferred** | Proposal §218: prove inline / value-log / micro-pages + temperature GC first. Needs real point_gets/scan_hits telemetry, space-amp policy, and op-type Hydra choice. `CompilerOp::DualRepresent` + `enable_dual_representation` already stubbed off by default. |
| **ZNS (or FDP) placement** | **No — keep deferred** | Hardware- and prototype-specific; not the portable default. Lifetime/temperature *logical* zones on ordinary NVMe come first; bind to ZNS only with device-backed evidence. |

**Net:** Do **not** implement write-time put compilation or dual-rep/ZNS now.
**Do** allow a compiler **worker** (execute plans on derived Chimera layouts) as
the next Chimera step when that lane is prioritized — separate from dual-rep/ZNS.

### Next steps towards maximum performance (2026-07-27 self-check)

**Question:** What are our next steps towards *maximum performance*?

**Short answer:** The write cliffs are closed. Maximum performance is three
path classes (OVERVIEW §12.1), not one knob. Next work is **ordered residuals
per class**, not another PrimaryIndex rewrite. Default program labor remains
gate-driven readiness; performance labor must attack a **measured** bottleneck
with before/after disclosure.

#### Where we already are (do not re-hunt)

| Cliff / cut | Status | Evidence |
|-------------|--------|----------|
| Write index asymptote | **Closed** (DEF-023) | late/early ≳ 0.7; ~µs steady-state puts |
| O(dataset) RSS | **Closed** (DEF-095) | 10 GiB pump peak RSS ~0.92 GiB |
| Lifecycle on put ack | **Closed** (DEF-096 Axis A) | seal pipeline + pending rotate |
| Sharded append layout | **Closed** (Axis B) | `create_with_shards` / `put_many` |
| Multi-process capacity harness | **Closed** (Axis C) | 4×stores 10 GiB ~17.7k ops/s, CPU% sum ~376 |
| Product multi-partition batch | **Started** | `Cluster::put_many` + `stage_def_096_product_capacity` |
| Hot get integrity | **Fixed** | PrimaryIndex first; no full `.cmr` on get |

Scoreboard (buffered 8 KiB, M4-class, diagnostic only — full tables in
[`BENCHMARK_DISCLOSURE.md`](BENCHMARK_DISCLOSURE.md)):

| Config | Wall ops/s (10 GiB class) | Multi-core? | Residual |
|--------|---------------------------|-------------|----------|
| Single | ~7.4k | no (~1 core) | — |
| Axis B shards=4 | ~8.1k (~+10%) | **no** (CPU% ~95) | serial index publish after `put_many` |
| Axis C stores=4 | ~17.7k (~2.4×) | **yes** (CPU% sum ~376) | harness ≠ product; not linear 4× |

#### Maximum performance = three destinations

| Class | Product target | Current ceiling | Next step toward max |
|-------|----------------|-----------------|----------------------|
| **Ingest** | Sustained multi-core firehose; durable/replicated modes disclosed | Single-process still ~1-core wall; Axis C proves media headroom | **P1 residual:** shrink serial PrimaryIndex publish on `put_many` (S2). **P1 product:** network multi-partition / multi-node capacity (S3). **P2:** durable + replicated append benches (DEF-093). |
| **Hot path** | Memory-store class p50/p99 on resident working set | µs-class gets via PrimaryIndex; Hydra/Chimera **not** on `Store::get` | **P1:** body-less sealed-segment get via **cached** Hydra locator + frame pread (read benches only). **P2:** Chimera compiler **worker** (relocate/GC/recluster on derived `.cmr` only). **Never:** full `.cmr` load per get. |
| **Archive** | Catalog prune + parallel stream; high latency OK | Scaffold / mirror tier; separate class | Only after hot/ingest product paths are real; never mix archive latency into hot claims. |

No single number is “maximum performance.” Claims must name the class and full
disclosure fields.

#### Ordered next steps (performance lane only)

Execute only when labor is intentionally on performance — otherwise prefer S4
gates from the earlier strategy self-check.

| Priority | Step | Why | Acceptance signal | Anti-goal |
|----------|------|-----|-------------------|-----------|
| **1** | **Axis B residual — serial index publish** | Measured: appends parallelize; `apply_durable_event` loop after `put_many` is serial → wall ~+10%, CPU% stays 1-core | Before/after on same machine: wall ops/s **and** process CPU% rise together at shards≥4; late/early still ≳ 0.7; crash matrix green | More shards with same serial publish; PrimaryIndex structure thrash |
| **2** | **Product capacity path (cluster)** | Axis C multiplies whole processes; product scale is independent partition leaders + network serve-cluster, not testrig `--stores N` | Multi-process / multi-node pump with honest RF/ack; `Cluster::put_many` beyond in-process; disclosure of concurrency + topology | Treating harness multi-store as multi-tenant product |
| **3** | **Durable / replicated ingest disclosure** | Buffered is the diagnostic default; “max performance” without durability modes is incomplete (DEF-093) | `write_latency_breakdown` + testrig runs for `durable` (and replicated when network path exists); p50/p95/p99 + fsync amp | Claiming durable ≈ buffered |
| **4** | **Hot sealed-segment reads (Hydra wire-up)** | Hydra sidecars already compile at seal; get still frontier-index only | Dedicated probe: open-once get via cached Hydra → frame pread; `read_latency_breakdown` attributes Hydra path; no `.cmr` full load | “Parallelize gets” by loading Chimera containers |
| **5** | **Chimera compiler worker** | Layouts freeze at seal without execution; worker unlocks recompilation amp wins later | Worker executes Relocate/Gc/Recluster/HotColdMigrate on derived layouts; frames stay authority; generation-safe swaps | Put-path authority flip; dual-rep/ZNS |
| **6** | **DEF-093 reproducible suite** | Maximum performance is not credible without published profiles | README links to commands + raw results; CI catastrophic regression guard; path classes separated | Marketing averages without disclosure |
| **7** | **Archive path (later)** | Separate performance class | Only after Milestone B/C maturity; archive bench already skeletoned (`stage9_archive_bench`) | Cold retrieval under hot SLOs |

#### Concrete code residual for step 1 (Axis B)

Today in `Store::put_many_parallel` (`dingo-store` `store.rs`):

1. Prepare envelopes (serial, needs `&mut self`).
2. Parallel shard appends (`thread::scope`) — **already multi-core capable**.
3. Serial loop: `apply_durable_event` + `note_collection_for_subject` for every item.

**Surgical targets (only with measurement):**

- Batch / sharded PrimaryIndex publish after append (reduce dual-apply tax on the batch path if durable_index apply dominates).
- Avoid redundant body clones on locator-first entries when the batch already owns the body.
- Ensure seal-pipeline worker count scales with `writer_shards` so late-run early-window rates hold (lifecycle pressure, not index asymptote).

Do **not** flip put authority to Chimera or multi-thread one active segment.

#### Labor split for “maximum performance” program (next few tranches)

```text
If program default (readiness):     ~0–10% perf residual; rest gates (Jepsen/fuzz/wire/security)
If intentionally maxing ingest:     ~50% step 1 (serial publish) + ~40% step 2 (cluster) + ~10% measure
If intentionally maxing hot reads:  ~70% step 4 (Hydra get) + ~20% step 5 (Chimera worker) + ~10% measure
Never:                              PrimaryIndex micro-rewrite / Chimera-on-every-get / rayon-on-single-put
```

#### Verdict table

| Claim | Verdict |
|-------|---------|
| Next steps exist toward max performance? | **Yes** — ordered residuals above. |
| Are write cliffs the main story? | **No** — closed; residual is efficiency + product scale + read path. |
| Single biggest single-node write lever left? | **Serial index publish after `put_many`** (step 1). |
| Single biggest capacity lever left? | **Product multi-partition / multi-node** (step 2), not more harness stores. |
| Single biggest hot-read lever left? | **Cached Hydra → frame pread** (step 4), not Chimera full sidecar. |
| Should we stop all performance work? | **No** — stop **unmeasured** cliff-hunting and path-class mixing. |
| Does max performance require production gates? | **Yes for product claims** — network maturity, wire freeze, security, DEF-093. Diagnostic micro-opts can proceed without them but must stay labeled. |

Companions: [`PARALLEL_INGEST.md`](PARALLEL_INGEST.md) §10, [`BENCHMARK_DISCLOSURE.md`](BENCHMARK_DISCLOSURE.md), DEF-023 / DEF-095 / DEF-096 / DEF-093 in [`DEFECTS.md`](../DEFECTS.md).

**Canonical write-up of all strategies (S1–S6, max-performance residuals, SDA
A1–A5):** [`PERFORMANCE_STRATEGIES.md`](PERFORMANCE_STRATEGIES.md).

### SDA performance harness (2026-07-27)

Started: pure-SDA path is a separate performance class from store ingest/hot-get.

| Hook | Role |
|------|------|
| `dingo-sda` `Program::parse` + `run_json` / `eval` | Compile-once host API |
| `dingo-sda` example `sda_latency_breakdown` | Phase attribution (diagnostic) |
| `dingo-sda` `sda_bench_skeleton` | CI absurdity bounds |
| SDK `Filter::compile_sda` / `matches_compiled_sda` | Multi-doc filter parity without re-parse |

Strategies: [`PERFORMANCE_STRATEGIES.md`](PERFORMANCE_STRATEGIES.md) § SDA.
Default collection find remains native `Filter::matches` (A3).

## What this document is not

- Not a commitment to a schedule.
- Not permission to reopen remediated DEF cuts without new evidence of failure.
- Not a claim that experimental network cluster is production-ready.

When the next self-check runs, update the date and re-score §16 checkboxes in
DEFECTS.md against current main.
