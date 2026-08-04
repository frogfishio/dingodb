# CSE-3 Stage 2 step 8 — RSHD0004 CSE matrix + flip labor (2026-08-04)

Status: **labor complete / awaiting principal review**

## Commands

```bash
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_rshd0004_matrix -- --test-threads=1
# → 16 passed (15 cells + charter meta)

cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_shadow_f0_f5 -- --test-threads=1
# → 15 passed
```

## Step 7 accept language (honest figures)

- Product figure: **~28K** sustained 8 KiB TPS with full-copy protection (~219 MiB/sec).
- ~2.25× prior Materialized full-product ~12.4K.
- Slower than unsafe Compact-only ~37.9K; recovery guarantee retained.
- **55.57 seg/s** = Shadow finalize capacity after dual-stream data writes — **not** DB TPS.

## Step 8 APIs

- `Store::prepare_flip_to_compact_shadow`
- `Store::activate_compact_shadow_mode`
- `Store::rollback_to_materialized_mode`
- Marker: `recovery/mode.v1` (`RMODE001`)

## Normative plan

[`CSE3_STAGE2_STEP8_RSHD0004_MATRIX.md`](../../../todo/performance-qualification/CSE3_STAGE2_STEP8_RSHD0004_MATRIX.md)
