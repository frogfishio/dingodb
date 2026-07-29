---
last_verified: 2026-07-30
claim_ids:
title: Pagination and ordering
description: Cursors, pages, deterministic order, and continuation limits.
class: how-to
status: experimental
section: guides
order: 7
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/dingo-sdk/README.md
owners:
  - sdk
keywords:
  - pagination
  - and
  - ordering
---

## Current APIs

Embedded paths support paged scans and filter scans with continuation tokens. Tokens are integrity-tagged and bound to store identity/generation.

## Requirements

- Deterministic subject-ascending order for stable paging
- Stale tokens fail closed (`CursorStale` / invalid)
- Partial coverage must not look like empty complete success

## DEF-097

Continuation-token **authentication** remains open. Do **not** treat tokens as attacker-authenticated secrets. See [known limitations](/status/known-limitations/).

## Design-only

Direct Access (DDA) and Order Wavelets are **design** until implementation evidence changes. See [specifications](/specifications/).
