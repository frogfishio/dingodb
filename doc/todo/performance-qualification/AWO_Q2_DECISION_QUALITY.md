# AWO-Q2 — Adaptive decision quality (labor evidence)

Status: **labor `in_review`** (not package accept / default-on)  
Card: `0a043642-5e79-4a18-8350-debbc63d6a6e`  
Feature: `d0ae3c06` (AWO-Q series)  
Date: 2026-08-03  

## 0. Hang root cause (why this turn was methodical)

Prior agent turns hung ~30 minutes because concurrent AWO façade tests
deadlocked on **StoreHost Drop / detach**:

1. `detach` held the physical store mutex.
2. `IndependentCollector::shutdown` **joined** the `awo-collect` thread under that lock.
3. Collector was blocked on `physical.lock()` (after collection delay) → join never returned.
4. Symptom amplified when a prior Adaptive host (Q2 warm cell) left scheduling
   that made the race hit on the next sparse concurrent cell in the same process.

**Fix:** flush + `request_shutdown` under the lock; `join_after_detach` only after
mutex release. Collector loop uses interruptible `wait_timeout` + `try_lock` and
exits on shutdown if the lock is held by detach.

## 1. What Q2 proves (labor)

| # | Claim | Test |
|---|-------|------|
| 1 | Static multi-item present takes full slice (explicit-batching ceiling) | `q2_static_full_batch_vs_adaptive_cold_natural_one` |
| 2 | Adaptive cold takes Natural-1 + records `natural_insufficient_evidence` | same |
| 3 | Warm Adaptive leaves cold-evidence path; Batch (entries>1) **or** honest Natural decline | `q2_adaptive_warm_can_select_batch_on_multi_item` |
| 4 | Saturated concurrent façade: both modes `file_sync < logical_ack`; Adaptive wall ≤3× Static (smoke) | `q2_concurrent_facade_static_vs_adaptive_saturated_envelope` |
| 5 | Sparse concurrent façade: Adaptive wall ≤3× Static (smoke) | `q2_concurrent_facade_sparse_adaptive_latency_envelope` |

Metric law: `file_sync / logical_acknowledged_operations` (not file_sync/append).

## 2. Explicit residual / non-claims

- IndependentCollector batch sizing is still delay/max-entries only —
  `select_plan` is **not** wired into collector flush. Adaptive vs Static
  **decision divergence** is proven on multi-item `admit_put_batch`.
- Concurrent independent singles share collector mechanics (amortization), not
  controller plan selection.
- Not: package accept, default-on, thr floors, PQH diagnostic class, sparse product bound, crash.

## 3. Re-verify (bounded — do not run workspace)

```bash
cargo test -p residiuum-store --features legacy-raw-store \
  --test awo_q2_decision_quality -- --test-threads=1 --nocapture
```

Labor host 2026-08-03: **4/4** in ~1.4s; stress 8× suite **0 hangs**.

## 4. Paths touched

| Path | Intent |
|------|--------|
| `crates/residiuum-store/src/adaptive_write/collection.rs` | request/join split; interruptible delay; try_lock on shutdown |
| `crates/residiuum-store/src/adaptive_write/runtime.rs` | detach signals only; `join_after_detach` |
| `crates/residiuum-store/src/heap/host.rs` | Drop/attach/reset join after unlock |
| `crates/residiuum-perf/src/store_driver/real.rs` | join after detach |
| `crates/residiuum-store/tests/awo_q2_decision_quality.rs` | Q2 decision + envelope cells |
