---
last_verified: 2026-07-30
claim_ids:
title: Choose Residiuum
description: When to evaluate Residiuum and when to pick a mature alternative.
class: tutorial
status: experimental
section: getting-started
order: 1
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: USP.md
owners:
  - product
keywords:
  - fit
  - evaluation
---

## Good fit to evaluate

- Embedded Rust applications needing local durable storage
- Irreplaceable local records and blobs where partial survival matters
- Systems that want inspectable recovery (holes, salvage, coverage)
- Research workloads exploring formal examination (SDA)

## Not yet

- Drop-in mature SQL database
- Production MongoDB replacement
- Production multi-node cluster
- Native object-store archive platform
- Public multi-tenant hosted service

## Decision table

| Need | Direction |
|------|-----------|
| Mature SQL + tooling | PostgreSQL or SQLite |
| Mature document ecosystem | MongoDB or similar |
| Embedded arbitrary data with explicit damage model | Evaluate Residiuum |
| Production network cluster today | Do not choose Residiuum yet |

Product homepage: [residiuumdb.org](https://residiuumdb.org/use-cases/).
