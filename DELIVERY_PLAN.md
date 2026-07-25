# DingoDB staged delivery plan

Status: Draft v0.19 (Stages 0–7 done; Stage 8a–8d cluster foundation + Raft + convergent-append + SDK routing; 8e+ open)  
Audience: implementers  
Depends on: [SDA_SPEC.md](SDA_SPEC.md), [SDA_PROFILE.md](SDA_PROFILE.md),
[FORMAT_SPEC.md](FORMAT_SPEC.md), [OVERVIEW.md](OVERVIEW.md),
[DX_SPEC.md](DX_SPEC.md), [CLUSTER_SPEC.md](CLUSTER_SPEC.md),
[USP.md](USP.md), [ARCHITECTURE.md](ARCHITECTURE.md)

## 1. Purpose

Specs are ahead of code. SDA is already specified; the rest of DingoDB is
specified as architecture, wire format, DX, and clustering.

This plan answers: **how do we ship DingoDB in stages** so each stage is
demoable, testable, and aligned with the product thesis, without waiting for
the full system.

The governing product rule stays:

> Put anything in. Keep it at scale. Damage it. Find what survived.

## 2. Delivery principles

1. **Spec before behavior.** Each stage implements a named slice of an existing
   normative document. New behavior needs a spec amendment first.
2. **Vertical slices over horizontal layers.** Prefer “one path that works end
   to end” over building every subsystem half-complete.
3. **Conformance tests are the gate.** A stage is done when its conformance
   suite (and for storage stages, destructive tests) pass—not when APIs exist.
4. **SDA is pure and early.** SDA has no IO dependency. Ship and lock it
   before coupling it to storage.
5. **Ordinary DX before deep machinery.** Everyday put/get/filter must feel
   boring before salvage, clusters, and tiering dominate the surface.
6. **Damage early, optimize later.** Island recovery and hole reporting land
   before Redis-class hot-path tuning claims.
7. **Derived state is never the only map.** Catalogs and indexes may lag
   authority; salvage must not depend on them.
8. **Language and packaging TBD.** Stages assume a host runtime and one primary
   SDK. Concrete crates/packages wait for the codebase.

## 3. Spec → stage map

| Spec | Primary stages |
|------|----------------|
| [SDA_SPEC.md](SDA_SPEC.md) | 0–1 |
| [FORMAT_SPEC.md](FORMAT_SPEC.md) | 2–3 |
| [OVERVIEW.md](OVERVIEW.md) storage + recovery | 2–4, 6 |
| [SDA_PROFILE.md](SDA_PROFILE.md) | 5 |
| [DX_SPEC.md](DX_SPEC.md) | 4–7 |
| [CLUSTER_SPEC.md](CLUSTER_SPEC.md) | 8+ |
| [USP.md](USP.md) | product framing for all stages |

## 4. Stage overview

```text
0  Repo + CI harness
1  SDA standalone (pure)
2  Wire format + salvage scanner
3  Single-node store (append journal + sealed segments)
4  Collection SDK (embedded put/get/delete/filter)
5  SDA examination profile over recovered units
6  Indexes, catalogs, history, chunked payloads
7  CLI (doctor, salvage) + server mode
8  Cluster (partition-local consensus, coverage)  — 8a foundation done
9  Tiering, archive path, long-retention polish
```

Stages 0–4 are the **minimum path to a useful embedded database**.  
Stages 5–7 complete the **README initial implementation target**.  
Stages 8–9 are **scale-out and retention** after the single-node product is real.
Stage **8a** (`dingo-cluster`) is the first scale-out vertical slice.

---

## Stage 0 — Repository and engineering harness

**Goal:** Make the empty or incoming codebase a place where later stages can
land without thrash.

**Deliverables**

- Repo layout (core library, SDA, format, store, SDK, CLI, tests).
- Build, format, lint, unit-test CI on every PR.
- Golden-test and property-test harnesses ready for conformance corpora.
- Documented decision: primary implementation language + first SDK language
  (DX examples are TypeScript-like; implementation need not be TS).
