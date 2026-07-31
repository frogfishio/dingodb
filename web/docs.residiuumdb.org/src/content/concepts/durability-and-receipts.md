---
last_verified: 2026-07-30
claim_ids:
title: Durability and receipts
description: Acknowledgement modes and honest commit evidence.
class: concept
status: experimental
section: concepts
order: 4
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: doc/reference/product/USP.md
owners:
  - docs
keywords:
  - durability-and-receipts
---

Writes acknowledge under explicit modes (for example memory, buffered, durable). Do not mix modes when comparing performance or planning crash behavior.

Remote receipts require server-proved `committed`, acknowledgement, and identity fields—missing fields fail closed rather than optimistic defaults.

See [operations/durability](/operations/durability/) and [receipts reference](/reference/receipts-and-evidence/).
