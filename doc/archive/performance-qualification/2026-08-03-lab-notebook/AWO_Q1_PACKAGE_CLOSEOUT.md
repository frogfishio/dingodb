# AWO-Q1 — Package labor freeze (principal accept gate)

Status: **principal package `done` 2026-08-03 (correctness scope only)**  
Umbrella: `b6e2a138-bfdd-4420-9ef1-847d6a48bc05`  
Feature: `d0ae3c06` (AWO-Q series)  
Date: 2026-08-03  

## 1. Slice board truth

| Slice | Card | Board stage | Evidence |
|-------|------|-------------|----------|
| Q1.1 Concurrent harness + ledger | `fab1a943` | **principal `done`** | `AWO_Q1_1_HARNESS.md` |
| Q1.2 Wait-outside-mutex façade | `37a4c1a2` | **principal `done`** | `AWO_Q1_2_FACADE.md` |
| Q1.3 Correctness + reopen freeze | `ba312673` | **principal `done`** | `AWO_Q1_3_REOPEN.md` |
| **AWO-Q1 umbrella** | `b6e2a138` | **principal `done`** | this note |

**Next series pull:** AWO-Q2 (`0a043642`) promoted **`todo`** — Adaptive decision quality / performance envelope.

## 2. Series claims (proven)

| # | Claim | Where |
|---|-------|--------|
| 1 | Distinct producer threads (not serial-outstanding map) | Q1.1 `execute_concurrent_admit_put` + Q1.2/Q1.3 façade threads |
| 2 | Deterministic per-seq ledger (exactly-once terminal) | Q1.1 fold + multi-terminal / missing_seq / issued_count |
| 3 | Caller waits outside physical mutex (Durable path) | Q1.2 `HeapStore::put_if` / façade concurrent test |
| 4 | No lost / dup / double-ack under concurrent admit | Q1.3 ack vector + digests |
| 5 | Clean close + normal product reopen integrity | Q1.3 pre/post blake3 digest |
| 6 | Segment rotation **during** concurrent collection | Q1.3 `rotate_during` from t0; `SegmentRotate≥5` |
| 7 | Exact reopened key/hash set == ledger | Q1.3 coverage-aware full scan |
| 8 | Concurrent-phase barrier counters only | Q1.3 snapshot after joins (no rotate epilogue mix) |

## 3. Product defect closed under Q1.3

Mid-batch auto-seal dropped `WriteReceipt`s in `put_many_*` → AWO collector short zip → false “awo draining”. Fixed by receipt accumulation + fail-closed install + completion-drop distinction. See `AWO_Q1_3_REOPEN.md` §2.

## 4. Re-verify (labor host 2026-08-03 package freeze)

```bash
# Q1.1 ledger + concurrent admit smoke
cargo test -p residiuum-perf --features store-driver concurrent_ledger -- --test-threads=1
cargo test -p residiuum-perf --features store-driver concurrent_admit -- --test-threads=1
# Q1.2 façade wait-outside-mutex
cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission \
  heap_store_facade_concurrent -- --test-threads=1
# Q1.3 reopen + concurrent rotate + exact scan
cargo test -p residiuum-store --features legacy-raw-store --test awo_q13_reopen_correctness \
  -- --test-threads=1 --nocapture
```

| Suite | Result |
|-------|--------|
| `concurrent_ledger_*` (5) | pass |
| `concurrent_admit_independent_awo_smoke_ledger` (1) | pass |
| `heap_store_facade_concurrent_put_collection_wait_outside_mutex` (1) | pass |
| `awo_q13_reopen_correctness` (3) | pass |

Sample Q1.3 concurrent-rotate cells:

```text
static  seed=42  acked=24 rotate=5 scan=24
static  seed=99  acked=32 rotate=7 scan=32
adaptive seed=1001 acked=32 rotate=7 scan=32
```

## 5. Explicit package non-claims

- AWO product package accept / default-on  
- Adaptive **decision quality** (AWO-Q2)  
- Sustained PQH diagnostic class (AWO-Q3)  
- Sparse latency product bound (AWO-Q4)  
- Crash-boundary / crash-matrix reopen  
- Thr floors for Off/Static/Adaptive (optional smoke residual only)  

## 6. Principal accept (recorded 2026-08-03)

1. ~~Review Q1.3 remediation → `ba312673` → `done`~~ **done**  
2. ~~Accept umbrella `b6e2a138` → `done`~~ **done**  
3. ~~Promote AWO-Q2 `backlog` → `todo`~~ **done** (`0a043642`)

## 7. Related paths

| Path | Role |
|------|------|
| `AWO_QUALIFICATION_SERIES.md` | Series SoT |
| `AWO_Q1_1_HARNESS.md` | Q1.1 evidence |
| `AWO_Q1_2_FACADE.md` | Q1.2 evidence |
| `AWO_Q1_3_REOPEN.md` | Q1.3 evidence |
| `crates/residiuum-perf/src/store_driver/real.rs` | concurrent producers + ledger |
| `crates/residiuum-store/tests/awo_static_admission.rs` | façade wait-outside-mutex |
| `crates/residiuum-store/tests/awo_q13_reopen_correctness.rs` | reopen + concurrent rotate + scan set |
| `crates/residiuum-store/src/store.rs` | mid-batch seal receipt accumulate |