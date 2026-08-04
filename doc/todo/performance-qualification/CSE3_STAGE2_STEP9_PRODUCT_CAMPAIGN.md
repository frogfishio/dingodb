# CSE-3 Stage 2 step 9 — CompactShadow product-API campaign

Status: **accepted product path** — CompactShadow fresh-store default (Stage 2k).  
Official sustained figure: **~21–23K** 8 KiB life=ack — **controlled campaign
result**, not a hardware-independent floor (Stage 2l saw ~13K–~27K under load).  
Not the ~30.9K candidate. Stage **2l** principal-accepted / closed.  
Depends: [`CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md`](./CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md).

## Honest accounting

| Run | Ack | Lifecycle | life/ack | Role |
|---|---|---|---|---|
| Prior (seal-excluded ack) | 23278 | 11845 | 0.51 | Burst; rejected |
| After 2h (serialized seals) | **12372** | **12372** | **1.00** | Materialized-class prior |
| 2i activate-path 2 GiB | **30960** | **30960** | **1.00** | Candidate only |
| **2k fresh-default 2 GiB** | **~21–23K** | **same** | **1.00** | **Campaign product figure** |

Product implications: ≈164–180 MiB/s @ 8 KiB; ≈1.7–1.9× the ~12.4K Materialized
product; ack≈lifecycle so the work is not borrowed.

Ops notes: [`CSE3_COMPACTSHADOW_OPS_NOTES.md`](./CSE3_COMPACTSHADOW_OPS_NOTES.md).  
A/B delta (candidate vs product): Stage **2l** **principal-accepted** — activate
not faster than fresh (B/A≈0.63); migration residual non-blocking; see
[`CSE3_STAGE2_STEP2L_TPS_AB.md`](./CSE3_STAGE2_STEP2L_TPS_AB.md).

## Posture

- Fresh `Store::create` → **CompactShadow**.
- Legacy migrate via prepare → activate; no silent open flip.
- Step 9 campaign uses product create only (no manual activation).

## Step 8 qual activation ceremony

Harness: `tests/cse3_stage2_step8_qual_activate.rs`

Still the migrate path for legacy Materialized trees (not required for fresh
stores after 2k).

## Step 9 product API

Harness: `tests/cse3_stage2_step9_product_campaign.rs`

Fresh create is CompactShadow — no prepare/activate. Campaign waits
`wait_seals_applied` after the put loop so async pairs complete before TPS.

| Gate | Bound | 2 GiB product result |
|---|---|---|
| Ack + lifecycle TPS | lifecycle ≥ 80% of ack | **life=ack ≈21–23K** |
| Shadow frontier | gap-free; verified `.rsh` | PASS |
| Compact amp | ≤5% | ~1.2% |
| Shadow amp | ≤130% | ~100% |
| Reopen + query / P★ / continue | exact | PASS |

```bash
CSE3_STEP9_TARGET_BYTES=2147483648 CSE3_STEP9_WORK=/tmp/cse3-step9-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step9_product_campaign step9_product_campaign -- --nocapture
```

## CI housekeeping

```bash
bash ./scripts/verify-cse3-rshd0004-matrix.sh
```
