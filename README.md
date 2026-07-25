# DingoDB

## Damage the database. Keep the data you did not destroy.

DingoDB is a damage-tolerant, high-performance database for arbitrary digital
material.

Put structured records, logs, documents, binary objects, application state,
unknown formats, or uninterpreted bytes into it. Read them immediately through
fast indexes, retain them across massive storage tiers, and return years later
to examine whatever remains.

DingoDB is designed around a simple recovery rule:

> What is gone is gone. What remains still lives.

```text
┌─────────────────────────────────────────────────────────┐
│ DATA │ DATA │ █ HOLE █ │ DATA │ SCRATCH │ DATA │ DATA │
│  ✓   │  ✓   │    ✗     │  ✓   │    ✗    │  ✓   │  ✓   │
└─────────────────────────────────────────────────────────┘
```

A hole in the middle does not poison the healthy data after it. DingoDB finds
and verifies every surviving data island instead of treating the store as one
fragile object.

## Ordinary to use

The unusual recovery model is not the everyday API.

```ts
const db = await Dingo.open("./app.dingo");
const users = db.collection("users");

await users.put("user-42", {
  name: "Alice",
  status: "active"
});

const alice = await users.get("user-42");

for await (const user of users.find({ status: "active" })) {
  console.log(user);
}
```

JSON and bytes are first-class. Collections are schemaless by default.
Ordinary filters do not require learning SDA. Embedded, server, and clustered
deployments use the same logical API.

## Core promises

### Independent survival

Records, payload chunks, and immutable segments are independently framed and
verified.

Corrupt a record, truncate a segment, lose an index, delete a catalog, or punch
a hole through the storage medium: unrelated intact data remains recoverable.

Recovery does not stop at the first damaged byte.

### Performance with disclosed modes

DingoDB separates durable truth from acceleration:

- append-oriented, sharded ingestion;
- memory-resident hot indexes;
- immutable segments for parallel reads;
- asynchronous, rebuildable search structures;
- explicit memory, buffered, durable, and (cluster-profile) replicated
  acknowledgement modes.

Performance claims are mode-specific and require reproducible disclosure; see
[doc/BENCHMARK_DISCLOSURE.md](doc/BENCHMARK_DISCLOSURE.md). Do not read this
section as a Redis-class latency guarantee for every deployment profile.

### Massive retention

One logical DingoDB store may span:

- memory and local flash;
- local or network disks;
- object storage;
- replicated or erasure-coded storage;
- offline archival media;
- multiple hardware and format generations.

Segments are independently movable and self-identifying. No operation requires
rewriting the entire store, and losing the global catalog does not make the
underlying segments meaningless.

Store it today. Find it fifteen years later.

### Clustering without a new black box

A DingoDB cluster is a federation of independently recoverable partitions and
segments.

Consensus controls partition ownership and strong writes, but it is never the
only map back to the bytes. Destroy the cluster catalog or remove a storage
node: its surviving segments remain ordinary, self-identifying DingoDB data.

Strong ordering is partition-local. There is no global lock or global sequence
on the hot path. Workloads that prefer ingestion availability can use
convergent append and retain both sides of a network split explicitly.

### SDA examination

[SDA](SDA_SPEC.md), the Structured Data Algebra, is DingoDB's deterministic
examination and transformation language.

SDA can inspect:

- verified envelopes and structured payloads;
- opaque-byte descriptors;
- recovered fragments;
- missing chunks;
- physical holes;
- incomplete and uncertain derived state.

If DingoDB can recover it, SDA can examine it.

## Preserve first, understand later

DingoDB does not need to understand a payload before preserving it.

Every item receives a durable, self-describing envelope. The payload may be
structured data, opaque bytes, or independently recoverable chunks.

New decoders, schemas, labels, full-text indexes, semantic indexes, and SDA
projections can be applied years after ingestion without rewriting the
original bytes.

```text
ingest → hot indexes → immutable segments → cold archive
          fast now        durable history     cheap retention
```

## Recovery is evidence, not optimism

DingoDB distinguishes:

- physically verified data;
- complete logical state;
- partial payloads;
- unsupported formats;
- encrypted data whose keys are unavailable;
- corruption;
- known holes;
- uncertain reconstructions.

It never silently converts “survived” into “complete.”

An event after a hole remains valid as an event. A current-state projection
that may depend on missing history is returned as incomplete or uncertain.

## What DingoDB is not

DingoDB is not magic. It cannot recover bytes after every physical copy has
been destroyed.

It does not claim:

- Redis-class latency for offline archival data;
- semantic understanding of arbitrary bytes;
- complete state when required history is missing;
- zero-cost durability;
- SQL compatibility or distributed transactions by default.

Its promise is narrower and stronger:

> Localized destruction causes localized loss.

## Status and maturity

**Not production-ready** as a network database or distributed storage system.
Support labels until the release gates in [DEFECTS.md](DEFECTS.md) §16 pass:

| Deployment | Label |
|------------|--------|
| Embedded single-node (`Dingo::open`) | experimental / early-access |
| Single-node TCP (`dingo serve`) | development only (loopback by default) |
| In-process cluster (`Dingo::open_cluster`) | deterministic integration-test harness |
| Network `dingo serve-cluster` | **routing / endpoint advertise prototype** — writes hit **one node**; no network quorum |
| S3/GCS | filesystem-mirror integration, not a native cloud backend |
| Erasure coding / lifecycle automation | scaffolds only |
| Wire format | `1.0-draft` (not frozen for long-term interop) |

