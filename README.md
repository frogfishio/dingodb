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

### Extreme speed

DingoDB separates durable truth from acceleration:

- append-oriented, sharded ingestion;
- memory-resident hot indexes;
- immutable segments for parallel reads;
- asynchronous, rebuildable search structures;
- explicit memory, buffered, durable, and replicated acknowledgement modes.

The hot path targets the performance class of dedicated in-memory stores.
Durability and verification modes are always disclosed with benchmark claims.

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

## Status

DingoDB is past pure-spec: Stages **0–9** land in-tree. SDA library + CLI, wire
format + salvage, single-node store, collection SDK, SDA examination,
indexes/history/chunks, Stage 7 operator surface (`dingo` CLI + remote parity),
`dingo-cluster` multi-node federation (partitions, coverage, Raft,
convergent-append, SDK route cache, find coverage, rebalance), and Stage 9
tiering/archive with a media-locator seam for object roots. Product follow-ons
landed: live S3/GCS mirrors (`DINGO_S3_ROOT` / `DINGO_GS_ROOT`), multi-hop
`dingo serve-cluster` client routing, freeze labels (SDK 1.0 / cluster v1 /
wire 1.0-draft), lifecycle + erasure scaffolds, benchmark disclosure checklist.

| Stage | Focus | Status |
|-------|--------|--------|
| 0 | Repo + CI | done |
| 1 | SDA standalone | library + CLI + §14 MUST lock (`sda-standalone-v1.0`) |
| 2 | Wire format + salvage | **2a–2d** frames, seal, scanners, §13 corpus |
| 3 | Single-node store | **3a–3c** put/get/delete, §16 suite, descriptor + index cache |
| 4 | Collection SDK | **4a–4d** open, JSON/bytes, scan/stream, filters, `ErrorCode` |
| 5 | SDA examination profile | **done** — `dingo-examine` |
| 6 | Indexes, catalogs, history, chunks | done |
| 7 | CLI, doctor, salvage, server | **done** — 7a–7f + full single-node remote parity |
| 8 | Cluster | **8a–8f done** — find coverage honesty + rebalance + §22 remainder |
| 9 | Tiering | **done** — segment move/copy, hierarchical catalogs, offline coverage, retention runbook |

Staged plan: [DELIVERY_PLAN.md](DELIVERY_PLAN.md).  
Crate map and language decisions: [ARCHITECTURE.md](ARCHITECTURE.md).  
How to contribute / apportion work: [CONTRIBUTING.md](CONTRIBUTING.md).

```sh
cargo test --workspace
cargo run -p sda --bin sda -- eval -e '1 + 2'
```

The initial implementation target is:

- zero-configuration embedded operation;
- one excellent collection-oriented SDK;
- JSON and bytes with put, get, delete, append, and streaming filters;
- a small, safe CLI with doctor and non-destructive salvage;
- a resynchronizable framed journal;
- immutable self-describing segments;
- inline and chunked payloads;
- independent verification and island recovery;
- rebuildable catalogs and indexes;
- SDA examination;
- reproducible corruption and performance tests.

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
