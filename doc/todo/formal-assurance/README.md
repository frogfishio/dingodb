# Formal Assurance Spine

State: **foundation in progress — FAS-0…FAS-4 scoreboard accept (MVP scopes);
FAS-5+ open.** Operator guide: [formal/HOW_TO_USE.md](../../../formal/HOW_TO_USE.md).

Program: `FAS`

The Formal Assurance Spine turns Residiuum’s principal product claims into
versioned theorem families connected to the production Rust implementation and
to adversarial qualification evidence.

| Document | Authority |
|---|---|
| [FORMAL_ASSURANCE_SPEC.md](FORMAL_ASSURANCE_SPEC.md) | Claim language, mathematical model, proof systems, theorem families, refinement and release evidence |
| [FORMAL_ASSURANCE_REGISTRY_CONTRACT.md](FORMAL_ASSURANCE_REGISTRY_CONTRACT.md) | Exact identifiers, schemas, initial theorem catalogue, evidence objects, status derivation and profile gates |
| [FORMAL_KERNEL_MODEL_CONTRACT.md](FORMAL_KERNEL_MODEL_CONTRACT.md) | Exact abstract types, state, operations, observations, well-formedness and Rust-refinement obligations |
| [FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md](FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md) | Package order, repositories, tooling, artifacts, tests and acceptance |

`CSQ-12` is scoreboard **accept**; FA0 foundation packages **FAS-0…FAS-4** are
scoreboard **accept** (FAS-4 = consistency MVP). Feature-specific families
continue with their features:

```text
CSQ-12 (accept)
  ↓
FAS-0…4 foundation + consistency MVP   ← delivered (see formal/HOW_TO_USE.md)
  ├── FAS-5 security/noninterference     ← next formal lane
  ├── FAS-6/7 Atomics/isolation when Atomics enters
  └── FAS-8 cluster before cluster product
FAS-9 public proof bundle after families land
```

Residiuum never claims “the whole database is formally verified.” It publishes
the exact theorem, assumptions, bounds, proof status, Rust connection and
physical qualification status for each claim.