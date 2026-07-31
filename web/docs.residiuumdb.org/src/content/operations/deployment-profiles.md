---
last_verified: 2026-07-30
claim_ids:
title: Deployment profiles
description: Supported ways to run Residiuum and their maturity.
class: operation
status: experimental
section: operations
order: 1
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - deployment
  - profiles
---

## Outcome

Identify which profile matches your intent and its public maturity label.

## Applies to / maturity

All profiles; labels from `doc/wip/status/CAPABILITY_MATRIX.md`.

## Risk level

Low (read-only evaluation of labels). Choosing network multi-node for production is **high risk** and unsupported.

## Profiles

| Profile | How | Status |
|---------|-----|--------|
| Embedded single-node | `Residiuum::open` | Experimental / early access |
| Single-node TCP | `residuum serve` | Development only |
| In-process cluster | `open_cluster` | Integration-test harness |
| Network multi-node | `serve-cluster` | Experimental — **not production** |
| S3/GCS placement | Mirror roots | Experimental mirror |
| Erasure / lifecycle | Scaffold | Scaffold |

## Verification

Cross-check [status/capabilities](/status/capabilities/) against repository matrix before release claims.

## Rollback

N/A — informational.

## Related evidence

`doc/wip/status/CAPABILITY_MATRIX.md`
