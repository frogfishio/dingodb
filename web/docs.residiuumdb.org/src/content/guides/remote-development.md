---
last_verified: 2026-07-30
claim_ids:
title: Remote development
description: Connect to a development-only single-node server.
class: how-to
status: experimental
section: guides
order: 12
applies_to:
  product: 0.2
  surface: single-node-tcp
source:
  path: crates/residuum-sdk/README.md
owners:
  - sdk
keywords:
  - remote
  - development
---

## Maturity

Single-node TCP (`residuum serve`) is **development only**. It is not a production deployment claim.

```rust
use residuum_sdk::{ConnectOptions, Residuum};
let mut db = Residuum::connect("residuum://127.0.0.1:7434/app")?;
```

Prefer loopback binds. Public plaintext binds are refused without explicit insecure override. TLS/mTLS paths exist—see [security operations](/operations/security/).

Cluster multi-node: [Clustering](/operations/clustering/) (**experimental, not production**).
