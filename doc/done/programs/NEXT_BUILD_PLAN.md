# Residiuum post-Heap build plan

Status: **archived sequencing source — superseded for work selection**

Date: 2026-07-30

Scope: single-node product path after the Heap isolation kernel

Integrated release ordering:
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md).

Package definitions remain useful design provenance. Execution order and
admission are governed exclusively by the master plan and living scoreboard.
Do not select work from this document.

Companions:
[HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md),
[doc/todo/heap-application-ready/HEAP_APPLICATION_READY_PLAN.md](../../todo/heap-application-ready/HEAP_APPLICATION_READY_PLAN.md),
[RRE_SPEC.md](../../todo/rre/RRE_SPEC.md),
[doc/todo/rre/RRE_IMPLEMENTATION_PLAN.md](../../todo/rre/RRE_IMPLEMENTATION_PLAN.md),
[ATOMICS_SPEC.md](../../todo/atomics/ATOMICS_SPEC.md),
[doc/todo/atomics/ATOMICS_IMPLEMENTATION_PLAN.md](../../todo/atomics/ATOMICS_IMPLEMENTATION_PLAN.md),
[DIRECT_ACCESS_SPEC.md](../../todo/direct-access/DIRECT_ACCESS_SPEC.md),
[doc/todo/direct-access/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md](../../todo/direct-access/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md),
[ORDER_WAVELET_SPEC.md](../../todo/order-wavelets/ORDER_WAVELET_SPEC.md), and
[doc/todo/order-wavelets/ORDER_WAVELET_IMPLEMENTATION_PLAN.md](../../todo/order-wavelets/ORDER_WAVELET_IMPLEMENTATION_PLAN.md)

## 1. Decision

The next product program is:

```text
Heap Application Ready
        ↓
document-local Data Rules
        ↓
LocalHeap Atomics
        ↓
cross-document Data Rules
        ↓
Direct Access
        ↓
Order Wavelets
```

This order is normative for the single-node product program.

Heaps establish the non-crossing authority domain. RRE gives that domain
database-owned truth. Atomics enforce truth across more than one document.
Direct Access gives exact ranked positioning. Order Wavelets give exact
filtered positioning in declared order.

Text, vector, geospatial, cluster expansion, and broad archive work MUST NOT
displace this critical path.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

Each work package has:

- an immutable package ID;
- entry conditions;
- owned code and artifacts;
- required tests;
- an exit gate;
- explicit non-goals.

A package is not complete because its happy-path API exists. Its exit gate must
pass in CI and its capability status must be updated honestly.

## 3. Product releases

### Release P1 — Heap Application Ready

Outcome:

> A developer can create two Heaps, issue separate application keys, create
> collections, use ordinary data APIs, rotate authority, back up and restore,
> and prove that neither Heap can observe the other.

Packages: `HAR-0` through `HAR-7`.

Product status after exit:

- Heap application path: Experimental / self-assessed;
- Heap pure kernel: machine-checked by Kani and Verus;
- independent-review qualification: pending;
- legacy flat/raw paths: opt-in and explicitly outside the Heap claim.

### Release P2 — Data Rules: document-local

Outcome:

> One Heap can activate a finite RRE ruleset and every subsequent committed
> document satisfies its active document-local rules.

Packages: `RRE-0` through `RRE-4`.

Initial rule classes:

- required/optional/forbidden fields;
- exact scalar types;
- integer and exact-decimal ranges;
- finite enums;
- conditional presence;

Transition, reference, uniqueness, and bounded-cardinality rules remain
unavailable and MUST be rejected by activation until their Atomic scope ships.

### Release P3 — Atomic Integrity

Outcome:

> Within one Heap, Residiuum commits one bounded serializable transition with a
> stable identity and independently examinable decision evidence.

Packages: `ATM-0` through `ATM-5`, `RRE-5`, `REL-0` through `REL-4`, then
`RRE-6`.

First user-visible cross-document guarantees:

- unique;
- parent exists;
- optional parent reference;
- `on delete restrict`;
- bounded relationship cardinality.

### Release P4 — Direct Rank

Outcome:

> Supported natural-order queries can return result rank `k` without work
> proportional to `k`, with exact coverage and Heap-bound cursor evidence.

Packages: `DDA-0` through `DDA-4`.

### Release P5 — Counted Order

Outcome:

> Supported filtered scalar-order queries navigate exact conditional counts
> instead of sorting every matching document at query time.

Packages: `DOW-0` through `DOW-4`.

Distributed packages remain later qualification work.

## 4. Dependency graph

```text
HAR-0 ─ HAR-1 ─ HAR-2 ─ HAR-3 ─ HAR-4 ─ HAR-5 ─ HAR-6 ─ HAR-7
                                                        │
                                                        ▼
RRE-0 ─ RRE-1 ─ RRE-2 ─ RRE-3 ─ RRE-4
          │                       │
          │                       ▼
          └──────────────► ATM-0 ─ ATM-1 ─ ATM-2 ─ ATM-3 ─ ATM-4 ─ ATM-5
                                      │                │
                                      ▼                ▼
                                    RRE-5            REL-0 ─ REL-1 ─ REL-2 ─ REL-3 ─ REL-4 ─ RRE-6

RRE-0 predicate semantics ───────────────┐
existing indexes + frozen views ────────┼─► DDA-0 ─ DDA-1 ─ DDA-2 ─ DDA-3 ─ DDA-4
Heap cursor/key binding ────────────────┘                         │
                                                                 ▼
                                           DOW-0 ─ DOW-1 ─ DOW-2 ─ DOW-3 ─ DOW-4
```

