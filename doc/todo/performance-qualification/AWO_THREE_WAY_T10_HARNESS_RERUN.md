# Measure AWO three-way — T10 harness independent admit_put + re-run

Status: **labor complete (self_check) — not package accept**  
Card: `a0766154-4ae4-45f8-a79f-58643c466555`  
Date: 2026-08-03  
Depends on: collection connect + T8/T9  

## 1. What changed in the harness

For **AWO static/adaptive** with **`batch_size == 1`**:

1. Wrap store in `Arc<Mutex<Store>>` and `bind_physical` for the collector  
2. Workload uses **`admit_put`** (enqueue) with **wait outside** the store lock  
3. Outstanding depth piles independent puts for collection  
4. Disabled still uses natural `put_many` (fair L-AWO comparison)

Messages: `awo_path=independent_admit_put+collection` / `awo_flush=admit_put_collect`.

Multi-thread admit remains residual (reopen integrity); conc>1 maps to serial admit with `outstanding = max(out, conc)`.

## 2. T10 smoke re-run (APFS, seed 42, max_cells=1)

Artifacts: `artifacts/awo-three-way-t10-apfs-smoke/summary.json`

### Sparse (b1 c1 o1)

| Mode | valid+reopen | thr med MiB/s | file_sync/append |
|------|--------------|---------------|------------------|
| disabled | yes | ~4.42 | **1.00** |
| static | yes | ~3.88 | **1.00** |
| adaptive | yes | ~3.52 | **1.00** |

Sparse: collection window does not force multi-item batches; Adaptive/Static slightly slower (collection delay) — honest latency tradeoff.

### Saturated (b1 c4 o8 → serial admit outstanding=8)

| Mode | valid+reopen | thr med MiB/s | file_sync med | file_sync/append |
|------|--------------|---------------|---------------|------------------|
| disabled | yes | ~4.18 | 24 | **1.00** |
| static | yes | ~**9.04** | **12** | **0.50** |
| adaptive | yes | ~**8.76** | **12** | **0.50** |

**Decisive product signal:** under independent singles with outstanding pile-up, Static/Adaptive **amortize Durable barriers** (~2 ops per sync) and **~2× thr** vs Disabled on this smoke cell. Disabled stays 1 sync/op (presentation still fair).

## 3. Claims

| Claim | Status |
|-------|--------|
| Harness exercises AWO collection for batch=1 | **Yes** |
| Saturated Static/Adaptive `file_sync/ops < 1` | **Yes** (smoke) |
| Adaptive ≈ Static thr under saturated smoke | **Yes** (this cell) |
| Sparse Adaptive does not explode latency | **OK** (slightly slower) |
| thr×2 ↔ sync/2 causal freeze (first positive L-AWO signal) | **Yes** → `AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md` |
| Diagnostic floors / product ranking / default-on | **No** |
| Multi-thread admit_put path | Residual |

## 4. Exit

```bash
cargo test -p residiuum-perf --features store-driver --lib real_store_smoke
# smoke re-run: see summary.json commands in meta/run logs
```