- Version and feature flags for draft wire formats (FORMAT_SPEC is not frozen
  until wire major 1).

**Exit criteria**

- `main` builds green on CI with a trivial smoke test.
- CONTRIBUTING / architecture map points at the normative specs.
- No product claims beyond “spec phase + scaffold.”

**Risks**

- Over-scaffolding packages for cluster/server before Stage 3 exists.

**Do not do yet**

- Cluster, network protocol, object-store backends.

---

## Stage 1 — SDA standalone engine

**Goal:** A pure, conformance-tested SDA implementation with no storage IO.

**Normative scope:** [SDA_SPEC.md](SDA_SPEC.md) core + standalone profile,
especially §1–§12 and §14.

**Deliverables**

1. Value model: `Null`, numbers, strings, bytes, `None`/`Some`, `Ok`/`Fail`,
   `Seq`/`Set`/`Bag`/`Map`/`Prod`/`BagKV`/`Bind`.
2. Lex/parse for the standalone surface (ASCII + Unicode operator spellings).
3. Static validity where required; evaluation with stable error tags (§12).
4. Three eliminators (`?`, `!`, and the strict form), normalization, carrier
   operators, comprehensions, pipe (`|>` with `_` / `•`).
5. Standalone helpers (`type`, `keys`, `values`, `count`, …) exactly as §11.2.
6. JSON bridge: host tree ↔ SDA values for CLI and tests.
7. Conformance corpus from §14.1 (and expanded golden vectors checked into
   `tests/sda/`).

**Exit criteria**

- All §14 MUST items for standalone conformance pass.
- Minimal suite in §14.1 fully automated.
- Determinism: same program + input ⇒ same value or stable `Fail`.
- Public surface: library API + optional `sda` CLI (`eval`, `check`).
- **No** DingoDB types, segments, or host IO inside the SDA core.

**Why first**

- SDA is already fully written and independent.
- Later query filters compile to SDA; examination is SDA over host-built
  values. Locking semantics early prevents storage APIs from inventing a second
  expression language.

**Suggested sub-milestones**

| 1a | Values, equality, literals, parse of closed expressions |
| 1b | Eliminators + absence/`Null` laws |
| 1c | Normalization + Set/Bag/Seq ops |
| 1d | Comprehensions + carrier preservation |
| 1e | Pipe + helpers + JSON bridge + full §14 suite |

---

## Stage 2 — Survival wire format and salvage scanner

**Goal:** Encode, decode, and rediscover frames without trusting catalogs.

**Normative scope:** [FORMAT_SPEC.md](FORMAT_SPEC.md) (frames, segments,
scanner, §13 wire tests); [OVERVIEW.md](OVERVIEW.md) §§4, 6.

**Deliverables**

1. Frame encode/decode with independent header and body integrity.
2. Segment header/trailer (or equivalent self-description) and sealing.
3. Forward salvage scanner with resync after garbage, truncations, and corrupt
   lengths.
4. Hole construction (explicit discontinuities; no silent join across damage).
5. Diagnostic projection of frames/holes suitable for later SDA examination.
6. Destructive test corpus from FORMAT_SPEC §13 and OVERVIEW §16 items that
   apply to offline segments.

**Exit criteria**

- Every FORMAT_SPEC §13 case automated.
- Later intact frames remain discoverable after earlier corruption.
- Corrupt candidates never labeled `verified`.
- Scan works with segment descriptor/summary deleted.
- Draft wire version tagged as draft until major 1 freeze.

**Why before the “database API”**

- Independent survival is the product differentiator. If salvage is bolted on
  after a normal key-value store, framing will be wrong.

**Suggested sub-milestones**

| 2a | Frame codec + unit integrity tests | **done** |
| 2b | Active append segment + seal | **done** |
| 2c | Forward scanner + hole reports | **done** |
| 2d | Full destructive corpus (§13) | **done** — `tests/section13_corpus.rs`; reverse scan; event conflicts; draft chunk maps |

---

## Stage 3 — Single-node authoritative store

**Goal:** Append-only store of items/events with durability modes and
catalog-independent recovery.

