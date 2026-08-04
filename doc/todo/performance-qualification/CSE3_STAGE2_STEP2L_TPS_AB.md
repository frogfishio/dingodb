# CSE-3 Stage 2l — A/B fresh-default vs activate TPS delta

Status: **principal-accepted / closed** (2026-08-04).  
Measurement answered its question. **Did not reopen** Stage 2k flip.  
No further sealing or Chimera work is required for correctness from this stage.

## Accepted conclusions

- **Fresh CompactShadow** is the correct product path.
- Historical **~30.9K** activated-path result was **not reproducible** under fair A/B.
- Existing-store activation carries additional reopen cost, apparently from
  **retained pre-flip Materialized** recovery media.
- That cost does **not** invalidate the fresh-store default or its safety guarantees.

## Product SoT (qualified)

**~21–23K** sustained 8 KiB life=ack is the **controlled campaign result**
(Compact Chimera, full P★ Recovery Shadow, exact recovery, lifecycle ≈ ack).

It is **not** a universal hardware-independent floor: absolute A/B arms ranged
**~13K–~27K** under host load. Quote 21–23K as the accepted campaign band, not
a portable minimum.

~30.9K remains an earlier **candidate** figure only.

## Non-blocking migration residual

> Activated legacy stores retain Materialized recovery media for rollback and
> may reopen more slowly than native CompactShadow stores.

**Do not optimize immediately.** If addressed later, the safe approach is to
mark retained Materialized files as **rollback-only** so ordinary open/query
paths ignore them while preserving non-destructive rollback.

## Method

Same release binary (`cse3_stage2_step2l_tps_ab`, `--release`):

| Arm | Setup (outside timed window) |
|-----|------------------------------|
| **A** fresh-default | `Store::create_with_shards` → CompactShadow; seed; seal; reopen |
| **B** activate | `create_with_shards_mode(Materialized)` → seed/seal → prepare → activate → reopen → seed; seal; reopen |

Timed window: Buffered 8 KiB puts, seal every 64 MiB, soft auto-seal threshold,
enrichment on, `wait_seals_applied` before stopping the wall. No optimization.

```text
CSE3_STEP2L_TARGET_BYTES=2147483648 CSE3_STEP2L_WORK=/tmp/cse3-2l-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step2l_tps_ab step2l_tps_ab -- --nocapture
```

## Results (2 GiB)

| Run | A life TPS | B life TPS | B/A | A reopen (s) | B reopen (s) | A seal tot (s) | B seal tot (s) |
|-----|------------|------------|-----|--------------|--------------|----------------|----------------|
| 1 | 27018 | 17166 | **0.635** | 0.472 | 0.974 | 2.182 | 2.917 |
| 2 | 13248 | 8373 | **0.632** | 0.966 | 2.167 | 2.648 | 3.882 |

256 MiB smoke: A=29011, B=24351, B/A=0.839 (same direction).

**Config at timed window (both arms, both runs):** `CompactShadow`,
`shadow_dual_stream=true`, enrichment on, identical soft seal threshold,
same dual published count (33 @ 2 GiB).

## Measurement verdict (pre-accept)

1. Activate is **not** faster than fresh-default (B/A ≈ 0.63 @ 2 GiB).
2. Absolute TPS swings with host contention (~13K–~27K on A).
3. B pays more **`reopen_active`** (≈2×); Shadow dual finalize similar.
4. “Fresh-default reopen/dual-attach explains ~30.9K→21–23K” is **falsified**.

## Non-goals honored

- Product default unchanged; flip acceptance stands.
- ~30.9K not promoted.
- No optimization during or after the experiment for this residual.
