# ENR+ Extension Draft

## Thesis

`ENR` core is match-bag algebra.
`ENR+` is the first serious extension layer over that core.

Its purpose is to make enrichment operationally serious without losing the kernel.

That means:

- keep the primitive relational center explicit
- add candidate-aware refinement and resolution on top of it
- preserve provenance and explanation as first-class values
- never hide multiplicity or invent implicit uniqueness

If `ENR` core answers:

```text
what matches?
```

then `ENR+` answers:

```text
which candidates remain?
why were they preferred?
how was the outcome resolved?
how can the outcome be explained?
```

## Relationship To ENR Core

`ENR+` is not a replacement for the core defined in `04-enrichment.md`.

The dependency is one-way:

- `ENR` core defines the match-bag kernel
- `ENR+` lifts that kernel into candidate-aware refinement and richer resolved outcomes

Rule:

Every `ENR+` operator must preserve the core laws:

- no implicit uniqueness
- no implicit dropping
- null is not no match
- duplicate preservation until explicit interpretation

## Primitive Extension Object

The primitive extension object is the candidate bag.

Conceptually:

```text
CandidateBag[L,R] = Bag[Candidate[L,R]]
```

Where a `Candidate` minimally contains:

- `left`
- `right`
- `source`
- `provenance`

And may additionally contain:

- `evidence`
- `score`
- `class`
- `rank`
- `annotations`

Rule:

Candidate metadata may refine and explain resolution, but it must not change candidate identity unless an explicit equivalence or deduplication operator says so.

## Candidate Formation

The core match-bag form remains the semantic center, but `ENR+` may expose a candidate-aware surface.

Canonical candidate form:

```text
{ cand r ∈ R | P(l, r) }
```

Conceptual meaning:

```text
candidates(l, R, pred) -> CandidateBag[L,R]
```

This is extension sugar over the same relational basis as the core match bag.

### Formation requirements

`candidates(...)` or `{ cand ... }` MUST:

- preserve duplicates from `R`
- produce one candidate per surviving right row
- attach source identity
- attach initial provenance
- optionally attach initial evidence

## Candidate Refinement Operators

These operators keep the same broad type:

```text
CandidateBag -> CandidateBag
```

They do not resolve candidates.
They only shape the candidate space.

### 1. `whereCandidate`

```text
whereCandidate(C, p)
```

Filter candidates by predicate.

Use cases:

- keep only active rows
- keep only exact-class candidates
- keep only candidates above threshold

Rule:

Filtering removes candidates but does not choose among remaining ones.

### 2. `annotateCandidate`

```text
annotateCandidate(C, f)
```

Add evidence, class, score, rank, or policy annotations.

Rule:

Annotation MUST NOT change left/right identity of a candidate.

### 3. `rankCandidates`

```text
rankCandidates(C, keyFn)
```

Compute an ordering signal.

Rule:

Ranking prepares for resolution but does not itself resolve.

### 4. `preferCandidates`

```text
preferCandidates(C, ord)
```

Keep only maximally preferred candidates under an ordering or policy.

Rule:

Preference is refinement, not final resolution. If multiple equally preferred candidates remain, ambiguity remains.

### 5. `dedupeCandidates`

```text
dedupeCandidates(C, eq)
```

Collapse equivalent candidates under an explicit equivalence.

Rule:

Deduplication MUST state the equivalence basis. It is not general silent duplicate dropping.

## Resolved Outcomes

`ENR+` promotes richer resolution values than the core `one?` / `one!` forms.

Minimum outcome variants:

- `NoMatch`
- `Unique(v)`
- `Chosen(v)`
- `Ambiguous(Bag[v])`
- `Rejected(code, msg)`

Rule:

Ambiguity is first-class. It MUST NOT be silently collapsed.

## Resolution Operators

These consume a `CandidateBag` and produce a `Resolved` value.

### 1. `resolveOptional`

```text
resolveOptional(C) -> Resolved[R]
```

Meaning:

- `0 -> NoMatch`
- `1 -> Unique`
- `>1 -> Ambiguous` or policy-visible rejection, depending on the chosen mode

Default recommendation:

- `0 -> NoMatch`
- `1 -> Unique`
- `>1 -> Ambiguous`

### 2. `resolveRequired`

```text
resolveRequired(C) -> Resolved[R]
```

Meaning:

- `0 -> Rejected(t_enr_missing, ...)`
- `1 -> Unique`
- `>1 -> Rejected(t_enr_duplicate, ...)` unless pre-refined to one

