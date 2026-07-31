# Residiuum Atomics v1 specification

Status: normative design v1.0-draft; implementation not yet qualified

Profiles:

```text
dingo-atomic-v1
dingo-atomic-plan-v1
dingo-atomic-evidence-v1
dingo-relationship-v1
```

Source proposal: [ATOMICS_PROPOSAL.md](ATOMICS_PROPOSAL.md)

Normative companions:
[HEAP_SPEC.md](HEAP_SPEC.md),
[RRE_SPEC.md](RRE_SPEC.md),
[COLLECTION_CONTRACT_SPEC.md](COLLECTION_CONTRACT_SPEC.md),
[RESIDIUUM_PREDICATE_SPEC.md](RESIDIUUM_PREDICATE_SPEC.md),
[FORMAT_SPEC.md](FORMAT_SPEC.md), and
[doc/ATOMICS_IMPLEMENTATION_PLAN.md](doc/ATOMICS_IMPLEMENTATION_PLAN.md)

## 1. Decision

An Atomic is one bounded, serializable state transition inside exactly one
declared coordination scope and exactly one Heap.

It has:

- a stable identity;
- a canonical closed plan;
- one serialization point;
- authoritative prepare/member/decision evidence;
- explicit retry behavior;
- explicit outcome uncertainty;
- independently examinable recovery semantics.

An Atomic is not defined by an API closure, adjacent writes, a process mutex,
or transaction-shaped syntax.

The product statement is:

> Within one Key, LocalHeap, or qualified Partition scope, Residiuum can commit
> one bounded serializable transition with durable identity and independently
> examinable outcome evidence.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 3. Scope

V1 defines:

- Key Atomics on every backend;
- LocalHeap Atomics for embedded and qualified single-server Heaps;
- Partition Atomics only after a partition profile passes its separate gate;
- create-if-absent, compare-version replace/delete, and bounded mutation plans;
- serializable read/write and absence predicates;
- RRE enforcement;
- uniqueness and scalar relationships;
- exact outcomes and recovery evidence;
- remote submit/status without interactive server-held transactions.

V1 excludes:

- cross-Heap Atomics;
- cross-partition Atomics;
- interactive sessions holding server locks;
- unbounded member generation;
- external services/effects;
- triggers and user code;
- arbitrary rollback of history;
- cascade delete;
- read-only snapshot sessions;
- distributed sagas presented as Atomics.

Cross-scope work is an explicit workflow with idempotent steps and compensation.

## 4. Coordination scopes

| Code | Scope | Boundary |
|---:|---|---|
| 1 | `Key` | one immutable collection/key identity |
| 2 | `LocalHeap` | bounded identities in one embedded/single-server Heap |
| 3 | `Partition` | bounded identities in one qualified strong partition |

Every scope contains exactly one `HeapId`.

There is no wildcard Heap, Heap set, deployment scope, or implicit scope.

Before prepare, the engine MUST close the plan over:

- caller mutations;
- read versions;
- absence/range predicates;
- active RRE revisions;
- relationship/unique consequences;
- history events;
- index invalidation/publication consequences;
- idempotency and decision evidence.

If closure escapes the declared scope, execution fails before prepare.

## 5. Atomic identity

```text
AtomicId = 32 opaque bytes
```

Caller-generated IDs MUST come from a cryptographically secure random source
or a caller-owned stable idempotency derivation.

Engine-generated IDs are:

```text
BLAKE3-256(
  "DINGODB-ATOMIC-ID-V1"
  || heap_id
  || source_operation_id
  || invariant_or_job_id
)
```

An Atomic plan has:

```text
content_root = BLAKE3-256(
  "DINGODB-ATOMIC-CONTENT-V1"
  || canonical_plan_bytes
)
```

Rules:

- same `AtomicId` + same `content_root` resolves the original outcome;
- same `AtomicId` + different root returns `atomic_id_conflict`;
- an expired detailed receipt never permits re-execution;
- a minimal decision tombstone remains until Heap purge.

## 6. Canonical plan

Logical plan:

```text
AtomicPlan {
    profile
    atomic_id
    heap_id
    scope
    expected_frontier?
    reads[]
    predicates[]
    mutations[]
    active_rule_revisions[]
    caller_context_hash
    limits
}
```

Members are canonically ordered by:

