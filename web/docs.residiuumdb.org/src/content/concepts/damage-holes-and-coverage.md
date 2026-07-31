---
last_verified: 2026-07-30
claim_ids:
title: Damage, holes, and coverage
description: How Residiuum reports damage without invalidating healthy neighbors.
class: concept
status: available
section: concepts
order: 3
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: USP.md
owners:
  - docs
keywords:
  - damage-holes-and-coverage
---

## Ordinary failure model

Critical damage → whole database unavailable even if healthy bytes remain.

## Residiuum model

Missing or unreadable material becomes an explicit **hole**. Neighbors can still verify.

## Coverage

Query and scan results should report what was searched. Incomplete tier/partition coverage must not look like empty success.

## Limits

Damage tolerance reduces blast radius. It does **not** recover overwritten or never-written bytes.

Product narrative: [residiuumdb.org/survival](https://residiuumdb.org/survival/).
