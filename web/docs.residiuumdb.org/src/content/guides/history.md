---
last_verified: 2026-07-30
claim_ids:
title: History
description: Inspect per-key revision history.
class: how-to
status: experimental
section: guides
order: 8
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - history
---

```rust
let hist = users.history("user-42")?;
```

History interacts with compaction and reclaim policies. Compaction that allows history loss is an explicit operator choice—see store/compaction docs in the repository.

Related: [Backup](/guides/backup-and-restore/) · [Salvage](/guides/scrub-and-salvage/)
