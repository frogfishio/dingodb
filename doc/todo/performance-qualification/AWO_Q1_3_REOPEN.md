# AWO-Q1.3 — Concurrent admit correctness + reopen evidence freeze

Status: **labor complete (in_review) — not package accept**  
Card: `ba312673-ce84-4c89-ae1f-46c61caf112f`  
Date: 2026-08-03  
Parent: AWO-Q1 (`b6e2a138`) · Series: `AWO_QUALIFICATION_SERIES.md`

## 1. What this proves

| Claim | Evidence |
|-------|----------|
| Static + Adaptive concurrent **façade** callers | `awo_q13_reopen_correctness` matrices |
| Every ack binds seq → key + body hash + length | oracle `ExpectedOp` + pre/post get |
| Exactly-once ack (no lost / double-ack) | per-seq bool vector under concurrent writers |
| Clean close + **normal product reopen** | `drop(host)` → `StoreHost::open_with_adaptive_write` → re-open heap |
| Pre-close digest == reopened digest | blake3 chain over (seq, key, body_hash, len) |
| Multi-seed / concurrency matrix | seeds 1/7/42/99/1001/2026; conc 4 and 8 |
| Segment rotation across reopen | `rotate_after` cells: seal threshold + sequential large puts **after** concurrent phase; `SegmentRotate≥3` observed |
| file_sync / logical_ack | printed per cell; concurrent cells show amortization (`file_sync < acks`) |
| Static ≡ Adaptive content for same seed | `q13_static_and_adaptive_same_seed_both_reopen_ok` |

## 2. Hang residual (fixed this labor)

Earlier draft used a **tight spin** global outstanding pool while Durable AWO
`put_collection` blocked → multi-minute 300%+ CPU hang (processes killed).

**Fix:** outstanding = concurrent façade threads with **partitioned seq ranges**
(no spin-wait credits). Segment rotation runs **after** the concurrent phase
(single-threaded) to avoid seal × collector interaction under load.

## 3. Verify

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_q13_reopen_correctness \
  -- --test-threads=1 --nocapture
# 3 passed in ~3s (labor host 2026-08-03)
```

Sample output:

```text
q13 static seed=1 conc=4 acked=16 file_sync=9 rotate=0 sync/ack=0.562
q13 static seed=99 conc=8 acked=38 file_sync=29 rotate=3 sync/ack=0.763
q13 adaptive seed=1001 conc=8 acked=38 file_sync=29 rotate=3 sync/ack=0.763
```

## 4. Explicit non-claims

- Crash-boundary / crash-matrix reopen (later)
- Thr ranking Off/Static/Adaptive product floors (optional residual / Q1 thr smoke)
- Package accept / default-on / Adaptive decision quality (Q2)
- PQH 120s qualification class

## 5. Q1 series posture (labor)

| Slice | Labor stage |
|-------|-------------|
| Q1.1 harness + ledger | `in_review` (principal may `done`) |
| Q1.2 façade wait-outside-mutex | `in_review` (principal may `done`) |
| Q1.3 reopen correctness | **`in_review` this card** |
| Umbrella AWO-Q1 | still open until principal accepts 1.1–1.3 |
