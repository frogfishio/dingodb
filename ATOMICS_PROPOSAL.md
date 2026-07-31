# Residiuum Atomics proposal

Status: Historical exploratory source; superseded for implementation by
[ATOMICS_SPEC.md](ATOMICS_SPEC.md)

Scope: Bounded indivisible state transitions, integrity invariants, commit
evidence, and transaction compatibility

Companions: `ATOMICS_SPEC.md`, `RRE_SPEC.md`, `HEAP_SPEC.md`, `DX_SPEC.md`,
`TRANSACTIONS.md`, `CLUSTER_SPEC.md`, and `DATABASE_DOCTRINE.md`

This document preserves the product reasoning and research space. Developers
MUST use `ATOMICS_SPEC.md` and `doc/ATOMICS_IMPLEMENTATION_PLAN.md` for v1
behavior. If this proposal conflicts with either, the normative specification
wins.

## 1. The idea

An **Atomic** is Residiuum's fundamental unit of indivisible logical change:

> Within one declared coordination scope, validate one closed set of
> preconditions and invariants, then make one bounded set of effects logically
> visible together—or make none of them visible.

Atomics may be used internally by Residiuum or submitted explicitly by an
application. Referential integrity is the first proposed invariant built on
the primitive; scoped transaction compatibility is one external
interpretation of it.

The ordinary experience remains:

> Put anything in. Get it back.

The optional experience becomes:

> For the parts that must agree, tell Residiuum once instead of defending the
> invariant in every service forever.

Ordinary single-record operations are already degenerate one-member Atomics.
Optional invariants are absent by default and impose no
integrity-maintenance cost on collections that do not use them.

## 2. Motivating example

Consider two document collections:

```text
customers
  customer-42 -> { "name": "Alice" }

orders
  order-9001 -> { "customer_id": "customer-42", "total": 125 }
```

The application intends:

```text
orders.customer_id references customers._key
```

Without database enforcement, every writer must remember to:

1. verify that the customer exists before writing an order;
2. prevent customer deletion while orders still refer to it;
3. update both sides correctly when an order changes customer;
4. handle concurrent insertion and deletion;
5. recover correctly after a crash between those steps.

One forgotten code path, old service, import tool, repair script, or race can
create an orphan.

The Data Rules declaration is:

```text
rules for orders

require customer_id as key

reference customer using customers
  matching customer_id = _key
  expect exactly_one
  on delete restrict
```

After activation:

- an order with a missing customer is rejected;
- changing `customer_id` validates the new customer;
- deleting a referenced customer is rejected;
- child insertion racing parent deletion cannot produce an orphan;
- the rule applies equally to SDK writes, batches, imports, and administrative
  tooling that claims normal integrity.

## 3. Why “Atomics”

Referential integrity is the first visible use case, but the underlying product
idea is larger:

```text
optional invariant
        +
database-owned supporting index
        +
atomic validation and mutation
        =
Atomic
```

Possible later atomics include:

- unique values;
- required fields;
- immutable fields;
- scalar type constraints;
- bounded child cardinality;
- deterministic document checks.

This proposal begins with references because they solve a painful,
well-understood document-database problem and expose the required concurrency
machinery honestly.

An Atomic is the primitive. A bounded Atomic may satisfy serializable
transaction semantics inside its declared scope, but “transaction” is not the
foundational abstraction and never expands that scope.

## 4. Product position

Atomics are progressive disclosure.

A user who does not want schemas or relationships never encounters them. A
user who wants one protected master/detail relationship can declare it without
turning the rest of the heap into a relational schema.

The intended position is:

> Residiuum remains an easygoing document store, but it does not make
> applications repeatedly solve integrity problems the database can solve
> once.

The feature must not become a disguised SQL engine, ORM, trigger runtime,
general-purpose server-side programming language, or unqualified
“ACID everywhere” promise.

## 5. Goals

The Atomic architecture should:

