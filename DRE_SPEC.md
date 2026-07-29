# Dingo Rule Expression (DRE) specification

Status: **Normative design v1.0-draft**

Language name: **Dingo Rule Expression (DRE)**

Product capability: **Data Rules**

Dialect identifier: `dre`

Artifact profile: `dingo-dre-artifact-v1`

Scope: Declarative document rules, transition rules, referential integrity,
formal semantics, proof obligations, and Atomic enforcement

Normative companions: `ATOMICS_PROPOSAL.md`, `HEAP_SPEC.md`, `DQL_SPEC.md`,
`DINGO_PREDICATE_SPEC.md`, `SDA_SPEC.md`, `SDA_PROFILE.md`, and `DX_SPEC.md`

## 1. Decision

DingoDB provides a small, declarative, non-Turing-complete language for
describing valid stored state and valid state transitions.

The language is named **Dingo Rule Expression**, abbreviated **DRE**. One DRE
denotes one invariant; a DRE ruleset is the immutable deployment container.
“Data Rules” remains the plain-English name of the product capability. Its
source dialect identifier is `dre`.

Its human surface is visually compatible with DQL and imports the exact
`dingo-predicate-v1` predicate profile. Its meaning is nevertheless a separate,
stricter mathematical invariant kernel.

DREs are not:

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
Invariant Core IR, semantic version, and verification evidence.

The engine admits a state transition only when the compiled predicate holds at
the Atomic serialization point.

The architectural boundary is final:

```text
DQL          read, relate, order, and shape data
DRE          declares legal stored states and transitions
SDA / ENR    provide the shared mathematical kernels
Atomics      enforce one admitted transition indivisibly
```

DRE is **not a clause of DQL**. A DQL query is caller-controlled,
ephemeral, may be incomplete, and may use resource-bounded retrieval. An active
rule is administrator-controlled, immutable, mandatory, and evaluated at the
commit gate over a proven complete dependency scope. Combining those authority
models would create a bypass.

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
| Verified artifact | Dependencies, scope, canonical IR, bounds, and verification record |
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

The surface is declarative, uses lower-case keywords in documentation, uses
the shared canonical paths, and borrows DQL's `using`, `matching`, and `expect`
vocabulary. ASCII keywords are case-insensitive. Identifiers remain
case-sensitive.

Visual compatibility does not make Data Rules part of DQL. DQL reads and
enriches artefacts. Data Rules constrain states and transitions.

### 5.1 Lexical rules and normative grammar

Source is UTF-8. Invalid UTF-8 is rejected. `--` begins a line comment.
Whitespace and comments otherwise have no meaning.

`identifier`, `path`, `literal`, and `predicate` are imported from
`dingo-predicate-v1`. Active rules accept no runtime parameters.

The following EBNF is normative:

```ebnf
ruleset          = "rules", "for", source-ref,
                   [ "named", identifier ],
                   [ "revision", unsigned ],
                   { declaration } ;

declaration      = require-rule
                 | optional-rule
                 | forbid-rule
                 | constrain-rule
                 | unique-rule
                 | freeze-rule
                 | allow-rule
                 | reference-rule ;

require-rule     = "require", path, [ "as", type ],
                   [ "when", predicate ] ;
optional-rule    = "optional", path, "as", type ;
forbid-rule      = "forbid", path, [ "when", predicate ] ;
constrain-rule   = "constrain", path, "where", predicate ;

unique-rule      = "unique", path, { ",", path },
                   [ "normalize", normalization ],
                   [ "compare", comparison-profile ] ;
normalization    = "none" | "unicode_nfc" ;
comparison-profile = "codepoint" | "binary" ;

freeze-rule      = "freeze", path, "after", "create" ;
allow-rule       = "allow", path, transition, { transition } ;
transition       = "from", literal, "to", literal ;

reference-rule   = "reference", identifier, "using", source-ref,
                   "matching", path, [ "each" ], "=", path,
                   "expect", reference-cardinality,
                   "on", "delete", "restrict" ;
reference-cardinality
                 = "exactly_one"
                 | "optional"
                 | "between", unsigned, "and", unsigned ;

type             = "any"
                 | "null"
                 | "boolean"
                 | "integer"
                 | "decimal"
                 | "string"
                 | "bytes"
                 | "key"
                 | "enum", "(", literal, { ",", literal }, ")"
                 | "nullable", "(", type, ")"
                 | [ "closed" ], "product", "{",
                     [ field-type, { [ "," ], field-type }, [ "," ] ],
                   "}"
                 | "sequence", "(", type,
                     [ ",", "min", unsigned ],
                     ",", "max", unsigned,
                   ")" ;
field-type       = [ "optional" ], identifier, ":", type ;
source-ref       = identifier | string ;
unsigned         = DIGIT, { DIGIT } ;
```

