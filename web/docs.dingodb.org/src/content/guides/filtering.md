---
last_verified: 2026-07-30
claim_ids:
title: Filtering
description: Find documents with Rust Filter builders and dialects.
class: how-to
status: experimental
section: guides
order: 3
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - filtering
---

## Rust Filter

```rust
use dingo_sdk::Filter;
let rows = users.find(&Filter::field("status").eq("active"))?;
```

## Dialects

Optional SQL / JSON / mongo-style strings compile to pure SDA:

```rust
let via_sql = users.find_dialect(
    "sql",
    "SELECT * WHERE status = 'active' AND age >= 18",
)?;
```

Dialects are convenience surfaces. Authoritative examination algebra is [SDA](/concepts/sda/).

Distinguish: Rust Filter · [DQL](/guides/dql/) · dialects · [raw SDA](/guides/raw-sda/).
