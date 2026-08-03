# What “batch = 1” means

Status: **labor explainer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“i don’t understand what do you mean batch = 1?”*

## One sentence

**Batch = 1 means the peer presents one key/value per store call** — write this
record, wait until it is acked, then present the next. Not “only one core,” and
not “only one byte.”

## Picture

Mode A (PEER “honest single put”):

```text
app:  put(k0) → ack → put(k1) → ack → put(k2) → ack → …
         │              │              │
         └─ each call’s item list has length 1  ← that is batch = 1
```

Mode B (PEER “bulk”):

```text
app:  put_many([k0..k127]) → ack-all → put_many([k128..k255]) → …
         │
         └─ each call’s item list has length 128  ← batch = 128
```

Same total keys over the run (e.g. 32 768). The difference is **how many keys
ride in one `put_many`**.

## Where it shows up in this repo

| Name | Value | Meaning |
|------|------:|---------|
| PEER Mode **A** | `batch_size() == 1` | Residiuum: one Buffered put per call; SQLite: one autocommit insert |
| PEER Mode **B** | `batch_size() == 128` | Residiuum: `put_many` of 128; SQLite: BEGIN…128 inserts…COMMIT |
| JSON field `put_batch_size` | 1 or 128 | Same number in peer-pump results |

Code: `PeerMode::A => 1`, `PeerMode::B => 128` in `crates/residiuum-testrig/src/peer.rs`.

## Why we said it about multicore

Parallel cook fans out **inside one `put_many`**: several records cook on
several cores at once, then install in order.

```text
put_many([item0, item1, item2, …]) + COOK_PARALLELISM=4
        → cook workers can split those items  ✓ needs ≥2 items in the list
```

```text
put_many([item0]) + COOK_PARALLELISM=4
        → only one item → nothing to split     ✗ batch = 1
```

Store gate (plain English): parallel cook runs only if cook workers > 1 **and**
the presented list has at least two items. Mode A always presents one →
`RESIDIUUM_COOK_PARALLELISM=4` does nothing useful. That is all “batch = 1”
was doing in the multicore writeup.

## What it is *not*

| Not this | Actually |
|----------|----------|
| “We only use one CPU forever” | Off Mode A still burns ~CPU on Blake/frame for each put, serially |
| “Payload size is 1” | Payload is still 8 KiB; batch is **count of keys per call** |
| “QD / concurrency” | QD=1 is “wait for ack before next present”; batch is “how many keys in that present.” Mode A uses both: batch=1 **and** QD=1 |
| “AWO Static/Adaptive” | Separate; those were about collection delay, not cook batch size |

## Tiny analogy

- **Batch = 1:** hand the kitchen one plate at a time; four chefs stand idle after the first plate.
- **Batch = 128:** hand 128 plates at once; four chefs can cook in parallel.

Multicore helps the second shape. Mode A is the first — on purpose — so it
matches “SQLite autocommit one row at a time.”

## Related

- Multicore results: [FIRM_NUMBERS_MULTICORE.md](FIRM_NUMBERS_MULTICORE.md)
- Mode A contract: `doc/wip/status/surveys/README-PEER-SQL.md`
