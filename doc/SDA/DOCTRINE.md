# SDA Doctrine

## Thesis

SDA exists only if it gives stronger guarantees than ad hoc host-language code.

If users can get the same level of certainty by writing Java, Rust, Go, or JavaScript with tests, then SDA has no serious reason to exist.

The point of SDA is not convenience.
The point of SDA is certainty.

SDA is a mathematically defined kernel for extracting dependable meaning from unreliable structured data.

That means:

- expressions must have stable meaning
- carrier behavior must be explicit
- failure boundaries must be explicit
- determinism must not depend on host accident
- ambiguity must never be silently reinterpreted as truth

In short:

SDA exists so that structured-data transformation is not guesswork disguised as code.

## Why This Matters

Real-world data is mostly garbage in the broad sense:

- partial
- duplicated
- stale
- contradictory
- lossy
- reordered
- schema-drifting
- operationally inaccessible except through logs or side channels

The job is not merely to "extract data."
The job is to recover semantically dependable meaning from messy observations.

Example:

```javascript
a = b.c.d.e;
```

In an ordinary host language, what is `a`?

The answer is often host-dependent and operationally contingent:

- the value at the path
- `undefined`
- an exception
- a proxy result
- a getter side effect
- an environment-specific default

That is not a mathematical answer.

In SDA, the analogous question must have one precise meaning:

- lawful projection returns a value
- missing data yields the defined missing condition
- wrong shape yields the defined wrong-shape condition
- multiplicity remains multiplicity unless explicitly normalized

If SDA cannot make that distinction precisely, it is not finished.

## Boundary Model

SDA should be understood in four layers.

### 1. Semantic Core

This is the algebraic kernel.

It defines:

- value kinds
- equality
- carriers
- eliminators
- normalization
- algebraic operators
- comprehensions
- composition semantics
- lawful failure conditions
- determinism requirements

This layer must be small, closed, pure, and defensible.

### 2. Embedding And Transformation Model

This defines how the semantic core is used as a transformation engine inside another system.

It defines:

- what a program denotes as a transformation
- how inputs are supplied
- how results are consumed
- how stages compose
- whether placeholder or pipe semantics exist
- what contracts hold when one SDA stage feeds another stage

This layer is not mere UX.
It is the operational model of the pure algebra.

### 3. Standalone Profile

This is a concrete executable profile over the core.

It may define:

- concrete grammar
- ASCII and Unicode spellings
- JSON bridge
- canonical wrapper values
- helper functions such as `typeOf`, `keys`, `values`, and `count`
- CLI behavior
- diagnostics and formatting

This layer must not redefine the semantic core.

### 4. Orchestration And Effects

This is outside SDA proper.

It includes:

- HTTP and file effects
- external source access
- retries and timeouts
- enrichment from multiple systems
- output shaping
- workflow control

This is where Axiom and later layers belong.

## Non-Negotiable Requirements

The following are the minimum conditions for SDA to be worth having.

1. Every core expression must have one stable meaning.
2. No core operation may silently coerce one carrier into another.
3. Null and absence must remain distinct.
4. Uniqueness must never be inferred from multiplicity.
5. Failure conditions must be explicit, classified, and stable.
6. Determinism must not depend on parse order, host map ordering, or incidental runtime behavior.
7. Equality must be defined exactly for every comparable core value kind.
8. Any surface sugar must desugar to a precise core meaning.
9. Extensions may add convenience, but may not weaken or reinterpret the core algebra.
10. Conformance must be about semantic law, not only example-based behavior.

## What SDA Is Not

SDA is not:

- a bag of convenient JSON tricks
- a replacement for host programming in general
- an effectful workflow engine
- a place to hide ambiguity behind defaults
- a thin prettier wrapper around informal data poking

If SDA becomes permissive in the name of convenience, it collapses back into ordinary host-language data munging.

## The Current Certainty Leaks

The current repository is already strong in several areas, but certainty still leaks at important boundaries.

### A. Spec Examples That Are Not Actually Supported

The specification currently contains examples that the implementation does not accept as written.

#### Pipe examples imply implicit argument insertion

The spec uses forms such as:

```text
headers
|> normalizeUnique()
```

See:

- `SDA/SDA_SPEC.md` §10 example usage
- `SDA/SDA_SPEC.md` §13.6 pipeline examples

