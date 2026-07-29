---
last_verified: 2026-07-30
claim_ids:
title: Known limitations
description: Material user-visible limitations for DingoDB 0.2.
class: status
status: experimental
section: status
order: 2
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - release
keywords:
  - limitations
---

- DingoDB is experimental software. No production-supported release line is claimed.
- Experimental network cluster is not a production deployment path.
- S3/GCS locators are filesystem mirrors, not native cloud object-store connectors.
- `WIRE_PROFILE_LABEL = 1.0-draft` is not an interoperability freeze.
- No public comparative performance result is currently claimed.
- Continuation-token authentication (DEF-097) remains open; do not claim authenticated continuation tokens.
- Damage tolerance reduces blast radius; missing or overwritten bytes are not recoverable.
- Multi-process Jepsen-style histories and long soak remain follow-on work.
- Formal models are not implementation proofs of the full product.

Defect-program prose is not dumped into primary navigation; see public matrix and DEF notes in repository where linked.
