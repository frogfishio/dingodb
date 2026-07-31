---
last_verified: 2026-07-30
claim_ids:
title: Indexes
description: Secondary indexes as derived accelerators.
class: how-to
status: experimental
section: guides
order: 6
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residuum-sdk/README.md
owners:
  - sdk
keywords:
  - indexes
---

```rust
users.indexes()?.create("by-status", &["status"])?;
```

Indexes and catalogs are **derived**. They speed access; they are not the only map back to surviving data. If wiped, surviving segments can rebuild them.

Lifecycle states (building / ready / stale / …) are documented in the capability matrix. Misses are authoritative only when the index claims complete coverage.

Related: [Authoritative and derived](/concepts/authoritative-and-derived/)