- define a small mathematical contract;
- declare scope before any effect;
- provide serializable equivalence for read-write groups inside that scope;
- classify logical decision separately from surviving material;
- support stable identity, retry, and outcome resolution;
- serve both internal invariants and optional external groups;
- reject rather than weaken an operation that escapes its scope;
- have zero request-path cost for unrelated collections;
- operate only inside one heap;
- use immutable `CollectionId` identity rather than collection names;
- protect parent existence and restricted parent deletion;
- remain correct under concurrent writes;
- remain correct across process crashes;
- maintain a rebuildable reverse-reference index;
- expose predictable SDK and CLI behavior;
- detect and report integrity degradation after physical damage;
- work identically through ordinary embedded and server APIs;
- define an honest path to clustered enforcement;
- remain inspectable through SDA and recovery tooling.

## 6. Non-goals for the first profile

Version 1 should not provide or imply:

- deployment-global or cross-heap Atomics;
- arbitrary cross-partition atomic commitment;
- one global serial order;
- globally consistent snapshots across unrelated partitions;
- external side-effect atomicity;
- invisible compensation presented as rollback;
- unbounded interactive sessions or locks;
- a generic trigger or stored-procedure language;
- cross-heap references;
- references to mutable collection names;
- arbitrary query expressions as reference selectors;
- cross-database or external-service references;
- automatic cascade delete;
- `set null` or `set default`;
- cyclic relation graphs;
- triggers, callbacks, scripts, or user code;
- temporal referential integrity across retained history;
- arrays containing an unbounded number of references;
- eventual enforcement presented as strong integrity;
- automatic deletion of surviving children after parent corruption;
- a claim of clustered integrity before distributed qualification passes.

These exclusions are product boundaries, not missing syntax.

## 7. Formal contract

Let:

- `H` be one heap;
- `S` be the logical committed state visible inside one coordination scope;
- `X` be every state outside that scope;
- `P(S)` be the complete precondition and invariant predicate;
- `F(S)` be a deterministic bounded transition;
- `E` be durable decision and member evidence.

Execution has exactly these logical results:

```text
Atomic(H, scope, P, F, S, X)
    -> Committed(S' = F(S), X, E)
     | NotCommitted(S, X, E?)
     | OutcomeUnknown(S?, X, E)
```

The required properties are:

```text
Committed implies P held at one serialization point.
Committed implies every member becomes logically visible together.
NotCommitted implies no member becomes logically visible.
For every result, state outside the declared scope is unchanged.
No ordinary observer sees prepared or intermediate members.
Unknown is reported, retained, and resolvable; it is never guessed.
```

An Atomic is not defined by an API closure or by adjacent writes. It exists
only when authoritative evidence supports one decision over the exact member
set.

## 8. Two-dimensional truth

Residiuum separates logical decision from present physical material.

Logical decision:

```text
committed
not_committed
unknown_commit
conflicting_decision_evidence
```

Material condition:

```text
complete
partial
missing
conflicting
coverage_incomplete
```

These axes do not collapse:

```text
committed + partial
```

means the Atomic historically committed, but later damage destroyed some
material.

```text
unknown_commit + complete
```

means all known members survived, but sufficient decision evidence did not.
The members remain examinable and do not silently enter current state.

This distinction is a defining Residiuum property rather than an exceptional
recovery footnote.

## 9. Coordination scopes

Every Atomic declares one scope before mutation:

| Scope | Meaning | Initial availability |
|---|---|---|
| `Key` | one immutable collection/key identity | every backend |
| `LocalHeap` | bounded keys and collections inside one embedded or single-server heap | local qualified profile |
| `Partition` | bounded keys and collections mapping to one strong cluster partition | partition-linearizable cluster |

All scopes bind exactly one `HeapId`. There is no `Deployment`, `HeapSet`,
`AnyPartition`, or implicit scope.

Internal versus external is an origin and authorization distinction, not
another coordination scope. An engine-created invariant Atomic must still
qualify as `Key`, `LocalHeap`, or `Partition`.

