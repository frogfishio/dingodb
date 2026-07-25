# DingoDB

## The database for data that outlives applications

**Store anything. Use it now. Understand it later.**

DingoDB is a retention-native database for data that must remain useful long
after the application, schema, storage system, and team that created it have
changed.

Use it like an ordinary database today: collections, JSON, bytes, keys,
filters, indexes, and history. Retain the same data across storage tiers
without surrendering it to a fragile catalog or proprietary control plane.
Return years later with new software, new indexes, or new questions.

An object store can keep bytes. DingoDB is designed to keep those bytes
**database-usable**.

> Your data should outlive your stack.

## Why DingoDB

Most databases are optimized around the application that uses them now.
Archives are optimized around keeping objects cheaply. Moving from one to the
other usually means giving up immediacy, history, queryability, or independence
from today's metadata.

DingoDB treats active use and long-term retention as one lifecycle:

```text
write once
    │
    ├── use now ─────── collections · keys · filters · indexes · history
    │
    ├── retain ──────── hot · warm · cold · archive
    │
    ├── reinterpret ─── new decoders · new indexes · SDA examination
    │
    └── recover ─────── verified data islands · explicit holes · provenance
```

Choose DingoDB when the data is expected to live longer than:

- the service that first wrote it;
- its original schema or file format;
- the current storage hardware or cloud provider;
- the indexes and catalogs used to find it today;
- the cluster or organization operating it.

Typical candidates include event histories, scientific and device output,
documents and media, AI datasets and artifacts, long-lived application state,
compliance evidence, logs worth retaining, and formats that may only become
valuable in the future.

## What “retention-native” means

### Useful immediately

DingoDB stores JSON and raw bytes behind an ordinary collection API. Data is
available for key lookup, filtering, indexing, streaming, and history without
first being exported into an archival system.

### Preserved without interpretive lock-in

Payloads carry small self-describing envelopes. DingoDB can preserve opaque or
unknown bytes without requiring a schema, decoder, or semantic model up front.
Understanding can be added later without rewriting the original material.

### One identity across storage tiers

Immutable segments retain their identity as they move between hot, warm, cold,
and archive media. Physical placement changes; the logical data does not.

### Derived state is acceleration, not authority

Indexes, catalogs, summaries, and placement maps make access fast. They are not
the only route back to the data. Delete them and DingoDB can rebuild from
independently framed segments.

### Damage is contained

Damage tolerance is not the product category; it is what makes the retention
promise credible.

A corrupt record, missing chunk, truncated segment, or lost catalog does not
invalidate unrelated healthy material after it. DingoDB scans for every
surviving verified island and reports the gaps instead of pretending the
database is wholly intact—or wholly lost.

```text
┌─────────────────────────────────────────────────────────┐
│ DATA │ DATA │ █ HOLE █ │ DATA │ SCRATCH │ DATA │ DATA │
│  ✓   │  ✓   │    ✗     │  ✓   │    ✗    │  ✓   │  ✓   │
└─────────────────────────────────────────────────────────┘
```

The recovery rule is simple:

> What is gone is gone. What remains still lives.

## Try the embedded database

DingoDB currently ships as a Rust workspace. The embedded single-node path is
the most complete product surface.

```sh
git clone https://github.com/frogfishio/dingodb
cd dingodb
cargo install --path crates/dingo-cli

dingo put ./demo.dingo users/user-42 \
  --json '{"name":"Alice","status":"active"}'

dingo get ./demo.dingo users/user-42
dingo history ./demo.dingo users/user-42
dingo doctor ./demo.dingo
```

The Rust SDK exposes the same logical model:

```rust
use dingo_sdk::{json, Dingo, Filter};

fn main() -> Result<(), dingo_sdk::Error> {
    let mut db = Dingo::open("./app.dingo")?;
    let mut users = db.collection("users")?;

    users.put(
        "user-42",
        &json!({ "name": "Alice", "status": "active" }),
    )?;

    let alice = users.get("user-42")?;
    let active = users.find(&Filter::field("status").eq("active"))?;
    println!("{alice:?} {active:?}");

    Ok(())
}
```

Collections are schemaless by default. JSON and bytes are first-class, and
ordinary filters do not require learning the storage format or recovery model.

