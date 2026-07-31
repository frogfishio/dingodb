---
last_verified: 2026-07-30
claim_ids:
title: Format migration
description: Phased format migration with evidence-preserving copy.
class: operation
status: available
section: operations
order: 7
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - migration
---

## Outcome

Run preflight/plan/apply/verify for `residiuum-migrate-v1` on a **copy**.

## Risk level

High if pointed at production without backup.

## Procedure

```bash
residiuum migrate "$STORE" --preflight
residiuum migrate "$STORE" --plan-only
# apply only after reviewing plan on non-production data
```

## Verification

Source remains readable on failed migration; verify phase checks hashes.

## Rollback

Incomplete apply can roll back per CLI; source never in-place rewritten.

## Related evidence

DEF-052