```text
(heap_id, collection_id, canonical_key_bytes, member_kind, ordinal)
```

The first component is constant inside one plan but remains in the definition
to prohibit accidental cross-Heap reuse.

Canonical key bytes use the Heap key profile:

```text
string UTF-8 bytes
opaque byte string
mathematical integer canonical signed encoding
exact decimal canonical coefficient + scale
```

Boolean, Null, products, sequences, and floating point are not relationship or
ordered-lock keys in v1.

Paths use the exact canonical RRE path profile. No host-language path syntax is
accepted after compilation.

## 7. Predicates and reads

V1 supports:

- exact version equality;
- key absence;
- key presence;
- exact scalar equality;
- bounded key-range absence/presence when the index declares exact coverage;
- active rule revision equality;
- collection/object lifecycle state;
- Heap authority/security revision.

Every read records:

```text
ReadWitness {
    object_identity
    observed_version_or_absent
    projection_hash
}
```

Every absence/range predicate records the exact index/order domain and frontier
under which absence was observed.

A candidate or damaged index cannot prove absence.

## 8. Serialization and isolation

Read/write Atomics are serializable.

The LocalHeap reference algorithm is:

1. bind current Heap commit frontier;
2. read and record versions/predicates;
3. build the closed mutation plan;
4. acquire the LocalHeap commit sequencer;
5. validate Heap authority, rights, lifecycle, reads, predicates, and active
   invariant revisions;
6. allocate one Heap commit position;
7. persist prepare, members, and decision under the crash protocol;
8. publish one logical committed delta;
9. release sequencer;
10. return receipt only after the requested durability boundary.

The sequencer is an implementation mechanism, not the semantics. An optimistic
or parallel implementation is conforming only if histories are equivalent to
the same serial contract.

The commit position is:

```text
HeapCommitPosition = monotonically increasing nonzero u64
```

It is allocated per Heap. Exhaustion fails closed and requires a new profile.
Positions are never reused, including after compaction or restore retaining
identity.

## 9. Evidence

Canonical authoritative evidence:

```text
AtomicPrepare {
    atomic_id
    heap_id
    scope
    content_root
    frontier
    ordered_member_manifest_root
    read_set_root
    predicate_set_root
    active_rule_revision_root
    limits
}

AtomicMember {
    atomic_id
    ordinal
    object_identity
    member_kind
    before_version?
    after_content_hash?
    event_id
}

AtomicDecision {
    atomic_id
    prepare_hash
    member_root
    member_count
    decision
    commit_position?
    durability
}
```

Decision codes:

| Code | Decision |
|---:|---|
| 1 | committed |
| 2 | not committed |

`unknown_commit` and `conflicting_decision_evidence` are examination outcomes,
not decisions an engine intentionally writes.

Domain separators:

```text
DINGODB-ATOMIC-PREPARE-V1
DINGODB-ATOMIC-MEMBER-V1
DINGODB-ATOMIC-DECISION-V1
DINGODB-ATOMIC-MANIFEST-V1
DINGODB-ATOMIC-READSET-V1
DINGODB-ATOMIC-PREDICATES-V1
```

Persistent v1 uses deterministic CBOR under the repository canonical-CBOR
profile. Exact numeric field assignments MUST land in
`spec/atomics/cbor-v1.json` before `ATM-1`.

## 10. Outcomes

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

Status returns two independent axes.

Logical:

```text
committed
not_committed
unknown_commit
conflicting_decision_evidence
```

Material:

```text
complete
partial
missing
conflicting
coverage_incomplete
```

`committed + partial` means the transition committed but later damage removed
material. Healthy members remain examinable; current logical materialization
MUST NOT invent missing values.

## 11. Crash protocol

The implementation MUST provide failpoints:

```text
before_prepare
after_prepare
after_member_n
before_decision
after_decision
before_publish
after_publish
before_ack
```

Allowed restart outcomes:

- no valid prepare: not committed;
- valid prepare, no valid decision: unknown until deterministic recovery
  resolves or records abort under the recovery profile;
- one valid commit decision with complete manifest: committed;
- conflicting valid decisions: conflicting evidence, Heap degraded;
- commit decision with damaged members: committed + partial/missing.

Prepared members are never visible to ordinary reads.

## 12. Retry and retention