**Normative scope:** OVERVIEW §§5–7 (model, layout, write path); FORMAT_SPEC
chunks as needed for inline-only first.

**Deliverables**

1. Store open/create on a filesystem path (zero ceremony directory layout).
2. Event kinds needed for MVP: at least `put`, `delete`; stub or defer
   `link` / `checkpoint` / `repair` / `purge` unless required by tests.
3. Append path: encode frame → append → publish visibility by durability mode
   (`memory`, `buffered`, `durable` minimum; `replicated` later).
4. Logical open path: subject/key → current event via rebuildable index **or**
   segment scan.
5. Delete of catalogs/indexes still allows full salvage scan of segments.
6. Interrupted append: incomplete tail does not poison earlier frames.

**Exit criteria**

- Put/get/delete at the store layer (may be internal API, not full SDK yet).
- Destroy catalogs/indexes → salvage reconstructs surviving items.
- Durability mode on every ack; docs state failure boundary per mode.
- OVERVIEW §16 cases 1–10 applicable to single-node segments pass.

**Suggested sub-milestones**

| 3a | Open/create, put/get/delete, durability modes, rebuildable index, catalog wipe salvage | **done** — `dingo-store` |
| 3b | Broader OVERVIEW §16 store-level suite (middle segment loss, reorder, etc.) | **done** — `tests/section16_store.rs` (cases 1–10) |
| 3c | Store descriptor frame + optional index cache on disk | **done** — `store-info/descriptor.dingo` + `indexes/primary.idx` |

**Explicit non-goals for Stage 3**

- Secondary indexes, query language, network, multi-writer consensus.
- Cold object storage tiers.

---

## Stage 4 — Embedded collection SDK (ordinary DX)

**Goal:** The boring happy path from [DX_SPEC.md](DX_SPEC.md) and README:

```ts
const db = await Dingo.open("./app.dingo");
await db.collection("users").put("user-42", { name: "Alice" });
```

**Normative scope:** DX_SPEC §§1–7 (journeys 1–3, 6 partial), progressive
disclosure layers 1–2.

**Deliverables**

1. `Dingo.open(path)` create-or-open with safe defaults.
2. Collections: `put`, `get`, `delete`, optional `append` for streams.
3. JSON and raw bytes as first-class payloads.
4. Simple filters without requiring callers to write SDA (builder or object
   filter → internal plan; may compile to SDA under the hood).
5. Streaming iteration for results larger than memory (bounded materialize).
6. Typed errors for missing key vs damaged vs incomplete (no silent empty
   success when coverage is broken).
7. Write receipts reporting actual durability mode.

**Exit criteria**

- DX journeys: open + store JSON in under one minute; bytes round-trip; common
  JSON field filter without learning SDA; stream larger-than-memory dataset.
- README-style sample works against a real store from Stage 3.
- No requirement that app developers understand frames/segments.

**Suggested sub-milestones**

| 4a | Open + put/get/delete JSON | **done** — `dingo-sdk` (`Dingo::open`, `collection`, JSON put/get/delete) |
| 4b | Bytes + streaming scan of collection | **done** — `put_bytes`/`get_bytes`, `scan_keys`/`scan_json`, `scan_json_iter` |
| 4c | Filter builder + limit/order basics | **done** — `Filter` AST, `find`/`find_json`, fluent `query()`, limit/order |
| 4d | Error taxonomy + durability receipts | **done** — `ErrorCode` + `Error::code`; receipts report achieved durability |

---

## Stage 5 — SDA examination profile (recovery as data)

**Goal:** Host builds [SDA_PROFILE.md](SDA_PROFILE.md) `ExaminationUnit` values;
SDA programs examine verified items, partial payloads, and holes.

**Normative scope:** SDA_PROFILE; OVERVIEW §11; DX progressive layer 3–4.

**Deliverables**

1. Map recovered frames/items/holes → normative `ExaminationUnit` product
   shape.
2. API: stream examination units (salvage or online), evaluate SDA program over
   each unit or over pages.
3. Status tags: `verified-complete`, `verified-partial`, holes, encryption
   unavailable, format unsupported—without collapsing them into one “error.”