Before prepare, the engine closes the plan over caller members and every
engine-generated invariant, index, history, and idempotency effect. If any
read, write, invariant, derived consequence, or lock falls outside the declared
scope, the Atomic fails before recording a member. It never splits, widens
itself, downgrades isolation, or becomes a workflow automatically.

## 10. Isolation and execution

Read-write Atomics provide one observable isolation contract inside their
scope:

```text
serializable
```

An implementation may validate optimistically:

1. bind a stable scope frontier;
2. record read versions and absence predicates;
3. buffer a closed mutation plan;
4. obtain the scope commit sequencer;
5. validate reads, predicates, rights, state, and invariants;
6. assign one commit position;
7. persist decision/member evidence;
8. publish one logical delta.

The resulting history must be equivalent to a serial order. Snapshot
isolation alone is insufficient because it permits write skew across exactly
the invariants Atomics are intended to protect.

Long-lived user locks are not part of the model. Construction is bounded by
time, members, bytes, read set, generated work, and affected collections.

## 11. Identity, retry, and evidence

Every Atomic has a caller- or engine-stable `AtomicId` and deterministic
content hash.

Retrying the same ID and content:

```text
returns or resolves the original outcome
```

Reusing the ID with different scope, preconditions, members, or content:

```text
AtomicIdConflict
```

The authoritative evidence model contains:

```text
AtomicPrepare {
    atomic_id
    heap_id
    scope
    frontier
    ordered member manifest
    preconditions
    invariant revisions
    content root
}

AtomicMember {
    atomic_id
    ordinal
    event_id
    object identity
    content hash
}

AtomicDecision {
    atomic_id
    prepare hash
    member root and count
    decision
    commit position
    achieved durability
    cluster evidence when applicable
}
```

Prepared members are independently recoverable but invisible to ordinary
state until the complete committed decision validates. Adjacency may optimize
recovery but is never authority.

## 12. Internal and external Atomics

Internal Atomics may protect:

- reverse-reference and uniqueness indexes;
- parent deletion and child insertion conflicts;
- catalog identity and rename publication;
- idempotency records plus their effects;
- bounded batch publication;
- index/frontier installation;
- state-machine commands.

Internal use does not bypass HeapKey, heap isolation, durability, recovery, or
resource limits. It uses a non-serializable engine capability with only the
required scope.

External Atomics submit a bounded plan:

```rust
let outcome = heap.atomic(
    Atomic::partition("account-42")
        .check_version(accounts.id(), "account-42", expected)
        .replace(accounts.id(), "account-42", account)
        .create(ledger.id(), "entry-901", entry)
)?;
```

For a remote server, the complete plan is submitted as one request. The server
does not hold an interactive transaction open across arbitrary network pauses.
A local ergonomic closure may compile into the same immutable plan before
commit.

## 13. Outcome API

The core result is:

```rust
pub enum AtomicOutcome {
    Committed(AtomicReceipt),
    NotCommitted {
        atomic_id: AtomicId,
        reason: AtomicAbortReason,
    },
    Unknown {
        atomic_id: AtomicId,
        resolution: AtomicResolutionHandle,
    },
}
```

Status resolution returns both axes from §8 plus coverage:

```rust
heap.atomic_status(atomic_id)?
```

`not_found` is permitted only when the declared scope, evidence-retention
window, and required tiers have complete coverage.

## 14. Transaction compatibility and workflows

A compatibility layer may expose familiar transaction terminology:

```text
serializable transaction within one local heap
    = LocalHeap Atomic

serializable transaction within one cluster partition
    = Partition Atomic
```

This is a truthful interpretation, not the architectural primitive.
`TRANSACTIONS.md` defines that adapter and must not broaden this contract.

Work crossing Atomic scopes is a **workflow**:

```text
durable steps
idempotent retry
explicit compensation
per-step outcome and coverage
no all-or-nothing claim
```

A workflow is never automatically named an Atomic or transaction. Compensation
is a new forward effect, not erasure or rollback of history.

## 15. First invariant: relationship profile

