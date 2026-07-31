# Residiuum document control

Status: **authoritative document index**

This directory is organized by execution state. The location of a document is
part of its status; headings inside archived documents describe their state at
the time they were archived.

## Authority

There is exactly one execution authority:
[MASTER_DELIVERY_PLAN.md](../MASTER_DELIVERY_PLAN.md).

There is exactly one living package scoreboard:
[wip/status/NEXT_BUILD_STATUS.md](wip/status/NEXT_BUILD_STATUS.md).

Strategic summaries and older queues do not select work. They are retained
under [done/programs/](done/programs/) as history.

## State directories

| Directory | Meaning | May developers start it? |
|---|---|---|
| [todo/](todo/) | Specified, developer-ready work not yet accepted | Yes, only when the master plan and dependencies admit it |
| [wip/](wip/) | Active, partial, under review, or awaiting qualification | Continue the admitted package |
| [done/](done/) | Completed, superseded, or historical delivery material | No; consult as evidence/history |
| [reference/](reference/) | Durable doctrine, contracts, policies, and engineering reference | Not an execution queue |

Crate READMEs, public website content, and operator/user manuals remain beside
the code or publishing system they document.

## Current critical path

```text
DONE: emergency storage defects + Residiuum rebrand
  ↓
TODO NOW: Core Storage Qualification (CSQ-0…CSQ-12)
  ├── TODO NEXT: Performance Qualification Harness (PQH-0…PQH-9)
  │              first post-C0 measurement lane
  └── Application Baseline (APB-0…APB-12), alongside PQH
       ↓
      Heap Application Ready completion
  ↓
RRE → Atomics → relationships → Direct Access → Order Wavelets
```

The Core Storage pair is intentionally colocated:

- [qualification specification](todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md)
  defines the invariants, failure model, claims, and acceptance standard;
- [implementation plan](todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md)
  defines packages, dependencies, artifacts, and execution order.

They are two layers of one program, not competing plans.

## Movement rules

1. A document moves from `todo` to `wip` only when its package is admitted.
2. A document moves from `wip` to `done` only when its acceptance evidence is
   recorded in the scoreboard.
3. A stable contract moves to `reference`, not `done`, when implementations
   must continue obeying it.
4. Superseded plans move to `done`; they never remain beside the live queue.
5. Every move must update links and the scoreboard in the same change.
6. New execution plans must subordinate themselves to the master plan and may
   not create a second priority list.

## Product entry documents

- [README](../README.md) — product introduction and maturity.
- [Architecture](../ARCHITECTURE.md) — normative technical map.
- [Master delivery plan](../MASTER_DELIVERY_PLAN.md) — sole priority authority.
- [Contributing](../CONTRIBUTING.md) — engineering workflow.
- [Security](../SECURITY.md) — security policy and reporting.
