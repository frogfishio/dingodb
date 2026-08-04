# CSE-3 Stage 2l — A/B fresh-default vs activate TPS delta

Status: **todo** (2026-08-04) — measurement only; **does not reopen** Stage 2k flip.

## Why

Product figure is **~21–23K** life=ack on fresh-default CompactShadow.
An earlier **activate-path** run printed ~30.9K. That candidate is not the
product number. Until A/B evidence exists, treat “reopen/dual-attach cost”
as a **hypothesis**.

## Method (narrow)

1. **Same** release binary / host.
2. Arm **A:** `Store::create` (fresh CompactShadow default).
3. Arm **B:** `create_with_shards_mode(..., Materialized)` → prepare → activate
   → reopen (manual CompactShadow).
4. Exclude create/seed/reopen from the acknowledgement window — start the wall
   after both arms are warm and ready to put.
5. Same payload (8 KiB), seal every 64 MiB, enrichment on, target ≥256 MiB
   (2 GiB preferred for parity with Step 9).
6. Record: ack TPS, life TPS, `SealStageBreakdown` fields, dual published count,
   mode flags (`recovery_mode`, `shadow_dual_stream`, enrichment).
7. Diff configs and call paths if timings diverge.

## Non-goals

- Do not change the product default.
- Do not re-litigate flip acceptance.
- Do not promote ~30.9K to product language.

## Done when

Reproducible table A vs B with stage timings + one sentence naming any real
path difference (or stating none found).
