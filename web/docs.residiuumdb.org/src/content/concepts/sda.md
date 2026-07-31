---
last_verified: 2026-07-30
claim_ids:
title: SDA concept
description: Structured Data Algebra for deterministic examination.
class: concept
status: experimental
section: concepts
order: 6
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/reference/product/USP.md
owners:
  - docs
keywords:
  - sda
---

SDA is a small pure algebra for filtering, projecting, and validating JSON-like values and recovery evidence. Implementation: `crates/sda-core` (MIT).

It can represent verified data, partial data, holes, and uncertainty without loading an entire store into memory.

[Reference](/reference/sda/) · [Specification materials](/specifications/sda/)
