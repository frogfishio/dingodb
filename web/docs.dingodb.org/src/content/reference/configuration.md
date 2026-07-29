---
last_verified: 2026-07-30
claim_ids:
title: Configuration reference
description: dingo-config-v1 process configuration.
class: reference
status: experimental
section: reference
order: 3
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - configuration
---

Profile: `dingo-config-v1`.

- Layering: defaults < file < env secrets < CLI
- Setting classes: static / restart-required / dynamic
- Secrets via env/file refs only
- Validate before serve

Source: `crates/dingo-server` config module; DEF-054.
