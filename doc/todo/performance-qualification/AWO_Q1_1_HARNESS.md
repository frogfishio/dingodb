# AWO-Q1.1 — Concurrent admit harness + ledger (labor evidence)

Status: **labor complete (in_review) — not package accept**  
Card: `fab1a943-8def-4ded-9bfe-0499a567da56`  
Date: 2026-08-03  
Brief: `AWO_Q1_1_IMPLEMENTER_BRIEF.md`

## 1. What changed

| Change | Location |
|--------|----------|
| Independent AWO path uses **real concurrent producers** when `concurrency > 1` | `store_driver/real.rs` `execute_workload_puts_independent` |
| Serial path retained for `workers == 1` | same |
| Concurrent outcomes → **deterministic per-seq ledger** (sorted seq; no `for i in 0..acked`) | `execute_concurrent_admit_put` |
| Double-ack detection + `ledger_balance=ok` notes | messages on concurrent path |
| Unit test | `concurrent_admit_independent_awo_smoke_ledger` |

### Removed residual (this slice)

```text
// OLD: always serial + note "conc=N mapped to serial admit_put outstanding=M"
// NEW: workers>1 → execute_concurrent_admit_put; workers==1 → serial
```

## 2. Verify

```bash
cargo test -p residiuum-perf --features store-driver --lib concurrent_admit_independent_awo_smoke_ledger
# exit 0 — 1 passed

cargo test -p residiuum-perf --features store-driver --lib real_store_smoke
# broader smoke (run as part of labor)
```

## 3. Claim table

| Claim | Status |
|-------|--------|
| Concurrent independent AWO path uses real threads (not serial map) | **Yes** (code + test notes) |
| Deterministic per-seq ledger; `attempted == ack + failed` | **Yes** (smoke) |
| Messages show `concurrent_admit_put workers=…` + `ledger_balance=ok` | **Yes** |
| Thr ranking / three-way re-run under concurrent | **No** (Q1.3) |
| Full reopen integrity campaign | Residual → Q1.3 |
| Product wait-outside-mutex redesign | Already present in `heap_store::put_if` (Q1.2 may skip) |
| Package accept / default-on | **No** |

## 4. Residual for Q1.2 / Q1.3

- **Q1.2:** Heap already waits outside mutex; confirm concurrent collection pile-up under product path if needed, else document skip.
- **Q1.3:** Reopen digest campaign under concurrent admit; optional thr Off/Static/Adaptive re-measure; freeze claim table.
