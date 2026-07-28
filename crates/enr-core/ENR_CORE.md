# ENR Core Draft

## Thesis

SDA is reduction.
ENR is expansion.

More precisely:

- SDA reduces noisy structured inputs into semantically sharper artefacts.
- ENR expands one artefact against another artefact or dataset, making relational possibility explicit.
- SDA can then reduce the enriched result into the next stable artefact.

ENR exists to relate reduced artefacts without hiding multiplicity, uniqueness assumptions, or source semantics.

If SDA is the certainty kernel for transformation, ENR should be the certainty kernel for relation.

## Position In The Family

The family rhythm is:

- reduce with SDA
- expand with ENR
- reduce again with SDA
- repeat until the target artefact is reached

Conceptually:

```text
raw input
  -> SDA
  -> artefact A

raw input 2
  -> SDA
  -> artefact B

(A, B)
  -> ENR
  -> relational artefact E

E
  -> SDA
  -> artefact C
```

So ENR is not a replacement for SDA.
It is the pure relational expansion layer between reduction passes.

## Core Boundary

ENR should be layered explicitly.

### 1. Match Algebra Core

This is the irreducible center.

It defines:

- formation of a match bag from a left value and a right dataset
- duplicate preservation
- explicit cardinality interpretation
- explicit row-combination operators
- carrier-preservation rules for grouped versus expanded enrichment
- stable enrichment failure conditions

### 2. Candidate And Resolution Extension Layer

This layer is valuable, but should not replace the core.

It may define:

- candidate bags
- ranking
- preference
- evidence and annotations
- explicit resolution objects
- ambiguity as a first-class result
- explanation and quality reporting

This is the first serious extension layer on top of the core, not the primitive center itself.

### 3. Source And Host Profile

This layer defines:

- source declarations
- uniqueness claims
- ordering claims
- source identity and provenance hooks
- host integration rules

It must not define acquisition mechanics.

### 4. Outside ENR Proper

The following do not belong to ENR meaning:

- HTTP
- file IO
- retries
- auth
- caching
- orchestration

Those belong to Axiom.

## Synthesis Of ENR1 And ENR2

The notes in `ENR1.md` and `ENR2.md` are not mutually exclusive.
They describe different semantic levels.

### What ENR1 gets right

`ENR1.md` identifies the kernel correctly:

- the primitive semantic object is the match bag
- multiplicity must be preserved
- cardinality interpretation must be explicit
- attach, merge, and expand are separate operations
- source declarations are semantic, not transport-level

This should be the base layer.

### What ENR2 gets right

`ENR2.md` identifies the serious next layer:

- candidate objects
- refinement before resolution
- explanation and provenance
- quality and operational reporting
- richer resolution outcomes than simple `one?` / `one!`

This should be preserved, but treated as an extension layer over the kernel rather than the kernel itself.

### The synthesis

The best current direction is:

- ENR core = explicit match-bag algebra
- ENR+ = candidate / resolution / explanation layer

In short:

- ENR1 supplies the semantic center
- ENR2 supplies the first serious growth path

## Primitive Core Object

For a current left value `l`, right dataset `R`, left key function `kL`, and right key function `kR`, define:

```text
Match(l, R, kL, kR) = { r ∈ R | kR(r) = kL(l) }
```

Core requirements:

1. the primitive result is always a bag of matches
2. duplicates from `R` are preserved
3. equality in the predicate uses SDA equality
4. `Null` is a value, not no match
5. no match means the bag is empty

This is the relational analogue of SDA's refusal to hide carrier meaning.

## Core Surface Form

The canonical surface form should preserve the visible relation:

```text
{ r ∈ R | kR(r) = kL(l) }
```

Example:

```text
{ c ∈ customers | c.id = l.customer_id }
```

This should remain the normative center even if shorter sugar is added later.

## Core Interpretation Operators

The first required operators are not dozens of helpers.
They are the minimal interpretations of a multiplicity-bearing match result.

### Optional exact match

```text
one?(B)
```

Meaning:

- `0 -> None`
- `1 -> Some(v)`
- `>1 -> Fail(t_enr_duplicate, "duplicate match")`

### Required exact match

```text
one!(B)
```

Meaning:

- `0 -> Fail(t_enr_missing, "missing match")`
- `1 -> v`
- `>1 -> Fail(t_enr_duplicate, "duplicate match")`

### Ordered choice

```text
first(B)
last(B)
```

Meaning:

- choose by source-defined order only
- fail with `t_enr_unordered_policy` if order is not defined

### Exact uniqueness spelling

```text
only(B)
```

This is the stricter exact-uniqueness form.

Rule:

cardinality is never implicit.
A program must say how a match bag is interpreted.

## Core Combination Operators

After interpretation, ENR must say how matches enter the result.

### Attach

```text
l + { field: E }
```

Attach an interpreted enrichment result to the current left row.

### Merge

```text
merge(l, r)
```

Merge two product-like values under explicit or default collision policy.

### Expand

Expansion is comprehension over the match bag, not a hidden join mode.

Canonical form:

```text
{ yield merge(l, r) | l ∈ L, r ∈ { s ∈ R | kR(s) = kL(l) } }
```

Rule:

one-to-many behavior must remain visible as comprehension over explicit multiplicity.

## Core Carrier Rules

### Match carrier

The primitive match result is always `Bag`.

Why:

- duplicates must survive
- uniqueness must not be assumed
- policy must be explicit

### Left-carrier preservation

For attachment-style enrichment:

- `Seq -> Seq`
- `Bag -> Bag`
- `Set -> Set`

For expansion-style enrichment:

- `Seq -> Seq`
- `Bag -> Bag`
- `Set -> Bag`

This should be normative in the core.

## Core Failure Set

Minimum stable failures:

- `t_enr_missing`
- `t_enr_duplicate`
- `t_enr_unordered_policy`
- `t_enr_field_collision`
- `t_enr_wrong_shape`
- `t_enr_invalid_key`

These are the ENR equivalents of SDA's stable semantic failures.

## Core Laws

The following should be non-negotiable.

### 1. Primitive reduction law

All core enrichment forms reduce to:

```text
{ r ∈ R | kR(r) = kL(l) }
```

plus explicit interpretation of the resulting bag.

### 2. No implicit uniqueness

A match bag is never implicitly collapsed.

### 3. No implicit dropping

A left row is not dropped unless the enclosing expression explicitly omits it.

### 4. Null is not no match

If a matched row contains `Null`, that row still counts as a match.

### 5. Duplicate preservation

Duplicates in the right dataset remain duplicates in the match bag until an explicit operator resolves or rejects them.

## What Belongs To ENR+

The following are important, but should sit on top of the kernel rather than define it.

### Candidate bags

`ENR2.md` introduces `CandidateBag` as the primitive value.

That is likely the right next step, but not the right first kernel.

Recommendation:

- define `CandidateBag` as the first major extension over the match-bag core
- preserve the law that candidate formation must still reduce to explicit match formation

### Refinement operators

These belong to ENR+:

- `whereCandidate`
- `annotateCandidate`
- `rankCandidates`
- `preferCandidates`
- `dedupeCandidates`

These are meaningful, but they presuppose a richer candidate object than the core requires.

### Rich resolution values

These also belong to ENR+:

- `Resolved`
- `Decision`
- `Provenance`
- `Quality`
- `explainResolved`
- `qualityOf`

These make ENR operationally serious, but they should not obscure the simpler core law that multiplicity must be explicit and interpreted, never hidden.

## Source Declarations

Source declarations should remain semantic, not transport-level.

Minimum source kinds:

- `Index[K,V]`
- `MultiIndex[K,V]`
- `Dataset[V]`

They declare:

- uniqueness expectations
- duplicate tolerance
- whether order is meaningful
- semantic source identity

They do **not** declare:

- HTTP
- auth
- retries
- caching
- file locations

Those belong to Axiom.

## Relationship To Axiom

ENR must stay pure.

ENR does not:

- fetch the right dataset
- decide retry policy
- perform HTTP or file calls
- orchestrate stage sequencing

Axiom does that.

So the family relationship is:

- Axiom acquires or routes artefacts
- SDA reduces raw or enriched artefacts
- ENR expands one reduced artefact against another
- SDA reduces again

## Minimal Useful ENR Core

If the goal is the smallest serious first version, it is just this:

- match bag comprehension
- `one?`
- `one!`
- `first`
- `last`
- `only`
- attach via `l + { field: E }`
- `merge(l, r)`
- dataset comprehensions using `yield`
- source declarations with semantic uniqueness kinds

That is enough to express:

- optional lookup
- required lookup
- grouped one-to-many
- expanded one-to-many

without prematurely turning ENR into a large decision-analysis language.

## Recommended Next Step

The right path from here is:

1. freeze this ENR kernel
2. define ENR+ candidate and resolution objects on top of it
3. keep provenance, explanation, and quality as explicit extension surfaces
4. let Axiom orchestrate acquisition and stage flow around the pure ENR layer

In short:

ENR should be to relation what SDA is to transformation.

That means keeping the kernel small enough to be correct, while preserving a serious growth path for candidate-aware enrichment later.