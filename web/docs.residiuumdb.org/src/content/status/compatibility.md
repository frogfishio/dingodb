---
last_verified: 2026-07-30
claim_ids:
title: Compatibility
description: Release and profile labels — not one collapsed version number.
class: status
status: experimental
section: status
order: 3
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - release
keywords:
  - compatibility
  - version
---

| Label | Value |
|-------|-------|
| Crate / workspace semver | 0.2.0 |
| SDK_API_VERSION | 1.0 |
| WIRE_PROFILE_LABEL | 1.0-draft |
| RPC_WIRE_LABEL | 1.0-draft |
| CLUSTER_PROFILE_VERSION | v1 |
| BACKUP_PROFILE | dingo-backup-v1 |
| SCRUB_PROFILE | dingo-scrub-v1 |
| MIGRATE_PROFILE | dingo-migrate-v1 |

These labels measure different things. Do not collapse them into a single “version” claim.
