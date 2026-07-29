---
last_verified: 2026-07-30
claim_ids:
title: Backup and restore
description: Full package backup and verified restore (single-node).
class: operation
status: available
section: operations
order: 4
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - backup
  - restore
---

## Outcome

Produce a `dingo-backup-v1` package and restore it.

## Risk level

Medium — restore can overwrite a destination root.

## Prerequisites

- Temporary source store you created for the exercise
- Destination path that does not hold irreplaceable data

## Pre-flight checks

```bash
WORKDIR="$(mktemp -d /tmp/dingo-br-XXXXXX)"
# ensure WORKDIR is the temp path printed above before continuing
```

## Procedure

```bash
# After creating data under $WORKDIR/store ...
dingo backup "$WORKDIR/store" --output "$WORKDIR/backup"
dingo restore "$WORKDIR/backup" --output "$WORKDIR/restored"
```

Identity-preserving restore is default; clone restore uses reassign-identity options in CLI help.

## Verification

Open restored store; get known keys; confirm manifest hashes if inspecting package.

## Rollback

Keep original source untouched (backup is read of source). Delete failed destination and re-restore.

## Failure meanings

Hash mismatch, incomplete package, destination not empty — fail closed.

## Related evidence

DEF-050, `stage_def_050_backup`