## Examine what survives

DingoDB distinguishes physical survival from logical completeness. It can
represent:

- verified complete values;
- verified partial payloads and surviving extents;
- missing chunks and physical holes;
- unsupported formats;
- conflicting evidence;
- unavailable tiers or keys;
- incomplete and uncertain projections.

`dingo doctor` inspects without modifying the source. `dingo salvage` writes
verified recovery evidence to a separate destination. A separate
`dingo export-live` command materializes only current complete state when that
is what the operator actually wants.

[SDA](SDA_SPEC.md), the Structured Data Algebra, provides deterministic
filtering and transformation over structured values and recovered evidence:

> If DingoDB can recover it, SDA can examine it.

## Where DingoDB fits

DingoDB does not try to replace every database:

- Use a relational database when joins, constraints, and general transactions
  are the center of the workload.
- Use an in-memory cache when ephemeral low-latency access is the only goal.
- Use a plain object store when retaining independent objects is enough.
- Use a warehouse when curated analytical tables are the product.
- Use DingoDB when arbitrary data must remain **active now, retainable at
  scale, reinterpret-able later, and independently recoverable**.

The concise answer is:

> We use DingoDB because our data needs to outlive the software and
> infrastructure that created it.

## Architecture

DingoDB separates durable truth from replaceable acceleration:

```text
application
    │
    ▼
collection SDK ───── JSON · bytes · filters · history
    │
    ▼
append store ─────── active segment · durability receipt
    │
    ▼
immutable segments ─ independently framed · verified · movable
    │
    ├── indexes and catalogs ─ rebuildable
    ├── SDA examination ───── deterministic
    └── storage tiers ─────── hot · warm · cold · archive
```

The workspace is organized into focused crates:

- `dingo-format` — survival wire format, integrity, and salvage scanning;
- `dingo-store` — single-node append store, history, chunks, and tiers;
- `dingo-sdk` — collection API for embedded, remote, and cluster backends;
- `dingo-examine` — recovered evidence projected into SDA values;
- `dingo-cli` — everyday and operator commands;
- `dingo-cluster` — partition, coverage, consensus, and rebalance model;
- `sda-lib` / `sda` — the pure SDA library and CLI.

## Current maturity

DingoDB is under active development. It is **not yet production-ready** as a
network database or distributed storage system.

| Surface | Current status |
|---------|----------------|
| Embedded single-node | Experimental / early access; strongest current path |
| Single-node TCP server | Development only |
| In-process cluster | Deterministic integration-test harness |
| Network `serve-cluster` | Routing prototype; no network quorum replication |
| S3/GCS | Filesystem-mirror adapter, not native cloud I/O |
| Lifecycle and erasure coding | Scaffolds |
| Wire format | `1.0-draft`, not frozen |

The project deliberately separates implementation milestones from production
qualification:

- [Capability matrix](doc/CAPABILITY_MATRIX.md)
- [Production-readiness work](DEFECTS.md)
- [Benchmark disclosure requirements](doc/BENCHMARK_DISCLOSURE.md)

Do not treat multiple `serve-cluster` processes as replicated durability.
Persistent network consensus, TLS, production observability, and other release
gates remain active work.

## Specifications and development

DingoDB is specification-driven:

- [System architecture](OVERVIEW.md)
- [Survival wire format](FORMAT_SPEC.md)
- [Developer experience](DX_SPEC.md)
- [Cluster architecture](CLUSTER_SPEC.md)
- [Scoped transaction extension proposal](TRANSACTIONS.md)
- [Jurisdiction and sovereign placement proposal](JURISDICTION_PROPOSAL.md)
- [Structured Data Algebra](SDA_SPEC.md)
- [DingoDB SDA profile](SDA_PROFILE.md)
- [Crate map](ARCHITECTURE.md)
- [Delivery plan](DELIVERY_PLAN.md)
- [Contributing](CONTRIBUTING.md)
- [Human demos](scripts/demos/)

Run the workspace:

```sh
cargo test --workspace
cargo run -p sda --bin sda -- eval -e '1 + 2'
cargo run -p dingo --bin dingo -- --help
```

## License

DingoDB is released under the MIT License.

The storage formats and specifications are intended to remain open, documented,
and implementable without a proprietary service.