`RRE_SPEC.md` governs the human declaration, formal predicate,
compilation proof obligations, lifecycle, and product position. This section
defines the first relationship enforcement profile built on the Atomic
primitive.

The smallest useful compiled relationship rule is:

```text
ReferenceRule {
    rule_id
    heap_id
    name
    child_collection_id
    child_field
    parent_collection_id
    parent_key = _key
    required
    on_delete = restrict
    state
    definition_revision
}
```

Rules:

- `rule_id` is immutable and never reused;
- both collections belong to the same `HeapId`;
- `child_field` is one deterministic document field path;
- the field value is one scalar key;
- the referenced parent identity is the parent document key;
- parent keys are immutable;
- `required = true` rejects missing and null references;
- `required = false` permits absence and validates any present value; explicit
  Null handling follows the frozen Data Rules type profile;
- `on_delete = restrict` is the only v1 delete action;
- collection rename does not affect the relationship;
- deleting and recreating a collection does not retarget it;
- one child field participates in at most one active reference rule in v1.

Document field-path syntax should reuse the frozen Residiuum query/path grammar.
It must not execute SDA or arbitrary application logic during admission.

## 16. Declarative and Rust experience

Illustrative fluent Rust:

```rust
let relation = heap
    .rules()
    .reference("order_customer")
    .from("orders", "customer_id")
    .references_key("customers")
    .required()
    .on_delete_restrict()
    .create()?;
```

Equivalent explicit form:

```rust
heap.rules().create_reference(
    ReferenceRule::builder()
        .name("order_customer")
        .child(orders.id(), FieldPath::parse("customer_id")?)
        .parent_key(customers.id())
        .required(true)
        .on_delete(DeleteAction::Restrict)
        .build()?,
)?;
```

Typical failures:

```text
ReferencedRecord
MissingReferencedRecord
RuleValidating
RuleIntegrityDegraded
RuleDefinitionConflict
```

Errors identify the rule and local collection when the caller is authorized
to observe them. They never reveal another heap.

## 17. Relationship internal model

Each active relationship maintains a derived reverse-reference index:

```text
(HeapId, RuleId, ParentKey, ChildKey)
```

Conceptually:

```text
order_customer / customer-42
  -> order-9001
  -> order-9017
  -> order-9044
```

The index answers:

```text
does this parent have at least one live child?
```

It may also support diagnostics and future bounded enumeration, but v1 parent
deletion needs only an efficient existence check.

The reverse index is derived:

- authoritative child documents remain the source of truth;
- loss or corruption of the index triggers rebuild or degraded status;
- a stale or incomplete index must never be trusted to authorize deletion;
- index entries carry `HeapId`, `RuleId`, immutable collection IDs, parent
  key, child key, and source event/version identity;
- rebuilding cannot combine equal names or keys from different heaps.

## 18. Relationship Atomic semantics

### 18.1 Insert child

Writing a child with a reference performs one internal Atomic:

```text
lock/order parent-reference domain
verify parent exists
write child
write reverse-reference entry
commit one outcome
```

The child and backlink become visible together or neither becomes visible.

### 18.2 Change child reference

Changing:

```text
customer_id: customer-42 -> customer-77
```

performs:

```text
verify customer-77 exists
remove backlink(customer-42, child)
add backlink(customer-77, child)
write child
commit one outcome
```

The Atomic never exposes a child whose visible reference and backlink
disagree.

### 18.3 Delete child

Deleting a child removes its backlink in the same Atomic as the logical
child deletion.

### 18.4 Delete parent

Deleting a parent performs:

```text
lock/order parent-reference domain
check reverse-reference index
if any live child exists:
    reject ReferencedRecord
else:
    delete parent
commit one outcome
```

An index that is building, stale, corrupt, incomplete, or unavailable cannot
prove the absence of children. Parent deletion fails closed.

## 19. Relationship concurrency requirement

The central race is:

```text
Writer A                              Writer B
--------                              --------
check parent exists
                                      check parent has no child
insert child
                                      delete parent
```

