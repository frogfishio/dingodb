# Formal Assurance Spine implementation plan

Status: **TODO — FAS-0 blocked by CSQ-12**

Program: `FAS`

Normative semantics:
[FORMAL_ASSURANCE_SPEC.md](FORMAL_ASSURANCE_SPEC.md).

## 1. Delivery law

The foundation begins immediately after `CSQ-12`. Consistency and security then
proceed alongside M1/PQH. Atomics and cluster theorem families are implemented
with—not after—the corresponding features.

No package may:

- claim implementation connection from duplicate example code;
- weaken or hide an assumption to make a proof pass;
- hide an unwinding/model bound;
- translate tests into “proofs”;
- mark a theorem proved because its file exists;
- introduce a second ungoverned theorem registry; or
- block a safety result on an optional liveness assumption.

## 2. Repository shape

```text
formal/
  registry/
    theorems-v1.json
    assumptions-v1.json
    tcb-v1.json
    claims-v1.json
    schemas/
    fixtures/
  lean/
    lakefile.toml
    lean-toolchain
    Residiuum/
      State.lean
      Observation.lean
      Refinement.lean
      Consistency/
      Security/
      Atomics/
      Cluster/
  tla/
    consistency/
    security/
    atomics/
    cluster/
  verus/
    foundation/
    consistency/
    security/
    atomics/
    cluster/
  kani/
    registry/
verification/
  formal-assurance/
    src/
    fixtures/
scripts/
  setup-formal-tools.sh
  check-formal-registry.sh
  check-formal-foundation.sh
  check-formal-consistency.sh
  check-formal-security.sh
  check-formal-atomics.sh
  check-formal-cluster.sh
  build-proof-bundle.sh
  verify-proof-bundle.sh
```

Existing `formal/heap/` and `verification/heap-verus/` are migrated into or
registered by this structure without discarding accepted evidence.

## 3. Package graph

```text
CSQ-12
  ↓
FAS-0  Doctrine, claims and theorem registry
  ├── FAS-1  Pinned tools + reproducible proof CI
  └── FAS-2  Abstract state/observation kernel
              ↓
FAS-3  Refinement bridge + production entrypoint census
  ├── FAS-4  Consistency theorem family
  └── FAS-5  Security/noninterference theorem family

ATM-1 + FAS-3/4/5
  ↓
FAS-6  Atomic safety and invariant preservation
  ↓
FAS-7  Isolation histories, recovery convergence and conditional liveness

cluster protocol freeze + FAS-3/4/5 (+ FAS-6/7 where used)
  ↓
FAS-8  Cluster agreement, fencing, membership and convergence

FAS-1/2/3 + any completed theorem family
  ↓
FAS-9  Public proof bundle, CLI and clean-room reproduction
```

## 4. FAS-0 — Doctrine and registries

Depends: `CSQ-12`

Deliver:

- theorem, assumption, TCB, claim and profile registries;
- JSON Schemas and accepted/rejected fixtures;
- assurance-status state machine and claim-language linter;
- theorem dependency graph;
- theorem-to-CSQ/test/proof ownership map;
- tool authority table; and
- migration map for existing Heap and CSQ formal artifacts.

Tests reject:

- unknown/missing status, tool, assumption and family;
- claim without theorem;
- proof without source/result hash;
- implementation connection without entrypoint/refinement;
- bounded result without bounds;
- liveness theorem labeled safety;
- stale theorem still public;
- circular theorem dependency; and
- forbidden broad formal-verification wording.

Exit:

- every existing formal artifact is registered, explicitly historical, or
  identified as unconnected;
- no existence/count proxy can satisfy proof status; and
- CI verifies the closed registries.

## 5. FAS-1 — Reproducible toolchain

Depends: `FAS-0`

Deliver:

