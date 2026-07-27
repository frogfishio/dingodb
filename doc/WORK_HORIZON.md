# Work horizon self-check

Status: snapshot as of 2026-07-27  
Audience: operators of the engineering program (humans + agents)  
Companion: [DELIVERY_PLAN.md](../DELIVERY_PLAN.md), [DEFECTS.md](../DEFECTS.md),
[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md)

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
| **DEF-063** | Threat model + independent security review (P0). |
| **DEF-091** | Continuous fuzz of every untrusted parser (P0). |
| **§16.1–16.3** | Most data-safety and distributed gates remain unchecked. |

### Product completeness (real surfaces, not nits)

| ID | Gap |
|----|-----|
| **DEF-039 / 040 network** | Anti-entropy repair and query paging still in-process only. |
| **DEF-050–052 follow-ons** | Incremental/encrypted backup; cluster-coordinated backup; scrub daemon; second wire major + rolling upgrade drills. |
| **DEF-070–074** | Native object stores, lifecycle scheduler, erasure coding, encryption, multi-decade retention proof — archive product, currently scaffold/mirror. |
| **DEF-080–084** | SDK MVP gaps, operator CLI depth, executable journeys, distribution, compatibility policy. |

### Engineering quality bar (release blockers, not vanity)

| ID | Gap |
|----|-----|
| **DEF-090** | CI quality bar (fmt/clippy/deny/doc/package) — partially landing (`scripts/quality.sh`, `deny.toml` may still be uncommitted). |
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
| `Store::get` | Prefers Chimera layout when present for live segment_id | **Wired** — fallback PrimaryIndex / chunk reassembly |
| Dual representation / ZNS | Design only | Deferred per proposal |

**Verdict:** Seal/compaction layout wire-up **landed**. Put still writes frames;
Chimera is a derived placement that get can resolve. Do not claim “primary
storage is workload-compiled” until put classifies at write time and segment
bodies can omit medium/large payloads. Next: optional put-path placement and
compiler worker — not more planner cosmetics.

## What this document is not

- Not a commitment to a schedule.
- Not permission to reopen remediated DEF cuts without new evidence of failure.
- Not a claim that experimental network cluster is production-ready.

When the next self-check runs, update the date and re-score §16 checkboxes in
DEFECTS.md against current main.
