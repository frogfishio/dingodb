---
last_verified: 2026-07-30
claim_ids:
title: Health and metrics
description: Liveness, readiness, and metrics scrape RPCs.
class: operation
status: development-only
section: operations
order: 9
applies_to:
  product: 0.2
  surface: single-node-tcp
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - health
  - and
  - metrics
---

## Outcome

Probe health and metrics endpoints/RPCs on a development server.

## Risk level

Low.

## Procedure

Use health_live / health_ready / metrics RPCs per protocol profile.

## Verification

Liveness succeeds when process up; readiness reflects store open state.

## Related evidence

DEF-061
