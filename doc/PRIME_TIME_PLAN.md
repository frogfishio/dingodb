# DingoDB prime-time plan

Status: living strategy snapshot (captured 2026-07-28)  
Audience: product + engineering program operators (humans + agents)  
Companions: [DEFECTS.md](../DEFECTS.md) (execution defects + §16 gates),
[WORK_HORIZON.md](WORK_HORIZON.md) (are we polishing?),
[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md) (what is honest today),
[DELIVERY_PLAN.md](../DELIVERY_PLAN.md) (Stages 0–9 — **done**),
[PERFORMANCE_STRATEGIES.md](PERFORMANCE_STRATEGIES.md) (write/read path only)

## 1. Purpose

Stages 0–9 are done. Doctrine is coherent. The repo is unusually honest about
maturity. **Prime time is not one more DEF cut** — it is choosing **which product
surface** we will stand behind, then closing the gates that make that claim true.

This document freezes the post-check-in strategic assessment so we can resume
without re-deriving it from chat. It does **not** replace DEFECTS (tactical work
list) or CAPABILITY_MATRIX (current labels). It decides **order and wedge**.

## 2. Where we actually are

| Surface | Reality today |
|--------|----------------|
| **Embedded single-node** | Strongest path: collections, filters, history, salvage, backup/scrub/migrate, resource budgets. Still **experimental / early-access**, not a production guarantee. |
| **Single-node TCP** | Real serve path with TLS, authz, admission, health/metrics/logs — labeled **development only**. |
| **Cluster** | Deep in-process model + experimental multi-process Raft. **Not** production multi-node storage. |
| **Archive / tiers / EC** | Doctrine is strong; implementation is **scaffold / FS mirror**. |
| **Wire** | `1.0-draft` — not frozen. |
| **Performance** | Real cliffs fixed (write path, RSS, parallel ingest harness). Not a published Redis-class product. |

We have built an **architecture and correctness program** that many “v1”
databases never write down. We have **not** yet shipped a product that a stranger
can put under a real business without reading DEFECTS.

## 3. What is already unusually good (do not dilute)

1. **Doctrine + honesty** — salvage vs export, holes vs silence, maturity labels,
   capability matrix. Trust is half the product.
2. **Authoritative vs derived** — frames survive; indexes/catalogs rebuild. That
   is the long-term differentiator.
3. **Embedded DX skeleton** — `open → put → get → find → history → doctor` is
   close to the everyday promise.
4. **Ops foundation for a server** — config profile, NDJSON logs, health/metrics,
   backup/scrub/migrate as real profiles, not vapor.
5. **SDA / DQL / ENR** — a serious examination story, not a bolted-on query toy.
6. **Engineering discipline** — crash matrix, testrig, fuzz hooks, release
   packaging discipline. Rare at this age.

If prime time means “people believe the README,” we are ahead of most early
systems. If it means “people run it on money,” we are not there yet.

## 4. What “prime time” means (pick a wedge)

Trying to go prime-time on **all four** promises at once (embedded + network +
cluster + multi-decade archive) will keep us experimental forever.

### Recommended first wedge

> **Embedded single-node as early-access production for irreplaceable local data**  
> (side projects → serious apps that must not lose files)

That matches the tagline, the strongest code path, and the USP’s everyday
promise. Network and cluster become **versioned maturity upgrades**, not
co-launch requirements.

### Later wedges (not launch blockers for the first story)

| Order | Wedge | When |
|------:|-------|------|
| 2 | Single-node `dingo serve` for small teams | After security review + wire freeze path + soak evidence |
| 3 | Cluster GA (Milestone C) | After multi-process Jepsen/soak and network repair/query |
| 4 | Archive / native object store (Milestone D) | Its own launch after B/C honesty |

## 5. What’s missing — ordered by leverage

### Tier 0 — Without these, do not say “production” for any network surface

| Gap | Why it blocks prime time |
|-----|---------------------------|
| **Multi-process / Jepsen-style evidence (DEF-041 follow-ons)** | In-process sim ≠ network truth. Until this exists, `serve-cluster` must stay experimental. |
| **Wire freeze qualification (DEF-053)** | Long-lived data without a frozen major is a trap: users store for years on a draft. |
| **Independent security review + disclosure (DEF-063)** | Threat model first cut is good; production needs external eyes + a way to report holes. |
| **Continuous fuzz / broader hostile surfaces (DEF-091)** | Format fuzz exists; parsers/RPC/SDA/control meta need continuous service. Damage-tolerant systems are high-value attack surfaces. |
| **§16 data-safety checkboxes** | Our own release bar; most still open. Closing DEFs without §16 is unfinished business. |

These are **trust gates**, not features.

### Tier 1 — Embedded early-access prime time (the right first product)

