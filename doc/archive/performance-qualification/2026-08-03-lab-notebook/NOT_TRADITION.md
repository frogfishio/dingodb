# Tradition call-out — are we just refusing to break with GrowOnAppend?

**Date:** 2026-08-03  
**Ask:** “so what you’re saying is: we append because that’s how it was always done and who are we to break with tradition?”

## Split the sentence

| Phrase | True? |
|--------|------:|
| We **append** (ordered frames at advancing head) | **Yes — keep.** That *is* the log. |
| We **grow EOF under the put** because tradition forbids otherwise | **No.** That is only the **default growth policy**, not sacred. |
| Who are we to break with it? | **We already did (opt-in).** Default not flipped because E2E TPS didn’t clearly win — not piety. |

So the sarcastic paraphrase is **half right**: keeping **`GrowOnAppend` as default** is largely **inertia + “watermark hasn’t proven ≫ grow on the peer meter yet.”** It is **not** “append logs must extend the file on every transaction.”

## What we are *not* defending

```text
❌  Grow-under-put is Residiuum identity
❌  Prealloc is immoral / anti-salvage
❌  Labor may not change the default without ancestral blessing
```

Principal already killed the morality play ([PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md](PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md)). Watermark + same-fd zero + bg preparer are **in tree**. Tradition is not the blocker; **missing TPS win + no default-on decision** are.

## What “append” still means after we stop growing under puts

```text
Same:   cook frame → write at durable_len → remember head → seal → next segment
Change: durable_len advances inside pre-zeroed capacity (or runway kept hot off-path)
        — not “seek past EOF and make the OS allocate on this put”
```

That breaks with **GrowOnAppend default**, not with **append-only segments**.

## Honest one-liner

```text
Yes — default grow-on-append is mostly “how it shipped / not flipped yet.”
No — we are not sworn to tradition. Append stays; growing under the put does not have to.
Flip when TPS evidence + principal say so — not when nostalgia says no.
```
