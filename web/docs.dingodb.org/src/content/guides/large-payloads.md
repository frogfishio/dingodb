---
last_verified: 2026-07-30
claim_ids:
title: Large payloads
description: Working with larger documents and blobs.
class: how-to
status: experimental
section: guides
order: 9
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - large
  - payloads
---

Prefer opaque bytes for large binary objects. Keep hot-path documents reasonably sized for memory-resident indexes.

Tiered placement (hot/warm/cold) and S3/GCS **mirrors** exist experimentally—they are not native cloud SDKs. See [deployment profiles](/operations/deployment-profiles/).

Resource budgets (query memory, admission) apply on server paths—see [resource/admission profiles](/status/compatibility/).
