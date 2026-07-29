# DingoDB Data Rules proposal

Status: Exploratory unified draft v0.1

Scope: Declarative document rules, transition rules, referential integrity,
formal semantics, proof obligations, and Atomic enforcement

Normative companions: `ATOMICS_PROPOSAL.md`, `HEAP_SPEC.md`, `DQL_SPEC.md`,
`SDA_SPEC.md`, `SDA_PROFILE.md`, and `DX_SPEC.md`

## 1. Decision

DingoDB should provide a small, declarative, non-Turing-complete language for
describing valid stored state and valid state transitions.

The language is called **Data Rules** in the product. Its human surface should
be visually compatible with DQL, while its meaning is a separate, stricter
mathematical invariant kernel.

Data Rules are not:

- callbacks;
- stored procedures;
- triggers;
- JavaScript;
- user-defined runtime functions;
- arbitrary SDA programs;
- arbitrary DQL queries;
- application code that DingoDB promises to remember to call.

A Data Rule denotes a finite mathematical predicate. Compilation determines
its complete dependency set, required Atomic scope, execution bound, canonical
bytecode, semantic version, and verification evidence.

The engine admits a state transition only when the compiled predicate holds at
the Atomic serialization point.

The intended product statement is:

> Flexible documents with mathematically specified, database-enforced truth.

## 2. The product quadrant

The market is commonly divided along two independent axes:

| | Truth maintained by applications | Truth maintained by the database |
|---|---|---|
| Fixed or relational shape | loosely governed relational use | relational constraints and foreign keys |
| Flexible or document shape | conventional document-store practice | **DingoDB Data Rules** |

DingoDB's intended quadrant is:

```text
document-native shape
        +
database-owned invariants
        +
formal semantics
        +
Atomic enforcement
```

Performance and physical survival are independent multipliers:

```text
Data Rules       define legal truth
Atomics          enforce legal transitions
heap isolation   bounds authority and state
storage evidence preserves what remains knowable
SDA              examines rules, decisions, data, and holes
bounded execution keeps the path fast
```

DingoDB is not claiming to reproduce PostgreSQL, SQL, joins, or arbitrary
distributed ACID. It is extracting a central value of a mature relational
database—the database owns declared integrity—without forcing documents into
one universal row shape.

It is also not differentiated merely by having validation or multi-document
writes. The differentiator is the combined contract:

> A declared rule has formal meaning, compilation is mechanically checkable,
> its dependencies must fit a proven Atomic scope, and invalid after-states
> are excluded from the commit relation.

## 3. One architecture, not three features

Referential integrity, conditional document validation, and transition rules
must not become separate enforcement engines.

The architecture is:

```text
Atomics
└── Data Rules
    ├── document predicates
    ├── conditional presence
    ├── scalar types and ranges
    ├── transition predicates
    ├── immutable fields
    ├── uniqueness
    ├── referential integrity
    └── bounded cardinality
```

Data Rules answer:

```text
Which states and transitions are legal?
```

Atomics answer:

```text
How does one legal transition become indivisibly visible?
```

SDA examination answers:

```text
Which declaration, compiled artifact, decision evidence, surviving values,
and holes can be established now?
```

Referential integrity is the first cross-document standard construct in Data
Rules. It is not a privileged alternative to the language.

## 4. Design principles

### 4.1 Declare states, do not program procedures

Users describe facts that must hold. They do not describe an execution order.

Good:

```text
require cup_size when sex = "F"
```

Not supported:

```text
if sex is F:
    call something
    modify another record
    retry until successful
```

### 4.2 Comfort never redefines truth

The relationship between the layers mirrors DQL and SDA:

| Layer | Role |
|---|---|
| Data Rules surface | Human-readable declarations |
| Invariant Core | Canonical mathematical meaning |
| Verified artifact | Dependencies, scope, bytecode, bounds, and certificate |
| Atomic engine | Serialization, evidence, and indivisible publication |

The friendly surface may evolve. Invariant Core semantics are versioned and
stable. If a surface construct cannot lower faithfully, compilation fails.

### 4.3 Absence and `null` are different

Data Rules preserve DingoDB's value distinctions:

```text
absent       the path is not present
null         the path is present and stores Null
value        the path is present and stores a typed value
unavailable  physical evidence is presently missing or damaged
```

