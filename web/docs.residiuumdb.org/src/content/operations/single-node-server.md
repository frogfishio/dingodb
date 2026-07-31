---
last_verified: 2026-07-30
claim_ids:
title: Single-node server
description: Run residiuum serve for development.
class: operation
status: development-only
section: operations
order: 11
applies_to:
  product: 0.2
  surface: single-node-tcp
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - single
  - node
  - server
---

## Outcome

Serve one local store over TCP for development clients.

## Maturity

**Development only** — not a production claim.

## Risk level

Medium (network exposure).

## Procedure

```bash
residiuum serve ./app.residiuum
# optional: --token, TLS flags per CLI help
```

## Verification

SDK `Residiuum::connect` put/get against loopback.

## Rollback

Stop process; store files remain.
