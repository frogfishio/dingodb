---
last_verified: 2026-07-30
claim_ids:
title: Development
description: Engineering rules and workspace commands.
class: how-to
status: experimental
section: contributing
order: 2
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: CONTRIBUTING.md
owners:
  - maintainers
keywords:
  - development
---

## Rules (summary)

1. SDA stays pure (no ambient IO in sda-core)
2. Authority before acceleration
3. Damage honesty — holes explicit
4. Conformance gates over API surface alone
5. Cluster does not own the bytes

```bash
cargo test --workspace
cargo fmt --all -- --check
```

Full detail: https://github.com/frogfishio/dingodb/blob/main/CONTRIBUTING.md
