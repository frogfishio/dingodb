---
last_verified: 2026-07-30
claim_ids:
title: Security operations
description: TLS, authz, admission, and reporting path.
class: operation
status: development-only
section: operations
order: 10
applies_to:
  product: 0.2
  surface: single-node-tcp
source:
  path: doc/CAPABILITY_MATRIX.md
owners:
  - ops
keywords:
  - security
---

## Outcome

Configure TLS/auth for development and know how to report vulnerabilities.

## Risk level

High if exposing plaintext publicly.

## Procedure

1. Default loopback plaintext only for local dev.
2. Use TLS 1.3 / mTLS for non-loopback.
3. Prefer auth tokens + privilege sets; audit chain on deny/sensitive allow.
4. Report vulns via GitHub security — **not** public issues with exploit details.

## Verification

Public plaintext bind refused without override; TLS handshakes succeed with expected identity.

## Related evidence

DEF-032–034, `doc/THREAT_MODEL.md` (draft)

Product page: [dingodb.org/security](https://dingodb.org/security/)