Capability matrix (what is tested vs experimental):  
[doc/CAPABILITY_MATRIX.md](doc/CAPABILITY_MATRIX.md).  
Known production-readiness gaps: [DEFECTS.md](DEFECTS.md).

### Version labels (do not conflate)

| Label | Where | Meaning |
|-------|--------|---------|
| Crate semver `0.1.0` | `[workspace.package].version` / crates.io | Package release number only |
| `SDK_API_VERSION` = `1.0` | `dingo-sdk` | Collection API freeze label after Stage 4+7 parity |
| `CLUSTER_PROFILE_VERSION` = `v1` | `dingo-cluster` | **In-process** cluster profile (8a–8f), not network Raft maturity |
| `WIRE_PROFILE_LABEL` = `1.0-draft` | `dingo-format` | Draft wire bytes until major-1 freeze |
| `CONFORMANCE_CORPUS_TAG` | `sda-lib` | SDA §14 corpus tag (`sda-standalone-v1.0`) |

Stages **0–9** and product follow-ons **1–4** are **implemented in-tree** (stage
exit criteria met). That is not the same as production qualification.

| Layer | Crate / binary | Maturity / notes |
|-------|----------------|------------------|
| SDA library + CLI | `sda-lib`, `sda` | conformance-locked `sda-standalone-v1.0` |
| Wire format + salvage | `dingo-format` | draft wire; §13 corpus |
| Single-node store | `dingo-store` | early-access embedded; put/get/delete, §16 suite |
| Collection SDK | `dingo-sdk` | `SDK_API_VERSION` = `1.0` (embedded + remote + in-process cluster) |
| SDA examination | `dingo-examine` | ExaminationUnit stream over salvage |
| Operator CLI | `dingo` | put/get/list, doctor, salvage, `serve`, experimental `serve-cluster` |
| Cluster federation | `dingo-cluster` | `CLUSTER_PROFILE_VERSION` = `v1` (**in-process** 8a–8f) |
| Tiering / media | store tiers + mirrors | filesystem + `object:local:` + **S3/GCS filesystem mirrors** |
| Network multi-hop | `serve-cluster` + client routes | experimental; **not** replicated durability |
| Lifecycle / erasure | store scaffolds | API scaffolds only |
| Benchmarks | [doc/BENCHMARK_DISCLOSURE.md](doc/BENCHMARK_DISCLOSURE.md) | disclosure template; no Redis-class claims |

| Stage | Focus | Implementation status |
|-------|--------|------------------------|
| 0 | Repo + CI | implemented |
| 1 | SDA standalone | implemented — §14 MUST lock (`sda-standalone-v1.0`) |
| 2 | Wire format + salvage | implemented — frames, seal, scanners, §13 corpus |
| 3 | Single-node store | implemented — put/get/delete, §16 suite |
| 4 | Collection SDK | implemented — open, JSON/bytes, scan/stream, filters |
| 5 | SDA examination profile | implemented — `dingo-examine` |
| 6 | Indexes, catalogs, history, chunks | implemented |
| 7 | CLI, doctor, salvage, server | implemented — development `serve` |
| 8 | Cluster | implemented **in-process** — 8a–8f (not network Raft) |
| 9 | Tiering | implemented — filesystem media roots + mirrors |

Still **not** production: network Raft log shipping / quorum over TCP, native
cloud object SDKs beyond mirrors, erasure codecs, exclusive store locks, and
the rest of [DEFECTS.md](DEFECTS.md).

Staged plan: [DELIVERY_PLAN.md](DELIVERY_PLAN.md).  
Crate map and language decisions: [ARCHITECTURE.md](ARCHITECTURE.md).  
How to contribute: [CONTRIBUTING.md](CONTRIBUTING.md).  
Human demos: [scripts/demos/](scripts/demos/).

```sh
cargo test --workspace
cargo run -p sda --bin sda -- eval -e '1 + 2'
cargo run -p dingo --bin dingo -- --help
```

### What is available in-tree today

- zero-configuration embedded operation (`Dingo::open`) — experimental;
- collection-oriented Rust SDK with JSON and bytes, put/get/delete, filters,
  indexes, history, and streaming scans;
- remote `Dingo::connect` / development-only `dingo serve` (loopback default;
  non-loopback plaintext requires `--allow-insecure-bind`);
- `dingo` CLI with doctor and non-destructive salvage;
- experimental `dingo serve-cluster` (**routing only**; requires
  `--experimental-network-cluster`; **three processes ≠ replicated durability**);
- resynchronizable framed journal and immutable self-describing segments;
- inline and chunked payloads with completeness-aware reads;
- independent verification and island recovery;
- rebuildable catalogs and indexes;
- SDA examination over recovered units (`dingo-examine` / doctor);
- **in-process** cluster federation with coverage honesty and rebalance;
- hot/warm/cold/archive filesystem tiering with offline coverage disclosure;
- S3/GCS **mirrors** (not native cloud backends);
- reproducible corruption and performance test packaging (nightly + demos).

See the [architecture specification](OVERVIEW.md) for normative requirements,
the [survival format](FORMAT_SPEC.md) for the draft wire profile, the
[SDA specification](SDA_SPEC.md) for the algebra, and the
[DingoDB SDA profile](SDA_PROFILE.md) for recovery examination semantics.
Distributed deployments are defined by the
[cluster architecture](CLUSTER_SPEC.md). The everyday API, CLI, defaults, and
progressive-disclosure rules are defined by the
[developer experience specification](DX_SPEC.md).

## License

DingoDB is released under the MIT License.

The storage formats and specifications are intended to remain open, documented,
and implementable without a proprietary service.