A compact exact tombstone:

```text
(atomic_id, content_root, decision, commit_position?, decision_hash)
```

is retained for the lifetime of the Heap identity and removed only by complete
Heap purge.

Detailed prepare/member/violation evidence is retained for at least:

```text
max(
  Heap history retention,
  active/retained RRE evidence requirement,
  configured Atomic detail retention
)
```

The default Atomic detail retention is 90 days. Heap policy MAY increase it.
It MAY lower it only when no active rule, legal hold, backup contract, or
history policy requires the evidence.

After detail removal, same-ID/same-root retry returns the retained decision
summary. It does not execute again.

## 13. Resource limits

V1 hard ceilings:

| Quantity | Key | LocalHeap / Partition |
|---|---:|---:|
| caller mutations | 1 | 256 |
| total generated members | 32 | 4,096 |
| canonical plan bytes | 256 KiB | 1 MiB |
| total proposed value bytes | 4 MiB | 8 MiB |
| read witnesses | 64 | 4,096 |
| predicates | 32 | 1,024 |
| affected collections | 1 | 64 |
| active rule revisions | 64 | 1,024 |
| construction deadline | 2 s | 5 s |
| emitted violations | 1,024 | 1,024 |

Heap policy MAY lower these ceilings and records the applied limits in the
prepare. Raising one requires a new Atomic profile.

Limit failure occurs before prepare whenever possible and never truncates
members, predicates, violations, or evidence silently.

## 14. Rights

Ordinary execution requires the union of rights for every proposed ordinary
data operation.

Administrative rights:

```text
RuleAdmin
AtomicAdmin
```

- `RuleAdmin`: create, validate, activate, replace, retire RRE rulesets.
- `AtomicAdmin`: create/change/validate/retire relationship, uniqueness, and
  other cross-document Atomic definitions.

These are independent HeapKey right bits and require a rights-registry version
amendment before network use.

Protected recovery modes require a non-serializable local `RecoveryCap`; they
are never granted by an ordinary application key.

Internal enforcement Atomics use a non-serializable engine capability derived
from the active rule/definition and still check the caller's ordinary data
rights.

## 15. External API

V1 external mutation is one-shot:

```rust
let plan = heap.atomic()
    .id(id)
    .expect_version(users, "u1", version)
    .put(users, "u1", updated)
    .build()?;

match heap.commit(plan)? {
    AtomicOutcome::Committed(receipt) => { /* visible */ }
    AtomicOutcome::NotCommitted { reason, .. } => { /* no effect */ }
    AtomicOutcome::Unknown { resolution, .. } => { /* resolve */ }
}
```

Remote API submits one immutable canonical plan. It does not open a transaction
session or hold a lock between client calls.

```rust
heap.atomic_status(atomic_id)?
```

resolves using evidence and explicit coverage.

Read-only snapshot sessions are deferred from v1.

## 16. RRE integration

At the serialization point:

1. load the exact active RRE revisions named by Heap state;
2. verify the plan's recorded revision root;
3. compute the complete affected projection;
4. evaluate every applicable invariant;
5. add derived consequences to the closed member set;
6. re-check package limits;
7. commit only when violations are empty.

There is no ordinary-write bypass.

Document-local rules use Key Atomic scope.
Reference, uniqueness, and bounded-cardinality rules require LocalHeap or
qualified Partition scope.

## 17. Relationship profile

V1 relationships support:

- required scalar reference;
- optional scalar reference;
- parent exists;
- `on delete restrict`;
- same-collection references;
- bounded sequence references when RRE declares a maximum;
- exact scalar key equality.

V1 permits relationship cycles because it has no cascade. Self-reference is
permitted only when the parent exists in the pre-state or is created in the
same Atomic and the final state satisfies the rule.

Relationship graphs need not be acyclic.

Parent/child conflicts use the canonical member ordering from §6 and
serializable validation. A concurrent parent deletion invalidates a child's
parent-exists predicate; a concurrent child insertion invalidates the parent's
no-children predicate. At most one conflicting transition commits.

The reverse-reference index is derived. It may nominate children, but absence
proves safety only when its declared coverage is complete and exact for the
Atomic frontier. Otherwise deletion refuses with `coverage_incomplete`.

## 18. Relationship activation

Activation over existing data:

