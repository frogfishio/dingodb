# Formal kernel model contract

Status: **normative v1.0-draft — required by FAS-2 and FAS-3**

This contract fixes the common mathematical vocabulary, state, transition and
refinement surface of the Residiuum Formal Assurance Spine. An implementation
MUST NOT invent a competing meaning for an identifier, observation,
commitment, absence, authority or recovery result.

## 1. Semantic authority

The authorities are deliberately split:

| Concern | Canonical authority |
|---|---|
| Abstract values, state, operations and invariant theorems | Lean 4 sources under `formal/lean/Residiuum/` |
| Temporal actions, fairness and liveness | TLA+/TLAPS sources under `formal/tla/` |
| Connection to production Rust | Verus specifications and proofs under `formal/verus/` |
| Bounded concrete Rust obligations | Kani harnesses under `formal/kani/` |
| Registry identity, dependency and achieved status | JSON registries under `formal/registry/` |

The notation in this document is a readable projection of the Lean model. If
the prose and type-checked source disagree, the profile fails; neither silently
overrides the other. TLA+, Lean and Verus correspondence follows the explicit
bridge contract in
[FORMAL_ASSURANCE_REGISTRY_CONTRACT.md](FORMAL_ASSURANCE_REGISTRY_CONTRACT.md).

## 2. Primitive types

The following are nonempty opaque types with decidable equality:

```text
HeapId CollectionId Key Generation EventId PrincipalId CredentialId
AtomicId NodeId Term LogIndex RuleId Value EvidenceDigest Time
```

No theorem may depend on their byte encoding, lexical ordering or hash value.
An order may be introduced only by a named feature contract.

```text
QualifiedCollection := HeapId × CollectionId
ItemId             := HeapId × CollectionId × Key
GenerationRef       := ItemId × Generation
```

Collection and key equality is always qualified by Heap. There is no
unqualified collection namespace in the abstract model.

## 3. Closed constructors

### 3.1 Observations

```text
Observation α :=
  | complete(value : α, evidence : EvidenceDigest)
  | absent_proved(evidence : EvidenceDigest)
  | partial(evidence : EvidenceDigest)
  | damaged(evidence : EvidenceDigest)
  | unknown(evidence : EvidenceDigest)
  | unauthorized
  | unavailable(evidence : EvidenceDigest)
```

`absent_proved` requires authoritative evidence that the item has no visible
committed generation. Failure to find, scan or decode evidence is `unknown`,
`partial` or `damaged`, never absence.

### 3.2 Outcomes

```text
Outcome α :=
  | ok(value : α)
  | rejected(reason : RejectReason)
  | indeterminate(evidence : EvidenceDigest)
  | unavailable(evidence : EvidenceDigest)

RejectReason :=
  unauthorized | invalid_input | invariant_violation | conflict |
  stale_epoch | blacklisted | forbidden_surface | unsupported
```

An operation reports `indeterminate` when durable evidence cannot establish
whether the requested effect became authoritative.

### 3.3 Publication and decisions

```text
Publication := unpublished | prepared | committed | retired
AtomicDecision := undecided | commit | abort
NodeRole := follower | candidate | leader
```

These are disjoint constructors, not integer conventions.

## 4. Abstract state

The canonical `State` record is:

```text
State := {
  heaps              : Finset HeapId,
  collections        : Finset QualifiedCollection,
  generations        : ItemId -> Finset Generation,
  values             : GenerationRef -> Option Value,
  publication        : GenerationRef -> Publication,
  current            : ItemId -> Option Generation,
  generation_events  : GenerationRef -> Finset EventId,
  durable_events     : Finset EventId,
  damaged_events     : Finset EventId,
  coverage           : Finset EventId,
  credentials        : CredentialId -> Option Credential,
  heap_epoch         : HeapId -> Nat,
  blacklist          : HeapId -> Finset CredentialId,
  active_rules       : QualifiedCollection -> Finset RuleId,
  atomics            : AtomicId -> Option AtomicState,
  nodes              : NodeId -> Option NodeState,
  log                 : NodeId -> LogIndex -> Option LogEntry,
  membership         : Term -> Option Membership
}
```

`Credential`, `AtomicState`, `NodeState`, `LogEntry` and `Membership` are
feature records, but they MUST contain at least:

```text
Credential := { heap, subject, epoch, rights, kind, parent, not_before,
                not_after, signature }
AtomicState := { heap, exact_members, prepared_members, decision, evidence }
NodeState := { term, role, voted_for, commit_index, applied_index }
LogEntry := { term, heap, operation_digest, decision_digest }
Membership := { old_voters, new_voters, phase }
```

The model may add fields only through a versioned model change. A proof-local
auxiliary field is permitted when it is erased by the abstraction function and
cannot alter observations.

## 5. Well-formedness

`WellFormed(s)` is the conjunction of these named predicates:

```text
WF_CollectionsQualified
WF_GenerationOwnership
WF_CurrentCommitted
WF_CurrentUnique
WF_ValueEvidence
WF_DamageHonesty
WF_CredentialHeapBinding
WF_DelegationConfinement
WF_AtomicMemberQualification
WF_AtomicDecisionUnique
WF_LogIndexUnique
WF_MembershipNonempty
```

At minimum they mean:

1. every collection's Heap exists;
2. every generation belongs to exactly one qualified item;
3. `current(i)=g` implies `publication(i,g)=committed`;
4. at most one generation is current per item;
5. a complete value has authoritative events for its exact generation;
6. missing/damaged required evidence cannot establish complete or absent;
7. every credential is cryptographically bound to one Heap and epoch;
8. delegated rights are a subset of parent rights and stay in the same Heap;
9. every Atomic member is in the Atomic's Heap and names an exact operation;
10. an Atomic cannot have both authoritative commit and abort evidence;
11. a node has at most one entry per log index; and
12. every active membership has at least one voter.