4. Determinism rules: host supplies ordered/bounded input; SDA remains pure.
5. Resource limits produce explicit incomplete results, never fake empty
   success.

**Exit criteria**

- “If DingoDB can recover it, SDA can examine it” holds for Stage 2–3 salvage
  outputs.
- Profile field set matches SDA_PROFILE (unknown future tags preserved).
- Golden tests: damaged segment → examination stream → SDA filter finds only
  verified islands / reports holes.

**Status**

| 5 | ExaminationUnit host + SDA over salvage | **done** — `dingo-examine` (`examine_store` / `examine_bytes`, `filter_units` / `map_units`, `ExaminePage` + limits); `Store::examination_sources`; golden tests in `stage5_examination.rs` |

**Dependency note**

- Stage 1 must be done. Stage 2–3 provide the host values. Stage 4 can partially
  proceed in parallel with 5, but public “raw SDA on collection” (DX §7.6)
  should not ship before profile shapes stabilize.

---

## Stage 6 — Indexes, catalogs, history, chunks

**Goal:** Fast path without violating “no essential derived state.”

**Normative scope:** OVERVIEW §§5.4–5.5, 6.7, 13; DX §§7–9; FORMAT chunk
manifests.

**Deliverables**

1. Rebuildable primary/current-state index and collection catalog.
2. Secondary indexes online, resumable, deletable; queries correct without
   them (scan + budget).
3. History / event stream for a subject key.
4. Chunked payloads: partial chunk maps, completeness never overstated.
5. Compaction that preserves identities and hole honesty.
6. Optional checkpoints as derived projections with declared coverage.

**Exit criteria**

- Delete all indexes/catalogs → rebuild from segments → same logical content.
- Index states (`building`, `ready`, `stale`, …) visible; queries never claim
  complete absence when tiers/indexes are incomplete without disclosure.
- Chunk tests: missing middle chunk → partial payload + completeness map.
- Benchmark harness skeleton (no marketing claims yet): point read, append by
  durability mode, salvage scan throughput.

| Slice | Deliverable | Status |
|-------|-------------|--------|
| 6a | Collection catalog + wipe/rebuild parity | **done** — `catalogs/collections.cat`; `rebuild_catalogs` / `list_collections` |
| 6b | Secondary indexes + states + scan budget | **done** — `indexes/sec/…/*.six`; SDK `indexes().create/drop/rebuild`; `QueryBudget` |
| 6c | Subject / key history | **done** — `Store::history`, SDK `Collection::history` |
| 6d | Chunked payloads + partial maps | **done** — threshold chunking, `PayloadResult`, manifest `DCHM0001` |
| 6e | Compaction + checkpoints | **done** — `compact_live` (sources retained), `checkpoint` under `snapshots/` |
| 6f | Bench skeleton | **done** — `tests/stage6_bench_skeleton.rs` |

---


## Stage 7 — CLI, doctor, salvage, server

**Goal:** Complete the README initial implementation target for single-node /
operator tooling and same-API remote access.

**Normative scope:** DX_SPEC CLI and doctor/salvage; DX server connect shape.

**Deliverables**

1. CLI mirroring logical API: put/get/list basics. **done** — `crates/dingo-cli` (`dingo` binary)
2. `dingo doctor` — read-only diagnostics by default. **done** — `Store::open_inspect`
3. `dingo salvage` — non-destructive recovery to a new store path. **done** — `Store::salvage_to`
4. Server process + `Dingo.connect("dingo://...")` with the same collection API
   as embedded. **done** — `dingo serve` + line-delimited JSON RPC; remote put/get/delete/scan
5. Authn/deadline/retry as connection options only (no app-level API split). **done** — `ConnectOptions` / `ServeOptions` / `Dingo::connect_with`; `dingo serve --token`; DX codes `authentication_failed` / `deadline_exceeded`
6. Reproducible corruption + performance test packaging for CI/nightly. **done** — `.github/workflows/nightly.yml` + `scripts/nightly.sh` run §13/§16/stage6 bench + Stage 7 CLI

