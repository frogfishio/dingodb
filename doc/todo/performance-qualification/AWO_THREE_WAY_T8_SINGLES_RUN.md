# Measure AWO three-way — T8 run (independent singles on APFS)

Status: **labor complete (self_check) — smoke presentation experiment, not package accept**  
Card: `2d12bec0-2046-4425-9505-80e6b78e9154`  
Date: 2026-08-03  
Plan: [AWO_THREE_WAY_T7_SPARSE_SATURATED.md](AWO_THREE_WAY_T7_SPARSE_SATURATED.md) v2  

---

## 1. What we ran

| Item | Value |
|------|--------|
| FS | **APFS** (`/System/Volumes/Data`, ~31 GiB free) |
| Class | **smoke** only (disk budget) |
| Seed | 42 |
| `max_cells` | 1 |
| Modes | disabled · static · adaptive |
| Presentation | **`--present-batch 1` always** (AWO-fair; not L-API `put_many(N)`) |
| Shape A sparse | `--present-concurrency 1 --present-outstanding 1` |
| Shape B saturated | `--present-concurrency 4 --present-outstanding 8` |
| Binary | `target/release/residiuum-perf` + store-driver |
| Harness | presentation pin (this labor) |

Primary matrix pick (seed 42) + pin suffix:

| Shape | cell_id |
|-------|---------|
| sparse | `L4-durable-s16384-…-pin-b1-c1-o1` |
| saturated | `L4-durable-s16384-…-pin-b1-c4-o8` |

Artifacts: `artifacts/awo-three-way-t7-apfs-smoke/` (`summary.json` + per-mode campaigns).

---

## 2. Results (median of 6 reps; thr = proxy MiB/s)

### Shape A — sparse singles (batch=1, conc=1, out=1)

| Mode | valid+reopen | thr med | e2e med ms | append | file_sync | sync/append |
|------|--------------|---------|------------|--------|-----------|-------------|
| disabled | yes | ~4.28 | ~92 | 24 | 24 | **1.00** |
| static | yes | ~4.53 | ~87 | 24 | 24 | **1.00** |
| adaptive | yes | ~4.15 | ~95 | 24 | 24 | **1.00** |

**Sparse read:** Adaptive thr ≈ Disabled (within smoke noise). No forced collection delay visible at this scale. All modes **1 Durable sync per op**.

### Shape B — saturated independent singles (batch=1, conc=4, out=8)

| Mode | valid+reopen | thr med | e2e med ms | append | file_sync | sync/append |
|------|--------------|---------|------------|--------|-----------|-------------|
| disabled | yes | ~4.30 | ~91 | 24 | 24 | **1.00** |
| static | yes | ~4.35 | ~90 | 24 | 24 | **1.00** |
| adaptive | yes | ~4.28 | ~92 | 24 | 24 | **1.00** |

**Saturated read:** Disabled stays **sync/op ≈ 1** (presentation correct — no accidental L-API multi-item).  
Static/Adaptive **do not** drop `file_sync/ops` under concurrent independent singles on this path. Thr stays same ballpark as Disabled.  
→ **AWO is not forming multi-write barriers from independent singles today** (matches `admit_put` / per-flush natural path honesty in T7).

---

## 3. Claim / non-claim

| Claim | Status |
|-------|--------|
| AWO-fair presentation pin works (`batch=1`, vary conc/out) | **Yes** |
| Sparse/saturated three-way smoke valid+reopen on APFS | **Yes** |
| Disabled under singles: `file_sync ≈ ops` | **Yes** (both shapes) |
| Static amortizes barriers by collecting concurrent singles | **No** (not yet) |
| Adaptive converges to Static under saturated singles | **No** (nothing to converge to; all ~1 sync/op) |
| Adaptive preserves sparse latency vs Disabled | **Consistent with smoke** (≈ tied thr) |
| Product ranking / default-on / diagnostic floors | **No** |

---

## 4. Harness change (this labor)

| Path | Change |
|------|--------|
| `campaign/run.rs` | `PresentationPin` + apply after matrix pick; multiproc finding off when pin active |
| `residiuum-perf` CLI | `--present-batch` / `--present-concurrency` / `--present-outstanding` |

Example:

```bash
PERF=target/release/residiuum-perf
$PERF run --work /tmp/w --driver real_store --seed 42 --class smoke \
  --max-cells 1 --awo-mode adaptive --no-spawn-workers \
  --present-batch 1 --present-concurrency 4 --present-outstanding 8
```

---

## 5. Residual (next product labor, not re-run smoke thr chase)

1. **Independent-single collection** under AWO lease (spec collection delay / queue) so Static can install multi-item batches from concurrent singles.
2. Harness may need to present concurrent `admit_put` (not only concurrent preparers + serial single-item `admit_put_batch([1])`) for collection to see a queue.
3. Re-run Shape B **diagnostic** on roomier APFS after collection exists; pass when Static `file_sync/ops ≪ 1` and Adaptive → Static.

---

## 6. Stop

| Criterion | Met? |
|-----------|------|
| T7 v2 shapes executed on APFS smoke | **Yes** |
| Presentation proven batch=1 | **Yes** |
| Barrier math recorded | **Yes** |
| Package accept | **No** |
