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
| [AWO_THREE_WAY_MEASURE_RUNBOOK.md](AWO_THREE_WAY_MEASURE_RUNBOOK.md) | Three-way measure T2/T3: fixed diagnostic matrix + correctness smoke gate (no throughput claims) |
| [AWO_THREE_WAY_T3_CORRECTNESS_SMOKE.md](AWO_THREE_WAY_T3_CORRECTNESS_SMOKE.md) | T3 evidence: three-mode unit + CLI driver-smoke green before numbers |
| [AWO_THREE_WAY_T4_DISKSAFE_MEASURE.md](AWO_THREE_WAY_T4_DISKSAFE_MEASURE.md) | T4 disk-safe first numbers (smoke slice); diagnostic residual; artifact paths |
| [AWO_THREE_WAY_T5_HONESTY.md](AWO_THREE_WAY_T5_HONESTY.md) | T5 honesty: claim table; ~30 GiB free host budget; no product ranking |
| [AWO_THREE_WAY_T6_INTERACTIVE.md](AWO_THREE_WAY_T6_INTERACTIVE.md) | T6 interactive re-run on Scratch; smoke OK; diagnostic exFAT residual |
| [AWO_THREE_WAY_T7_SPARSE_SATURATED.md](AWO_THREE_WAY_T7_SPARSE_SATURATED.md) | T7 v2: sparse/saturated **independent singles** (not harness batch_size=N); L-API vs L-AWO |
| [AWO_THREE_WAY_T8_SINGLES_RUN.md](AWO_THREE_WAY_T8_SINGLES_RUN.md) | T8 APFS smoke run: pin batch=1; all modes sync/op=1; collection residual |
| [AWO_THREE_WAY_T9_DECISIVE_FINDING.md](AWO_THREE_WAY_T9_DECISIVE_FINDING.md) | **Decisive (pre-connect):** harness OK; independent path was natural-only |
| [AWO_INDEPENDENT_COLLECTION_CONNECT.md](AWO_INDEPENDENT_COLLECTION_CONNECT.md) | Collection connect labor: queue+collector; concurrent file_sync amortize test |
| [AWO_THREE_WAY_T10_HARNESS_RERUN.md](AWO_THREE_WAY_T10_HARNESS_RERUN.md) | T10: PQH admit_put path + re-run; saturated sync/op=0.5 thr~2× |
| [AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md](AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md) | **T11 evidence freeze principal `done`:** saturated thr×2 + sparse 11–20% smoke penalty (card only; not package accept) |
| [AWO_QUALIFICATION_SERIES.md](AWO_QUALIFICATION_SERIES.md) | **AWO-Q series plan:** Q1 multi-thread admit → Q2 adaptive quality → Q3 sustained → Q4 sparse product bound |
| [PERF_BEARINGS_2026-08-03.md](PERF_BEARINGS_2026-08-03.md) | **Post-hang bearings:** where we are vs bigger truth (T11 + Q1/Q2 + PEER-SQL + PQH); not package accept |
| [AWO_Q1_1_IMPLEMENTER_BRIEF.md](AWO_Q1_1_IMPLEMENTER_BRIEF.md) | Q1.1 brief (anchors) |
| [AWO_Q1_1_HARNESS.md](AWO_Q1_1_HARNESS.md) | **Q1.1 labor:** concurrent path wired + per-seq ledger; test green |
| [artifacts/awo-three-way-t10-apfs-smoke/](artifacts/awo-three-way-t10-apfs-smoke/) | T10 smoke numbers (SoT for T11 freeze) |
| [artifacts/awo-three-way-t7-apfs-smoke/](artifacts/awo-three-way-t7-apfs-smoke/) | T8 numeric summary + campaigns |
| [artifacts/awo-three-way-t4-disksafe/](artifacts/awo-three-way-t4-disksafe/) | T4 JSON evidence only (no store trees) |
| [artifacts/awo-three-way-t6-scratch-smoke/](artifacts/awo-three-way-t6-scratch-smoke/) | T6 Scratch smoke three-way |
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