`unavailable` is a recovery/material condition, not a document value and not
an input that ordinary writes may manufacture.

Business states such as `unknown`, `withheld`, and `inapplicable` should be
modeled explicitly as tagged document values when the application needs those
distinctions.

### 4.4 Rules are optional and progressive

A heap with no Data Rules retains the ordinary experience:

> Put anything in. Get it back.

Collections without active rules pay no rule-evaluation cost. Adding one rule
does not impose a universal schema on unrelated collections.

### 4.5 No silent weakening

DingoDB rejects a declaration when it cannot prove:

- total lowering;
- finite evaluation;
- complete dependencies;
- an available coordination scope;
- supported placement;
- stable semantics for every operator.

It never converts a strong rule into a warning, an eventual check, an
application responsibility, or a best-effort background task without an
explicitly different declaration and name.

## 5. DQL-compatible human surface

The surface is line-oriented, uses lower-case keywords in documentation, uses
dot paths, and borrows DQL's `using`, `matching`, and `expect` vocabulary.
Keywords should be case-insensitive as in DQL. Identifiers remain
case-sensitive.

Visual compatibility does not make Data Rules part of DQL. DQL reads and
enriches artefacts. Data Rules constrain states and transitions.

### 5.1 Document rules

Illustrative surface:

```text
rules for people

require name as string
require age as integer
require sex as enum("F", "M", "X")

constrain age
  where age >= 0 and age <= 150

optional cup_size as string
require cup_size
  when sex = "F"
```

Meaning:

- `name`, `age`, and `sex` must be present and non-null with the declared
  types;
- `age` must be within the declared range;
- `cup_size`, when present, must be a string;
- when `sex = "F"`, `cup_size` must be present.

The example demonstrates conditional presence. It is not a claim that this is
a correct universal model of people. A semantically richer application may
write:

```text
rules for people

require bra_size_status as enum(
  "known",
  "unknown",
  "inapplicable",
  "withheld"
)

optional bra_size as product {
  band: integer,
  cup: string,
  system: enum("UK", "US", "EU")
}

require bra_size
  when bra_size_status = "known"

forbid bra_size
  when bra_size_status != "known"
```

### 5.2 Referential integrity

The declaration deliberately resembles DQL enrichment:

```text
rules for orders

require customer_id as key

reference customer using customers
  matching customer_id = _key
  expect exactly_one
  on delete restrict
```

Compare the read-side DQL:

```text
from orders

enrich customer using customers
  matching customer_id = _key
  expect exactly_one
```

The shared visual grammar teaches one relationship:

```text
orders.customer_id → customers._key
```

The meanings remain distinct:

```text
enrich      read and attach the related artefact
reference   preserve the relationship as an invariant
```

`expect exactly_one` is semantic, not documentation. For every live order,
there must be exactly one live matching customer at the serialization point.

An optional reference is:

```text
reference sponsor using sponsors
  matching sponsor_id = id
  expect optional
  on delete restrict
```

Here absence of `sponsor_id` is legal. When the path is present, no more than
one matching parent is legal, and the first profile requires it to exist.

### 5.3 Uniqueness

Illustrative surface:

```text
rules for users

unique email
  normalize unicode_nfc
  compare codepoint
```

Every normalization and comparison operator must name frozen semantics.
Locale-dependent ambient behavior is forbidden.

### 5.4 Transition rules

Some truths concern change rather than one isolated document:

```text
rules for accounts

freeze account_id
  after create

allow status
  from "draft" to "submitted"
  from "submitted" to "approved"
  from "submitted" to "rejected"
  from "approved" to "closed"
```

The normalized meaning is a relation over before-state and after-state. This
is not an instruction to execute transitions automatically.

### 5.5 Bounded cardinality

Illustrative surface:

```text
rules for teams

reference members using users
  matching member_ids each = id
  expect between 1 and 20
  on delete restrict
```

Unbounded traversal is not implied. The declaration supplies a statically
enforceable maximum. Exact v1 collection-path syntax remains an open decision.

### 5.6 Composition

Rules compose by conjunction:

```text
all active rules must hold
```

There is no declaration order and no short-circuit meaning. An implementation
may optimize evaluation order only when the observable violation set and
resource semantics remain equivalent.

