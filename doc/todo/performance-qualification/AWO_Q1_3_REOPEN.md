# AWO-Q1.3 — Concurrent admit correctness + reopen evidence freeze

Status: **principal `done` 2026-08-03** (correctness scope; AWO-Q1 package closed)  
Card: `ba312673-ce84-4c89-ae1f-46c61caf112f`  
Date: 2026-08-03 (remediation: concurrent rotation + exact scan set)  
Parent: AWO-Q1 (`b6e2a138`) · Series: `AWO_QUALIFICATION_SERIES.md`

## 1. What this proves

| Claim | Evidence |
|-------|----------|
| Static + Adaptive concurrent **façade** callers | `awo_q13_reopen_correctness` matrices |
| Every ack binds seq → key + body hash + length | oracle `ExpectedOp` + pre/post get |
| Exactly-once ack (no lost / double-ack) | per-seq bool vector under concurrent writers |
| Clean close + **normal product reopen** | `drop(host)` → `StoreHost::open_with_adaptive_write` → re-open heap |
| Pre-close digest == reopened digest | blake3 chain over (seq, key, body_hash, len) |
| **Exact reopened key/hash set == ledger** | coverage-aware `scan_collection_page` full walk; set equality (no unexpected records) |
| Multi-seed / concurrency matrix | seeds 1/7/42/99/1001/2026; conc 4 and 8 |
| **Segment rotation during concurrent collection** | `rotate_during` cells: seal threshold **from t0**; `SegmentRotate≥5` observed under load |
| Concurrent-phase barrier counters | snapshot immediately after worker joins (no sequential rotate epilogue) |
| file_sync / logical_ack | printed per cell; non-rotate cells show `file_sync < acks`; rotate cells report sync + rotate |
| Static ≡ Adaptive content for same seed | `q13_static_and_adaptive_same_seed_both_reopen_ok` (with concurrent rotate) |

## 2. Product defect closed (collector × seal)

**Symptom:** concurrent façade puts with low seal threshold failed with
`io: awo draining` (or hung in earlier drafts).

**Root cause:** `put_many_single_shard_batched[_bytes]` discarded
`WriteReceipt`s from mid-batch `finish_staged_batch_persist_publish` when
`maybe_auto_seal` fired. AWO `IndependentCollector` zips one receipt per
enqueued put; short vectors left `PendingPut` senders dropped → waiters saw
channel disconnect (mis-labeled as draining).

**Fix:**

- Accumulate receipts across seal boundaries in both batched put_many paths
  (`store.rs`).
- Fail-closed if install receipt count ≠ batch length (`collection.rs`).
- Distinguish completion-drop from runtime drain in `WriteCompletion::wait`.

## 3. Harness posture (no avoidance)

Earlier labor deferred seal to a sequential phase after concurrent collection.
That **avoided** the requirement. Current harness:

1. Low seal threshold active **before** concurrent writers start when
   `rotate_during`.
2. No sequential rotate epilogue mixed into barrier counters.
3. After reopen: full coverage-aware scan vs acknowledged ledger set.

## 4. Verify

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_q13_reopen_correctness \
  -- --test-threads=1 --nocapture
# 3 passed (labor host 2026-08-03 remediation)
```

Sample output (labor host re-verify 2026-08-03):

```text
q13 static seed=42 conc=4 acked=24 file_sync=23 rotate=5 sync/ack=0.958 scan=24
q13 static seed=99 conc=8 acked=32 file_sync=31 rotate=7 sync/ack=0.969 scan=32
q13 adaptive seed=42 conc=4 acked=24 file_sync=27 rotate=5 sync/ack=1.125 scan=24
q13 adaptive seed=1001 conc=8 acked=32 file_sync=33 rotate=7 sync/ack=1.031 scan=32
# same-seed Static≡Adaptive digest + scan count: ok
```

## 5. Explicit non-claims

- Crash-boundary / crash-matrix reopen (later)
- Thr ranking Off/Static/Adaptive product floors (optional residual / Q1 thr smoke)
- Package accept / default-on / Adaptive decision quality (Q2)
- PQH 120s qualification class
- Barrier amortization (`file_sync < acks`) under forced high-frequency rotate
  (seal FileSyncs dominate; non-rotate cells still prove amortization)

## 6. Q1 series posture (labor)

| Slice | Labor stage |
|-------|-------------|
| Q1.1 harness + ledger | `done` (principal) |
| Q1.2 façade wait-outside-mutex | `done` (principal) |
| Q1.3 reopen + concurrent rotate + exact scan | **principal `done`** |
| Umbrella AWO-Q1 | **principal `done`** — next **AWO-Q2** `todo` |