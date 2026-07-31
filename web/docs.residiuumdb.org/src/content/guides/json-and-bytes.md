---
last_verified: 2026-07-30
claim_ids:
title: JSON and bytes
description: Store JSON documents or opaque byte payloads.
class: how-to
status: experimental
section: guides
order: 2
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - json
  - and
  - bytes
---

## JSON

JSON values enable filters, secondary indexes, and dialect queries.

```rust
users.put("user-42", &json!({ "name": "Alice", "status": "active" }))?;
```

## Opaque bytes

Bytes remain first-class durable material. Do not force every payload into a document shape.

Prefer the SDK byte APIs documented in `crates/residiuum-sdk` for binary objects, then examine survivors with salvage/SDA when needed.
