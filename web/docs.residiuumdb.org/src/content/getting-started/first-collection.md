---
last_verified: 2026-07-30
claim_ids:
title: First collection
description: Collections, keys, JSON documents, and opaque bytes.
class: tutorial
status: experimental
section: getting-started
order: 4
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residuum-sdk/README.md
owners:
  - sdk
keywords:
  - collection
  - put
  - get
  - bytes
---

A **collection** is a named namespace of keys (subjects). Values may be JSON or opaque bytes.

```rust
let mut users = db.collection("users")?;
users.put("user-42", &json!({ "name": "Alice" }))?;
let doc = users.get("user-42")?;
users.delete("user-42")?;
```

Bytes path (API names follow the SDK; see [JSON and bytes](/guides/json-and-bytes/)):

```rust
// Opaque payloads remain first-class without forced document interpretation.
users.put_bytes("blob-1", b"\x00\x01raw")?;
```

Keys are application-defined strings. History, indexes, and filters apply primarily to the JSON surface.

Related: [Collections guide](/guides/collections/) · [Data model](/concepts/data-model/)