1. create immutable definition;
2. install prospective enforcement barrier;
3. capture frontier;
4. scan complete parent/child scope;
5. build reverse index and report violations;
6. replay changes after frontier;
7. obtain serialization point;
8. validate no uncovered gap;
9. activate only with complete coverage and zero unaccepted violations.

Concurrent parent deletion conflicts with validation through the same
predicate/read-set mechanism. It cannot pass between scan and activation
unobserved.

Integrity status is exposed through dedicated rule/relationship inspection and
optional `read_with_integrity`. Ordinary reads do not automatically claim
relationship completeness and need not carry the full status payload.

## 19. Uniqueness

Unique values use:

- RRE canonical path;
- frozen normalization/comparison profile;
- Heap-bound exact reverse map;
- absence predicate at the Atomic frontier;
- canonical member ordering.

Null/Absent participation is declared by the rule. It is never inferred from
SQL convention.

Damage or incomplete coverage cannot prove uniqueness and causes refusal or
explicit degraded status.

## 20. Backup, restore, import, and salvage

Every data-entry mode is one of:

```text
enforce
validate_then_commit
quarantine_violations
trusted_rebuild
```

Ordinary import uses `enforce` or `validate_then_commit`.

`quarantine_violations` and `trusted_rebuild` require local `RecoveryCap` and
produce explicit evidence. They cannot publish violating records into ordinary
active state.

Payload restore to a new Heap:

- rewrites Heap identity;
- does not preserve source Atomic authority/cursors/capabilities;
- preserves historical decision/material evidence with provenance;
- revalidates active rules before ordinary service.

Salvage reports decisions, members, holes, conflicts, and coverage without
manufacturing a clean current state.

## 21. Partition profile

Partition Atomic v1 requires explicit co-partitioning of every read, predicate,
mutation, rule dependency, and generated consequence.

The partition consensus decision is authoritative for logical commit.
`Committed` acknowledgement requires:

- quorum commit of the canonical Atomic command;
- local application of the decision and members;
- requested durability evidence.

Follower material may apply later but cannot contradict the committed decision.
Cluster relationship rules remain disabled until placement and partition
qualification pass.

## 22. Error codes

Minimum stable codes:

```text
atomic_id_conflict
atomic_id_invalid
atomic_scope_escape
atomic_scope_unavailable
atomic_limit_exceeded
atomic_deadline_exceeded
atomic_read_conflict
atomic_predicate_conflict
atomic_rule_changed
atomic_rule_violation
atomic_not_committed
atomic_outcome_unknown
atomic_evidence_conflicting
atomic_coverage_incomplete
atomic_material_partial
atomic_right_denied
relationship_parent_missing
relationship_children_exist
relationship_degraded
unique_value_exists
```

Errors reveal nothing outside the caller's Heap and collection constraints.

## 23. Conformance

V1 requires:

- canonical encoding corpus;
- ID/content-root retry corpus;
- serial history model check;
- write-skew and phantom tests;
- crash at every §11 failpoint;
- two-Heap noninterference;
- rights matrix;
- RRE enforcement;
- parent insert/update/delete races;
- relationship activation with concurrent mutation;
- unique contention;
- damage to prepare/member/decision/index;
- backup/restore/salvage;
- remote timeout/reconnect/status;
- limit and hostile-plan corpus.

No capability is advertised beyond the scopes that pass.

## 24. Closed decisions

This specification resolves every open question from
`ATOMICS_PROPOSAL.md` §31:

1. IDs/evidence use §5 and §9 canonical profiles.
2. Limits are fixed by §13.
3. Retention is fixed by §12.
4. Heap commit positions and predicate witnesses are fixed by §7–§8.
5. Partition decision/material durability is fixed by §21.
6. Read-only snapshots are deferred.
7. Required and optional scalar relationships are included.
8. Paths and scalar key profiles are fixed by §6.
9. Same-collection references are allowed.
10. Acyclic graphs are not required without cascade.
11. Conflict ordering is fixed by §6.
12. Clustered v1 requires explicit co-partitioning.
13. Validation concurrency is fixed by §18.
14. Evidence retention is fixed by §12.
15. Integrity status uses dedicated/optional read surfaces.
16. Administration and recovery authority are fixed by §14.

An implementer has no remaining semantic choice in this list.
