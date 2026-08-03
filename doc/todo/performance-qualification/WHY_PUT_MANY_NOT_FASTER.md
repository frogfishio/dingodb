# Why isn’t `put_many(N)` much faster than N × put(1)?

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“so why is put_many N slower than put 1 * N?”*

## First: the premise

**For Residiuum on our peer beds, `put_many(N)` is not slower than N singles.**
It is **about the same** (sometimes a touch faster).

| Bed | Mode A (N × put/batch=1) | Mode B (`put_many` 128) | B vs A |
|-----|-------------------------:|------------------------:|-------:|
| Scratch 2026-08-01 | 9 925/s | 10 221/s | **~1.03×** |
| APFS multicore FN | 13 192/s | 13 627/s | **~1.03×** |

So: not “batch is slower.” The surprise is **batch barely helps Residiuum**,
while it **does** help SQLite a lot (APFS: 29.7k → 50.0k, ~**1.7×**).

If the question meant *“why isn’t `put_many(N)` faster?”* — that is the real
one. Answer below.

## What each shape does

```text
put 1 × N     =  N separate presents, each list length 1   (Mode A)
put_many(N)   =  one present with N keys in the list       (Mode B uses 128)
```

Same total keys. Different **packaging** of the API call.

## Why Residiuum A ≈ B (batch doesn’t unlock thr)

Buffered Mode B still pays **per key**:

1. **Cook each record** — envelope + Blake + frame (integrity tax)
2. **Index each subject** — dual-index work scales with keys, not with “one batch”
3. **Seal policy** — long 256 MiB peer still hits soft seals; batching doesn’t remove them
4. **One tail write per batch** — that *is* amortized, but on this bed it is a
   **small** fraction of wall vs cook/index/seal

SQLite Mode B wins by amortizing **commit/durability** across 128 inserts
(`BEGIN`…`COMMIT`). Residiuum Mode B’s Buffered ack rules are **not** that
COMMIT bargain — README-PEER-SQL already: *per-put Buffered ≠ multi-row COMMIT*;
*at 8 KiB, batch=1 ≈ batch=128*.

```text
SQLite:   many inserts, one COMMIT boundary     → big thr jump A→B
Residiuum: many keys, still ~per-key integrity  → A≈B on peer beds
```

Campaign E / PEER fairness background said the same: “forgot to batch OS writes”
is **not** Residiuum’s main story.

## When `put_many(N)` *does* help Residiuum

| Shape | Batch helps? |
|-------|----------------|
| PEER long Mode A/B (8 KiB, seals, ~256 MiB) | **Barely** (~1.03×) — this table |
| Short micro, batch=128, parallel cook | **Yes** — cook1→cook4 ~1.8× (PARKED micro; different bed) |
| Parallel cook with batch=1 | **No** — nothing to split ([WHAT_BATCH_1_MEANS.md](WHAT_BATCH_1_MEANS.md)) |

So batch depth matters for **multicore cook micros**, not for “Mode B will look
like SQLite txn-128” on the long peer.

## Short answers

| Question | Answer |
|----------|--------|
| Is `put_many(N)` slower than N×put? | **No** (Residiuum peer) — ≈ same |
| Why doesn’t it soar like SQLite B? | We still pay Blake/frame/index per key; we don’t amortize a SQLite-style COMMIT |
| Where did “batch=1” fit? | Mode A’s packaging; explains multicore no-op, not “batch is slow” |

## Non-claims

Not a product txn API design. Not “never batch.” Not AWO. Diagnostic peer only.
