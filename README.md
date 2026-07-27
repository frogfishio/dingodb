# DingoDB

## Simple enough for a side project. Built for data that cannot be replaced.

DingoDB is an embedded-first database for projects that want to start small
without treating their data as disposable.

Open a file. Create a collection. Store JSON or bytes. There is no server to
provision and no schema ceremony before the first write. If the project grows,
the same data model is designed to gain history, indexes, storage tiers,
replication, jurisdiction controls, and forensic recovery without first being
exported into an entirely different kind of system.

Most side projects disappear. Some become businesses. A few create data that
matters long after the original application is gone. You should not need to
predict which one you are building before choosing a database that respects
the result.

> Start with one file. Keep the data for as long as it matters.

## See it work

DingoDB currently ships as a Rust workspace. Its embedded single-node path is
the strongest and simplest product surface.

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

The Rust SDK uses the same ordinary collection model:

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

Collections are schemaless by default. JSON and raw bytes are first-class.
Keys, filters, indexes, streaming, and history are available without requiring
an operator—or requiring the application to understand DingoDB's storage
format.

## The doctrine

The tagline is not a choice between simplicity and serious storage. It is the
design constraint that connects them.

### Simplicity is the default, not the ceiling

A small application should be able to use DingoDB as a local file with a
collection API. Distribution, tiering, recovery tools, and policy controls
should appear only when the data or workload earns that complexity.

### The importance of data is not measured by team size

A solo developer can hold the only copy of an irreplaceable dataset. A large
company can hold disposable cache entries. DingoDB is built around the value
and lifetime of the data, not the size of the organization operating it.

### Durable truth must not depend on replaceable machinery

Indexes, catalogs, summaries, and placement maps make access fast, but they are
not the data's sole authority. They are designed to be rebuilt from immutable,
independently framed segments.

### Preserve first; interpret continuously

DingoDB can retain opaque or unfamiliar bytes without demanding a complete
schema or decoder up front. New software, indexes, and interpretations can be
added later without rewriting the original material.

### Failure must be reported honestly

DingoDB distinguishes complete values, partial evidence, missing extents,
unsupported formats, unavailable keys, and conflicting evidence. It does not
turn uncertainty into silent success, nor one damaged region into total loss.

### The data should outlive the stack

Applications, schemas, indexes, storage hardware, cloud providers, clusters,
and teams all change. DingoDB's job is to keep the data usable across those
changes.

## One data lifecycle

Conventional databases optimize for active use. Archives optimize for cheap
retention. DingoDB is designed so a project does not have to abandon one model
for the other as its data ages.

```text
one write
    │
    ├── use now ─────── collections · keys · filters · indexes · history
    │
    ├── retain ──────── hot · warm · cold · archive
    │
    ├── reinterpret ─── new decoders · new indexes · SDA examination
    │
    └── recover ─────── verified data islands · explicit holes · provenance
```

Immutable segments keep their identity as they move through storage tiers.
Physical placement can change while the logical data remains the same. An
object store can keep bytes; DingoDB is designed to keep those bytes
**database-usable**.

This is useful for projects of any size that accumulate data which may become
impossible, expensive, or unethical to reproduce: event histories, scientific
and device output, documents and media, AI datasets and artifacts, long-lived
application state, compliance evidence, and records whose future value is not
yet known.

## Recovery is a consequence of the promise

Damage recovery is not the product category. It is one of the proofs that
DingoDB takes irreplaceable data seriously.

A corrupt record, missing chunk, truncated segment, or lost catalog should not
invalidate unrelated healthy material. DingoDB scans for every surviving,
verified island and reports the gaps.

```text
┌─────────────────────────────────────────────────────────┐
│ DATA │ DATA │ █ HOLE █ │ DATA │ SCRATCH │ DATA │ DATA │
│  ✓   │  ✓   │    ✗     │  ✓   │    ✗    │  ✓   │  ✓   │
└─────────────────────────────────────────────────────────┘
```

The rule is deliberately unsentimental:

> What is gone is gone. What remains still lives.

`dingo doctor` inspects without modifying the source. `dingo salvage` writes
verified recovery evidence to a separate destination. `dingo export-live`
materializes only current complete state when that is what the operator wants.

[SDA](SDA_SPEC.md), the Structured Data Algebra, provides deterministic
filtering and transformation over both normal structured values and recovered
evidence:

> If DingoDB can recover it, SDA can examine it.

## Grow only as far as the project needs

DingoDB's intended progression is additive:

```text
embedded file
    └── remote service
          └── partitioned cluster
                ├── hot / warm / cold / archive tiers
                ├── scoped transactions
                └── jurisdiction-aware placement
```

