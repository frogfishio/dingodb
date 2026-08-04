# CSE-3 Stage 2j — CompactShadow flip package (principal gate)

Status: **superseded by Stage 2k default flip** (2026-08-04) —
[`CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md`](./CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md).
This package prepared the gate; 2k executed the ceremony.

## Why this package exists

Stage 2i overlapped auth+Shadow finalize. Step 9 at 2 GiB release now shows:

- **ack = life ≈ 30.9K** 8 KiB TPS (was ~12.4K with serialized seals)
- All Step 9 product gates PASS (`gates_pass=true`)
- RSHD0004 + segment-ID never-reuse matrices green

That meets the historical “~28K band / life≈ack” bar for a **candidate**
CompactShadow product path. Flipping the universal default is still a
**principal** decision (LAWS: code ≠ accept).

## Evidence checklist (for principal)

| Item | Evidence |
|---|---|
| Seal-pair pipeline | [`CSE3_STAGE2_STEP2I_SEAL_PAIR_PIPELINE.md`](./CSE3_STAGE2_STEP2I_SEAL_PAIR_PIPELINE.md) |
| Step 9 campaign | [`CSE3_STAGE2_STEP9_PRODUCT_CAMPAIGN.md`](./CSE3_STAGE2_STEP9_PRODUCT_CAMPAIGN.md) |
| 2 GiB release run | `ack=30960 life=30960 pub≈123 seg/s frontier/rsh/recovery OK` |
| RSHD0004 | `cargo test … cse3_stage2_rshd0004_matrix -- --test-threads=1` → 16/16 |
| SegID never-reuse | `cse3_stage2_segment_id_never_reuse` → 8/8 |
| Pair crash recover | `cargo test … --lib protected_pair` (incl. shard sidecar) |
| Qual activation | Step 8 ceremony still required per store |

## What principal flip would mean (not done here)

1. Change `Store::create` / open default recovery mode to CompactShadow **or**
   document operator opt-in as the only path (current).
2. Update scoreboard + MASTER language: Materialized no longer release default.
3. Re-run Step 9 + RSHD0004 on the flipped default path.
4. Archive under `doc/archive/performance-qualification/`.

## Labor delivered in 2j

- Shard sidecar (`*.rsh.dual.shard`) for protected-pair crash recover
- Step 9 / Stage 2 / scoreboard docs updated for pipeline PASS
- Flip package written; **default not flipped**

## Residual

- Principal accept / reject of CompactShadow as Residiuum proper
- Optional archive folder once principal accepts Step 9
- CSE-1 / CSE-2 backlog items remain separate (not this gate)
