# Why can’t we microbatch? (FN-2 Mode A)

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“so why can’t we microbatch?”* (after FN-2 Static/Adaptive ~2.5k)

## One line

```text
We can microbatch — just not on this bed.
FN-2 Mode A is QD=1: wait for ack before the next put → collector never holds ≥2 keys.
```

**Principal sharpening:** this bed is **embedded sync feed**, not server async.
Judging AWO here is mixing apples and oranges —
[EMBEDDED_SYNC_VS_SERVER_ASYNC.md](EMBEDDED_SYNC_VS_SERVER_ASYNC.md).

## What microbatch needs

AWO independent collection coalesces **several in-flight single puts** into one
flush (or you present a multi-item `admit_put_batch` / `put_many`).

```text
Need either:
  (1) pile-up: put0 outstanding, put1 admitted before put0 acks  → queue depth ≥2
  (2) explicit multi-item present: put_many / admit_put_batch([k0..kN])
```

FN-2 Mode A PEER gives **neither**:

```text
put0 → wait until acked → put1 → wait until acked → …
         ▲
         only one key ever in the collector
```

That **QD=1** rule is intentional: fair peer vs SQLite autocommit (one present,
one ack). It is also exactly what starves microbatch.

## So we “can’t” only under that contract

| Claim | True? |
|-------|-------|
| AWO cannot microbatch at all | **False** — T11 Durable saturated: outstanding pile-up → ~2 acks/sync, thr×2 |
| FN-2 Mode A Static/Adaptive microbatched | **False** — delay then flush 1 → ~2.5k |
| PEER Mode A QD=1 forbids pile-up | **True** — by measurement design |
| Off ~12.6k is “we refused to batch” | **N/A** — off has no collector; it’s natural single puts on the CPU wall |

## Picture

```text
Microbatch works when time looks like:

  admit  admit  admit  …  flush(3)  ack ack ack
  |------overlap-------|

FN-2 Mode A looks like:

  admit → [collection delay] → flush(1) → ack → admit → …
           ▲
           no second in-flight key yet (QD=1); delay was pure tax
```

(“Partner put” = that second key in the queue — see
[WHAT_PARTNER_PUT_MEANS.md](WHAT_PARTNER_PUT_MEANS.md).)
## Ways we *could* microbatch (different question / bed)

| Lever | What changes | Still “Mode A”? |
|-------|----------------|-----------------|
| Raise outstanding / QD > 1 | Allows pile-up for collector | **No** — not PEER Mode A QD=1 |
| Client `put_many(N)` / Mode B | Explicit multi-item present | **No** — Mode B (and Residiuum A≈B on long peer anyway) |
| Skip/collect-delay when depth=1 | Avoid ~2.5k delay tax; still flush 1 | Still Mode A; doesn’t create batch thr, just stops the self-own |
| T11-style saturated Durable | Shows AWO batching works | Different durability / concurrency bed |

So: **product can microbatch under concurrency or multi-item presents.**  
**This odometer cannot**, without changing what Mode A means.

## Tie-back to the FN-2 numbers

| Cell | ops/s | Microbatch? |
|------|------:|-------------|
| SQLite A | ~29 200 | N/A (SQL autocommit) |
| Residiuum-off | ~12 600 | No AWO — CPU/cook wall |
| Static / Adaptive | ~2 460 | **Tried**, failed to form a batch — delay tax |

Fixing “why ~2.5k” ≠ inventing batch thr on QD=1. First hygiene: don’t wait
full collection delay when depth cannot exceed 1. Separate program: beds where
pile-up is allowed if we want AWO thr wins.

## Related

- [STATIC_IS_NOT_BATCHED_ON_FN2.md](STATIC_IS_NOT_BATCHED_ON_FN2.md)
- [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md)
- [WHAT_BATCH_1_MEANS.md](WHAT_BATCH_1_MEANS.md)
