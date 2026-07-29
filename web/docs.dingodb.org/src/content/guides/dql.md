---
last_verified: 2026-07-30
claim_ids:
title: DQL guide
description: Human query language surface for collections.
class: how-to
status: experimental
section: guides
order: 4
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - dql
---

Source: `doc/DQL/USER_GUIDE.md`.

DQL is the official human query surface. The **implemented** surface may be smaller than the full DQL v1 design document—do not assume every design clause is available.

## Orientation

- Prefer the user guide in the repository for exact syntax that works today
- For enrichment/join designs, check capability status before coding
- Raw SDA remains available for deterministic examination

References: [DQL concept](/concepts/dql/) · [DQL reference](/reference/dql/) · [DQL specification](/specifications/dql/)
