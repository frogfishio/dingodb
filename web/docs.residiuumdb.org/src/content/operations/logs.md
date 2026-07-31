---
last_verified: 2026-07-30
claim_ids:
title: Structured logs
description: NDJSON process logs and correlation fields.
class: operation
status: development-only
section: operations
order: 8
applies_to:
  product: 0.2
  surface: single-node-tcp
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - logs
---

## Outcome

Locate and read structured process logs (`dingo-log-v1`).

## Risk level

Low.

## Procedure

Configure logging via process config; collect NDJSON for support.

## Verification

Required correlation fields present on request paths.

## Related evidence

DEF-060