Named reusable fragments may be added later if expansion is finite, hygienic,
and visible in the compiled artifact. User-defined executable functions are
not permitted.

## 6. Invariant Core

### 6.1 State model

Let:

- `H` be one heap;
- `S` be one complete logical committed state of `H`;
- `S'` be a proposed after-state;
- `o` be one proposed operation;
- `⊥` denote path absence;
- `Null` denote the stored Null value;
- `R` be the finite set of active rule revisions.

Absence and Null are distinct:

```text
⊥ ≠ Null
```

Each rule revision denotes a total predicate:

```text
⟦r⟧ : State × State × Operation → Bool
```

Document-only rules may ignore `S` and `o`. Transition rules may inspect
bounded projections of `S`, `S'`, and `o`. Cross-document rules inspect only
their statically declared dependencies.

The active invariant is:

```text
I_R(S, S', o) ≜ ∧ r ∈ R • ⟦r⟧(S, S', o)
```

### 6.2 Conditional presence

The declaration:

```text
require cup_size
  when sex = "F"
```

normalizes to:

```text
sex(d) ≠ "F" ∨ cup_size(d) ≠ ⊥
```

equivalently:

```text
sex(d) = "F" ⇒ defined(cup_size(d))
```

If `cup_size` is declared `as string`, explicit Null does not satisfy the
requirement.

### 6.3 Referential integrity

For child collection `C`, parent collection `P`, child path `f`, and immutable
parent key `k`, a required exactly-one reference means:

```text
∀ c ∈ Live(C) •
    defined(c.f)
    ∧
    ∃! p ∈ Live(P) • p.k = c.f
```

`∃!` means “there exists exactly one.”

For an optional reference:

```text
∀ c ∈ Live(C) •
    ¬defined(c.f)
    ∨
    ∃! p ∈ Live(P) • p.k = c.f
```

`on delete restrict` constrains the transition relation:

```text
DeleteParent(p, S, S') is legal
    ⇔
Referrers(p, S at serialization point) = ∅
```

Child insertion, child reference change, and parent deletion share one
ordering domain. A check followed by an unrelated write is not sufficient.

### 6.4 Uniqueness

For a normalized key function `N`:

```text
∀ a, b ∈ Live(C) •
    defined(a.f)
    ∧ defined(b.f)
    ∧ N(a.f) = N(b.f)
    ⇒ a.id = b.id
```

The compiler includes `N` and its semantic version in the artifact identity.

### 6.5 Transition relations

Immutability of field `f` after creation means:

```text
exists_before(d)
    ⇒
value(S, d, f) = value(S', d, f)
```

An allowed transition graph is a finite relation:

```text
Allowed ⊆ Status × Status
```

and:

```text
(before.status, after.status) ∈ Allowed
```

### 6.6 Violation result

Evaluation returns a finite, deterministic result:

```text
RuleResult =
    Valid
  | Violations(ordered finite set of Violation)
```

Each violation contains:

```text
rule_id
rule_revision
stable_code
bounded paths
expected proposition
observed value hashes or safe summaries
```

Values and secrets are excluded from diagnostics by default.

## 7. Compilation and proof artifact

Compilation is the expensive semantic step. Execution is shared and bounded.

```text
Data Rules source
        ↓ parse
canonical AST
        ↓ normalize
Invariant Core predicate
        ↓ analyze
dependencies + required scope + cost bound
        ↓ compile
canonical bytecode
        ↓ certify
verification artifact
```

Conceptual artifact:

```text
InvariantArtifact {
    heap_id
    rule_id
    revision
    source
    source_hash
    canonical_ast
    normalized_predicate
    dependency_set
    required_scope
    maximum_cost
    bytecode
    semantics_version
    compiler_profile
    artifact_hash
    proof_certificate
}
```

Rule revisions and artifacts are immutable. A changed declaration creates a
new revision and activation protocol.

### 7.1 Compiler-correctness obligation

Let:

```text
Compile(r) = b
```

The required semantic equivalence is:

```text
∀ r, x • Evaluate(Compile(r), x) = ⟦r⟧(x)
```

A verifier accepting an artifact must establish:

