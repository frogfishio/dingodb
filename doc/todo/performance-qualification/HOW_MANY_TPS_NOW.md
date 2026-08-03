# How many TPS can we have now?

**Principal metric (locked):** **TPS only** = acked puts/s (`ops_per_sec`). See [TPS_ONLY.md](TPS_ONLY.md). Do not answer with component timings.

**Short answer:**

| | TPS |
|--|----:|
| Residiuum default (quiet) | **~12–14k** |
| Residiuum default (disk nearly full) | **~6.5–8k** |
| Residiuum watermark (opt-in) | **≈ default** — no proven TPS win |
| SQLite same recipe | **~25–30k** |

We are roughly **half of SQLite** when the disk is quiet. That is the whole plot.

---

## Labor-only detail (not the principal answer)

Do **not** lead principal replies with this section. TPS table above is SoT for “how fast are we.”

## What “now” means (three bands)

| Band | TPS (acked puts/s) | Status |
|------|-------------------:|--------|
| **Default Real** (Residiuum-off, grow-on-append) | **~10–14k** quiet · **~7–9k** noisy/full disk | What you get **today** without opt-in growth |
| **Opt-in watermark** (product API; not default) | **≈ Real** on honest end-to-end seal ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md)) | Shipped opt-in — space amp; thr win **unproven** after cheat correction |
| **Ceiling probes** (not durable / not honest product) | Discard **~120k** · overwrite **~100k** · full-zero pump **~48–51k** | Upper bounds only |

SQLite Mode A concurrent peer: **~25–30k** quiet · lower when disk is tight.

```text
Discard/overwrite ceilings     ~100–120k   ← not product
SQLite A                       ~25–30k     ← peer target band (quiet)
Residiuum Real (default)       ~10–14k     ← “we have now” (quiet)
Watermark (opt-in)             ≈ Real      ← API shipped; no honest 30k claim
AWO Adaptive c=8               ~13–14k     ≈ Real off (no thr win yet)
AWO Adaptive c=1 (FN-2)        ~2.5k       ← feed tax; ignore for multi-client
```

## Detail (same recipe family)

### Concurrent feed (`--concurrency 8`) — judge multi-client here

| Engine / sink | ops/s | Source |
|---------------|------:|--------|
| SQLite A | ~29 700 | [FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md) |
| Residiuum-off Real | ~13 200 (earlier quiet) · **~6.7–7.7k** on 93–95% full beds | concurrent feed / [TRY_WM_64MIB.md](TRY_WM_64MIB.md) / product-wm |
| Residiuum Adaptive | ~13 600 | concurrent feed |
| **Watermark** (product opt-in, honest seal) | **≈ Real on that bed** | [FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md), [TRY_WM_64MIB.md](TRY_WM_64MIB.md) — prior ~32k **withdrawn** |
| Full-zero diag | ~48 000 pump · ~25 000 E2E | [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md) |

Run-to-run Real varies (~7–14k depending on disk fullness); quote a **band**, not one integer. See [WHY_7K_VS_12K.md](WHY_7K_VS_12K.md).

### Embedded sync (`c=1`) — autocommit peer only

| Cell | ops/s |
|------|------:|
| SQLite A | ~29 200 |
| Residiuum-off | ~12 600 |
| Adaptive/Static | ~2 500 (delay tax — not a smart-mode ceiling) |

## Plain language

- **Have now (default product):** about **ten to fourteen thousand** acked Mode A puts per second on fast local disk with a concurrent client feed — still about **half of SQLite**.
- **Can have (opt-in, not default):** watermark API exists (`set_segment_growth_policy` / `--segment-growth watermark`) with ~512 MiB/active space amp — **honest thr ≈ Real so far**, not a proven SQLite-band unlock.
- **Cannot claim:** 50k/100k/120k as product TPS from full-zero / overwrite / Discard; prior diag watermark ~32k; watermark as default-on SLO.

## Non-claims

Not Scratch PEER accept. Not Durable. Not default-on AWO. Not Windows/Linux measured on product flag. Not that watermark is production-safe without CSQ/space disclosure. Policy is process-local (not sticky across reopen) until a config surface lands.
