---
last_verified: 2026-07-30
claim_ids:
title: Clustering
description: Experimental network multi-node — not production.
class: operation
status: experimental
section: operations
order: 12
applies_to:
  product: 0.2
  surface: network-multi-node
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - clustering
---

## Outcome

Understand experimental cluster flags and why this is **not** production.

## Maturity

**Experimental.** Requires `--experimental-network-cluster`. Multi-process Jepsen/long soak still open.

## Risk level

**Critical** if used as a production system of record.

## Pre-flight checks

Read [known limitations](/status/known-limitations/) and capability matrix Raft sections.

## Procedure

Do **not** follow a production deployment runbook here — none is supported.

For lab evaluation only, see repository cluster tests (`stage_def_036`, `stage_def_037`) and CLI help for `serve-cluster`.

## Verification

Lab only: quorum commit tests in repository. No production SLO.

## Rollback

N/A — do not place irreplaceable data solely on experimental cluster.

## Related evidence

`doc/wip/status/CAPABILITY_MATRIX.md`, CLUSTER_SPEC.md
