---
last_verified: 2026-07-30
claim_ids:
title: Scrub and salvage
description: Integrity scrub, quarantine, and catalog-independent salvage.
class: how-to
status: experimental
section: guides
order: 11
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - scrub
  - and
  - salvage
---

## Scrub

Bounded integrity verification with findings and optional quarantine of corrupt targets (never silent delete of originals).

## Salvage

Walk surviving frames; emit verified data and **holes**. Salvage does not invent missing bytes.

## Safety

Destructive demos MUST use a newly created temporary directory. Never paste paths that point at home directories or production data.

Operator procedures: [Scrub](/operations/scrub/) · [Salvage](/operations/salvage/). Concept: [Damage, holes, coverage](/concepts/damage-holes-and-coverage/).