Allowed parallelism:

- `RRE-0` reference semantics MAY begin while `HAR-4`–`HAR-7` finish.
- `ATM-0` encoding design MAY begin after `HAR-2` freezes Heap object identity.
- `DDA-0` oracle work MAY begin after RRE shared predicate semantics freeze.
- `DOW-0` mathematical reference MAY begin after DDA order-domain identity
  freezes.

No later package may publish a product surface before its upstream exit gate.

## 5. Universal Heap rule

Every new durable or cached artifact MUST bind:

```text
HeapId
profile/version
semantic content identity
read-view or commit frontier where applicable
integrity/authentication tag where applicable
```

This includes:

- RRE source, artifacts, activations, and decisions;
- Atomic IDs, prepares, members, decisions, and deduplication;
- predicate bitmaps and rank blocks;
- DDA selection artifacts and cursors;
- Order Wavelet dictionaries, blocks, caches, and cursors;
- backup, restore, salvage, logs, metrics, and support material.

There is no deployment-global rule, Atomic, index, cursor, or selection cache
with a Heap filter added at lookup time.

## 6. Shared engineering gates

Every package MUST satisfy:

1. **Semantics:** normative examples and counterexamples are executable.
2. **Identity:** canonical bytes and domain separators are versioned.
3. **Containment:** two-Heap differential noninterference tests pass.
4. **Crash:** every authoritative multi-step publication has failpoints.
5. **Damage:** missing/corrupt evidence produces explicit incomplete/unknown,
   never silent success.
6. **Bounds:** bytes, members, depth, work, and retained evidence have hard
   ceilings.
7. **Retry:** mutation identity and retry behavior are specified and tested.
8. **Recovery:** backup, restore, migration, and salvage treatment is explicit.
9. **Examination:** SDA can inspect authoritative evidence and holes.
10. **DX:** Rust and CLI examples use the qualified Heap path.
11. **Maturity:** capability data and website/docs status are updated.
12. **No legacy leak:** new code does not import flat/raw compatibility paths.

## 7. Evidence ladder

Package evidence uses these labels:

| Label | Meaning |
|---|---|
| `Unit` | Deterministic local behavior |
| `Property` | Generated/exhaustive algebraic laws |
| `Differential` | Fast implementation equals slow oracle |
| `Isolation` | Other-Heap mutations cannot affect target observation |
| `Crash` | Failpoint restart converges to an allowed outcome |
| `Damage` | Missing/corrupt material remains explicit |
| `Model` | Kani, Verus, TLA+, or linearizability/model check |
| `Journey` | Public SDK/CLI path from clean state |
| `Performance` | Reproducible disclosure with raw artifact |

Each exit gate names the evidence classes it requires.

## 8. Product truth rules

- “Machine-checked Heap kernel” is allowed because Kani and Verus are connected.
- “Independently reviewed Heap isolation” is prohibited until CPR-005 closes.
- Document-local RRE MUST NOT be described as referential integrity.
- Key Atomic MUST NOT be described as a general transaction.
- LocalHeap Atomic MUST NOT imply cross-Heap or cross-partition Atomicity.
- Cursor paging MUST NOT be described as direct ranked access.
- DDA design MUST NOT be described as implemented before its exit gate.
- DOW design MUST NOT be described as implemented before its exit gate.
- A performance number requires the benchmark-disclosure class appropriate to
  the wording used.

## 9. Program scoreboard

The repository SHOULD maintain:

```text
doc/wip/status/NEXT_BUILD_STATUS.md
```

with one row per package:

```text
package
state: not_started | active | blocked | accept | deferred
owner
source revision
evidence
open defects
capability impact
```

The scoreboard is derived program state. Normative semantics remain in the
specifications.

## 10. Stop conditions

Work stops and returns to specification when:

- an Atomic implementation requires an unresolved semantic choice;
- RRE compilation cannot derive a finite dependency or cost bound;
- a direct query cannot prove exact membership/count/order;
- damage can change an answer while coverage still reports complete;
- an artifact can be reused across Heaps, views, rules, or plans;
- a retry can produce a second logical effect;
- a product surface would need to weaken a named invariant silently.

Refusal is an acceptable product outcome. Silent fallback is not.

## 11. Immediate queue

The next executable queue is:

1. `HAR-0` — truth cleanup and qualification-script consistency;
2. `HAR-1` — collection creation over the Heap protocol;
3. `HAR-2` — local Heap creation ceremony;
4. `HAR-3` — application-key lifecycle;
5. `HAR-4` — qualified remote listener as default Heap server;
6. `HAR-5` — Heap lifecycle/backup operational path;
7. `HAR-6` — SDK/CLI journey;
8. `HAR-7` — Heap Application Ready release evidence;
9. `RRE-0` — crate and conformance harness;
10. `RRE-1` — parser and canonical AST.

External review remains a qualification opportunity. It is not a code-package
dependency for P1–P5 while product language remains honest.
