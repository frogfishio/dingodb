# RRE implementation plan

Status: developer-ready v1.0

Program release: P2, with cross-document continuation in P3

Normative source: [RRE_SPEC.md](../RRE_SPEC.md)

Atomic companion: [ATOMICS_SPEC.md](../ATOMICS_SPEC.md)

## 1. Decision

RRE implementation begins with document-local rules only.

The first shipped profile MUST compile finite declarative source into canonical
Invariant Core, independently verify the artifact, and enforce it on every
applicable committed put/replace.

Reference, uniqueness, and bounded-cardinality declarations MAY parse during
P2 but MUST fail activation with `atomic_scope_unavailable` until the required
LocalHeap Atomic packages pass.

## 2. Crate ownership

Create:

```text
crates/residiuum-dre/
  src/
    lib.rs
    token.rs
    parser.rs
    ast.rs
    path.rs
    types.rs
    normalize.rs
    invariant_core.rs
    artifact.rs
    verify.rs
    evaluate.rs
    violation.rs
    limits.rs
    encoding.rs
  tests/
    grammar.rs
    normalization.rs
    conformance.rs
    properties.rs
    hostile.rs
```

`dingo-dre` is a pure, deterministic kernel:

- no filesystem;
- no clock;
- no network;
- no callbacks;
- no host database access;
- no floating point;
- no random behavior;
- no dependence on `residiuum-store`.

Host integration lives in:

```text
residiuum-store    artifact/activation persistence and commit gate
residiuum-sdk      rule administration and violations
residiuum-server   Heap-bound protocol dispatch
residiuum-examine  SDA projection of rule evidence
residiuum-cli      compile/check/install/status/examine
```

## 3. Artifact boundary

Before `DRE-3`, `RRE_SPEC.md` MUST receive a canonical encoding amendment for:

```text
dingo-dre-source-v1
dingo-dre-artifact-v1
dingo-dre-ruleset-v1
dingo-dre-verification-v1
dingo-dre-decision-v1
```

The encoding MUST define:

- canonical CBOR field numbers;
- domain separators;
- semantic/compiler profile versions;
- HeapId, collection IDs, source hash, artifact hash;
- dependency set;
- required Atomic scope;
- maximum cost;
- normalized IR;
- verifier record;
- ordered violation encoding.

No persistent artifact may ship with implementation-defined serialization.

## 4. Work packages

### DRE-0 — Semantic oracle and corpus

Work:

- create crate and profile constants;
- import `dingo-predicate-v1` semantics without copying them;
- implement a deliberately slow reference evaluator;
- translate RRE_SPEC examples/counterexamples into fixtures;
- create JSON corpus schema for source, input, expected normalized form,
  violations, and required scope.

Required corpus axes:

- Absent versus Null;
- integer versus exact decimal;
- Unicode code-point comparison;
- open/closed products;
- bounded sequences;
- conditional presence;
- hostile depth/size limits;
- deterministic violation order.

Exit:

- corpus has at least one positive and negative case per grammar construct;
- repeated evaluation is byte-identical;
- no host I/O dependency exists.

Evidence: Unit, Property.

### DRE-1 — Parser and canonical AST

Work:

- lexer/token boundaries;
- parser for the exact §5.1 grammar;
- canonical paths;
- AST with source spans excluded from semantic identity;
- duplicate declaration detection;
- stable diagnostics;
- depth, token, identifier, and source-byte ceilings.

Rules:

- unknown syntax is rejected;
- no error recovery may produce an activatable partial ruleset;
- equivalent whitespace/comments do not change semantic identity;
- path resolution is against immutable collection identities at activation,
  not mutable names in the artifact.

Tests:

- grammar corpus;
- round-trip pretty-print only as a display aid;
- fuzz lexer/parser;
- Unicode and malformed UTF-8 boundaries;
- resource-limit failures.

Exit: every valid source has one AST; invalid source has no artifact.

Evidence: Unit, Property.

### DRE-2 — Normalization and Invariant Core

Work:

- normalize products, types, paths, predicates, and conditions;
- compute complete dependency set;
- compute required scope (`Key`, `LocalHeap`, `Partition`, or unavailable);
- derive maximum evaluation cost;
- emit canonical Invariant Core;
- implement reference evaluation and canonical violations.

Proof/property obligations:

```text
normalize(normalize(x)) = normalize(x)
equivalent source forms → equal canonical IR
equal canonical IR → equal evaluation for all bounded inputs
evaluation terminates within declared bound
violation order is total and stable
```

Use exhaustive small models and property generation.

Exit: corpus source and direct Invariant Core fixtures agree.

Evidence: Property, Differential, Model where useful.

### DRE-3 — Artifact encoding and independent verifier

Work:

