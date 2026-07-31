---
last_verified: 2026-07-30
claim_ids:
title: Wire reference
description: Frame and RPC wire labels.
class: reference
status: experimental
section: reference
order: 8
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
owners:
  - sdk
keywords:
  - wire
---

| Label | Value | Note |
|-------|-------|------|
| WIRE_PROFILE_LABEL | 1.0-draft | Not interoperability freeze |
| RPC_WIRE_LABEL | 1.0-draft | Network RPC draft |
| Protocol | dingo-rpc-v1 | Framed length + JSON |

See FORMAT_SPEC and protocol fixtures under `crates/residiuum-sdk/tests/fixtures`.
