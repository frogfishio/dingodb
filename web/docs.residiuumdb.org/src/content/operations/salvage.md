---
last_verified: 2026-07-30
claim_ids:
title: Salvage
description: Catalog-independent evidence recovery of verified frames.
class: operation
status: available
section: operations
order: 6
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - salvage
---

## Outcome

Salvage verified frames from a damaged **copy** into an output store.

## Risk level

Medium — wrong paths waste time; never salvage “in place” over the only copy.

## Prerequisites

- Source is disposable or a copy
- Output directory empty/new

## Pre-flight checks

Print and confirm absolute paths:

```bash
SRC="$(mktemp -d /tmp/dingo-src-XXXXXX)"
DST="$(mktemp -d /tmp/dingo-dst-XXXXXX)"
echo "SRC=$SRC DST=$DST"
```

## Procedure

```bash
dingo salvage "$SRC" --output "$DST"
```

## Verification

Inspect salvage manifest under recovery/; holes listed explicitly; verified frames readable.

## Rollback

Source is not mutated. Discard DST if unsatisfied.

## Failure meanings

Unreadable media, empty survivors — report holes; no invented bytes.

## Related evidence

DEF-011