- freeze canonical encodings;
- compile source to artifact;
- implement verifier through an independent reconstruction path;
- reject altered source/artifact/profile/dependency/bound;
- retain verification record;
- provide `dingo dre compile` and `dingo dre verify`.

The verifier MUST NOT trust cached evaluator state. It recompiles retained
source and compares canonical components.

Exit:

- bit-flip corpus fails closed;
- compiler and verifier share semantic definitions but not one unchecked
  serialized object;
- artifact identity is stable across supported platforms.

Evidence: Differential, Damage.

### DRE-4 — Document-local activation and enforcement

Work:

- Heap-bound immutable ruleset revisions;
- collection binding by immutable ID;
- activation frontier;
- prospective barrier during validation;
- enforcement in embedded Heap put/replace/delete path;
- remote parity;
- canonical rejection response;
- success decision evidence;
- history and backup integration;
- SDA examination.

Activation states:

```text
draft
validating
active
degraded
retired
```

P2 activation for document-local rules requires:

- artifact verified;
- all referenced collections exist;
- current live collection coverage complete;
- every existing live document validates, or activation returns violations;
- writes after barrier validate under the prospective revision;
- replay reaches the captured frontier;
- publication atomically selects one active ruleset revision.

Required tests:

- activate empty/non-empty collection;
- concurrent writes during activation;
- invalid existing records;
- rule replacement;
- crash at each activation phase;
- missing/corrupt artifact;
- backup/restore;
- same rules in two Heaps remain distinct;
- foreign artifact rejected;
- ordinary write has no bypass flag;
- remote/embedded violation parity.

Exit:

> Every reachable committed document after activation satisfies the active
> document-local ruleset, or the Heap is explicitly degraded/unavailable rather
> than silently bypassing enforcement.

Evidence: Isolation, Crash, Damage, Journey.

### DRE-5 — Transition rules

Entry: `ATM-1` Key Atomic accepted.

Work:

- before/after values;
- immutable fields;
- finite allowed transitions;
- delete rules;
- history/retry integration.

Exit: transition evaluation occurs at the Key Atomic serialization point and
cannot be bypassed by remote, import, or recovery paths.

### DRE-6 — Cross-document rules

Entry: `REL-0` through `REL-4` accepted.

The enforcement machinery is delivered by `REL-*` in the Atomic
implementation plan. This package closes the RRE language integration:

- compile reference, uniqueness, and bounded-cardinality declarations to the
  exact qualified relationship artifact;
- report `required_atomic_scope = LocalHeap`;
- reject unsupported cascade, cross-Heap, and Partition declarations;
- prove activation revision, Atomic plan, reverse index, and violation
  evidence refer to the same ruleset and Heap;
- run source-to-artifact-to-enforcement conformance journeys.

Exit: each advertised cross-document RRE has one canonical relationship
meaning, is enforced at the LocalHeap Atomic serialization point, and cannot
be bypassed by any mutation surface.

Evidence: Differential, Isolation, Crash, Damage, Journey.

## 5. Public API

Target Rust administration:

```rust
let source = RreSource::parse(include_str!("customer.dre"))?;
let compiled = heap.rules().compile(source)?;
let report = heap.rules().validate(&compiled)?;
let revision = heap.rules().activate(compiled, report)?;
```

Inspection:

```rust
let active = heap.rules().active()?;
let violations = heap.rules().violations(revision.id())?;
```

Ordinary writes do not receive a `skip_rules` option.

CLI:

```text
dingo dre check FILE
dingo dre compile FILE --out ARTIFACT
dingo heap rules validate STORE --heap H --key KEY FILE
dingo heap rules activate STORE --heap H --key KEY FILE
dingo heap rules status STORE --heap H --key KEY
dingo heap rules violations STORE --heap H --key KEY
dingo heap rules retire STORE --heap H --key KEY REVISION
```

Rule administration requires a deliberately added `RuleAdmin` right before
remote activation ships. Until the rights-registry revision is frozen, remote
rule administration MUST remain unavailable; local development MAY use
`HeapAdmin` only behind an explicitly provisional profile.

## 6. Performance gates

Measure separately:

- parse;
- normalize;
- verify;
- activation validation;
- per-write evaluation;
- violation construction;
- artifact cache hit/miss.

P2 MUST publish:

- per-rule-class cost curves;
- maximum rule-set ceiling;
- p50/p95/p99 write overhead for valid and rejected documents;
- no-rule control;
- cache cold/warm;
- payload sizes and durability mode.

Performance optimization may change representation, not semantics, dependency
scope, violation order, or evidence.

## 7. Explicit non-goals for P2

- referential integrity;
- uniqueness across documents;
- cascade;
- cross-Heap rules;
- arbitrary scripts/functions;
- external lookups;
- cluster enforcement;
- direct user-supplied Invariant Core activation;
- proof-certificate marketing;
- rule bypass on ordinary writes.
