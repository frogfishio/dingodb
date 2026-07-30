---
last_verified: 2026-07-30
claim_ids:
title: CLI reference
description: dingo operator CLI command map.
class: reference
status: experimental
section: reference
order: 2
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - cli
---

Binary: `dingo` (AGPL networked product tier for serve paths).

Common commands (see `dingo --help` for flags):

| Command | Purpose |
|---------|---------|
| `console` | Interactive/scripted RQL-ish console |
| `serve` | Single-node TCP (development only) |
| `serve-cluster` | Experimental multi-node |
| `backup` / `restore` | Package backup |
| `scrub` | Integrity scrub |
| `salvage` | Evidence salvage |
| `migrate` | Format migration |
| `doctor` | Inspection |
| `config validate\|show` | Config |

Generate flag tables from `--help` in CI to avoid drift (launch: consult binary help as source of truth).
