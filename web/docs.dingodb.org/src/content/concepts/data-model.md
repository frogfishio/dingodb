---
last_verified: 2026-07-30
claim_ids:
title: Data model
description: Stores, collections, keys, JSON, and bytes.
class: concept
status: experimental
section: concepts
order: 1
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: USP.md
owners:
  - docs
keywords:
  - data-model
---

An application opens a **store** root, names a **collection**, and addresses values by **key** (subject).

- JSON documents support filters, indexes, and dialects
- Opaque bytes are first-class
- History retains prior versions subject to retention/compaction policy

Authority is in self-verifying storage units (frames/segments). Application APIs hide that machinery until recovery time.
