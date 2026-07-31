---
last_verified: 2026-07-30
claim_ids:
title: Rust quickstart
description: Create an embedded Residiuum store, put JSON, filter, and reopen from a clean directory.
class: tutorial
status: experimental
section: getting-started
order: 2
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - rust
  - sdk
  - quickstart
  - embedded
---

This walkthrough starts from an empty directory and ends with a store you can reopen. Maturity: **experimental / early access** on embedded single-node. See [limitations](/status/known-limitations/).

## Prerequisites

- Rust toolchain (MSRV target: **1.75+**; use a current stable release)
- Network access once to fetch crates
- A scratch directory you can delete

## 1. Create a project

```bash
cargo new dingodb-quickstart
cd dingodb-quickstart
```

## 2. Add the dependency

```toml
# Cargo.toml
[dependencies]
residiuum-sdk = "0.2"
```

**Licensing:** default features are **MPL-2.0** (embedded + remote client). Enabling `features = ["cluster"]` pulls **AGPL-3.0-or-later** `residiuum-cluster`. See [project licensing](https://residuumdb.org/project/#licensing).

## 3. Complete program

Replace `src/main.rs` with:

```rust
// tested: repository SDK examples / stage suites exercise this surface
use residiuum_sdk::{json, Residiuum, Filter};

fn main() -> Result<(), residiuum_sdk::Error> {
    let mut db = Residiuum::open("./app.dingo")?;
    {
        let mut users = db.collection("users")?;
        users.put(
            "user-42",
            &json!({ "name": "Alice", "status": "active" }),
        )?;

        let active = users.find(&Filter::field("status").eq("active"))?;
        println!("active rows: {}", active.len());
        for row in &active {
            println!("  {} -> {:?}", row.key, row.value);
        }
    }
    // Dropping scopes releases the exclusive writer; reopen proves persistence.
    let mut db2 = Residiuum::open("./app.dingo")?;
    let users = db2.collection("users")?;
    let again = users.get("user-42")?;
    println!("reopened user-42: {:?}", again);
    Ok(())
}
```

## 4. First run

```bash
cargo run
```

**Expected output (shape):** at least one active row printed, then a reopened value for `user-42`. Exact `Debug` formatting may vary.

## 5. Second run

Run `cargo run` again. The store directory already exists; the put overwrites the same key and find still returns active rows. Persistence is under `./app.dingo/` (segment files and control metadata).

## 6. Where the store lives

The path passed to `Residiuum::open` is a **directory** tree managed by `residiuum-store`. Do not hand-edit files inside it. For inspection, prefer CLI `doctor` / `scrub` / salvage tools.

## 7. Durability note

Default durability mode depends on open options. For production-shaped experiments, read [Durability](/operations/durability/) and [Receipts](/concepts/durability-and-receipts/) before trusting process crash behavior. Memory vs buffered vs durable acknowledgements are **not** interchangeable.

## 8. Cleanup

Only after you no longer need the data:

```bash
rm -rf ./app.dingo
```

## Next

- [First collection](/getting-started/first-collection/)
- [Durability](/operations/durability/)
- [Backup and restore](/guides/backup-and-restore/)
- [Scrub and salvage](/guides/scrub-and-salvage/)