**Exit criteria**

- DX journeys 4–5, 7, 9–10 satisfied for single-node. **done** (CLI + doctor + salvage + serve/connect)
- Doctor never writes by default; salvage does not mutate the source store. **done** — tests in `dingo-cli/tests/cli.rs`, `dingo-store/tests/salvage.rs`
- Embedded and server pass the same logical SDK put/get path (transport differs). **done** — `serve_and_sdk_connect_parity`
- README “initial implementation target” checklist checked item-by-item. **done** for single-node items

**Suggested sub-milestones**

| 7a | `dingo` CLI put/get/list/delete/put-bytes | **done** |
| 7b | `dingo doctor` read-only | **done** |
| 7c | `dingo salvage --output` | **done** |
| 7d | `dingo serve` + `Dingo::connect` | **done** |
| 7e | Authn/deadline/retry connection options | **done** |
| 7f | Nightly corruption/perf packaging polish | **done** |

**README initial target mapping**

| README bullet | Stage |
|---------------|-------|
| zero-config embedded | 3–4 |
| collection-oriented SDK | 4 |
| JSON/bytes put get delete append filters | 4 |
| CLI doctor + non-destructive salvage | 7 |
| resync framed journal | 2–3 |
| immutable self-describing segments | 2–3 |
| inline and chunked payloads | 3 + 6 |
| independent verification + island recovery | 2–3, 5 |
| rebuildable catalogs and indexes | 6 |
| SDA examination | 1 + 5 |
| reproducible corruption and performance tests | 2–3, 6–7 |

---

## Stage 8 — Cluster (after single-node is real)

**Goal:** Federation of independently salvageable nodes without making the
control plane payload authority.

**Normative scope:** [CLUSTER_SPEC.md](CLUSTER_SPEC.md).

**Deliverables (high level)**

1. Partitioned keyspace; partition-local consensus for strong writes.
2. Frame replication + ack modes including `replicated`.
3. Coverage records on every distributed result.
4. Convergent-append mode for split-friendly immutable events.
5. Node salvage without cluster software still yields ordinary segments.
6. Same SDK API as embedded/server; routing cached and refreshed safely.

**Suggested sub-milestones**

| 8a | Foundation: `dingo-cluster` crate; virtual partitions; coverage; placement directory; development + dependable-local profiles; quorum-style put/delete; node salvage without cluster | **done** — `tests/stage8a_cluster.rs` |
| 8b | Real per-partition Raft (or equivalent) elections, log matching, commit evidence | **done** — `src/raft.rs`, `tests/stage8b_raft.rs` |
| 8c | Convergent-append path + split dual-accept tests | **done** — `src/convergent.rs`, `tests/stage8c_convergent.rs` |
| 8d | SDK routing (`Dingo::connect` cluster URLs) + client directory cache | **done** — `ClientDirectoryCache`, `Dingo::open_cluster` / `create_cluster`, multi-seed URL parse, `directory` RPC; tests `stage8d_routing.rs` |
| 8e | Distributed scan/find coverage + partial-query honesty | open |
| 8f | CLUSTER_SPEC §22 remaining conformance + rebalance | open |

**Exit criteria**

- CLUSTER_SPEC conformance tests (§22) for the chosen deployment profile.
- Control plane loss does not make surviving segments unreadable.
- No global hot-path lock; strong ordering remains partition-local.

**Do not start Stage 8 until**

- Stages 2–4 destructive and DX gates are green.
- Salvage and doctor work on a single node without cluster metadata.

**Stage 8a notes**

- Leadership was **static primary** from the placement directory (term fencing
  on the assignment only). Superseded by 8b for live writes/reads.
- Balanced placement puts every voting node as a replica of every virtual
  partition (simple for tests; rebalance is later).
- Development profile explicitly warns that replicated durability is
  unavailable.

**Stage 8b notes**

- Per-partition Raft-equivalent groups (`dingo_cluster::raft`) with published
  election, log-matching, and commit rules (CLUSTER_SPEC §10.1).
- Leadership is elected among online voters; quorum is majority of the
  **configured** voter set. Leader loss with a live majority re-elects.