The beginning should remain recognizable at every stage: collections, keys,
JSON, bytes, filters, indexes, and history. Operational machinery grows around
the data model instead of replacing it.

Transactions are deliberately scoped rather than presented as universal
distributed ACID. Jurisdiction is designed as enforceable placement and
movement policy rather than descriptive metadata. Both are currently design
proposals, not completed product claims:

- [Scoped transaction proposal](TRANSACTIONS.md)
- [Jurisdiction and sovereign placement proposal](JURISDICTION_PROPOSAL.md)

## Where DingoDB fits

DingoDB does not try to replace every database.

- Use a relational database when joins, relational constraints, and general
  multi-table transactions define the workload.
- Use an in-memory cache when the data is intentionally ephemeral.
- Use a plain object store when preserving independent objects is sufficient.
- Use a warehouse when curated analytical tables are the product.
- Use DingoDB when starting simply matters, but losing the accumulated data
  would eventually be unacceptable.

The concise answer is:

> We use DingoDB because the project was easy to start, and the data became too
> important to lose.

## How the design supports the doctrine

DingoDB separates the interface an application needs from the machinery that
keeps its data viable:

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

The workspace is split along those responsibilities:

- `dingo-format` — survival wire format, integrity, and salvage scanning;
- `dingo-store` — single-node append store, history, chunks, and tiers;
- `dingo-sdk` — collection API for embedded, remote, and cluster backends;
- `dingo-examine` — recovered evidence projected into SDA values;
- `dingo-cli` — everyday and operator commands;
- `dingo-cluster` — partition, coverage, consensus, and rebalance model;
- `sda-lib` / `sda` — the pure SDA library and CLI.

## Current maturity

DingoDB is under active development. It is **not yet production-ready** as a
network database or distributed storage system. The doctrine describes the
design standard; it does not erase the distance between the current
implementation and that standard.

- **Embedded single-node:** experimental / early access; strongest current path
- **Single-node TCP server:** development only
- **In-process cluster:** deterministic integration-test harness
- **Network `serve-cluster`:** experimental multi-process Raft (control plane
  DEF-036 + data-plane commit DEF-037); still not production-ready
- **S3/GCS:** filesystem-mirror adapter, not native cloud I/O
- **Lifecycle and erasure coding:** scaffolds
- **Wire format:** `1.0-draft`, not frozen

Network quorum commit exists on the experimental path when Raft attaches, but
do **not** treat it as a production release: distributed query completeness,
Jepsen-style verification, and other §16 gates remain open (DEF-040+; durable
rebalance is DEF-038; in-process anti-entropy repair is DEF-039).

Progress is tracked openly:

- [Capability matrix](doc/CAPABILITY_MATRIX.md)
- [Production-readiness work](DEFECTS.md)
- [Release artifacts (DEF-003)](doc/RELEASE_ARTIFACTS.md)
- [Benchmark disclosure requirements](doc/BENCHMARK_DISCLOSURE.md)
- [Delivery plan](DELIVERY_PLAN.md)

## Specifications and development

The implementation is backed by public specifications so that durable data
does not depend on private institutional memory:

- [System architecture](OVERVIEW.md)
- [Survival wire format](FORMAT_SPEC.md)
- [Developer experience](DX_SPEC.md)
- [Cluster architecture](CLUSTER_SPEC.md)
- [Structured Data Algebra](SDA_SPEC.md)
- [DingoDB SDA profile](SDA_PROFILE.md)
- [Crate map](ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)
- [Human demos](scripts/demos/)

Run the workspace:

```sh
cargo test --workspace
cargo run -p sda --bin sda -- eval -e '1 + 2'
cargo run -p dingo --bin dingo -- --help
```

## License

DingoDB is **multi-licensed** (not uniform MIT). MIT was temporary scaffolding
during project setup. Adopted policy: [doc/LICENSING.md](doc/LICENSING.md).

| Tier | SPDX | Crates (today) |
|------|------|----------------|
| Permissive | MIT | `sda-lib`, `sda`, `dingo-format` |
| Weak copyleft | MPL-2.0 | `dingo-store`, `dingo-examine`, `dingo-sdk` (default; optional `cluster` feature pulls AGPL) |
| Network copyleft | AGPL-3.0-or-later | `dingo-cluster`, `dingo-server`, `dingo-cli` |

Full texts: [LICENSE-MIT](LICENSE-MIT), [LICENSE-MPL-2.0](LICENSE-MPL-2.0),
[LICENSE-AGPL-3.0](LICENSE-AGPL-3.0). Overview: [LICENSE](LICENSE).

The storage formats and specifications are intended to remain open, documented,
and implementable without a proprietary service. Data that cannot be replaced
should not be held hostage by the software that first wrote it.
