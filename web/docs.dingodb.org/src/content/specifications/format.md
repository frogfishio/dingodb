---
last_verified: 2026-07-30
claim_ids:
title: Format specification
description: On-disk frames, segments, integrity.
class: specification
status: experimental
section: specifications
order: 1
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: FORMAT_SPEC.md
owners:
  - specs
keywords:
  - format
spec_state: draft
---

## Document state

**Draft** wire profile (`1.0-draft`) — not an interoperability freeze.

## Product capability

Implemented store format with draft labels; see capability matrix.

## Source

- Path: `FORMAT_SPEC.md`
- Raw: https://github.com/frogfishio/dingodb/blob/main/FORMAT_SPEC.md
- Companions: `crates/residuum-format`

## Summary

Authoritative self-verifying records, segment layout, integrity hashes, and scan/salvage implications. Implementation evidence in format tests and store stage suites.
