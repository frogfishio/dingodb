# Answer — what does grow-on-append get us apart from low performance?

**Date:** 2026-08-03  
**Card:** `49a35967`  
**Ask:** “but what does that get us apart from low performance?”

> **Correction:** Principal rejected the soft “buys” below. See
> [GROW_ON_APPEND_BUYS_RETRACT.md](GROW_ON_APPEND_BUYS_RETRACT.md).
> Honest summary: status-quo default + known thr tax; space/salvage lines were
> overclaimed. Next is a principal call on default-on pre-touch/watermark.

## Short answer (original — partly retracted)

~~**Space honesty, empty-store frugality, and a simpler salvage story.**~~  
**Mostly inertia.** Thr is the known tax. Speed is currently opt-in because we
never decided the space/thr trade as product law.

## What was claimed (now graded)

| Claim | Grade |
|-------|-------|
| Pay for bytes you wrote / no forced 512 MiB | **Weak** as a default justification (host-dependent) |
| Cheap create / many small stores | **Situational** — not proven binding |
| Salvage needs grow-on-append | **False as stated** — we can scan either way |
| Policy outside the model / opt-in speed | **True as process**, not a thr excuse |

## What you do **not** buy

- Mode A thr on fast disk (~10–14k quiet Real vs SQLite ~25–30k)  
- “SQLite parity” as a free default  
- A story where first-touch is free

See [FIFTY_TO_TEN.md](FIFTY_TO_TEN.md), [WHY_EXTEND_EACH_TIME.md](WHY_EXTEND_EACH_TIME.md),
[GROW_ON_APPEND_BUYS_RETRACT.md](GROW_ON_APPEND_BUYS_RETRACT.md).

## Blunt trade (still true)

```text
GrowOnAppend (default)     →  ~10k Real class (status quo)
Watermark / pre-touch      →  space amp + zero work; can lift put-path thr if honest
```

If the product priority is **beats SQLite on Mode A concurrent by default**,
grow-on-append should lose a principal decision — with space disclosure.
Do not dress the current default as deep virtue.

## Non-claims

Not that grow-on-append is sacred. Not that watermark is proven E2E faster after
the seal fix. Not package accept / default-on either way.