Declarations are self-delimiting because every declaration and continuation
clause begins with a reserved keyword. Newlines and indentation are stylistic.

If `named` is omitted, the administration request MUST supply an immutable
ruleset identifier separately. If `revision` is omitted, the administration
request MUST supply it separately. Neither value may be guessed from mutable
collection names in an activated artifact.

Quoted source references permit any valid collection name. Compilation binds
every source reference to an immutable collection identity in the artifact.

An empty ruleset is legal but has no enforcement effect.

Reserved words are all terminals in the grammar plus the predicate profile's
reserved words. A reserved word can occur as a bracketed path segment, never
as a bare identifier.

### 5.2 Type semantics

Types denote finite predicates:

| Type | Accepted present values |
|---|---|
| `any` | every stored value, including Null |
| `null` | Null only |
| `boolean` | Boolean |
| `integer` | mathematical integer |
| `decimal` | exact base-10 decimal; integers are not silently promoted for type checking |
| `string` | Unicode string |
| `bytes` | byte string |
| `key` | a value accepted by the frozen Heap key profile |
| `enum(...)` | one listed value by SDA equality |
| `nullable(T)` | Null or a value satisfying `T` |
| `product { ... }` | product/map satisfying every field declaration; extra fields allowed |
| `closed product { ... }` | same, with unlisted fields forbidden |
| `sequence(T, min n, max m)` | sequence of `T` with bounded length |

`max` is mandatory for sequences. `min` defaults to zero. `min > max` is a
static error.

Inside a product, an ordinary field is required and non-null unless its type
admits Null. `optional field: T` permits absence and validates `T` when present.

Top-level declarations mean:

- `require p` — `p` is present and, when a type is supplied, satisfies it;
- `optional p as T` — `p` may be absent and must satisfy `T` when present;
- `forbid p` — `p` must be absent;
- `when q` — the requirement or prohibition applies exactly when `q` is true.

Explicit Null does not satisfy an untyped `require`; use `require p as
nullable(any)` when Null is intentionally legal.

### 5.3 Document rules

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

### 5.4 Referential integrity

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

An optional reference permits path absence. Explicit Null is invalid. If Null
is a business state, it must be modeled separately from the reference path.

For a scalar reference, `each` is absent and only `exactly_one` or `optional`
is legal. For a bounded sequence reference, `each` is required, the child path
must be typed as `sequence(T, ... max m)`, and `between n and m` counts
successfully resolved distinct child elements. Duplicate sequence elements are
not collapsed and cause `reference_duplicate_element`.

V1 supports only `on delete restrict`. Cascade, set-null, and callbacks are
not accepted.

The parent path must be either:

- the collection's immutable `_key`; or
- covered by an active DRE unique declaration with the identical
  type, normalization, and comparison profile.

The compiler rejects any other parent path. Child and parent key types and
comparison profiles must agree. Reference equality never performs coercion.

Changing a referenced parent key is logically removal of the old parent key
plus insertion of the new key. `restrict` therefore rejects the change while
any live child refers to the old key, unless the same qualified Atomic also
changes or removes every such child and the final state satisfies the rule.

### 5.5 Uniqueness

Illustrative surface:

```text
rules for users

unique email
  normalize unicode_nfc
  compare codepoint
```

Every normalization and comparison operator must name frozen semantics.
Locale-dependent ambient behavior is forbidden.

Defaults are `normalize none` and `compare codepoint` for strings, `binary` for
bytes, and exact numeric comparison for numbers. `unicode_nfc` uses the Unicode
version recorded in the artifact.

If any unique path is absent, that document does not participate in the unique
domain. Present Null participates only when its declared type admits Null.
Composite uniqueness compares the ordered tuple of normalized path values.

### 5.6 Transition rules

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

Creation is legal for an `allow` rule when the after-value is one of the
relation's source or target values; deletion is not constrained by `allow`.
On update, both before and after paths must be present and their pair must
belong to the declared finite relation. Absence, Null not explicitly listed,
or a type mismatch is a violation.

`freeze p after create` requires exact SDA equality of the before and after
values on every update. It also preserves absence: adding or removing the path
after creation is forbidden.

### 5.7 Bounded cardinality

Illustrative surface:

```text
rules for teams

reference members using users
  matching member_ids each = id
  expect between 1 and 20
  on delete restrict
```

Unbounded traversal is not implied. The `each` keyword is the exact v1
collection-path syntax. The referenced sequence has a statically declared
maximum and the rule supplies the accepted resolved cardinality.

### 5.8 Composition

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