```text
Verify(r, b, certificate) = true
    ⇒
∀ x • Evaluate(b, x) = ⟦r⟧(x)
```

The certificate format may evolve, but the product must never describe
ordinary compiler output as a proof without an independently checkable
relationship.

### 7.2 Totality

For every accepted artifact and every well-formed bounded input:

```text
Evaluate(b, x) ∈ RuleResult
```

The evaluator does not panic, diverge, consult ambient state, or return an
implementation-defined result.

### 7.3 Termination and resource bound

For every accepted artifact:

```text
∀ x • Evaluate(b, x) terminates
```

and:

```text
Cost(b, x) ≤ Bound(b, Size(x))
```

The artifact records the bound. The compiler rejects declarations whose bound
cannot be derived or whose configured maximum exceeds heap policy.

### 7.4 Determinism

For identical:

```text
artifact
input state projection
operation
semantics version
```

evaluation returns byte-for-byte equivalent ordered results.

No rule may depend on:

- wall-clock `now`;
- random values;
- process environment;
- filesystem or network state;
- node identity;
- iteration encounter order;
- locale;
- platform floating-point behavior;
- mutable external code.

### 7.5 Dependency completeness

For rule `r`, the compiler derives:

```text
Dependencies(r)
```

Every value capable of changing the result must appear in that set:

```text
EqualProjection(S1, S2, Dependencies(r))
    ⇒
⟦r⟧(S1) = ⟦r⟧(S2)
```

This is the dependency-completeness obligation. An opaque query or function
whose dependencies cannot be established is not a legal rule.

## 8. Atomic enforcement

### 8.1 Commit relation

Let:

- `T(S, o) = S'` be the deterministic proposed transition;
- `P(S, o)` be ordinary operation preconditions;
- `I_R(S, S', o)` be all applicable active rules.

An Atomic may commit only when:

```text
S' = T(S, o)
∧ P(S, o)
∧ I_R(S, S', o)
```

The evaluation occurs at the Atomic serialization point over the complete
dependency projection.

If any predicate is false:

```text
NotCommitted
```

and no member becomes logically visible.

### 8.2 Preservation theorem

Assume an initial committed state:

```text
I(S₀)
```

and a sequence of committed Atomics:

```text
S₀ → S₁ → S₂ → ... → Sₙ
```

The commit relation requires:

```text
Commit(Sᵢ, Sᵢ₊₁) ⇒ I(Sᵢ₊₁)
```

Therefore, by induction:

```text
∀ n • I(Sₙ)
```

Proof:

```text
Base:
    I(S₀)

Step:
    assume I(Sᵢ)
    a committed transition is admitted only if I(Sᵢ₊₁)
    therefore I(Sᵢ₊₁)

Conclusion:
    every reachable committed state satisfies the active invariant
```

This is the central Data Rules guarantee.

### 8.3 No callback bypass

Rule enforcement belongs to the single Atomic commit gate. It is not attached
independently to SDK methods, HTTP routes, import tools, administrative
commands, or storage adapters.

Any path claiming normal committed writes must pass the same gate. Recovery
and raw salvage may preserve violating or unproven material, but cannot publish
it as ordinary committed state.

### 8.4 Serialization requirement

Serializable equivalence is required inside the rule's declared scope.
Snapshot validation alone is insufficient for invariants vulnerable to write
skew.

For a reference, the child write, parent existence, reverse-reference update,
and conflicting parent deletion must be ordered together.

## 9. Scope inference and proof

Each rule compiles to one required Atomic scope:

| Rule class | Inputs | Initial scope |
|---|---|---|
| shape, type, range | proposed document | `Key` |
| conditional presence | proposed document | `Key` |
| transition, immutable field | before and proposed document | `Key` |
| uniqueness | collection key domain | `LocalHeap` or qualified `Partition` |
| reference | child, parent, reverse-reference domain | `LocalHeap` or qualified `Partition` |
| bounded cardinality | protected relationship domain | `LocalHeap` or qualified `Partition` |

The proof obligation is:

```text
Dependencies(r) ⊆ AtomicScope(o)
```

Before prepare, DingoDB closes the Atomic plan over:

- caller mutations;
- rule reads;
- reverse-reference changes;
- unique-key reservations;
- index consequences required for enforcement;
- history and idempotency evidence;
- conflicting operations that require ordering.

