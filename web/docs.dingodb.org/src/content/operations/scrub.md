---
last_verified: 2026-07-30
claim_ids:
title: Integrity scrub
description: Bounded scrub with findings quarantine.
class: operation
status: available
section: operations
order: 5
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - scrub
---

## Outcome

Run a bounded scrub and inspect findings.

## Risk level

Low–medium (read-heavy; quarantine copies corrupt targets, does not delete originals).

## Prerequisites

Store path you control (prefer temp).

## Procedure

```bash
dingo scrub "$STORE" --status
# scrub_once / pause / resume per CLI help
```

## Verification

Clean store reports no open findings; metrics show coverage age.

## Rollback

N/A for read-only scrub; quarantine is additive.

## Related evidence

DEF-051
