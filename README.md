# DingoDB

DingoDB is a deterministic relational document engine that lets developers build nested application artefacts directly from relational data, using explicit enrichment semantics instead of hidden joins and ORM hydration.

DingoDB is not trying to be another SQL replacement or another document database. It is built around a different idea:

- **ENR** relates artefacts without hiding multiplicity.
- **SDA** transforms artefacts with deterministic semantics.
- The storage engine preserves data long enough for those artefacts to remain useful.

The result is a database designed for applications that need both relational correctness and document-shaped output.

---

## The problem

Modern applications usually choose between two compromises.

Relational databases provide:

- strong relationships;
- mature indexing;
- transactional semantics.

But application developers often rebuild the final shape through:

```
database rows
    ↓
SQL joins
    ↓
ORM hydration
    ↓
application objects
    ↓
API JSON
```

Document databases provide convenient shapes, but relationships often move into application code.

DingoDB treats relationship formation and document construction as first-class database operations.

---

## See the model

Example:

```text
enrich customer using customers
  matching customer_id = id
  expect exactly_one

enrich items using items
  matching id = order_id
  expect many

enrich product using products
  matching product_id = id
  expect exactly_one

project {
  order_id,
  customer.name,
  items {
    quantity,
    product.name
  }
}
```

This is not a hidden join.

Each relationship declares:

- what is being added;
- what source provides it;
- how records relate;
- what multiplicity is valid.

The engine can reason about:

```
customer_id → customer
        exactly one

order_id → items
        many

product_id → product
        exactly one
```

Ambiguity is not silently converted into a result.

---

## ENR + SDA

DingoDB separates two operations.

### ENR: relationship formation

ENR answers:

> How does this artefact relate to another artefact?

The primitive operation is:

```
Match(l, R, kL, kR)
=
{ r ∈ R | kR(r) = kL(l) }
```

The result is always a match bag.

Multiplicity is preserved until explicitly interpreted:

```
expect exactly_one
expect optional
expect many
```

ENR does not perform acquisition, orchestration, authentication, or transport.

Those belong outside the language boundary.

### SDA: deterministic transformation

SDA answers:

> How should this artefact be reshaped?

SDA provides deterministic projection, normalization, filtering, and transformation over structured values.

See:

- [SDA specification](SDA_SPEC.md)
- [DingoDB SDA profile](SDA_PROFILE.md)

---

## Simple enough for a side project. Built for data that cannot be replaced.

DingoDB is embedded-first.

Open a file. Create a collection. Store JSON or bytes.

No server is required before the first write.

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

Install:

```sh
cargo add dingo-sdk
cargo install dingo-cli
```

CLI:

```sh
dingo put ./demo.dingo users/user-42 \
  --json '{"name":"Alice","status":"active"}'

dingo get ./demo.dingo users/user-42
dingo history ./demo.dingo users/user-42
dingo doctor ./demo.dingo
```

Collections are schemaless by default.

JSON and raw bytes are first-class.

---

## The doctrine

DingoDB is built around one principle:

> The importance of data is not measured by the size of the team holding it.

A solo developer may hold the only copy of a valuable dataset.

A large company may hold disposable cache entries.

DingoDB separates:

- application convenience;
- storage durability;
- interpretation;
- recovery.

---

## Preserve first. Interpret continuously.

Indexes, catalogs, summaries, and placement information accelerate access.

They are not the sole authority of the data.

The design goal:

```
immutable data
      |
      +-- rebuild indexes
      |
      +-- add new interpretations
      |
      +-- inspect with SDA
      |
      +-- recover surviving evidence
```

A dataset should not become unusable because the original application disappeared.

---

## Recovery is a consequence of the promise

Recovery is not the product category.

It is a demonstration of the storage philosophy.

A damaged region should not invalidate unrelated healthy data.

```
DATA | DATA | HOLE | DATA | SCRATCH | DATA
 ✓      ✓      ✗      ✓       ✗        ✓
```

