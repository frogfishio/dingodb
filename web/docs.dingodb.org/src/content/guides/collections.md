---
last_verified: 2026-07-30
claim_ids:
title: Collections
description: Open collections, put/get/delete, and naming rules.
class: how-to
status: experimental
section: guides
order: 1
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - collections
---

## Outcome

Create and use a named collection for application keys.

```rust
let mut db = Dingo::open("./app.dingo")?;
let mut users = db.collection("users")?;
users.put("k1", &json!({"v": 1}))?;
let got = users.get("k1")?;
users.delete("k1")?;
```

Collections are application-level namespaces. Authority lives in self-verifying storage units underneath; catalogs are derived.

See [Rust SDK reference](/reference/rust-sdk/) and [data model](/concepts/data-model/).
