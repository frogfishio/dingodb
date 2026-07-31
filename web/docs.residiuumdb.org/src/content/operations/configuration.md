---
last_verified: 2026-07-30
claim_ids:
title: Configuration
description: Validate process configuration before serve.
class: operation
status: development-only
section: operations
order: 2
applies_to:
  product: 0.2
  surface: single-node-tcp
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - configuration
---

## Outcome

Validate a `residiuum-config-v1` document before serving.

## Applies to / maturity

Development-only server path primarily; config tooling is shipped for process validation.

## Risk level

Medium if mis-binding network interfaces.

## Prerequisites

- Built `residiuum` CLI
- Config file under your control

## Pre-flight checks

```bash
residiuum config validate --config ./residiuum.json
residiuum config show --config ./residiuum.json
```

## Procedure

1. Prefer defaults < file < env secrets < CLI flags layering.
2. Never inline secrets; use env/file refs.
3. Refuse unsafe combos (e.g. public plaintext without override).

## Verification

`validate` exits 0; show output redacts secrets.

## Rollback

Fix config and re-validate; do not serve invalid config.

## Failure meanings

Unsafe bind, missing secrets, replication claim below policy — see CLI error text.

## Related evidence

DEF-054, `stage_def_054_config`