- pinned Lean 4, Verus, Kani, TLA+/TLC/TLAPS toolchain;
- checksummed installation/bootstrap;
- offline/cacheable CI layout;
- per-tool result adapters and deterministic result schema;
- negative-control runner;
- timeout/resource classification;
- changed-proof dependency selection; and
- dedicated CI lanes.

Tests cover corrupt/missing/wrong-version tools, stale cached results, timeout
versus failed theorem, skipped proof discovery, accepted false companions,
source/result mismatch and clean-machine bootstrap.

Exit:

- one command reproduces a small proof in every selected system;
- negative controls demonstrably fail; and
- CI cannot report green when a mandatory tool did not run.

## 6. FAS-2 — Abstract semantic kernel

Depends: `FAS-0`, `FAS-1`

Deliver:

- Lean definitions for identities, qualified collections, keys, generations,
  values, authority, durability, coverage/damage, evidence and observations;
- typed outcomes preserving partial/damaged/unknown distinctions;
- abstract operations and crash relations;
- invariant-composition framework;
- safety/liveness classification;
- theorem notation rendered into documentation; and
- independent finite vectors for model/implementation comparison.

Proofs/tests establish initial-state well-formedness, operation totality over
declared inputs, observation-constructor separation, no forbidden projection,
and deterministic-versus-relational operation classification.

Exit:

- consistency and security theorems use one common state/observation universe;
- no feature owns conflicting private definitions of commitment, absence,
  uncertainty or authority.

## 7. FAS-3 — Refinement bridge

Depends: `FAS-2`

Deliver:

- Rust entrypoint census for each target claim;
- concrete-to-abstract type map and abstraction functions;
- initial-state and operation forward simulation;
- observation/error and crash/recovery refinement;
- feature/profile/build-flag binding;
- reachable unsafe/FFI census; and
- Verus connection architecture.

The package distinguishes:

```text
abstract theorem only
independent executable agreement
bounded concrete proof
Rust-connected refinement
```

Tests prove that renamed/deleted/reachable-changed entrypoints invalidate the
connection, feature changes revoke profiles, forbidden-collapse mutants die,
duplicate demonstration implementations are rejected, and vector agreement is
not mislabeled proof.

Exit:

- one end-to-end theorem is Lean-proved, Verus/Rust-connected, Kani-bounded
  where applicable and linked to CSQ evidence;
- a controlled code change revokes the correct dependent closure.

## 8. FAS-4 — Consistency theorem family

Depends: `FAS-3`, accepted applicable CSQ packages

Deliver proofs/connections for:

- no fabricated value;
- generation-exact reconstruction;
- publication old/new/unknown but never hybrid;
- durable acknowledgement under the filesystem assumption ledger;
- recovery idempotence;
- derived-state non-authority;
- damage honesty; and
- healthy-island locality.

Reuse CSQ publication/chunk kernels, independent model/reference reader,
crash/corruption evidence and genuinely connected Kani work.

Exit:

- `residiuum-formal-consistency-v1` bundle verifies;
- every theorem has live negative controls;
- Rust entrypoints and CSQ evidence are connected; and
- wording names filesystem/corruption assumptions.

## 9. FAS-5 — Security theorem family

Depends: `FAS-3`, `FAS-4`, frozen Heap authority/admission contracts

Deliver proofs/connections for:

- Heap noninterference;
- authority confinement and delegation monotonicity;
- epoch rotation/grace and blacklist soundness;
- master-key non-serving;
- ordinary/wildcard scope behavior; and
- consistency preservation under authorized operations.

Migrate and strengthen `formal/heap/*.tla`, `verification/heap-verus/`,
`residiuum_heap::pure_proofs` and existing Kani/Verus CI.

Exit:

- the existing eight-proof Heap bundle is represented honestly;
- missing complete-path refinements remain visible;
- `residiuum-formal-security-v1` verifies for its achieved scope; and
- cryptographic assumptions are not presented as primitive proofs.

## 10. FAS-6 — Atomic safety

Depends: `FAS-3`, `FAS-4`, `FAS-5`, `ATM-1`