- Client commands enter the Raft log first; stores are applied only after
  `commit_index` advances (commit evidence on acks).
- Membership changes, leases, and log snapshots remain later work.

**Stage 8c notes**

- `convergent-append` writes skip Raft: any online replica may accept; acks
  report `commit_status: prepared` and `committed: false` (not linearizable).
- `append_local` targets one ingest node for dual-accept split tests.
- `reconcile` fans out missing `(subject, body)` by content hash; same subject
  with differing live bodies is reported as an explicit conflict (both retained
  in history). Linearizable reads return `consistency_violation` in this mode.

**Stage 8d notes**

- Same collection API over an in-process cluster via `Dingo::create_cluster` /
  `open_cluster`; client holds a [`ClientDirectoryCache`] of partition → leader
  routes and refreshes on stale placement (CLUSTER_SPEC §13, §22.5).
- Multi-seed `dingo://h1:p1,h2:p2[/label]` URLs parse and try seeds in order.
- Single-node `dingo serve` answers `directory` with a synthetic all-local
  placement snapshot for uniform client caching.

---

## Stage 9 — Tiering, archive, long retention

**Goal:** One logical store across hot/warm/cold/archive without rewrite-the-
world migrations.

**Normative scope:** OVERVIEW retention/tiering; CLUSTER_SPEC tiered cluster;
USP long retention.

**Deliverables**

1. Segment move/copy to colder media with stable identities.
2. Hierarchical catalogs for cold search; rebuild after catalog loss.
3. Archive-path performance class benchmarks (separate from hot path).
4. Media migration and multi-generation format readers
   (`format-unsupported` + byte preservation).

**Exit criteria**

- Cold retrieval never claimed under hot-path latency SLOs.
- Offline tier unavailable → explicit coverage hole, not empty success.
- Multi-year story is operationally documented (runbooks), not only aspirational.

---

## 5. Parallelism after Stage 1

Once Stage 1 is locked, work can fan out carefully:

```text
        ┌── Stage 2 (format/salvage) ── Stage 3 (store) ──┐
Stage 1 ┤                                                   ├── Stage 5 (profile)
        └── Stage 4 stubs (API design, fakes) ─────────────┘
                              │
                         Stage 4 full
                              │
                    Stage 6 ── Stage 7 ── Stage 8 ── Stage 9
```

- **Do parallelize:** SDA conformance growth vs format codec experiments.
- **Do not parallelize naively:** SDK release vs unfinished frame identity
  rules; cluster vs unproven single-node salvage.

## 6. Quality bars by stage type

| Stage type | Required gates |
|------------|----------------|
| SDA (1) | Semantic golden tests, error-tag stability, determinism |
| Format/store (2–3, 6) | Destructive island tests, hole honesty, rebuild from authority |
| DX (4, 7) | Journey tests, progressive disclosure (no salvage jargon on happy path) |
| Examination (5) | Profile shape tests, damaged-store SDA scripts |
| Cluster (8) | Coverage, split behavior, node-local salvage |
| Perf claims (6+) | Benchmark disclosure checklist from OVERVIEW §12.2 |

## 7. Freezes and versioning

| Artifact | Freeze target |
|----------|----------------|
| SDA core semantics | After Stage 1 exit; changes need explicit versioning |
| Standalone SDA surface | With core; helpers only via profile version |
| Wire format major 1 | After Stage 2–3 production soak; until then draft bytes |
| Collection SDK 1.0 | After Stage 4 + 7 embedded/server parity |
| Cluster profile v1 | After Stage 8 conformance; no cross-partition atomic writes (per CLUSTER_SPEC) |

## 8. Suggested first demo milestones (human-facing)

These are narrative checkpoints for users and sponsors, not separate engineering
tracks:

1. **“Algebra works”** — Stage 1: paste JSON, run SDA, get deterministic tree.
2. **“Punch a hole”** — Stage 2: corrupt a segment file; scanner lists islands
   and holes.
3. **“Database that survives”** — Stage 3–4: app puts data; wipe indexes;
   salvage; app still reads survivors.