If closure escapes the declared scope, the operation fails before recording an
Atomic member.

No rule can request:

- an implicit cross-heap scope;
- an implicit cross-partition scope;
- a global cluster scan;
- eventual validation presented as strong enforcement.

## 10. Heap noninterference

Every artifact is bound to exactly one immutable `HeapId`.

For distinct heaps:

```text
H₁ ≠ H₂
```

a transition authorized for `H₁` must satisfy:

```text
πH₂(T_H₁(S)) = πH₂(S)
```

where `πH₂` projects the entire logical state of `H₂`.

Rule evaluation for `H₁` receives no representable binding for state in `H₂`:

```text
Inputs(Evaluate(r_H₁, S)) ⊆ State(H₁)
```

Cross-heap references are therefore not merely rejected by a naming
convention. They are outside the rule artifact's authority and dependency
type.

Data Rules do not weaken the heap separation proof in `HEAP_SPEC.md`.

## 11. Rule lifecycle

### 11.1 Definition

Creating a rule:

1. parses and normalizes the declaration;
2. derives dependencies and scope;
3. derives resource bounds;
4. compiles canonical bytecode;
5. verifies the artifact;
6. stores an immutable proposed revision.

Definition does not imply activation.

### 11.2 Activation over existing data

The preservation theorem requires a valid base state:

```text
I(S₀)
```

Activation must therefore:

1. bind a validation frontier `F`;
2. validate all in-scope existing state at `F`;
3. order concurrent mutations after or within the validation protocol;
4. record every violation and coverage hole;
5. commit the rule revision and frontier atomically only when validation is
   complete and valid.

Lifecycle states:

```text
proposed
compiled
validating
violated
coverage_incomplete
active
retiring
retired
```

Only `active` rules participate in the normal commit relation.

### 11.3 Revision

Changing a rule creates a new immutable revision. The new revision undergoes
the activation protocol. The old revision remains active until the switch
Atomic commits.

Receipts and historical events retain the exact rule revisions applicable at
their serialization point.

### 11.4 Retirement

Retirement stops future enforcement only after an authorized Atomic decision.
It does not erase:

- the declaration;
- compiled artifacts;
- historical applicability;
- violation evidence;
- receipts;
- recovery provenance.

Derived enforcement indexes may be reclaimed only after retention and recovery
requirements permit it.

## 12. Damage, recovery, and truth

Data Rules preserve DingoDB's two-dimensional truth.

An event may be:

```text
logically committed
        +
physically partial
```

This does not mean the event violated its rules at commit. It means later
damage destroyed some present material.

Recovery must distinguish:

```text
rule held at original commit
rule evaluation evidence survives
current document material is partial
current re-evaluation is impossible
```

It must not rewrite historical commitment because current bytes are missing.

Likewise, surviving prepared members that appear rule-valid do not become
committed without valid Atomic decision evidence.

Rule artifacts are historical evidence and must be:

- content addressed;
- versioned;
- self-describing enough for long-term interpretation;
- preserved through compaction;
- copied by evidence-preserving salvage;
- examinable when unsupported by the current runtime.

## 13. SDA examination

SDA does not become the rule authority. It examines the evidence supplied by
the host.

The DingoDB SDA profile should expose:

```text
rule source
canonical predicate
artifact hash
semantics version
dependencies
required scope
activation frontier
lifecycle state
evaluation result or witness
Atomic ID and decision
coverage
surviving material condition
```

An examiner must be able to ask:

```text
Which rule revision applied?
What mathematical predicate did it denote?
Was its artifact verified?
Which state dependencies were covered?
Which Atomic decision admitted the transition?
What material survives now?
Which conclusions are impossible because of holes?
```

Unknown future artifacts are preserved losslessly and reported as unsupported.
They are never interpreted using a convenient older rule version.

## 14. Stable errors and receipts

Initial stable error families:

```text
rule_parse_error
rule_unsupported_construct
rule_unsatisfiable
rule_dependency_unprovable
rule_scope_unavailable
rule_cost_unbounded
rule_artifact_invalid
rule_violation
rule_activation_violated
rule_activation_coverage_incomplete
rule_revision_conflict
reference_missing
reference_duplicate
reference_restrict_delete
unique_conflict
transition_forbidden
```