Develop this package with the Atomic state machine, before persistence
implementation is admitted.

Deliver:

- canonical Atomic state/decision/member model;
- all-or-none visibility, prepare completeness and decision uniqueness;
- prepared-state invisibility and retry idempotence;
- RRE/relationship invariant preservation;
- consistency/security preservation; and
- false-hybrid/dual-decision mutants.

Exit:

- Atomic implementation packages cite FAS-6 theorem IDs;
- no implementation transition exists outside the formal registry;
- bounded and machine proofs pass before authoritative persistence is enabled.

## 11. FAS-7 — Isolation and Atomic liveness

Depends: `FAS-6`, Atomic recovery protocol freeze

Deliver:

- exact named isolation history predicate;
- concurrent-history refinement and conflict rules;
- crash/retry/recovery temporal model;
- recovery convergence;
- safety independent of liveness;
- conditional liveness with fairness/storage assumptions; and
- Loom/Shuttle/model-check counterexample replay.

Exit:

- every advertised isolation word maps to a proved predicate;
- safety remains green when fairness is removed;
- liveness fails honestly when assumptions are removed; and
- `residiuum-formal-atomics-v1` verifies.

## 12. FAS-8 — Cluster safety and liveness

Depends: cluster protocol freeze, `FAS-3`…`FAS-5`, plus `FAS-6/7` for
distributed Atomics

Begin before cluster protocol implementation.

Deliver:

- named consensus protocol mapping;
- quorum intersection including joint membership;
- unique-term authority and leader fencing;
- log/decision agreement and acknowledgement survival;
- partition honesty, membership safety and replica convergence;
- Heap confinement through routing/replication/repair;
- protocol-message/storage refinement; and
- deterministic simulation/Jepsen counterexample replay.

Exit:

- no undefined “Raft-like” protocol remains;
- safety passes for every declared failure/membership cell;
- liveness assumptions are explicit;
- wire/implementation state refines the model; and
- `residiuum-formal-cluster-v1` verifies before a strong cluster claim.

## 13. FAS-9 — Public proof product

Depends: `FAS-1`, `FAS-2`, `FAS-3`, and one accepted theorem family

Deliver:

- canonical proof bundle builder/verifier;
- `residiuum claims`, `theorem`, `assumptions`, and `verify-proofs`;
- source/binary/profile binding;
- signing support when release signing exists;
- theorem pages generated from registry truth;
- clean-room reproduction; and
- release capability-language integration.

Tests reject tampered/omitted/stale results, forged higher status, missing
negative controls, unavailable tools, wrong binaries and partial bundles.

Exit:

- an outsider can inspect a claim, theorem and assumptions, rerun its proof,
  verify the Rust connection and follow the CSQ evidence;
- every overstated fixture is refused; and
- the proof bundle is a release artifact.

## 14. Execution waves

### Wave A — immediately after CSQ

```text
FAS-0 → FAS-1/FAS-2 → FAS-3 → FAS-4
```

Runs alongside PQH and M1.

### Wave B — Heap/security consolidation

`FAS-5` reuses existing connected Heap proofs and closes their theorem/Rust/
claim chain.

### Wave C — Atomics

`FAS-6 → FAS-7` runs as part of Atomics development, not retrospective
certification.

### Wave D — Cluster

`FAS-8` starts from the protocol model before production protocol code.

### Wave E — public proof product

`FAS-9` starts after FAS-3 and publishes incrementally with accepted families.

## 15. Definition of done

The program succeeds when a release can truthfully expose:

```text
$ residiuum theorem FAS-CON-GENERATION-EXACT-001

Mathematical statement:       available
Machine proof:                passed
Production Rust refinement:   connected
Bounded concrete proof:       passed; bounds shown
Physical qualification:       passed; CSQ evidence linked
Assumptions:                  listed
Negative controls:            rejected
Release/source hashes:        match
```

and a critic can reproduce that result without trusting Residiuum’s runtime or
team.

