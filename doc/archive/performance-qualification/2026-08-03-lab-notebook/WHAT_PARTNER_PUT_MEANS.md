# What “partner” meant (collector jargon)

Status: **labor explainer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“what partner? what are you talking about?”*

## One sentence

**“Partner” = a second (or third…) put sitting in the AWO collector at the same
time** — another key to flush together. Not a person, not SQLite, not another
machine.

## Plain picture

When Static/Adaptive take an independent single put, the collector does roughly:

```text
1. Accept key A into a small queue
2. Wait a short time (collection delay, ~250µs) hoping more keys show up
3. Flush whatever is in the queue (ideally A+B+C… as one microbatch)
```

In step 2, “partner” was casual language for **those other keys** — e.g. key B
arriving while A is still waiting.

```text
Good microbatch moment:

  queue: [A, B, C]  ← B and C are the “partners” for A
  flush once

FN-2 Mode A (QD=1):

  queue: [A]        ← nothing else is allowed in yet
  wait…
  flush just A
  then later: [B] alone, etc.
```

Under QD=1 the client **must not** send B until A is fully acked. So while A
waits in the collector, B cannot exist yet. The delay runs; the queue stays
size 1; we flush alone. That is all “no partner showed up” meant.

## Better words (use these)

| Avoid | Prefer |
|-------|--------|
| partner put | **second in-flight key** / **another queued put** / **pile-up** |
| waiting for a partner | **waiting for queue depth ≥ 2 before flush** |

## Related

- [WHY_CANT_WE_MICROBATCH.md](WHY_CANT_WE_MICROBATCH.md)
- [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md)