Both checks can succeed unless they participate in the same ordering domain.

Therefore a conforming implementation must make:

```text
insert/update child reference to P
```

conflict or serialize with:

```text
delete parent P
```

The implementation may use an Atomic sequencer, predicate/key-range lock,
partition command ordering, or another proven mechanism. A preflight read
followed by unrelated writes is not referential integrity.

The required invariant for active relation `R` is:

```text
for every live child C:
    reference(C, R) = P
    implies live(parent(P, R))
```

and:

```text
visible_backlink(R, P, C)
    iff live(C) and reference(C, R) = P
```

## 20. Atomic and partition implications

Single-node enforcement requires atomic commitment across:

- the child or parent record;
- the reverse-reference entry;
- Atomic/dedup evidence;
- any affected history and indexes.

Clustered enforcement is easy only when parent, child, and reference-index
commands share one ordering scope.

The initial cluster implementation has three honest options:

1. require related records to share a partition;
2. introduce a qualified distributed Atomic coordinator;
3. report the rule unsupported for that placement.

It must not perform an eventually consistent check and call the result
referential integrity.

A useful v1 cluster restriction may be:

> A hard reference rule requires child and parent partitioning to derive
> the same integrity partition from the parent key.

The exact cluster algorithm belongs in a later implementation specification
and must be model-checked before the strong claim is enabled.

## 21. Relationship lifecycle

A rule revision has explicit states:

```text
draft
  -> validating
  -> active
  -> suspended_degraded
  -> rebuilding
  -> active
  -> retired
```

### 21.1 Creation over existing data

Creating a rule does not instantly make existing data valid.

The safe sequence is:

1. persist the immutable draft definition;
2. install a prospective enforcement barrier for new and changed records
   without yet reporting the rule as active;
3. scan a stable snapshot of existing children;
4. build the reverse-reference index;
5. replay changes after the snapshot boundary;
6. report every pre-existing violation;
7. activate only when coverage is complete and violations are zero or have
   been explicitly repaired.

While validating, parent deletion fails closed for the relationship. The
product must distinguish:

```text
declared
validating
active
degraded
```

It must never display “protected” merely because a declaration exists.

### 21.2 Definition changes

Changing child field, parent collection, or semantics creates a new immutable
definition revision and repeats validation. It does not reinterpret the old
reverse index in place.

### 21.3 Retirement

Retiring an atomic stops future enforcement only after:

- authorization and explicit confirmation;
- active Atomics using it have drained or fenced;
- the retirement revision is durable;
- SDK/schema caches observe the new revision.

Derived reverse indexes may then be deleted. Authoritative documents are
unchanged.

## 22. Damage and survival

Physical survival and logical integrity are separate claims.

If a parent frame is destroyed while child frames survive, Residiuum preserves
the healthy children. It does not delete them to make the relationship appear
valid.

The relationship becomes:

```text
suspended_degraded {
    reason: parent_or_evidence_missing
    coverage: known coverage
    violations: known orphan references
}
```

In degraded state:

- ordinary reads may return surviving documents with explicit integrity
  status where the API requests it;
- new writes that can be proven valid may be allowed only by a separately
  specified degraded-write policy;
- parent deletion and any operation relying on absence fail closed;
- repair and examination preserve heap and atomic identity;
- rebuild reports `Known`, `Unknown`, and `Conflict` evidence;
- no repair guesses that equal names or keys imply identity.

The operator may:

- restore the missing parent;
- update or remove the child reference;
- delete the child explicitly;
- retire the atomic;
- accept a documented integrity violation through a protected repair
  ceremony.

No choice happens silently.

## 23. History semantics

V1 atomics protect the current live state.

Historical versions may legitimately refer to a parent that was live at their
commit time but is no longer live. Residiuum does not rewrite retained history
when a relationship changes.

Claims such as:

```text
every historical child version resolves to the corresponding historical
parent version
```

belong to a future temporal-integrity profile. The ordinary `history` API must
state whether a returned version was committed while the atomic was active,
validating, degraded, or absent.

