# ENR Doctrine

## Thesis

SDA is reduction.
ENR is expansion.

SDA reduces noisy structured inputs into semantically sharper artefacts.
ENR expands one reduced artefact against another artefact or dataset so that relation, multiplicity, and policy become explicit.

If SDA is the certainty kernel for transformation, ENR should be the certainty kernel for relation.

## Why ENR Exists

After reduction, a system still often lacks enough context to produce the needed result.

It must:

- look up related rows
- preserve duplicates honestly
- decide whether uniqueness is required or optional
- decide whether grouped or expanded results are needed
- combine the outcome back into the left artefact explicitly

That is ENR's job.

ENR is not "just joins."
ENR is explicit relation plus explicit interpretation of multiplicity.

## Non-Negotiable Principle

Multiplicity must remain visible until a program explicitly interprets it.

This is the core law.

If ENR hides multiplicity, it becomes ordinary host-language lookup logic again.

## Core Boundary

The ENR kernel should stay small.

Its primitive semantic object is the match bag:

```text
{ r ∈ R | kR(r) = kL(l) }
```

Everything in the kernel is an interpretation of that object.

The kernel therefore includes:

- match-bag formation
- explicit cardinality operators
- explicit row-combination operators
- explicit carrier rules
- stable failures
- algebraic laws

## Extension Boundary

Candidate bags, ranking, provenance, explanation, and quality are important, but they should begin as the first serious extension layer over the kernel rather than replace it.

This preserves two truths at once:

- ENR1 correctly identifies the kernel
- ENR2 correctly identifies the growth path

So the right answer is not ENR1 or ENR2 in isolation.
It is kernel first, extension second.

## What ENR Must Never Do

ENR must not:

- perform acquisition
- perform orchestration
- hide uniqueness assumptions in host code
- silently collapse duplicates
- silently drop left rows
- treat `Null` as no match
- let provenance or explanation replace semantic law

Those are the ways ENR would lose its reason to exist.

## Relationship To SDA

The family rhythm is:

- reduce with SDA
- expand with ENR
- reduce again with SDA

So ENR is not a competing transform language.
It is the pure relational expansion layer between reduction passes.

## Relationship To Axiom

Axiom acquires and routes artefacts.
ENR does not.

ENR is pure.
Axiom owns effects.

If a system runs inside a host such as AWS Lambda, then:

- the host supplies noisy event artefacts
- SDA reduces that noise into usable artefacts
- ENR expands one artefact against another
- SDA reduces again
- Axiom orchestrates the stages and effects around them

## Standard For A Sound ENR Core

ENR is in a sound core state only when:

1. the primitive match result is explicit and multiplicity-preserving
2. every collapse of multiplicity is explicit
3. grouped and expanded outcomes remain distinct
4. source semantics are explicit and non-transport-level
5. failures are stable and policy-visible
6. candidate/provenance features, if added, preserve the kernel rather than replace it

In short:

ENR should be to relation what SDA is to transformation.