4. **“Examine the damage”** — Stage 5: SDA over examination units filters
   verified vs holes.
5. **“Ordinary product”** — Stage 6–7: indexes, doctor, CLI, server.
6. **“Federation”** — Stage 8: kill a node; others serve; dead node’s disks
   still salvage offline.
7. **“Keep it fifteen years”** — Stage 9: tier move + cold search story.

## 9. Open decisions

Record answers in-repo; they block packaging, not the stage order:

| # | Decision | Status |
|---|----------|--------|
| 1 | Implementation language(s) for core vs SDK | **Resolved (Stage 0):** Rust core; first SDK is Rust lib API; DX TypeScript-like samples remain the product shape (other language SDKs later). See [ARCHITECTURE.md](ARCHITECTURE.md). |
| 2 | Sync marker, integrity algorithms, draft wire constants | **Open** — resolve at Stage 2a against [FORMAT_SPEC.md](FORMAT_SPEC.md). |
| 3 | Default durability mode for embedded open | **Open** — DX says safe/durable default; confirm at Stage 3–4. |
| 4 | First secondary-index implementation | **Done (in-process)** — Stage 6 field indexes under `indexes/sec/`. |
| 5 | Consensus library vs purpose-built leadership | **Open** — Stage 8 only. |
| 6 | Whether `sda` ships inside `dingo` or separate | **Resolved for now:** separate `sda` binary (Stage 1). Stage 7 may add `dingo` without removing `sda`. |

## 10. Work apportionment (streams)

Parallel streams after Stage 0; refuse out-of-order starts:

| Stream | Stages | Start when |
|--------|--------|------------|
| **A — SDA** | 1 | Stage 0 builds green |
| **B — Survival format** | 2 | Stage 0; full freeze after 1 exits preferred |
| **C — Store** | 3 | Frame codec + scanner (2a–2c) usable |
| **D — Collection SDK** | 4 | Store put/get/delete path (3) |
| **E — Examination** | 5 | SDA locked (1) + salvage units (2–3) |
| **F — Operator path** | 6–7 | Ordinary DX (4) solid |
| **G — Cluster / tiering** | 8–9 | Single-node salvage + doctor (2–4, 7) |

**Do not start G before C salvage is proven.**

## 11. What “done” means for this plan document

This plan is successful if an implementer can:

- pick up work at Stage 0 or 1 without rereading every spec end-to-end;
- know which spec sections gate each stage;
- refuse out-of-order work (especially cluster-before-salvage);
- map the README initial target to concrete exit criteria.

When the codebase arrives, convert each stage into issues/milestones and attach
the cited conformance suites as required checks—not optional polish.

## 12. Immediate next steps (codebase has landed)

1. ~~Land Stage 0 scaffold when the codebase is added.~~ **Done** — workspace,
   CI, architecture map, language decision.
2. ~~Stage 1 SDA §14.1 minimal suite automated.~~ **Done** — `crates/sda-core/tests/sda_conformance.rs`
   module `section_14_1_minimal_suite` covers placeholder scoping, BagKV
   duplicates, `normalizeUnique`, equality, standalone helpers, carrier
   preservation, null-vs-absence, Unicode/ASCII synonyms, and `Bind`. Core
   no longer exposes `normalizeFirst`/`normalizeLast` (§7.2).
3. Expand beyond §14.1 to full §14 MUST lock (remaining edge cases, versioned
   golden corpus under `tests/sda/` if desired); keep `sda-core` pure.
4. Freeze SDA standalone behavior behind a versioned conformance corpus tag.
5. ~~Open Stage 2 format work (`dingo-format` frame codec).~~ **Done (2a)** —
   `crates/dingo-format` encodes/decodes FORMAT_SPEC frames with CRC32C +
   BLAKE3-256 body hash and structural `verified-complete` checks (envelope
   still opaque bytes; deterministic CBOR rules not yet enforced).
6. ~~Stage 2b–2c — active segment seal + forward salvage scanner.~~ **Done** —
   `ActiveSegment` / `SealedSegment` (draft descriptor & summary bodies);
   `scan_forward` with hole reports; later islands remain discoverable after
   corrupt candidates (search resumes at `q + 1`).
