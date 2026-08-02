# Performance Qualification Harness

State: **ACTIVE — PQH-0 labor floor delivered (principal accept open)**

Program: `PQH`

This program builds the controlled laboratory used to explain Residiuum
performance from the storage device up to the acknowledged database operation.
It is not a marketing benchmark and it is not the existing damage/scale
testrig.

| Document | Authority |
|---|---|
| [PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md) | Measurement semantics, experiment matrix, metrics, attribution mathematics, safety and acceptance |
| [PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md](PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md) | Packages, dependencies, artifacts, tests and delivery order |
| [ADAPTIVE_WRITE_OPTIMISER_SPEC.md](ADAPTIVE_WRITE_OPTIMISER_SPEC.md) | Post-PQH adaptive intake, cooking, write-pipeline, acknowledgement, control, proof and qualification contract |
| [ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md](ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md) | Exact std-thread architecture, Rust contracts, algorithms, defaults, files, packages, tests and acceptance commands |
| [AWO_LABOR_EXECUTION.md](AWO_LABOR_EXECUTION.md) | Developer start pack: entry honesty E1–E6, package DAG, board tasks, first-pull order (does not amend norms) |
| [AWO-0_T1_CONTRACT_RESIDUAL_CHECKLIST.md](AWO-0_T1_CONTRACT_RESIDUAL_CHECKLIST.md) | AWO-0 T1 evidence: E1–E6 stamp, contract inventory, plan §15 residual (not package accept) |
| [../../../spec/performance/README.md](../../../spec/performance/README.md) | PQH-0 live registries |
| [../../../spec/performance/awo/README.md](../../../spec/performance/awo/README.md) | AWO executable contracts (`verify-awo-contract.sh`) |

Profile: `residiuum-performance-qualification-v1`

Entry dependency: `CSQ-12 = accept`.

Execution position: the first post-C0 measurement lane. It may run alongside
M1 feature work, but no performance optimization or new quantitative product
claim may be selected from intuition once `PQH-0` begins. Optimization must
follow a reproduced PQH finding.

**PQH-0 evidence:** `bash scripts/verify-performance-registry.sh` +
`cargo test -p residiuum-perf --lib`.

**Next:** PQH-1 safe runner (after PQH-0 accept).

The Adaptive Write Optimiser is a specified post-PQH implementation candidate.
Its presence here does not admit it ahead of the master delivery plan.

**AWO labor (2026-08-02):** Full labor plan in `AWO_LABOR_EXECUTION.md`. Kanban
Feature **AWO — Adaptive Write Optimiser** pre-staged (AWO-0 T1–T3 + AWO-1…7).
**AWO-0 T1–T3 labor floor complete** + **AWO-1 deepen:**
persist-before-publish on single-shard, parallel-cook, and multi-shard `put_many`
(all-or-nothing publish; checkpoint restore on clean fail; poison on short write).
`awo_persist_before_publish` 4/4. Residuals: full AdaptiveWriteLease, full crash matrix.
Master-plan AWO admission residual (E1).