## 24. Heap and security rules

Atomics obey the heap isolation contract:

- a definition binds exactly one `HeapId`;
- child and parent collections must belong to that heap;
- no atomic contains a heap set or wildcard;
- all reverse indexes, caches, locks, jobs, receipts, audits, and recovery
  output remain heap-owned;
- an atomic cannot widen a `HeapCap`;
- creating, changing, validating, rebuilding, or retiring an atomic requires
  a dedicated administrative right;
- ordinary writes still require their normal data rights;
- a relationship error cannot reveal data outside the caller's heap and
  collection constraints.

The likely right is:

```text
AtomicAdmin
```

Adding it requires a deliberate HeapKey rights-registry revision; it must not
be smuggled into `HeapAdmin` as an implementation convenience.

## 25. Import, restore, and migration

Every data-entry path must declare its integrity mode:

```text
enforce
validate_then_commit
quarantine_violations
trusted_rebuild
```

An ordinary import uses `enforce` or `validate_then_commit`.

`quarantine_violations` is a protected recovery/import mode. Violating records
do not become ordinary live records.

`trusted_rebuild` is available only to the recovery TCB while reconstructing
derived reverse indexes from authoritative records. It cannot manufacture
missing parents or relabel records.

Restoring payload data without the atomic definition and its durable state
cannot silently claim integrity. Restoring a definition without complete data
produces validating or degraded status, never active.

## 26. Performance model

No configured atomic:

```text
no constraint lookup
no reverse-index write
no relationship lock
no additional read
```

An unrelated collection should not pay for atomics configured elsewhere in
the heap.

A related child mutation normally adds:

- one definition lookup from resident immutable metadata;
- one parent-existence check;
- one reverse-index mutation;
- Atomic coordination in the relevant integrity scope.

A parent deletion adds one reverse-index existence probe.

Reads do not validate the relationship on every access. They rely on the
committed invariant and expose integrity status only when requested or when
the atomic is degraded.

Quantitative claims require benchmarks for:

- child insert/update/delete;
- unreferenced and referenced parent delete;
- high fan-out parents;
- relationship creation/backfill;
- rebuild after index loss;
- unrelated collection operations with atomics elsewhere.

## 27. Why cascade is deferred

`on_delete cascade` appears convenient but changes the problem substantially.

One parent may have millions of descendants. Cascades may:

- cross partitions;
- form cycles;
- exceed Atomic limits;
- conflict with retention or legal holds;
- fail after partial progress;
- amplify one accidental deletion catastrophically;
- make latency and acknowledgement meaning unclear.

An asynchronous cascade is a job, not one atomic delete. A synchronous cascade
needs bounded fan-out and an Atomic capable of covering the whole closure.

V1 therefore supports only:

```text
on_delete restrict
```

Future cascade support requires its own proposal and must never weaken
`restrict` semantics.

## 28. Integrity extension candidates

Once the atomic framework is trustworthy, the next low-ambiguity constraints
could be:

### 28.1 Unique

```text
unique users.email
```

Backed by an atomically maintained value-to-record index.

### 28.2 Required

```text
required orders.customer_id
```

This already participates in relationship v1.

### 28.3 Immutable

```text
immutable invoices.issued_number
```

Once present, the value cannot change through ordinary writes.

### 28.4 Deterministic type

```text
type accounts.balance = decimal
```

Validation uses a closed type vocabulary, not scripts or coercive user code.

### 28.5 Bounded cardinality

```text
customer has at_most 5 active_addresses
```

This requires a contention-safe counter and therefore belongs after uniqueness
and references.

## 29. What would make this proposal fail

The design should be rejected or narrowed if implementation requires:

- deployment-global data access;
- a hidden cross-heap index;
- request-time scans to check parent deletion;
- eventual enforcement marketed as atomic;
- a general trigger language;
- mandatory schema declarations for ordinary document use;
- unrelated writes paying global constraint costs;
- trusting a damaged or incomplete reverse index to prove absence;
- cascading destructive work without bounded Atomic semantics;
- weakening Residiuum's survival rule by deleting healthy orphaned data.

