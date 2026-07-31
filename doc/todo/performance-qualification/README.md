# Performance Qualification Harness

State: **TODO — blocked by CSQ-12**

Program: `PQH`

This program builds the controlled laboratory used to explain Residiuum
performance from the storage device up to the acknowledged database operation.
It is not a marketing benchmark and it is not the existing damage/scale
testrig.

| Document | Authority |
|---|---|
| [PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md) | Measurement semantics, experiment matrix, metrics, attribution mathematics, safety and acceptance |
| [PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md](PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md) | Packages, dependencies, artifacts, tests and delivery order |

Entry dependency: `CSQ-12 = accept`.

Execution position: the first post-C0 measurement lane. It may run alongside
M1 feature work, but no performance optimization or new quantitative product
claim may be selected from intuition once `PQH-0` begins. Optimization must
follow a reproduced PQH finding.

