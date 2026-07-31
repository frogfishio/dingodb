---
last_verified: 2026-07-30
claim_ids:
title: Capabilities
description: User-oriented capability matrix for Residiuum 0.2.
class: status
status: experimental
section: status
order: 1
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: doc/wip/status/CAPABILITY_MATRIX.md
owners:
  - release
keywords:
  - capabilities
  - matrix
---

Generated from structured docs data checked against `doc/wip/status/CAPABILITY_MATRIX.md`.

## Deployment profiles

| Profile | Status | Note |
|---------|--------|------|
| Embedded single-node | Experimental | Strongest path / early access |
| Single-node TCP | Development only | Not a production claim |
| In-process cluster | Development only | Test harness |
| Network multi-node | Experimental | **Not production** |
| S3/GCS placement | Experimental | Filesystem mirror |
| Erasure / lifecycle | Scaffold | Not protection |

## Embedded surfaces (summary)

| Capability | Status |
|------------|--------|
| Open/create, JSON/bytes CRUD | Experimental |
| Filters, secondary indexes, history | Experimental |
| Backup / verified restore | Available (single-node) |
| Scrub | Available (single-node) |
| Salvage / examination | Available |

## Design-only

Heaps, RRE, Atomics, DDA, Order Wavelets — **Design** unless the matrix gains implementation evidence.

Full repository matrix: https://github.com/frogfishio/dingodb/blob/main/doc/wip/status/CAPABILITY_MATRIX.md
