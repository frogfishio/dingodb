---
last_verified: 2026-07-30
claim_ids:
title: Errors reference
description: How failures surface in SDK and CLI.
class: reference
status: experimental
section: reference
order: 4
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residuum-sdk/README.md
owners:
  - sdk
keywords:
  - errors
---

Errors fail closed on integrity and protocol violations. Examples:

- Corrupt control documents → recovery-oriented errors, not silent invent
- Cursor stale/invalid → explicit
- Resource limits → `resource_limit`
- Auth failures → generic authentication failure after lockout policy

Exact codes evolve with the crate; treat repository error types as authoritative.
