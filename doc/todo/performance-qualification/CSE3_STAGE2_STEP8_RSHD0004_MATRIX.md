# CSE-3 Stage 2 step 8 — RSHD0004 CSE matrix + controlled flip

Status: **labor complete / in_review** (2026-08-04)  
Depends: Step 7 dual-stream evidence
([`CSE3_STAGE2_STEP7_SHADOW_PERF.md`](./CSE3_STAGE2_STEP7_SHADOW_PERF.md)).

## Honest product figure (Step 7 accept language)

| Metric | Value | Notes |
|---|---|---|
| Sustained 8 KiB ack TPS | **~28K** | Candidate product figure with full-copy protection |
| Logical payload | ~219 MiB/sec | 28K × 8 KiB |
| vs Materialized full-product | ~2.25× (~12.4K) | Same recovery information, faster representation |
| vs unsafe Compact-only | slower than ~37.9K | Integrity retained |
| Shadow finalize capacity | median **55.57** seg/s | **Not** database TPS — post dual-stream data writes |

## RSHD0004 CSE matrix (15 cells)

Harness:

```bash
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_rshd0004_matrix -- --test-threads=1
```

| # | Cell | Failpoint / scenario |
|---|---|---|
| 1 | `r4_f0_healthy_dual_stream_recovery` | Healthy dual-stream + auth wipe recovery |
| 2 | `r4_fail_shadow_append_no_p_star` | Auth wrote; Shadow append fails → poison; seal refuses P★ |
| 3 | `r4_fail_shadow_flush` | Staging buffer sync (`rshd4.shadow.flush`) |
| 4 | `r4_fail_finalize_summary` | Summary append |
| 5 | `r4_fail_finalize_sync` | Shadow staging `sync_all` |
| 6 | `r4_fail_finalize_rename` | Atomic rename |
| 7 | `r4_fail_finalize_dir_sync` | Parent directory sync |
| 8 | `r4_fail_frontier_publish` | Frontier publication after Shadow file |
| 9 | `r4_multi_shard_rotation` | Multi-shard dual-stream; per-shard gap-free |
| 10 | `r4_overwrite_tombstone_recovery` | Overwrites + tombstones |
| 11 | `r4_chunked_value_recovery` | Chunked frames mirrored in RSHD0004 image |
| 12 | `r4_step8_flip_activate_rollback` | prepare → activate → omit new Materialized → rollback keeps sources |
| 13 | `r4_step8_mode_persist_failpoint` | Marker persist crash boundary |
| 14 | `r4_step8_activate_requires_gap_free` | Activate refuses sealed-without-durable hole |
| 15 | `r4_step8_reopen_loads_marker` | Reopen loads CompactShadow marker |

Legacy F0–F5 suite (`cse3_stage2_shadow_f0_f5`) remains green (15/15).

## Step 8 controlled activation

1. Backfill / verify Shadows for sealed segments (`backfill_shadows_for_sealed`).
2. Require per-shard gap-free protected coverage (`protected_frontier_gap_free`).
3. Persist atomic `recovery/mode.v1` marker (`RMODE001`).
4. New seals: Compact + Shadow when mode is `CompactShadow`.
5. Stop new Materialized only after CompactShadow marker is durable.
6. Retain existing Materialized `.cmr` until post-switch verification.
7. Crash-test transition boundaries (mode persist + finalize failpoints).
8. Rollback → Materialized without deleting Shadows or Materialized files.

## Non-claims

- Labor does **not** mark scoreboard package `accept` without principal review.
- Aggregate `max(sealed)−min(prefix)` lag may be >0 under interleaved multi-shard
  seqs; gap-free is **per-shard** completeness.
- Chunk reassembly via `project_live` on ItemEvent bodies is a separate salvage
  path; cell 11 proves mirrored chunk frames in the RSHD0004 image.

## Evidence

Archive: [`…/2026-08-04-cse3-stage2-step8-rshd0004-matrix/`](../../archive/performance-qualification/2026-08-04-cse3-stage2-step8-rshd0004-matrix/)