Feature packages MAY strengthen `WellFormed`; they may not weaken these
predicates. Every state-changing operation proves preservation.

## 6. Initial state

`Init` contains no Heaps, collections, generations, credentials, Atomics,
nodes, logs or memberships. Total maps return their neutral value.

FAS-2 MUST prove:

```text
theorem init_well_formed : WellFormed Init
```

Bootstrap and Heap-creation ceremonies are operations from `Init`; they are
not assumed pre-existing state.

## 7. Operation vocabulary

`Input` is a closed tagged union:

```text
create_heap | create_collection
put | delete | get | scan
recover | reassemble | repair
issue_credential | blacklist_credential | rotate_epoch
atomic_prepare | atomic_decide | atomic_recover
cluster_append | cluster_elect | cluster_change_membership | cluster_repair
```

Every constructor carries all qualified identities, credential/evidence and
request identity required by its feature contract. There are no ambient
current-Heap or current-collection variables.

The transition relation is:

```text
Step : State -> Input -> State -> Outcome (Observation Value) -> Prop
```

For read-only operations, `s' = s`. For rejected operations, `s' = s` unless
the operation's contract explicitly permits append-only rejection evidence;
that evidence MUST NOT change a data observation.

Crash behavior is expressed by a separate relation:

```text
CrashStep : ConcreteExecutionPrefix -> ConcreteState -> Prop
RecoverStep : State -> State -> Outcome Unit -> Prop
```

A crash is not an ordinary input and cannot authorize an otherwise forbidden
state transition.

## 8. Operation obligations

Every operation constructor has one `OperationContract` registry entry:

```text
operation_id
input_type
precondition
transition_relation
outcome_relation
read_or_write
deterministic_or_relational
preserved_invariants
authorization_predicate
crash_points
Rust entrypoints
Lean symbols
TLA+ actions, if temporal
Verus spec functions
```

FAS-2 rejects an operation with no abstract contract. FAS-3 rejects a claimed
production entrypoint with no operation mapping, and a mapped state-changing
entrypoint with an empty preservation set.

## 9. Observation law

```text
Observe : PrincipalId -> Scope -> State -> Observation Value
```

Observation is pure and total. It first establishes authority, then derives
knowledge from authoritative evidence. It does not use a derived index as
truth.

The forbidden-collapse relation contains exactly:

```text
partial       -> absent_proved
partial       -> complete
damaged       -> absent_proved
damaged       -> complete
unknown       -> absent_proved
unknown       -> complete
unauthorized  -> absent_proved
unavailable   -> absent_proved
prepared      -> committed
stale_epoch   -> authorized
minority      -> quorum_committed
```

A public projection may coarsen multiple failure states to a generic error,
but it MUST preserve machine-readable discrimination and cannot project one
of these pairs.

## 10. Refinement contract

FAS-3 defines total abstraction functions over every reachable concrete state:

```text
alpha_state       : ConcreteState -> State
alpha_input       : ConcreteInput -> Input
alpha_outcome     : ConcreteOutcome -> Outcome Observation
alpha_observation : ConcreteObservation -> Observation Value
```

For every production entrypoint and reachable concrete transition
`c --x/o--> c'`, forward simulation requires:

```text
Step (alpha_state c) (alpha_input x) (alpha_state c') (alpha_outcome o)
```

For crash prefixes, the abstract post-state must be one permitted by the
registered crash/publication relation. Stuttering is allowed only when the
abstract state and observation are unchanged.

The bridge MUST prove or explicitly register:

- initial-state correspondence;
- state invariant correspondence;
- operation forward simulation;
- observation preservation;
- error/outcome preservation;
- crash/recovery simulation;
- feature-flag and profile binding; and
- reachable unsafe/FFI boundaries.

Testing common vectors across tools is evidence for semantic-map review. It is
not a proof that the tools' models are identical.

## 11. Model-change law

A change to a primitive type, closed constructor, state field semantics,
`WellFormed`, `Input`, `Step`, `Observe`, forbidden collapse or abstraction
function is a semantic model change. It SHALL:

1. increment the relevant model version;
2. identify superseded symbols;
3. revoke the dependent theorem closure;
4. rerun cross-tool differential vectors and negative controls; and
5. require principal acceptance before a public profile is restored.

Adding a lemma without changing definitions is not a model change, but it
still changes source and result hashes.

## 12. FAS-2 acceptance

FAS-2 is accepted only when:

```text
bash scripts/check-formal-foundation.sh
```

passes and emits:

```text
target/formal-assurance/fas2-foundation-report.json
```

The command MUST prove/type-check:

- the closed constructors and their separation;
- `init_well_formed`;
- operation contract completeness;
- total observation construction;
- forbidden-collapse impossibility;
- `WellFormed` preservation for the foundation operations; and
- accepted/rejected finite model vectors.

## 13. FAS-3 acceptance

FAS-3 is accepted only when:

```text
bash scripts/check-formal-refinement.sh
```

passes and emits:

```text
target/formal-assurance/fas3-refinement-report.json
```

It MUST include one complete vertical slice through registry, Lean statement,
Verus-to-production connection, bounded Kani obligation where applicable,
negative controls, concrete/abstract vector agreement and CSQ evidence link.
Renaming or mutating the connected Rust entrypoint MUST make the slice fail.
