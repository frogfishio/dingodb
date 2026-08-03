# FN-2 table: is Static “batched”?

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: pointing at FN-2 cells (SQLite ~29k / off ~12.6k / Static ~2.5k /
Adaptive ~2.5k) — *“I assume static = batched.”*

## Short answer

**Static means “AWO will try to coalesce / microbatch.”**  
On **this table**, Static did **not** successfully batch — it paid the
**wait-to-batch** tax and still flushed **one key at a time**. That is why it is
~2.5k, not faster than off ~12.6k.

```text
“Static = batched”     →  intent / capability when pile-up or multi-item presents exist
“Static on FN-2 Mode A” →  collector + QD=1 + batch=1 presents → no pile-up → delay, then flush 1
```

## The four cells (same Mode A knobs)

| Cell | ops/s | What it actually was |
|------|------:|----------------------|
| SQLite A | ~29 200 | One SQLite insert at a time (autocommit) |
| Residiuum-off | ~12 600 | Natural `put_many([1])` — **no AWO** |
| Residiuum Static | ~2 460 | AWO Static + `admit_put` + collector, **QD=1** |
| Residiuum Adaptive | ~2 470 | AWO Adaptive, same path as Static here |

All four are still **PEER Mode A packaging**: one logical key per present
(batch size 1). Static/Adaptive are **not** Mode B `put_many(128)`.

## What “Static” is (vs off)

| | Residiuum-off | Residiuum Static |
|--|---------------|------------------|
| Lease | Off | On (fences natural mutation) |
| Path (FN-2) | `put_many` of 1 | `independent_admit_put+collection` |
| Batching intent | None | Coalesce independent puts when several are in flight |
| What happened on FN-2 | Cook+ack one key, next | Enqueue one key, **wait ~collection delay**, flush one key, next |

Static’s batching needs **more than one put outstanding** (pile-up) or a
**multi-item** `admit_put_batch`. Mode A PEER is QD=1 + one key per present →
neither. So Static ≈ Adaptive ≈ “delay then single flush.”

T11 showed Static/Adaptive **can** batch (Durable, outstanding pile-up → ~2
acks/sync, thr×2). Different bed. FN-2 Mode A is the bed where that win does
not appear.

## Do not confuse three “batch” words

| Word | Means here |
|------|------------|
| PEER **batch size** (Mode A=1 / B=128) | Keys in one API present ([WHAT_BATCH_1_MEANS.md](WHAT_BATCH_1_MEANS.md)) |
| AWO **Static** “batched” | Optimiser tries to coalesce independent admits / fixed plan |
| SQLite **txn batch** (Mode B) | Many inserts, one COMMIT — not Residiuum Static |

Static ≠ Mode B. Static ≠ “we ran put_many(128).”  
Static on this table ≠ “we got a batched thr win.”

## One line

```text
Static = “allowed to batch” ≠ “batched on FN-2 Mode A”
FN-2 Static/Adaptive ~2.5k = delay tax, not a successful batch ceiling
```

**Why we can’t microbatch here:** Mode A QD=1 — see
[WHY_CANT_WE_MICROBATCH.md](WHY_CANT_WE_MICROBATCH.md).

## Related

- Delay tax: [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md)
- Numbers: [FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md)
- put_many vs singles: [WHY_PUT_MANY_NOT_FASTER.md](WHY_PUT_MANY_NOT_FASTER.md)
