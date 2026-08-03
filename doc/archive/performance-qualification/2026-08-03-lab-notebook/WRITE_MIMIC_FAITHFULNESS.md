# Is write-mimic faithful? Is it the true write-transaction model?

**Date:** 2026-08-03  
**Ask:** “We have a program that mimics the writes. Does it do it faithfully or not? Is this the true model of what happens during the write transaction?”

## Direct answer

**No — not faithful to the write transaction. No — it is not the true model of a put.**

It is a **size-calibrated disk ceiling check** for one step: “if we only did N×~8440 B `seek`+`write_all` grow appends, how fast is this bed?”

That is still useful. It is **not** a replay of Residiuum’s put.

## What a Mode A Buffered put actually does (product)

Rough order (omitting failpoints / AWO):

```text
key/payload in
  → cook / Blake / encode frame into active segment buffer
  → dual-index publish (in-memory locators)          [every put]
  → collection / derived bookkeeping (rate-limited)
  → seek(durable_len) + write_all(frame bytes)       [FileWrite]
  → advance durable_len / discard through
  → ack
(+ end-of-run seal_active, possible persist_index_cache, …)
```

Peer c=8: many client threads, **one store mutex** → those steps are **serialized** on the writer.

## What write-mimic `data` actually does

```text
loop N:
  seek(off) + write_all(8440 fixed bytes of 0xA5)
  off += 8440
flush
```

No cook, no Blake, no frame layout, no dual-index, no collection, no seal pipeline, no store lock, no watermark runway, no real payload entropy. Byte count ≈ mean skip-index growth/op — **not** per-put frame length measured from the encoder.

## Faithfulness scorecard

| Aspect | Faithful? | Notes |
|--------|:---------:|-------|
| Op count (32 768) | **Yes** | Peer recipe |
| Mean data bytes/op (~8440) | **Approx** | skip-index store÷ops, not live frame_len |
| `seek` + `write_all` grow | **Yes (shape)** | Same syscall shape as FileWrite under GrowOnAppend |
| Cook / hash / encode | **No** | Deliberately omitted |
| In-memory index publish | **No** | Omitted (cheap in probe ~11 ms anyway) |
| Derived checkpoint / seal | **No** in `data`; crude optional in `atomic` | End 270 MiB blob ≠ real seal/index encoding |
| Per-op 64 B index append | **Hypothesis only** | Not what product does on disk each put |
| Concurrency / mutex | **No** | Single-threaded mimic |
| **Whole write transaction** | **No** | Ceiling for one I/O step |

## So what may we conclude from ~129k?

**Allowed:** On this bed, **those sized grow-appends alone** are ~129k put-shaped ops/s — so “disk can’t absorb ~85 MiB/s of this shape” is false.

**Not allowed:** “Residiuum’s write transaction is modeled at 129k and therefore the only bug is tradition.” The transaction includes everything mimic drops; Discard (~128k) already shows cook+index without Real grow is also ~129k. The gap to Real ~10k is about **how Residiuum does Real FileWrite + the rest of the path**, which mimic does not reproduce.

## One line

```text
Write-mimic ≈ faithful to “N grow appends of ~8440 B.”
It is not a faithful model of the write transaction.
Use it as a disk ceiling, not as a put simulator.
```
