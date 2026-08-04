# CompactShadow default — migration & protection-frontier ops notes

Status: **Stage 2k product default** (2026-08-04) — principal-accepted.  
Fresh `Store::create` / `create_with_shards` persist `RMODE001=compact_shadow`.

## Product performance

**~21–23K** sustained 8 KiB writes/sec (life=ack), Compact Chimera + Recovery
Shadow, no growing deferred-work debt. ≈164–180 MiB/s; ≈1.7–1.9× Materialized
~12.4K. Do not use ~30.9K as the product figure (candidate-only; see Stage 2l).

## Operator invariants

1. **Fresh stores** start in CompactShadow: dual-stream Shadow + Compact Chimera;
   reclaim policy `RequireReplacementShadow`.
2. **Existing stores** without `recovery/mode.v1` remain **Materialized** on open —
   there is **no silent flip**. Migrate with the Step 8 ceremony.
3. **Migration (legacy Materialized → CompactShadow):**
   - `prepare_flip_to_compact_shadow` — Transitioning; backfill `.rsh`; gap-free check
   - `activate_compact_shadow_mode` — durable CompactShadow marker
   - Retain Materialized `.cmr` until post-reopen P★ verification
4. **Rollback:** `rollback_to_materialized_mode` keeps Shadows and Materialized files.
5. **P★:** advance `protected_frontier` only after both auth `.residiuum` and verified
   `.rsh` are durable (seal-pair pipeline + crash recover).

## Protection frontier

- Gap-aware, per-shard; aggregate claim is **min** prefix.
- Partial pairs recover via `recover_protected_pairs` (+ shard sidecar).
- Incomplete / corrupt `.rsh` never claims protection.

## Evidence commands

```bash
cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_shadow_f0_f5 -- --test-threads=1

bash ./scripts/verify-cse3-rshd0004-matrix.sh
cargo test -p residiuum-store --features legacy-raw-store --lib protected_pair
bash ./scripts/verify-cse3-segment-id-never-reuse.sh

cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_default_flip_ceremony -- --test-threads=1

CSE3_STEP9_TARGET_BYTES=2147483648 CSE3_STEP9_WORK=/tmp/cse3-step9-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step9_product_campaign step9_product_campaign -- --nocapture
```

## Related

- Flip ceremony: [`CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md`](./CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md)
- Prior flip package: [`CSE3_STAGE2_STEP2J_FLIP_PACKAGE.md`](./CSE3_STAGE2_STEP2J_FLIP_PACKAGE.md)