| Gap | Why it matters for real users |
|-----|-------------------------------|
| **Crash-consistency residual (DEF-022 buffered power-loss, etc.)** | Embedded users kill processes and pull plugs. Brand is survival; this must be boring. |
| **Quality + coverage bar (DEF-090 done-ish; DEF-092 models/coverage)** | Early-access needs CI that fails closed and evidence we can point at. |
| **SDK MVP completeness (DEF-080)** | Streaming, errors, receipts, docs that match reality — one coherent Rust story. |
| **Operator journeys as scripts, not prose (DEF-082)** | “Install → put → crash → doctor → salvage → restore” as one-command demos people can run. |
| **Compatibility / deprecation policy (DEF-084)** | Even embedded needs “what we break when.” |
| **Distribution that non-contributors use (DEF-083)** | crates.io is started; also versioned binaries, install docs, MSRV story, signed artifacts later. |
| **Disclosed benchmarks (DEF-093)** | Never claim memory-store class without reproducible numbers and durability mode named. |

**Prime-time embedded test:** a careful stranger can put important JSON in a
file, survive crash/kill, inspect holes, restore from backup, and never need to
understand Raft.

### Tier 2 — Single-node server as a real product

| Gap | Why |
|-----|-----|
| **Remote bounded cursors / page RPC** | Server without honest large scans will OOM or lie. |
| **Concurrent reads without global store mutex** | Throughput and multi-client reality. |
| **Auth model beyond shared-token superuser** | Real multi-user needs durable principals/ACLs, not only RPS admission. |
| **Config live-reload + ops runbooks** | Production ops expect reload/audit trails, not only validate-before-serve. |
| **Incident / release process (DEF-094)** | Who cuts releases; what happens when salvage finds a bug in the format. |

### Tier 3 — Cluster as a product (after Tier 0)

| Gap | Why |
|-----|-----|
| **Network anti-entropy + query paging over the wire** | In-process repair/paging does not operate a fleet. |
| **Cluster-coordinated backup** | Single-node backup is not HA ops. |
| **Long soak / rolling restart** | The only way “experimental Raft” becomes “we trust RF=3.” |
| **Product multi-partition capacity path** | Axis C harness ≠ multi-tenant product; `Cluster::put_many` is the start, not the finish. |

### Tier 4 — Archive / multi-decade promise (the USP’s long tail)

| Gap | Why |
|-----|-----|
| **Native S3/GCS (not FS mirror)** | Without this, “massive retention” is a local-disk story. |
| **Lifecycle scheduler, erasure coding, encryption/KMS** | Scaffolds today; doctrine without media reality. |
| **Multi-decade retention proof (DEF-074)** | Format stability, media refresh, migration drills — evidence, not essays. |

Ship this **after** embedded (and preferably server) is honest. Expanding
EC/lifecycle now is the classic diluting move.

### Tier 5 — Product surface gaps people will feel immediately

These are not always P0 in DEFECTS, but they decide adoption:

1. **Language surface area** — Rust-only is fine for v0.2; broader prime time
   needs at least one **stable FFI/HTTP client story** or a clear “Rust first,
   others later” positioning so people don’t bounce.
2. **Watches / change feeds** — DX progressive disclosure mentions watches; many
   apps need “notify me when this key changes.”
3. **Unique constraints / multi-doc transactions** — TRANSACTIONS.md is proposal;
   either ship a **scoped** transaction MVP or keep marketing strictly
   single-key/receipt-level.
4. **Query ergonomics** — DQL/SDA are powerful; everyday `Filter` must stay the
   default path so SDA stays progressive disclosure, not homework.
5. **Hydra/Chimera as product, not research** — impressive engineering; hot path
   still PrimaryIndex. Don’t market adaptive indexes until they own `get` with
   rebuildable authority intact.
6. **Ecosystem gravity** — migrations from SQLite/Rocks/JSON files, backup into
   object storage, observability into Prometheus/OTel (metrics exist;
   packaging/dashboards/tracing still thin).
7. **Narrative compression** — specs are excellent for implementers; a **one-page
   “when to use / not use” + 15-minute tutorial** is what converts. Pieces exist;
   the funnel is still engineer-to-engineer.

## 6. What to *stop* treating as the main program

From [WORK_HORIZON.md](WORK_HORIZON.md) — still correct:

- Extra metrics series, log join fields, dynamic config polish
- Unique indexes without partition design
- Put-path Chimera authority flip / dual-rep / ZNS
- PrimaryIndex micro-opts
- Growing network cluster features without multi-process proof

Those make an experimental system prettier. They do not create a prime-time
product.

## 7. Path to prime time (practical sequence)

```text
1. Declare wedge: Embedded early-access for irreplaceable local data
2. Close embedded trust: crash residual, SDK MVP, executable journeys, compat policy
3. Freeze path: continuous fuzz + wire major-1 qualification criteria (even if freeze is later)
4. Security: independent review + disclosure before any non-loopback default story
5. Only then: single-node serve as “supported development → production candidate”
6. Only then: multi-process Jepsen/soak → cluster experimental → cluster GA
7. Archive product (native object store + EC + lifecycle) as its own launch
```

### Labor split (next phase of the program, not one PR)

