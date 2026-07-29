---
last_verified: 2026-07-30
claim_ids:
title: Durability modes
description: Choose acknowledgement modes deliberately.
class: operation
status: experimental
section: operations
order: 3
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - durability
---

## Outcome

Select memory / buffered / durable (and replicated where applicable) with honest expectations.

## Risk level

High if you assume durable semantics while using memory acks.

## Procedure

1. Read SDK open options for durability mode.
2. Match benchmark disclosure fields if measuring.
3. On remote paths, require server-proved receipts.

## Verification

Crash/restart tests on a **temporary** store; confirm expected survival for the chosen mode.

## Rollback

N/A for mode selection; restore from backup if data loss under weaker modes.
