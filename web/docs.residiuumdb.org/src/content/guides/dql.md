---
last_verified: 2026-07-30
claim_ids:
title: RQL guide
description: Human query language surface for collections.
class: how-to
status: experimental
section: guides
order: 4
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residuum-sdk/README.md
owners:
  - sdk
keywords:
  - dql
---

Source: `doc/RQL/USER_GUIDE.md`.

RQL is the official human query surface. The **implemented** surface may be smaller than the full RQL v1 design document—do not assume every design clause is available.

## Orientation

- Prefer the user guide in the repository for exact syntax that works today
- For enrichment/join designs, check capability status before coding
- Raw SDA remains available for deterministic examination

References: [RQL concept](/concepts/dql/) · [RQL reference](/reference/dql/) · [RQL specification](/specifications/dql/)
