# Next build status

Status: program scoreboard

Source: [NEXT_BUILD_PLAN.md](../NEXT_BUILD_PLAN.md)

Updated: 2026-07-30

This file records delivery state. It does not change normative semantics.

Allowed states:

```text
not_started | active | blocked | accept | deferred
```

| Package | State | Owner | Source revision | Evidence | Open defects | Capability impact |
|---|---|---|---|---|---|---|
| HAR-0 | not_started | unassigned | — | — | stale Heap qualification truth/check | none |
| HAR-1 | not_started | unassigned | — | — | — | collection creation |
| HAR-2 | not_started | unassigned | — | — | — | local Heap ceremony |
| HAR-3 | not_started | unassigned | — | — | — | application-key lifecycle |
| HAR-4 | not_started | unassigned | — | — | — | qualified remote Heap path |
| HAR-5 | not_started | unassigned | — | — | — | Heap operations |
| HAR-6 | not_started | unassigned | — | — | — | SDK/CLI journey |
| HAR-7 | not_started | unassigned | — | — | — | P1 release gate |
| DRE-0 | not_started | unassigned | — | — | — | semantic oracle |
| DRE-1 | not_started | unassigned | — | — | — | source language |
| DRE-2 | not_started | unassigned | — | — | — | canonical invariant core |
| DRE-3 | not_started | unassigned | — | — | encoding amendment required | verified artifact |
| DRE-4 | not_started | unassigned | — | — | — | document-local enforcement |
| DRE-5 | not_started | unassigned | — | — | — | operational lifecycle |
| DRE-6 | not_started | unassigned | — | — | — | P2 release gate |
| ATM-0 | not_started | unassigned | — | — | — | semantic oracle |
| ATM-1 | not_started | unassigned | — | — | — | canonical plans |
| ATM-2 | not_started | unassigned | — | — | — | prepare/member evidence |
| ATM-3 | not_started | unassigned | — | — | — | durable decision |
| ATM-4 | not_started | unassigned | — | — | — | recovery/convergence |
| ATM-5 | not_started | unassigned | — | — | — | LocalHeap Atomic API |
| REL-0 | not_started | unassigned | — | — | — | reference metadata |
| REL-1 | not_started | unassigned | — | — | — | parent-exists/restrict |
| REL-2 | not_started | unassigned | — | — | — | uniqueness |
| REL-3 | not_started | unassigned | — | — | — | activation/validation |
| REL-4 | not_started | unassigned | — | — | — | P3 release gate |
| DDA-0 | not_started | unassigned | — | — | profile amendment required | rank oracle |
| DDA-1 | not_started | unassigned | — | — | — | natural direct rank |
| DDA-2 | not_started | unassigned | — | — | — | filtered direct rank |
| DDA-3 | not_started | unassigned | — | — | — | ordered admission seam |
| DDA-4 | not_started | unassigned | — | — | cursor profile required | P4 public surface |
| DDA-5 | deferred | unassigned | — | — | cluster profile unavailable | distributed rank |
| DDA-6 | deferred | unassigned | — | — | P4 not accepted | adaptive optimization |
| DOW-0 | not_started | unassigned | — | — | — | mathematical oracle |
| DOW-1 | not_started | unassigned | — | — | — | immutable order blocks |
| DOW-2 | not_started | unassigned | — | — | — | compressed exact indexes |
| DOW-3 | not_started | unassigned | — | — | — | P5 immutable path |
| DOW-4 | not_started | unassigned | — | — | — | mutable order path |
| DOW-5 | deferred | unassigned | — | — | cluster profile unavailable | distributed order |

When a package enters `accept`, replace `—` in source revision with the commit
hash and link every required evidence artifact. A package cannot be accepted
with an unresolved exit-gate defect.