| Share | Focus |
|------:|--------|
| ~50% | Trust gates (crash, fuzz, wire path, security, §16) |
| ~25% | Embedded product completeness (SDK, CLI journeys, distribution, docs funnel) |
| ~15% | Server honesty (cursors, concurrency, multi-user auth depth) |
| ~10% | Measured performance residuals *or* cluster capacity — only with a named bottleneck |

Avoid 50% cluster + 50% archive while embedded is still “experimental.”

Maps to DEFECTS milestones: **A** (embedded) → **B** (server) → **C** (cluster)
→ **D** (archive). Do not reorder for novelty.

## 8. Definition of done — embedded early-access only

This is the first claim we are allowed to defend. **Pass all must-haves before
raising CAPABILITY_MATRIX / README maturity for embedded beyond experimental.**

### Must-pass (embedded)

- [ ] Stranger journey: install crate/binary → put JSON → kill process → reopen →
      get same data (scripted; no Raft knowledge).
- [ ] Crash / kill residual: documented durability modes; buffered power-loss
      story honest; crash matrix residual for create/append/delete/seal/checkpoint
      still green (DEF-022 and friends).
- [ ] Salvage honesty: holes, partials, conflicts reportable; ordinary get never
      invents absence from incomplete coverage.
- [ ] Wipe-derived recovery: open after deleting `indexes/` + `catalogs/` still
      recovers authority (already claimed — keep proof green).
- [ ] Backup → restore from released artifact path (DEF-050 profile exercised).
- [ ] Scrub finds integrity issues without silent data invent (DEF-051 profile).
- [ ] SDK MVP: put/get/delete/find/history/errors/receipts documented and tested
      (DEF-080).
- [ ] Executable demos: at least one scripted “break and recover” journey
      (DEF-082 / `scripts/demos`).
- [ ] Compatibility policy draft: what breaks at 0.x, what is freeze-labeled
      (DEF-084).
- [ ] Quality bar: fmt/clippy/deny/doc/package + crash/testrig evidence linked
      from README (DEF-090–092 core).
- [ ] Performance: no Redis-class or “max cores” claims without
      [BENCHMARK_DISCLOSURE.md](BENCHMARK_DISCLOSURE.md).

### Forbidden claims (until later milestones)

- Do **not** call network `serve-cluster` production (needs Tier 0 + Milestone C).
- Do **not** call single-node TCP production until Milestone B (security review,
  wire path, soak, multi-user honesty).
- Do **not** claim native S3/GCS, erasure coding, or multi-decade retention as
  product (scaffold/mirror only — Milestone D).
- Do **not** market Hydra/Chimera as the hot get path until they own `get` with
  authority still wipeable.
- Do **not** imply multi-doc transactions or unique global indexes unless shipped
  and scoped in docs.

### Exit label when this section is green

**Recommended README / matrix language:**

> Embedded single-node: **early-access** — suitable for serious local apps that
> accept 0.x compatibility risk; not a multi-node or long-retention platform.

Not “production database.” Early-access with honest limits *is* the first
prime-time win.

## 9. Competitive / category honesty

Category: **damage-tolerant universal store** with ordinary DX. Competitors own
pieces (SQLite ease, Rocks speed, S3 scale, Postgres transactions). We win if:

- starting is as easy as SQLite,
- surviving damage is better than almost everything,
- examination (SDA) works on what salvage returns,
- and we **never** claim cluster/archive maturity we haven’t proved.

The risk is not “missing a feature.” The risk is **over-building distributed and
archive surfaces while the first trustable product (embedded file that refuses
to die) is still labeled experimental.**

## 10. Bottom line

**We are missing fewer “ideas” than “commitments.”**

The architecture thesis is clear. The staged product is largely *built*. What is
missing for prime time:

1. **A single supported product wedge** (embedded first).
2. **Trust evidence** (crash, fuzz, wire freeze path, security review, §16).
3. **Stranger-ready packaging and journeys** (install → use → break → recover
   without reading DEFECTS).
4. **Deliberate deferral** of cluster-as-product and archive-as-product until that
   wedge is defensible.
5. **Adoption surfaces** (language/clients, watches, scoped tx or honest limits,
   non-Rust path later).

WORK_HORIZON and CAPABILITY_MATRIX already say this. The discipline now is
**gate-driven productization**, not another brilliant subsystem.

## 11. How to use this document

| Need | Open |
|------|------|
| What to build next (tactical IDs) | [DEFECTS.md](../DEFECTS.md) |
| Whether current labor is polish thrash | [WORK_HORIZON.md](WORK_HORIZON.md) |
| What we may claim today | [CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md) |
| Performance residual order only | [PERFORMANCE_STRATEGIES.md](PERFORMANCE_STRATEGIES.md) |
| **Wedge, sequence, labor split, forbidden claims** | **This file** |

When the embedded DoD checkboxes flip, update the date, CAPABILITY_MATRIX labels,
and README maturity in the same change. Do not raise maturity labels from chat
alone.

---

*Origin: objective product assessment after Stages 0–9 check-in (2026-07-28).
Kept in-tree so the program can resume without re-deriving strategy from session
history.*
