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
| Concurrent outcomes → **deterministic per-seq ledger** | `fold_concurrent_op_outcomes` + `execute_concurrent_admit_put` |
| **Reject carries issued `seq`** (not anonymous) | `ConcurrentOpOutcome::Reject { seq }` |
| **Multi-terminal** detection (Ack+Ack, Ack+Fail, Fail+Fail, Reject+*, …) | fold; fail-closed run error |
| **Missing-seq / hole** detection on issued span `0..=max_seq` | fold; fail-closed run error |
| Unit tests (pure fold + integration smoke) | `concurrent_ledger_fold_*`, `concurrent_admit_independent_awo_smoke_ledger` |

### Removed residual (this slice)

```text
// OLD: always serial + note "conc=N mapped to serial admit_put outstanding=M"
// NEW: workers>1 → execute_concurrent_admit_put; workers==1 → serial
```

### Principal ledger gaps closed (post-review remediation)

| # | Gap | Fix |
|---|-----|-----|
| 1 | `OpOutcome::Reject` had **no seq** — cannot prove issued → terminal | `Reject { seq }` on lock-poison and admit reject |
| 2 | Duplicate detect only **double-ack** | Any second terminal for same seq → `multi_terminal` |
| 3 | Same seq **Ack then Fail** (or Fail+Fail, Reject+Ack, …) silent | Covered by multi-terminal; `multi_mixed` / `multi_ack` notes |
| 4 | `attempted == ack+failed` was **circular** (built only from outcomes) | Balance also requires `multi_terminal==0` and `missing_seq==0` on span; fail-closed if not |

## 2. Verify

```bash
cargo test -p residiuum-perf --features store-driver --lib concurrent_
# fold unit tests + concurrent_admit_independent_awo_smoke_ledger
```

## 3. Claim table

| Claim | Status |
|-------|--------|
| Concurrent independent AWO path uses real threads (not serial map) | **Yes** (code + test notes) |
| Every terminal outcome carries a seq (including Reject) | **Yes** |
| Exactly-one-terminal per seq (multi-terminal detected + fail-closed) | **Yes** (unit + smoke) |
| Issued span has no holes (missing_seq) | **Yes** (unit + smoke) |
| `ledger_balance=ok` only when count + multi + missing all clean | **Yes** |
| Thr ranking / three-way re-run under concurrent | **No** (Q1.3) |
| Full reopen integrity campaign | Residual → Q1.3 |
| Product wait-outside-mutex redesign | Already present in `heap_store::put_if` (Q1.2 may skip) |
| Package accept / default-on | **No** |

## 4. Residual for Q1.2 / Q1.3

- **Q1.2:** Heap already waits outside mutex; confirm concurrent collection pile-up under product path if needed, else document skip.
- **Q1.3:** Reopen digest campaign under concurrent admit; optional thr Off/Static/Adaptive re-measure; freeze claim table. Dropped outcomes if `outcomes_mu` lock fails still surface as `missing_seq` (fail-closed).
