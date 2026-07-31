---
last_verified: 2026-07-30
claim_ids:
title: Authoritative and derived state
description: Segments are authority; catalogs and indexes are rebuildable accelerators.
class: concept
status: available
section: concepts
order: 2
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: USP.md
owners:
  - docs
keywords:
  - authoritative-and-derived
---

## Authoritative

Immutable, integrity-checked storage units hold the durable event history.

## Derived

Primary indexes, secondary indexes, and catalogs accelerate reads. They can lag or be rebuilt. Salvage and open recovery must not require them as the only map.

This separation is the basis of catalog-independent salvage.
