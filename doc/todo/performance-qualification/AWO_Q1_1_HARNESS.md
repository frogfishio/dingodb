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
| **Missing-seq / hole** detection on authoritative `0..issued_count` | fold; fail-closed run error |
| **Out-of-range** terminal seq rejection | fold; fail-closed |
| **Global outstanding** credit pool (not per-worker) | `GlobalOutstanding` shared across workers |
| **Worker panic** / **poisoned outcomes_mu** → run failure | hard `join` + lock error (not empty fold) |
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
| 4 | `attempted == ack+failed` was **circular** (built only from outcomes) | Balance also requires multi/missing/out-of-range clean; fail-closed |
| 5 | Span `0..=max_seq` hides missing **tail** when final issued ops disappear | Authoritative `issued_count`; require exactly `0..issued_count` |
| 6 | `let _ = h.join()` swallowed worker panics | Hard error: panicked producer invalidates leg |
| 7 | Per-worker outstanding → N×M silent in-flight | Shared `GlobalOutstanding` (PQH global limit) |
| 8 | Poisoned `outcomes_mu` → empty vec fold | Direct run failure |

## 2. Verify

```bash
cargo test -p residiuum-perf --features store-driver --lib concurrent_
# fold unit tests + concurrent_admit_independent_awo_smoke_ledger

cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission \
  heap_store_facade_concurrent -- --test-threads=1
# Q1.2 façade concurrent path
```

## 3. Claim table

| Claim | Status |
|-------|--------|
| Concurrent independent AWO path uses real threads (not serial map) | **Yes** (code + test notes) |
| Every terminal outcome carries a seq (including Reject) | **Yes** |
| Exactly-one-terminal per seq (multi-terminal detected + fail-closed) | **Yes** (unit + smoke) |
| Issued span has no holes (missing_seq) on `0..issued_count` | **Yes** (unit + smoke) |
| Missing **tail** detected via authoritative issued_count | **Yes** (unit `concurrent_ledger_fold_missing_tail_via_issued_count`) |
| Out-of-range terminal seq rejected | **Yes** (unit) |
| Outstanding is **global** shared credit (not per-worker) | **Yes** (code + unit + smoke note `outstanding_global=`) |
| Worker panic / poisoned outcomes_mu fail the run | **Yes** (code) |
| `ledger_balance=ok` only when count + multi + missing + OOR clean | **Yes** |
| Thr ranking / three-way re-run under concurrent | **No** (Q1.3) |
| Full reopen integrity campaign | Residual → Q1.3 |
| Product wait-outside-mutex under façade | **Q1.2** — `heap_store_facade_concurrent_put_collection_wait_outside_mutex` |
| Package accept / default-on | **No** |

## 4. Residual for Q1.2 / Q1.3

- **Q1.2:** Labor evidence — concurrent `HeapStore::put_collection` (façade) amortizes Durable file_sync; wait-outside-mutex product wiring proven. Direct-handle test remains in `independent_puts_collect_amortize_file_sync`.
- **Q1.3:** Reopen digest campaign under concurrent admit; optional thr Off/Static/Adaptive re-measure; freeze claim table.