DingoDB reports what exists and what does not.

It does not convert uncertainty into success.

Tools:

- `dingo doctor`
- `dingo salvage`
- `dingo export-live`
- `dingo backup`
- `dingo restore`
- `dingo scrub`
- `dingo migrate`

See:

- DEF-050
- DEF-051
- DEF-052
- DEF-054
- DEF-060
- DEF-061

---

## One data lifecycle

```
one write
    |
    +-- use now
    |      collections · keys · filters · indexes · history
    |
    +-- retain
    |      hot · warm · cold · archive
    |
    +-- reinterpret
    |      new indexes · new decoders · SDA examination
    |
    +-- recover
           verified data islands · provenance · explicit holes
```

Physical placement can change while logical identity remains.

---

## Where DingoDB fits

Use a relational database when:

- relational constraints define the workload;
- SQL is the primary interface;
- flat relational output is acceptable.

Use a document database when:

- isolated document retrieval dominates.

Use an object store when:

- preserving independent objects is enough.

Use DingoDB when:

- applications need nested artefacts;
- relationships matter;
- ambiguity must be explicit;
- data must remain usable over time.

The concise answer:

> DingoDB is for projects that start simple but cannot afford their data becoming disposable.

---

## Architecture

DingoDB separates application interfaces from storage machinery:

```
application
    |
    v
collection SDK
    |
    v
append store
    |
    v
immutable segments
    |
    +-- rebuildable indexes
    +-- SDA examination
    +-- storage tiers
    +-- recovery tooling
```

Workspace responsibilities:

- `dingo-format` — survival wire format, integrity, salvage scanning;
- `dingo-store` — single-node append store, history, chunks, tiers;
- `dingo-sdk` — collection API for embedded, remote, and cluster backends;
- `dingo-examine` — recovered evidence projected into SDA values;
- `dingo-cli` — everyday and operator commands;
- `dingo-cluster` — partition, coverage, consensus, and rebalance model;
- `dingo-sda` / `dingo-sda-cli` — SDA + ENR evaluator.

---

## Current maturity

DingoDB is under active development.

It is not yet production-ready as a distributed database.

Current status:

- **Embedded single-node:** experimental / early access; strongest current path
- **Single-node TCP server:** development only
- **In-process cluster:** deterministic integration harness
- **Network cluster:** experimental
- **Cloud adapters:** filesystem-mirror adapters
- **Wire format:** `1.0-draft`, not frozen

Progress:

- [Capability matrix](doc/CAPABILITY_MATRIX.md)
- [Production-readiness work](DEFECTS.md)
- [Prime-time plan](doc/PRIME_TIME_PLAN.md)
- [Work horizon](doc/WORK_HORIZON.md)
- [Release artifacts](doc/RELEASE_ARTIFACTS.md)
- [Benchmark disclosure](doc/BENCHMARK_DISCLOSURE.md)
- [Delivery plan](DELIVERY_PLAN.md)

---

## Specifications

The design is backed by public specifications:

- [System architecture](OVERVIEW.md)
- [Survival wire format](FORMAT_SPEC.md)
- [Developer experience](DX_SPEC.md)
- [Cluster architecture](CLUSTER_SPEC.md)
- [Structured Data Algebra](SDA_SPEC.md)
- [DingoDB SDA profile](SDA_PROFILE.md)
- [Crate map](ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)
- [Human demos](scripts/demos/)

Run:

```sh
cargo test --workspace
cargo run -p dingo-sda-cli --bin dingo-sda -- eval -e '1 + 2'
cargo run -p dingo --bin dingo -- --help
```

---

## License

DingoDB is multi-licensed.

See:

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-MPL-2.0](LICENSE-MPL-2.0)
- [LICENSE-AGPL-3.0](LICENSE-AGPL-3.0)
- [Licensing overview](doc/LICENSING.md)

The storage formats and specifications are intended to remain open, documented, and implementable without a proprietary service.

Data that cannot be replaced should not be held hostage by the software that first wrote it.
