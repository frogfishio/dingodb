# Atomics implementation plan

Status: developer-ready v1.0

Program release: P3

Normative source: [ATOMICS_SPEC.md](../ATOMICS_SPEC.md)

## 1. Crate and module ownership

Create:

```text
crates/residuum-atomics/
  src/
    lib.rs
    id.rs
    plan.rs
    canonical.rs
    limits.rs
    evidence.rs
    outcome.rs
    status.rs
    oracle.rs
    relationship.rs
    uniqueness.rs
  tests/
    encoding.rs
    retry.rs
    oracle.rs
    hostile.rs
```

`dingo-atomics` owns pure identities, plans, canonical encoding, evidence
types, outcome derivation, and slow semantic oracles.

Host implementation:

```text
residuum-store::atomics       LocalHeap sequencer, journal, publication, recovery
residuum-sdk::atomics         builders, submit, status, receipts
residuum-server::atomics      Heap-bound one-shot RPC
residuum-examine::atomics     SDA evidence projection
residuum-cluster::atomics     later Partition profile
```

The pure crate MUST NOT import store/server/cluster code.

## 2. Work packages

### ATM-0 — Canonical profiles and oracle

Deliver:

- `spec/atomics/cbor-v1.json`;
- accepted/rejected fixtures;
- profile constants;
- `AtomicId`, scope, plan, evidence, outcome types;
- canonical member ordering;
- content-root implementation;
- slow in-memory serial oracle;
- hostile decoding limits.

Tests:

- byte-identical vectors;
- field permutation rejected/canonicalized as specified;
- same plan equal root;
- one semantic change changes root;
- cross-Heap plan rejected;
- ID conflict;
- corpus round trip.

Exit: future implementations can be compared to the oracle and fixture bytes.

Evidence: Unit, Property.

### ATM-1 — Key Atomic

Deliver on embedded Heap:

- create-if-absent;
- replace-if-version;
- delete-if-version;
- stable retry;
- prepare/member/decision frames;
- key commit position;
- status resolution;
- crash failpoints;
- history/index consequences in the closed plan.

Required tests:

- every conditional mutation;
- same-ID retry before/after restart;
- ID conflict;
- concurrent compare-version;
- crash matrix;
- damaged evidence;
- two Heaps same key/ID;
- RRE document-local commit integration.

Exit: one-key transitions match the oracle under randomized histories and
return no ambiguous successful acknowledgement.

Evidence: Differential, Isolation, Crash, Damage.

### ATM-2 — LocalHeap bounded publication

Deliver:

- per-Heap sequencer/frontier;
- bounded multi-key closed plan;
- canonical lock/member order;
- one logical publication;
- prepared-member invisibility;
- detailed and tombstone retention;
- backup/restore/salvage;
- SDA evidence.

This package may initially serialize all LocalHeap commits. It MUST expose the
semantic frontier so a future parallel implementation can preserve the same
contract.

Required tests:

- 2–256 user mutations;
- generated history/index members;
- no partial ordinary visibility;
- crash after each member;
- detail compaction retains exact retry tombstone;
- same AtomicId across two Heaps remains independent;
- resource ceilings.

Exit: LocalHeap all-or-nothing publication works, but serializable read
predicates are not yet advertised until `ATM-3`.

Evidence: Isolation, Crash, Damage, Journey.

### ATM-3 — Serializable LocalHeap validation

Deliver:

- read witnesses;
- absence/range predicates;
- validation under sequencer;
- phantom protection;
- read-your-writes;
- active RRE revision validation;
- randomized history recorder and serializability checker.

Required adversarial histories:

- write skew;
- lost update;
- parent delete versus child insert;
- two same-unique-value inserts;
- rule replacement versus write;
- stale exact index;
- damaged index claiming absence;
- timeout before/after decision.

Exit: every accepted generated history has a valid serial order and every
known anomaly is rejected.

Evidence: Differential, Property, Model, Crash.

### ATM-4 — Remote one-shot plan

Deliver:

- Heap-bound plan submit;
- status by AtomicId;
- timeout/reconnect;
- stable operation admission;
- request/response fixtures;
- server resource admission;
- no interactive lock/session API.

Required tests:

- lost request;
- lost response after commit;
- reconnect to same server;
- retry identical/different root;
- malformed/oversized plan;
- foreign Heap;
- hidden Heap error shape;
- server restart and status.

Exit: remote outcomes equal embedded outcomes for the same plan/evidence.

Evidence: Isolation, Crash, Journey.

### ATM-5 — Release and performance gate

Required:

- full conformance;
- fuzz plan/evidence parsers;
- 24-hour randomized local history soak or recorded equivalent;
- write overhead disclosure by member count and durability;
- recovery-time disclosure;
- docs and capability status.

Product wording:

> Bounded serializable Atomics within one local Heap.

It MUST NOT imply cross-Heap or cluster transaction support.

## 3. Relationship packages

### REL-0 — Definition and reverse index

Deliver:

- `dingo-relationship-v1` canonical definition;
- `AtomicAdmin` registry amendment;
- required/optional scalar references;
- same-collection support;
- Heap-bound reverse-reference index;
- build/drop/rebuild;
- coverage/frontier status;
- RRE compilation integration.

The reverse index is derived and disposable.

Exit: index contents equal a complete scan oracle.

### REL-1 — Enforcement

Use `ATM-3` to enforce:

- child insert requires parent;
- child reference change;
- child delete;
- parent delete restrict;
- same-Atomic parent+child creation;
- conflict races.

Absence under incomplete coverage refuses.

Exit: no generated serial history commits a state violating the active
relationship.

### REL-2 — Online activation

Deliver barrier → snapshot → scan → replay → serialize → activate.

Tests include concurrent writes/deletes, crash, violations, index damage, and
coverage loss.

Exit: active status is possible only with complete coverage and zero unresolved
violations.

### REL-3 — Uniqueness and bounded cardinality

Deliver:

- exact normalization;
- unique reverse map;
- contention;
- optional Null/Absent policy;
- bounded-many relationship counts;
- rebuild/salvage.

Exit: generated concurrent histories equal the uniqueness/cardinality oracle.

### REL-4 — DX and recovery

Deliver:

- Rust fluent administration;
- CLI create/validate/status/violations/retire;
- backup/restore/import/migration;
- SDA inspection;
- one parent/delete-restrict public demonstration.

Exit journey:

```text
create parent/child collections
activate relationship
insert valid child
reject missing parent
reject referenced parent delete
punch reverse-index hole
refuse unsafe delete with coverage_incomplete
rebuild index
delete child then parent atomically
```

## 4. Deferred packages

- `ATM-6` Partition Atomic;
- clustered relationship placement;
- cascade;
- cross-scope workflows;
- read-only snapshot sessions;
- long interactive transactions;
- external effects.