### 6.2 Document declaration semantics

Let `resolve(d, p)` have the exact meaning in `dingo-predicate-v1` and let
`Conforms(v, T)` be the recursive type predicate from §5.2.

```text
⟦require p⟧(d)
    ≜ resolve(d,p) = Present(v) ∧ v ≠ Null

⟦require p as T⟧(d)
    ≜ resolve(d,p) = Present(v) ∧ Conforms(v,T)

⟦optional p as T⟧(d)
    ≜ resolve(d,p) = Absent
       ∨ (resolve(d,p) = Present(v) ∧ Conforms(v,T))

⟦forbid p⟧(d)
    ≜ resolve(d,p) = Absent

⟦constrain p where q⟧(d)
    ≜ EvaluatePredicate(q,d)
```

The attribution path `p` in `constrain` MUST occur in `Dependencies(q)`. It
does not change the predicate's mathematical meaning.

For conditional declarations:

```text
⟦require p ... when q⟧(d)
    ≜ ¬EvaluatePredicate(q,d) ∨ ⟦require p ...⟧(d)

⟦forbid p when q⟧(d)
    ≜ ¬EvaluatePredicate(q,d) ∨ ⟦forbid p⟧(d)
```

The negation above is mathematical Boolean negation of the total predicate. It
must not be rewritten as a field `!=` comparison because `!=` deliberately
does not match absent operands.

### 6.3 Conditional presence

The declaration:

```text
require cup_size
  when sex = "F"
```

normalizes to:

```text
¬(sex(d) = "F") ∨ cup_size(d) ≠ ⊥
```

equivalently:

```text
sex(d) = "F" ⇒ defined(cup_size(d))
```

If `cup_size` is declared `as string`, explicit Null does not satisfy the
requirement.

### 6.4 Referential integrity

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

For `each` reference over bounded sequence `c.f = [v₁ ... vₙ]`, every element
must resolve to exactly one parent and the sequence length must lie in the
declared interval:

```text
n ∈ [minimum, maximum]
∧
∀ i ∈ 1..n • ∃! p ∈ Live(P) • p.k = vᵢ
∧
∀ i,j ∈ 1..n • i ≠ j ⇒ vᵢ ≠ vⱼ
```

### 6.5 Uniqueness

For a normalized key function `N`:

```text
∀ a, b ∈ Live(C) •
    defined(a.f)
    ∧ defined(b.f)
    ∧ N(a.f) = N(b.f)
    ⇒ a.id = b.id
```

The compiler includes `N` and its semantic version in the artifact identity.

### 6.6 Transition relations

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

### 6.7 Violation result

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

Violations are canonically ordered by:

```text
(rule_id bytes,
 rule_revision,
 collection_id bytes,
 document_id bytes,
 canonical_path bytes,
 stable_code bytes)
```

Execution and short-circuit order are never observable.

## 7. Compilation and proof artifact

Compilation is the expensive semantic step. Execution is shared and bounded.

```text
DRE source
        ↓ parse
canonical AST
        ↓ normalize
Invariant Core predicate
        ↓ analyze
dependencies + required scope + cost bound
        ↓ compile
canonical Invariant Core IR
        ↓ verify
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
    canonical_ir
    semantics_version
    compiler_profile
    artifact_hash
    verification_record
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

A v1 verifier independently parses and normalizes the retained source,
reconstructs the canonical IR and metadata, and accepts only byte-for-byte
identity with the artifact:

```text
VerifyV1(source, artifact) = true
    ⇔