A normal rule rejection is definitely not committed.

Conceptual receipt:

```text
RuleDecision {
    atomic_id
    heap_id
    rule_revisions
    artifact_hashes
    input_projection_root
    result
    bounded violation witnesses
    serialization_position
    achieved_durability
}
```

The receipt is a projection of Atomic evidence, not a second source of truth.

## 15. Security and administration

Rule administration requires an explicit heap-bound administrative capability.

Ordinary write capabilities cannot:

- create or activate rules;
- replace artifact bytecode;
- change semantic versions;
- bypass an active rule;
- write enforcement indexes directly;
- bind another heap;
- reinterpret recovery material as committed.

There is no normal network “disable validation for this write” flag.

Repair and import tools must choose an explicit mode:

```text
normal       enforce active rules and publish ordinary committed state
quarantine   preserve material without ordinary visibility
salvage      preserve evidence, violations, holes, and provenance
```

Administrative actions are themselves heap-bound, evidenced, and examinable.

## 16. Performance model

The formal restrictions are performance features:

- compilation is amortized;
- runtime bytecode is canonical and bounded;
- no user script interpreter exists;
- no network, clock, or filesystem access exists;
- dependency access is explicit;
- enforcement indexes are chosen from known predicates;
- scope is known before execution;
- no global coordinator is introduced implicitly;
- collections without rules have no rule-evaluation path.

The hot path is:

```text
load verified artifact
        ↓
load bounded dependency projection
        ↓
evaluate deterministic bytecode
        ↓
commit or reject one Atomic
```

Required benchmarks:

- no-rule write baseline;
- one shape rule;
- ten document predicates;
- conditional presence;
- immutable-field transition;
- unique insertion under low and high contention;
- reference insertion;
- restricted parent deletion;
- validation artifact cache miss and hit;
- local and partition Atomic enforcement;
- violation diagnostics;
- recovery with missing artifact or member evidence.

Reports must disclose rule count, instruction bound, dependency reads,
contention, durability, verification mode, and hardware.

## 17. Trusted computing base

Mathematics does not excuse implementation bugs. The guarantee is relative to
an explicit trusted computing base:

```text
Invariant Core semantics
canonical decoder
certificate verifier
bounded evaluator
dependency and scope enforcement
Atomic commit gate
heap authority enforcement
cryptographic primitives
declared runtime and hardware assumptions
```

The goal is to keep this base small, auditable, fuzzable, and progressively
verifiable.

DingoDB must not use “formally verified database” as a product claim until the
relevant implementation has actually reached that standard.

Defensible staged claims include:

```text
formally specified
mathematically defined
mechanically checkable
proof-carrying artifacts
model-checked protocols
bounded deterministic execution
```

## 18. Conformance and proof-derived testing

Tests derive from the formal obligations.

### 18.1 Surface equivalence

- equivalent source formatting produces one canonical predicate;
- Data Rules and direct Invariant Core forms produce equivalent artifacts;
- unsupported constructs fail rather than lower approximately;
- keyword case does not change meaning;
- DQL-like visual constructs retain their documented direction and
  cardinality.

### 18.2 Value semantics

- absent differs from Null;
- required rejects absent and Null unless Null is explicitly in the type;
- optional permits absence but validates present values;
- Unicode, decimals, dates, overflow, and comparison use frozen semantics;
- damaged/unavailable material never becomes an ordinary document value.

### 18.3 Compiler and verifier

- randomized source/AST/bytecode equivalence;
- malformed artifact and certificate corpora;
- dependency mutation checks;
- instruction and memory bound enforcement;
- architecture-independent golden results;
- unsupported semantic versions preserve but do not execute.

### 18.4 Atomic preservation

- crash at every prepare/member/decision boundary;
- concurrent child insertion and parent deletion;
- concurrent unique insertions;
- write-skew attempts;
- rule revision during writes;
- retry with identical and conflicting Atomic IDs;
- no partial visibility after violation or crash.

### 18.5 Lifecycle

- activation over valid existing data;
- activation with violations;
- activation with coverage holes or offline tiers;
- concurrent mutation at validation frontier;
- revision with narrower and wider predicates;
- retirement and derived-index reclamation;
- backup, restore, migration, compaction, and salvage.

