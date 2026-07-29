---
last_verified: 2026-07-30
claim_ids:
title: Raw SDA
description: Deterministic examination algebra for filters and recovery evidence.
class: how-to
status: experimental
section: guides
order: 5
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - raw
  - sda
---

SDA (Structured Data Algebra) evaluates pure expressions over JSON-like values and recovery evidence. It has **no** ambient IO in `sda-core`.

Use cases:

- Filter and project surviving material
- Represent verified / partial / hole evidence
- Power dialects (SQL mimicry, etc.) by compilation to SDA

Start with [SDA concept](/concepts/sda/) and [SDA reference](/reference/sda/). Normative materials live under `doc/SDA/` and `crates/sda-core`.