## 30. Proposed delivery sequence

### A0 — Core semantics and evidence

Freeze Atomic IDs, scopes, content roots, prepare/member/decision evidence,
outcome states, coverage, errors, limits, and golden recovery fixtures.

### A1 — Key Atomic

Implement create-if-absent, replace/delete-if-version, stable retry identity,
and committed/not-committed/unknown outcomes on every backend.

### A2 — LocalHeap write Atomic

Implement bounded write plans, one logical publication, prepare/member/decision
evidence, crash injection, salvage, history, indexes, and watches.

### A3 — Serializable LocalHeap Atomic

Add frontiers, read and predicate sets, optimistic validation,
read-your-writes, phantom protection, and randomized serializability checking.

### A4 — Remote Atomic plan

Submit one bounded immutable plan, preserve identity through timeout and
reconnect, and expose status resolution without server-held interactive locks.

### A5 — Partition Atomic

Encode one plan as one partition state-machine command and qualify quorum
decision evidence, leader failure, retry, fencing, and replica application.

### R0 — Relationship definition and reverse index

Freeze field paths, states, rights, errors, and heap-owned reverse-reference
index encoding; build, discard, and reconstruct the derived index.

### R1 — Relationship enforcement

Use internal Atomics to enforce child insert/update/delete and parent
`restrict` under adversarial concurrency and crash injection.

### R2 — Online relationship validation

Create reference rules over existing collections with snapshot, change
replay, violation reporting, activation, degradation, and rebuild.

### R3 — Relationship DX and lifecycle

Provide fluent creation, inspection, status, violation examination, explicit
repair, retirement, and stable errors.

### R4 — Relationship recovery integration

Preserve definitions and status through backup, restore, import, migration,
damage, and salvage without treating derived indexes as authoritative.

### R5 — Relationship cluster qualification

Choose colocated integrity partitions or a separately qualified coordinator,
model-check it, and execute partition/leader/crash tests before enabling the
strong clustered-invariant claim.

## 31. Questions for the next draft

The next draft must decide:

1. the exact Atomic ID, prepare, member, and decision encodings;
2. initial member, byte, read-set, duration, and generated-work limits;
3. decision and deduplication evidence retention;
4. exact local commit-position and predicate representation;
5. the durability boundary between cluster consensus and segment evidence;
6. whether read-only snapshots belong to the first external profile;
7. whether v1 relationships support optional references or only `required`;
8. the exact field-path and scalar-key profile;
9. whether a child key may reference a parent in the same collection;
10. whether relationship graphs must be acyclic even without cascade;
11. the Atomic ordering key used for parent/reference conflicts;
12. whether clustered relationship v1 requires explicit co-partitioning;
13. how validating atomics handle concurrent parent deletion;
14. how long completed relationship receipts and violation evidence are kept;
15. whether integrity status is attached to ordinary reads or exposed only
   through inspection APIs;
16. the exact `AtomicAdmin` and recovery authorization boundaries.

## 32. Initial recommendation

Proceed with Atomics as the primitive and keep every initial promise
deliberately scoped:

> Within one declared Key, LocalHeap, or qualified Partition scope, Residiuum
> can commit one bounded serializable state transition with durable identity
> and independently examinable outcome evidence.

Use that primitive for the first optional invariant:

> Within one heap, Residiuum can enforce that a scalar child reference names a
> live parent and can atomically prevent deletion of a referenced parent.

Transaction APIs may adapt LocalHeap and Partition Atomics, but do not define
or widen them. Cross-scope work remains an explicit workflow.

The relationship capability solves a real document-database pain without
imposing a relational model on users who do not want one.

The difficult work is not evaluating `parent_exists`. It is preserving the
invariant through races, crashes, backfill, clustering, restoration, and
damage. Those are exactly the areas in which Residiuum should prefer a narrow
truthful guarantee over a broad convenient approximation.
