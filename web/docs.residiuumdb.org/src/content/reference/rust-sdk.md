---
last_verified: 2026-07-30
claim_ids:
title: Rust SDK reference
description: Collection SDK entry points for embedded and remote use.
class: reference
status: experimental
section: reference
order: 1
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - rust-sdk
---

**Crate:** `residiuum-sdk` **0.2** · `SDK_API_VERSION` = `1.0` · Status: experimental embedded path.

## Install

```toml
residiuum-sdk = "0.2"
# optional AGPL cluster:
# residiuum-sdk = { version = "0.2", features = ["cluster"] }
```

## Core types

| Type | Role |
|------|------|
| `Residuum` | Open embedded / connect remote / optional cluster |
| `Collection` | put/get/delete/find/history/indexes |
| `Filter` | Typed JSON predicates |
| `QueryOptions` | Coverage and query budgets |

Authoritative examples: `crates/residiuum-sdk/README.md` and crate tests.
