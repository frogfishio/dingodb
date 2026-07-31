---
last_verified: 2026-07-30
claim_ids:
title: Benchmark disclosure
description: Rules for publishing performance numbers; no comparative result claimed.
class: status
status: available
section: status
order: 4
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: doc/reference/operations/BENCHMARK_DISCLOSURE.md
owners:
  - release
keywords:
  - benchmarks
---

## Status

**No public comparative benchmark result is currently claimed.**

Methodology checklist lives in `doc/reference/operations/BENCHMARK_DISCLOSURE.md`. Required fields include version/git SHA, durability mode, hardware, dataset, concurrency, warm/cold state, latency percentiles, and competing system configuration if any comparison is made.

Design language such as “designed for a fast hot path” is not a measured ranking.
