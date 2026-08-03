# How many TPS can we have now?

**Date:** 2026-08-03 · Odometer = **acked puts/s** (Mode A · 8 KiB · APFS `/var/tmp` unless noted)  
**Short answer:** **Product path today ≈ 10–14k TPS.** Diagnostic watermark shows **~32k pump / ~28k E2E** is reachable on the same bed — **not shipped**.

## What “now” means (three bands)

| Band | TPS (acked puts/s) | Status |
|------|-------------------:|--------|
| **Shipped Real** (Residiuum-off, real segment append) | **~10–14k** | What you get **today** without diag sinks |
| **Diagnostic watermark** (`realpreallocwm`) | **~32k** pump · **~28k** E2E wall | Proven spike — **not product** |
| **Ceiling probes** (not durable / not honest product) | Discard **~120k** · overwrite **~100k** · full-zero pump **~48–51k** | Upper bounds only |

SQLite Mode A on the same concurrent bed: **~30k**.

```text
Discard/overwrite ceilings     ~100–120k   ← not product
Watermark (diag)               ~28–32k     ← “we can,” if we ship alloc+zero
SQLite A                       ~30k        ← peer target band
Residiuum Real (shipped)       ~10–14k     ← “we have now”
AWO Adaptive c=8               ~13–14k     ≈ Real off (no thr win yet)
AWO Adaptive c=1 (FN-2)        ~2.5k       ← feed tax; ignore for multi-client
```

## Detail (same recipe family)

### Concurrent feed (`--concurrency 8`) — judge multi-client here

| Engine / sink | ops/s | Source |
|---------------|------:|--------|
| SQLite A | ~29 700 | [FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md) |
| Residiuum-off Real | ~13 200 (earlier) · ~9k on later noisy runs | concurrent feed / watermark baseline |
| Residiuum Adaptive | ~13 600 | concurrent feed |
| **Watermark diag** | **~32 300** pump · **~28 500** E2E | [PREALLOC_WATERMARK_SPIKE.md](PREALLOC_WATERMARK_SPIKE.md) |
| Full-zero diag | ~48 000 pump · ~25 000 E2E | [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md) |

Run-to-run Real varies (~9–14k); quote a **band**, not one integer.

### Embedded sync (`c=1`) — autocommit peer only

| Cell | ops/s |
|------|------:|
| SQLite A | ~29 200 |
| Residiuum-off | ~12 600 |
| Adaptive/Static | ~2 500 (delay tax — not a smart-mode ceiling) |

## Plain language

- **Have now (product):** about **ten to fourteen thousand** acked Mode A puts per second on fast local disk with a concurrent client feed — still about **half of SQLite**.
- **Can have (evidence, not merged):** about **thirty thousand** (watermark E2E ≈ SQLite; pump a bit above) if we ship something like ahead-of-write physical zeroing — still diagnostic.
- **Cannot claim:** 50k/100k/120k as product TPS from full-zero / overwrite / Discard.

## Non-claims

Not Scratch PEER accept. Not Durable. Not default-on AWO. Not Windows/Linux measured. Not that watermark is production-safe (space, crash semantics, CSQ labels untouched only because this is a diag sink).