RecompileV1(source) = artifact.canonical_components
```

`verification_record` stores the verifier profile, artifact hash, result, and
bounded diagnostics. It is not described as a formal proof certificate.

The canonical IR is the Invariant Core tree defined by §6; it is not an
underspecified virtual-machine instruction set. An implementation may compile
it into a private evaluator program for speed, but that program is derived
cache state, excluded from artifact identity, and disposable.

The semantic equivalence between the evaluator and canonical Invariant Core
remains a proof obligation discharged through specification, conformance,
property-based differential testing, model checking where applicable, and
progressive formal verification. A future profile may introduce a separately
checkable proof certificate under a new artifact version.

The logical artifact fields are normative. V1 does not assign their persistent
binary field numbers in this language document. No implementation may persist
or exchange a purported `dingo-dre-artifact-v1` until a companion encoding
profile fixes its canonical bytes. An implementation may begin with an
ephemeral in-process representation.

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

The v1 profile ceilings are:

| Quantity | Maximum |
|---|---:|
| decoded source | 262,144 bytes |
| declarations per ruleset | 1,024 |
| canonical AST nodes | 65,536 |
| predicate/type nesting | 32 |
| path segments | 32 |
| decoded path bytes | 4,096 |
| dependencies per ruleset | 4,096 |
| members in one enum | 4,096 |
| maximum declared sequence length | 4,096 |
| canonical artifact | 1,048,576 bytes |
| evaluator instructions per affected document | 1,000,000 |
| emitted violations per attempted Atomic | 1,024 |

A Heap policy may set lower ceilings and records them in the artifact. Raising
a ceiling beyond this table requires a new evaluator profile. Exceeding a
ceiling is a compile or pre-commit failure, never truncation.

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
4. emits canonical Invariant Core IR;
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

The activation frontier is encoded as:

```text
RuleActivationFrontier {
    heap_id
    ruleset_id
    revision
    collection_id
    partition_frontiers: ordered Map<PartitionId, AtomicPosition>
    validation_coverage_root
    activation_atomic_id
}
```

Every participating partition is named explicitly. Missing or indeterminate
coverage produces `coverage_incomplete`; it cannot activate a rule.

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

For a successful commit, durable Atomic evidence stores the exact active
ruleset revisions, artifact hashes, dependency-projection root, serialization
position, and aggregate `Valid` result. Per-rule success traces are derivable
and need not be stored.

For rejection, the bounded canonical violation set is returned and may be
audited according to Heap policy, but rejection does not create a committed
business event. Sensitive observed values are represented by domain-separated
hashes or safe type/length summaries.

## 15. Security and administration

Rule administration requires an explicit heap-bound administrative capability.

Ordinary write capabilities cannot:

- create or activate rules;
- replace artifact canonical IR or derived evaluator state;
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
- runtime evaluator programs are derived from canonical bounded IR;
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
evaluate deterministic derived program
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
artifact reconstruction verifier
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
model-checked protocols
bounded deterministic execution
```

“Proof-carrying artifact” is reserved for a future profile with a separately
checkable proof object. It is not a v1 claim.

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
- Unicode, decimals, overflow, and comparison use frozen semantics;
- damaged/unavailable material never becomes an ordinary document value.

### 18.3 Compiler and verifier

- randomized source/AST/IR/evaluator equivalence;
- malformed artifact and verification-record corpora;
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

- canonical Invariant Core IR and derived evaluator;
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

## 21. Closed v1 decisions

The exploratory questions are resolved as follows:

1. The language is **Dingo Rule Expression (DRE)**; the product capability is
   **Data Rules**; the dialect identifier is `dre`.
2. Paths and predicates use `dingo-predicate-v1`.
3. Scalar and composite types are exactly those in §5.1–§5.2. Numeric
   evaluation is integer/exact decimal; binary floating point is excluded.
4. Boolean syntax and semantics are those in the shared predicate profile.
5. V1 includes open/closed products and statically bounded sequences.
6. Violation ordering is fixed by §6.7.
7. V1 uses canonical Invariant Core IR plus a reconstruction verification record, not
   an unearned “proof certificate” claim.
8. The verifier recompiles retained source independently and checks canonical
   identity.
9. Resource ceilings are fixed by §7.3; Heap policy may only lower them.
10. Scalar references omit `each`; bounded-many references require `each`.
11. Optional references permit absence and reject explicit Null.
12. Activation uses the explicit per-partition frontier in §11.2 and is retained
    with the immutable rule revision for at least as long as any event governed
    by that revision.
13. Successful commits retain revision/artifact/projection/decision evidence;
    detailed success traces are derivable. Rejections return bounded canonical
    violations.
14. Invariant Core notation is normative documentation and an examination
    format in v1. It is not a remotely accepted rule-source dialect.
15. DQL `enrich` and DRE `reference` share the same present-key match
    bag and cardinality functions. DQL observes/interprets the bag for a read;
    DRE requires the corresponding proposition at every relevant
    serialization point.

No open semantic choice in this list is delegated to an implementer. A change
requires a new profile or a normative amendment.

## 22. Development readiness

DRE is ready to enter implementation planning when the companion
Atomic scope named by a rule is itself implemented and qualified.

Developers may implement document-local rules before cross-document scope
exists. They MUST reject, rather than emulate, reference or uniqueness rules
whose required serialization scope is unavailable.

The implementation contract is now:

1. parse only the §5.1 grammar;
2. import the exact shared predicate profile;
3. normalize to the §6 Invariant Core;
4. produce the immutable §7 artifact within fixed ceilings;
5. independently reconstruct and verify it;
6. activate only through §11;
7. enforce only through the Atomic commit gate;
8. preserve Heap noninterference, recovery evidence, and stable diagnostics;
9. pass §18 conformance before claiming the corresponding rule class.

## 23. Recommendation

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
