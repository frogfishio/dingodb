# CSE-3 Stage 2 step 7 — write-time dual-stream Shadow (2026-08-04)

Status: **labor complete / experimental dual-stream PASS on sustained 2 GiB**  
**No product flip.** Materialized remains product recovery authority. Step 8 still
requires principal accept of the same campaign.

## Design (RSHD0004)

```text
Cook encoded frame once
 ├── append authoritative active
 └── append shadow staging (independent allocation; no reflink)
```

At seal: append the same summary to staging → `sync_all` → atomic publish →
advance `protected_frontier` only after Shadow publication.

Commitment: `blake3(store‖seg‖encoded_len‖ ordered(body_hash‖offset‖len‖gen))`
— payloads are not re-hashed; body hashes come from the cooked frame suffix.

Async rotate is disabled while dual-stream is armed (finalize is on the sync
seal path).

## Harness

```bash
CSE3_STEP7_DUAL_STREAM=1 CSE3_STEP7_REPEATS=3 \
CSE3_STEP7_TARGET_BYTES=2147483648 \
CSE3_STEP7_WORK=/tmp/cse3-dual-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step7_shadow_perf step7_smoke -- --nocapture
```

Shadow pub rate = dual-stream finalize service (summary+sync+rename+dir).
Lifecycle ≈ ack (Shadow folded into seal).

## 2 GiB / 64 MiB × 3 sustained runs

Source: `cse3-dual-2g.log`

| Run | Shadow pub | Ack TPS | Lifecycle | Amp | Recovery | Pass |
|---:|---:|---:|---:|---:|---|---|
| 0 | 63.48 | 27.7K | =ack | 100% | ok | **PASS** |
| 1 | 55.57 | 28.3K | =ack | 100% | ok | **PASS** |
| 2 | 37.69 | 27.8K | =ack | 100% | ok | **PASS** |
| **median** | **55.57** | — | — | — | — | **PASS ≥7** |

Frontier gap-free, verified `.rsh`, Compact ≤5%, backlog slope 0 — all PASS.
Ack TPS remains in the prior ~26–28K band (no unacceptable foreground regression
vs RSHD0003 post-seal campaigns on this host).

## Residual

- Experimental only — product seal still Materialized until step 8.
- Dual-stream forces sync auto-seal (no async rotate) while armed.
- Step 8 still needs principal accept of perf + P★ in the same campaign.
