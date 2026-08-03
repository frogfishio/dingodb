# Principal charter — honest maximum performance

Status: **principal intent recorded** (labor evidence; not package accept)  
Card: `91453b0f-d719-494b-aab8-dab109e32a62`  
Date: 2026-08-03  

## Charter (verbatim intent)

> Do not cheat just to pump up numbers. Not vanity. Squeeze every last ounce of
> performance we can. If ~10K/s is the true max under honest product semantics,
> that is fine. If we can do better — all the way until every legitimate trick
> is exhausted — pursue that. Numbers yes, but **honest** numbers, without
> sacrificing consistency, stability, or other features.

**Yes — that makes sense.** It is the standing bar for AWO / PQH / peer work.

## Binding rules (labor must obey)

### Pursue

1. **True headroom** under the **named** durability / ack / consistency contract
   (Buffered, Durable, coverage grade, etc. — never silently weakened).
2. **Every legitimate trick** that preserves those contracts: Adaptive collection
   depth, cook/seal policy, write-through, parallel cook where seal-safe,
   presentation-fair batching from 1:1 admits, PQH attribution.
3. **Stop when residual is real physics or contract**, not when a vanity cell
   looks bad — document the wall (disk, Blake, seal, sync) with evidence.

### Forbid (vanity / cheat)

| Cheat | Why forbidden |
|-------|----------------|
| Label weaker semantics as Buffered/Durable | Contract lie |
| Harness `put_many(N)` presented as Adaptive 1:1 win | Presentation cheat (T7/T11 law) |
| Quote short no-seal micro as PEER long-peer floor | Band mixing (`AWO_10X_VS_2X_ACCOUNTING.md`) |
| Drop reopen / ledger / crash honesty for thr | Stability sacrificed |
| Soft-skip damage / incomplete scan for speed | Consistency sacrificed |
| Default-on without principal + evidence chain | Product claim cheat |

### Acceptable outcome language

```text
IF honest product path saturates at ~10k ops/s under stated mode+bed
  → report that as the floor/ceiling with disclosure; do not invent 120k
ELSE IF deeper k / seal / cook / Adaptive still moves thr under same contract
  → keep squeezing; each step needs claim → evidence → residual
```

10K is **not** declared “the max” today. It is the **PEER Mode A long-peer band**.
Higher bands exist under **different** beds (short Buffered, seals avoided).
Charter = find the **honest** max **per named contract+bed**, then exhaust
legitimate tricks inside that box — and only then raise the box via **named**
product features (e.g. explicit batch/txn API), never by silent weakening.

## Ties to existing law

- LAWS §12 claims need chain; §7 damage honesty; §15 smallest correct change  
- PQH: not marketing bench; no floors without disclosure  
- AWO T11: thr×2 only with sync/2 causal match; sparse negative control required  
- Campaign H three-band rule: do not mix ~10k / ~100–160k / ~330k  

## Non-claims

This note does not accept AWO/PQH packages, set product floors, or flip default-on.