### 18.6 Heap and cluster scope

- cross-heap identifiers and names;
- stale collection names with immutable identities;
- same-partition enforcement;
- accidental second partition;
- placement change during evaluation;
- unsupported convergent mode;
- damaged consensus evidence.

## 19. Non-goals for the first profile

The first profile does not include:

- arbitrary theorem proving at write time;
- arbitrary first-order quantification over an unbounded heap;
- user recursion or loops;
- arbitrary regex engines;
- floating-point predicates;
- locale-dependent comparison;
- wall-clock-relative rules;
- network or service lookups;
- scripts, callbacks, or user functions;
- cascade delete;
- cyclic cascading effects;
- cross-heap rules;
- arbitrary cross-partition rules;
- triggers that manufacture new business effects;
- rules over external side effects;
- retroactive claims that damaged or unvalidated historical data was valid;
- a bypass flag on ordinary writes.

These are boundaries required for truth, boundedness, and speed.

## 20. Delivery sequence

### D0 — Freeze the semantic subset

- value kinds and absence;
- canonical paths;
- Boolean and comparison semantics;
- document and transition predicate forms;
- canonical AST and normalization;
- stable violation model.

### D1 — Document rules

- shape, type, required, optional, forbid, range, enum;
- conditional presence;
- immutable revisions and artifacts;
- reference evaluator and conformance corpus.

### D2 — Verified execution

- canonical bytecode;
- static bounds;
- artifact verifier;
- Atomic `Key` enforcement;
- receipts and SDA examination.

### D3 — Transition rules

- before/after semantics;
- immutable fields;
- finite transition relations;
- retry and history integration.

### D4 — Referential integrity

- immutable collection identities;
- reverse-reference index;
- exactly-one and optional reference;
- `on delete restrict`;
- LocalHeap Atomic enforcement;
- activation and recovery.

### D5 — Uniqueness and bounded cardinality

- frozen normalization;
- contention behavior;
- bounded relationship collections;
- rebuild and salvage.

### D6 — Partition qualification

- placement proof;
- Partition Atomic integration;
- leader, epoch, quorum, and damage tests;
- explicit refusal outside qualified placement.

## 21. Open decisions

Before D0 closes:

1. Final product name and dialect identifier for Data Rules.
2. Exact canonical path grammar.
3. Exact scalar types, decimal model, and Unicode profile.
4. Whether Boolean syntax uses `=`, `!=`, `and`, `or`, and `not` exactly as
   illustrated.
5. Whether v1 supports product and bounded sequence types.
6. Canonical violation ordering.
7. Initial bytecode and certificate form.
8. Whether proof checking reconstructs canonical bytecode or validates a
   separate certificate.
9. Maximum instructions, paths, dependencies, and bounded collection size.
10. Exact relationship syntax for scalar and bounded-many references.
11. Whether optional references permit explicit Null or require absence.
12. Rule activation frontier and retention encoding.
13. Which rule-evaluation evidence is stored per commit versus derivable.
14. How much Invariant Core notation is exposed as a pure expert surface.
15. The formal connection between DQL `enrich` cardinality and Data Rules
    `reference` cardinality.

## 22. Recommendation

Proceed with Data Rules as the unified declarative layer above Atomics.

The governing statement is:

> DingoDB does not execute user programs. It enforces finite declarations
> about valid data and valid state transitions.

The central mathematical claim is:

> Given a valid activation frontier, every reachable committed state satisfies
> every active rule whose dependencies are contained by the Atomic scope.

The product claim is:

> Flexible documents. Mathematical rules. Atomic enforcement. Surviving
> truth.

Referential integrity should be the first cross-document demonstration because
it solves a familiar document-database pain and exercises the complete
architecture: dependency inference, supporting indexes, concurrency,
activation, recovery, damage, and examination.

The surface should feel like DQL:

```text
from orders
enrich customer using customers
  matching customer_id = _key
  expect exactly_one
```

becomes, on the enforcement side:

```text
rules for orders
reference customer using customers
  matching customer_id = _key
  expect exactly_one
  on delete restrict
```

The resemblance is intentional. One describes how related data is read. The
other declares that the relationship must remain true.
