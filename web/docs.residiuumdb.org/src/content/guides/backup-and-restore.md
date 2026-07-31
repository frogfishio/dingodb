---
last_verified: 2026-07-30
claim_ids:
title: Backup and restore guide
description: Full single-node backup package and verified restore.
class: how-to
status: experimental
section: guides
order: 10
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - backup
  - and
  - restore
---

## Terms

| Term | Meaning |
|------|---------|
| Backup | Intentional package of a store |
| Restore | Materialize a package to a store root |
| Salvage | Evidence recovery from damaged media |
| Replication | Live copies under cluster policy |

## CLI sketch

Operate only on a **temporary** store you create for practice:

```bash
WORKDIR="$(mktemp -d /tmp/dingo-bak-XXXXXX)"
# ... populate store ...
dingo backup "$WORKDIR" --output "$WORKDIR/backup-out"
dingo restore "$WORKDIR/backup-out" --output "$WORKDIR/restored"
rm -rf "$WORKDIR"
```

Profile: `dingo-backup-v1`. Full operator procedure: [operations/backup-restore](/operations/backup-restore/).