### 3. `resolveFirst`

```text
resolveFirst(C) -> Resolved[R]
```

Meaning:

- choose first candidate under defined order
- record rejected alternatives in the decision object

If order is undefined, fail with `t_enr_unordered_policy`.

### 4. `resolveBest`

```text
resolveBest(C, ord) -> Resolved[R]
```

Meaning:

- choose the best candidate under explicit ordering
- if a tie remains, return `Ambiguous(...)` or a policy-visible rejection

Rule:

`resolveBest` MUST NOT choose arbitrarily when the maximum is not unique.

### 5. `resolveGroup`

```text
resolveGroup(C) -> Resolved[Bag[R]]
```

Meaning:

- preserve all surviving candidates as a grouped value
- still emit decision and provenance

### 6. `resolveReduce`

```text
resolveReduce(C, agg) -> Resolved[T]
```

Meaning:

- aggregate multiple candidates into one summary result

Use cases:

- sum balances
- choose canonical merged record
- summarize evidence

Rule:

Reduction summarizes multiplicity; it does not pretend multiplicity was absent.

## Resolved Eliminators

If rich resolved values exist, eliminators are required.

### `value(res)`

Extract the chosen value.

Strict behavior:

- `Unique(v) -> v`
- `Chosen(v) -> v`
- `NoMatch -> Fail(t_enr_missing, ...)`
- `Ambiguous(...) -> Fail(t_enr_duplicate, ...)`
- `Rejected(code, msg) -> Fail(code, msg)`

### `decision(res)`

Return the decision object of the resolved outcome.

### `provenance(res)`

Return the provenance object of the resolved outcome.

## Combination Operators Over Rich Results

These combine a left row with resolved or candidate-aware enrichment results.

### 1. `attachResolved`

```text
attachResolved(l, name, res) -> Prod
```

Attach a resolved result under a field.

### 2. `mergeResolved`

```text
mergeResolved(l, res, policy) -> Prod | Fail
```

Merge a resolved value into the left row under explicit collision policy.

### 3. `expandResolved`

```text
expandResolved(l, C, f) -> Bag[T]
```

Emit one output row per surviving candidate.

### 4. `attachCandidates`

```text
attachCandidates(l, name, C) -> Prod
```

Attach the entire candidate bag when the correct action is to preserve ambiguity rather than resolve immediately.

## Explanation And Quality

These make ENR operationally serious without redefining its kernel.

### 1. `explainResolved`

```text
explainResolved(res) -> Prod
```

Return structured explanation such as:

- chosen candidate
- rejected candidates
- policy used
- evidence summary
- provenance

### 2. `qualityOf`

```text
qualityOf(Bag[Resolved[T]]) -> Quality
```

Produce summary stats such as:

- total
- unmatched
- unique
- chosen
- ambiguous
- failed

### 3. `assertQuality`

```text
assertQuality(Q, predicate) -> Res[Quality]
```

This supports operational thresholds such as:

- unmatched must be zero
- ambiguity rate must stay below threshold

## Surface Guidance

The surface should remain SDA-like, not command-like.

That means:

- candidate comprehension
- ordinary function application
- explicit operator wrapping
- explicit construction

Recommended reading:

- `ENR` core remains the normative center
- `ENR+` may add candidate comprehensions such as `{ cand c ∈ customers | ... }`
- richer syntax should preserve visible relation formation rather than replace it with opaque commands

## Laws

### 1. Formation before resolution

All resolution operators consume candidate bags, not raw datasets.

### 2. Refinement preserves candidate identity

Filtering, ranking, and annotation do not create fake left/right identities.

### 3. Resolution is explicit

No candidate bag is implicitly collapsed.

### 4. Explanation survives resolution

A resolved result must retain enough information to explain the decision.

### 5. Combination does not rewrite prior decisions

Attach, merge, and expand consume a resolved or candidate form; they do not retroactively alter the earlier matching decision.

## Minimal Serious ENR+

If the goal is the smallest useful extension over the kernel, keep:

formation:

- `candidates`
- `unionCandidates`

refinement:

- `whereCandidate`
- `annotateCandidate`
- `preferCandidates`

resolution:

- `resolveOptional`
- `resolveRequired`
- `resolveBest`
- `resolveGroup`

combination:

- `attachResolved`
- `mergeResolved`
- `expandResolved`
- `attachCandidates`

explanation:

- `explainResolved`
- `qualityOf`

That is enough to make enrichment operationally serious without collapsing the kernel into a large everything-language.