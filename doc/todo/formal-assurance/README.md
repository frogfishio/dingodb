# Formal Assurance Spine

State: **TODO — FAS-0 blocked by CSQ-12**

Program: `FAS`

The Formal Assurance Spine turns Residiuum’s principal product claims into
versioned theorem families connected to the production Rust implementation and
to adversarial qualification evidence.

| Document | Authority |
|---|---|
| [FORMAL_ASSURANCE_SPEC.md](FORMAL_ASSURANCE_SPEC.md) | Claim language, mathematical model, proof systems, theorem families, refinement and release evidence |
| [FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md](FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md) | Package order, repositories, tooling, artifacts, tests and acceptance |

The foundation starts immediately after `CSQ-12` alongside PQH and M1.
Feature-specific theorem families enter with their feature:

```text
CSQ-12
  ↓
FAS foundation + consistency
  ├── security/noninterference
  ├── Atomics/isolation when Atomics enters
  └── cluster agreement/convergence before cluster implementation
```

Residiuum never claims “the whole database is formally verified.” It publishes
the exact theorem, assumptions, bounds, proof status, Rust connection and
physical qualification status for each claim.