The current implementation does not treat `|>` as implicit function application.
It binds the left-hand side only to the placeholder `_` or `•` on the right-hand side.

Today, this fails with arity mismatch rather than behaving like a stage application.

Implication:

- either implicit application belongs to the embedding model and must be specified rigorously
- or those examples must be rewritten to use explicit placeholder form

#### Comprehension examples imply `yield k -> v` as a general expression form

The spec allows examples such as:

```text
{ yield "x" -> 1 | a ∈ Seq[1,2,3] }
```

The current parser accepts `->` only in map and bagkv literal entry positions, not as a general expression that yields `Bind(k, v)`.

Implication:

- either `->` becomes a first-class expression former with precise desugaring to `Bind`
- or the spec must require `yield Bind("x", 1)` instead

### B. The Failure Boundary Is Still Split

The current system has two different failure domains:

- stable SDA `Fail(code, msg)` results for some semantic conditions
- host/runtime evaluation errors for others

Examples currently represented as host-side evaluation errors include:

- arity mismatch
- not callable
- division by zero
- unbound variable
- type mismatch

This is not automatically wrong, but it is not yet sharply specified.

The spec must decide:

- which conditions belong to the core SDA failure algebra
- which conditions are static rejection
- which conditions are host/runtime errors outside the core algebra

Until this boundary is fixed, conformance remains partially ambiguous.

### C. Some Implemented Operations Are Broader Than The Written Spec

The evaluator currently supports membership on more carriers than the written spec explicitly defines.

The written spec defines membership clearly for `Set` and `Bag`.
The implementation also supports `Seq`, `Map`, and `Prod` membership forms.

Implication:

- either this broader behavior is part of the algebra and must be specified completely
- or it is a profile extension and must be documented as such
- or it should be removed from the standalone core surface

An unstated widening of lawful domains is a certainty leak.

### D. The Conformance Suite Is Smaller Than The Real Semantic Surface

The repository contains meaningful conformance tests, but much actual behavior is still proved only by implementation-local tests.

That means part of the language contract lives in the implementation rather than in spec-indexed conformance.

The missing high-value areas are:

- comprehensions as a complete section-level conformance module
- pipe composition semantics beyond the simplest placeholder case
- null vs absence at the conformance level
- broader algebraic law tests
- executable replay of worked examples from the spec

### E. Prod Is Operationally Distinct But Still Conceptually Underdefined

The implementation already distinguishes `Prod` from `Map` operationally, especially for total projection.

That is good.

But the phrase "known shape" still needs sharper meaning in the standalone and embedding story.

The spec should state whether `Prod` in standalone SDA is:

- a runtime carrier with distinct lawful eliminators
- a host-declared schema-aware value kind
- or a future stronger static concept not yet enforced by the standalone profile

## What Must Be Decided Next

The following decisions should be made before adding more language surface.

### 1. Pipe Semantics

Decide whether `|>` means:

- placeholder-only substitution on the RHS
- or a richer stage-application form in the embedding model

If the latter, the desugaring must be exact.

### 2. Binding Syntax

Decide whether `k -> v` is:

- only literal-entry syntax
- or a general expression form denoting `Bind(k, v)`

The current spec examples and parser are not aligned.

### 3. Failure Taxonomy

Fix the classification of:

- static invalidity
- dynamic SDA failure values
- host/runtime exceptions outside the SDA algebra

### 4. Lawful Domains

For each operation, specify exactly which carriers are lawful.

No implicit widening.
No convenience by accident.

### 5. Conformance Scope

Promote the real contract into a spec-indexed conformance suite rather than relying primarily on implementation-local tests.

## Immediate Roadmap

1. Freeze the semantic-core boundary.
2. Separate embedding-model semantics from standalone-profile conveniences.
3. Repair the spec so every normative example is executable.
4. Expand conformance to prove laws, not only examples.
5. Align the implementation to the frozen contract.
6. Only then expand CLI or outer-language ergonomics.

## Standard For Completion

SDA is complete enough for its purpose only when:

- every lawful operation has one defined meaning
- every unlawful operation has one defined boundary
- every example in the normative spec is executable or explicitly informative only
- profile sugar desugars exactly
- conformance proves algebraic law and determinism
- host extensions cannot silently change core meaning

If those conditions are not met, SDA may still be useful, but it is not yet the certainty kernel it is intended to be.