7. ~~Stage 2d — full FORMAT_SPEC §13 destructive corpus.~~ **Done** —
   `tests/section13_corpus.rs` automates every §13 bullet; `scan_reverse`
   (§7.4); `group_by_event_id` (§9); draft `reassemble_chunks` partial maps
   (§8). Envelope CBOR (§5 condition 6) still deferred.
8. ~~Stage 3 store (`dingo-store`).~~ **Done (3a–3c)** — put/get/delete,
   durability, salvage, §16 suite, descriptor + index cache.
9. ~~Stage 4 collection SDK (`dingo-sdk`).~~ **Done (4a–4d)** — open,
   JSON/bytes, scan/stream, filters, `ErrorCode`, receipts.
10. ~~Stage 5 SDA examination (`dingo-examine`).~~ **Done** — ExaminationUnit
    projection from salvage, `examine_store` / `examine_bytes`, SDA
    `filter_units` / `map_units`, bounded `ExaminePage` with resource-limit
    honesty; golden tests `stage5_examination.rs`.
11. ~~Stage 6 indexes/catalogs/history/chunks.~~ **Done** — rebuildable
    collection catalog; secondary field indexes (`building`/`ready`/`stale`/…);
    scan + `QueryBudget`; per-key history; chunked puts with partial maps;
    live-state compaction (sources retained); derived checkpoints; bench
    skeleton (`stage6_store.rs`, `stage6_indexes_history.rs`,
    `stage6_bench_skeleton.rs`).
12. ~~Stage 7 CLI/doctor/salvage/server.~~ **Done** — `dingo-cli` (`dingo`
    put/get/list/delete/put-bytes/history/doctor/salvage/serve);
    `Store::open_inspect` + `salvage_to`; `Dingo::connect("dingo://...")`
    line-delimited JSON RPC; tests `dingo-cli/tests/cli.rs`.
13. ~~Stage 7e–7f tighten (authn/deadline/retry + nightly packaging).~~ **Done** —
    `ConnectOptions` / `ServeOptions`, remote receipt ids, nightly workflow.
14. ~~Remote parity for history + secondary indexes.~~ **Done** — RPC ops
    `history`, `index_list` / `index_create` / `index_drop` / `index_rebuild`;
    server marks indexes stale on put/delete; tests
    `stage7_remote_parity.rs`.
15. ~~Remote `get_payload` + server-side find.~~ **Done** — RPC ops
    `get_payload` (complete/partial/unavailable/conflicting maps) and `find`
    (JSON filter, limit/order/budget/`force_scan`, index-accelerated via
    shared `find_on_store`); tests in `stage7_remote_parity.rs`.
16. ~~Stage 8a cluster foundation.~~ **Done** — `dingo-cluster` crate:
    virtual partitions (`blake3-mod-v1`), coverage records, placement
    directory, development (1-node) + dependable-local (3-node) profiles,
    quorum-style put/delete, node salvage without cluster software;
    tests `stage8a_cluster.rs`.
17. ~~Stage 8b per-partition Raft.~~ **Done** — elections, log matching,
    commit evidence (`src/raft.rs`); linearizable path elects/re-elects,
    proposes through the log, applies after quorum commit; tests
    `stage8b_raft.rs`.
18. ~~Stage 8c convergent-append.~~ **Done** — dual-accept without quorum,
    `append_local` + `reconcile` with explicit subject conflicts; tests
    `stage8c_convergent.rs` (§22 items 7–8).
19. ~~Stage 8d SDK routing + client directory cache.~~ **Done** —
    `ClientDirectoryCache`, `Dingo::open_cluster` / `create_cluster`, multi-seed
    URL parse, `directory` RPC; tests `stage8d_routing.rs` (§13, §22.5).
20. **Next:** Stage 8e–8f (distributed find coverage, §22 remainder);
    optional deterministic CBOR envelope validation (FORMAT_SPEC §5
    condition 6); Stage 9 tiering.
