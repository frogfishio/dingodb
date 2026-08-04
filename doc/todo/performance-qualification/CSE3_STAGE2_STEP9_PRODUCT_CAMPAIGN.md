# CSE-3 Stage 2 step 9 — CompactShadow product-API campaign

Status: **labor PASS as diagnostic / product-perf FAIL for default flip** (2026-08-04)  
Depends: Step 8 activation machinery
([`CSE3_STAGE2_STEP8_RSHD0004_MATRIX.md`](./CSE3_STAGE2_STEP8_RSHD0004_MATRIX.md)).

## Honest accounting (Stage 2h)

| Run | Ack | Lifecycle | life/ack | Notes |
|---|---|---|---|---|
| Prior (seal-excluded ack) | 23278 | 11845 | 0.51 | Burst; rejected |
| After 2h (same wall) | **12372** | **12372** | **1.00** | Sustainable ≈ Materialized class |

Default remains **Materialized**. See
[`CSE3_STAGE2_STEP2H_SEGID_LIFECYCLE.md`](./CSE3_STAGE2_STEP2H_SEGID_LIFECYCLE.md).

## Posture

- **Qual-store activation only** — not the universal release default.
- Until Step 9 passes, released/default product remains **Materialized**.
- Step 8 delivered safe switching; Step 9 decides whether CompactShadow
  becomes Residiuum proper.

## Step 8 qual activation ceremony

Harness: `tests/cse3_stage2_step8_qual_activate.rs`

1. Prepare Shadows for every sealed segment  
2. Per-shard gap-free protection coverage  
3. Existing Materialized `.cmr` intact  
4. Persist `RMODE001 = CompactShadow`  
5. Reopen — mode survives  
6. New seals: locator-only Compact + complete Shadows; no new Materialized payloads  
7. Rollback — neither recovery source deleted  

Fresh `Store::create` remains Materialized.

## Step 9 product API (no harness-only overrides)

Harness: `tests/cse3_stage2_step9_product_campaign.rs`

Mode is armed only via `prepare_flip_to_compact_shadow` /
`activate_compact_shadow_mode` / reopen — **not** `CSE3_STEP7_DUAL_STREAM` or
test-only `attach_shadow_dual_to_actives`.

| Gate | Bound |
|---|---|
| Ack + complete-lifecycle TPS | lifecycle ≥ 80% of ack (dual-stream folds Shadow into seal) |
| Shadow frontier | per-shard gap-free; verified `.rsh` |
| Compact amp | ≤5% of auth segment (locator-only) |
| Shadow amp | ≤130% of auth segment |
| Reopen + query | exact values |
| P★ recovery | auth + Compact delete → Shadow reconstruct |
| Restart + continue | reopen CompactShadow, write, seal, recover |

```bash
# Smoke
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step9_product_campaign -- --nocapture

# Full 2 GiB / 64 MiB
CSE3_STEP9_TARGET_BYTES=2147483648 CSE3_STEP9_WORK=/tmp/cse3-step9-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step9_product_campaign step9_product_campaign -- --nocapture
```

Expected candidate band ~**28K** 8 KiB TPS; Step 9 must establish it independently
of Step 7 dual-stream harness numbers.

## CI housekeeping

RSHD0004 failpoints are process-global. CI invokes:

```bash
bash ./scripts/verify-cse3-rshd0004-matrix.sh
# → cargo test … --test cse3_stage2_rshd0004_matrix -- --test-threads=1
```

## Product bugs fixed in this labor

- Seal path no longer double-increments `segment_seq` before `next_segment_id`
  (skipped ids / reopen collisions).
- Resume refuses an active whose segment id is already sealed (would overwrite
  `.residiuum` / `.rsh`).
- On resume / after seal, `segment_seq` is bumped to at least the active/sealed
  id seq. Active files are named `active.residiuum` (no seq in the filename), so
  open’s `max_segment_seq_from_paths` under-counted and the next mint reused the
  just-sealed id — overwriting large CompactShadow seals on the next
  `seal_active` (blocked ≥64 MiB continue / Step 9 scale).
