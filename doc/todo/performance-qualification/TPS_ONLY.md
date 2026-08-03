# Principal lock — TPS is the only scoreboard

**Date:** 2026-08-03  
**Card:** `1a1595a8`  
**Steer (accepted):** Principal cares about **TPS only**. Component timings, “put-path vs E2E reinterpretations,” seal shares, and other meter archaeology as *the answer* are **noise**. Stop leading with them.

## The one number

**TPS** = peer-pump `ops_per_sec` = **acked puts per second** under a named recipe  
(Mode A · c=8 · 8 KiB · APFS `/var/tmp` unless we say otherwise).

That is the car’s speed. Everything else is garage talk — optional, labeled, never the headline.

## Where we are (TPS only)

| What | TPS |
|------|----:|
| Residiuum **default** (grow) — quiet bed | **~12 000–14 000** |
| Residiuum **default** (grow) — disk nearly full | **~6 500–8 000** |
| Residiuum **watermark** opt-in — same beds | **≈ default** (no honest TPS win yet) |
| SQLite peer (same recipe) — quiet / try bed | **~25 000–30 000** |
| Discard / overwrite ceilings | **not product TPS** — ignore for “how fast are we” |

**One sentence:** We are about **half of SQLite** on a quiet bed (~12–14k vs ~25–30k). Recent ~6.5k was the same default path on a full disk — not a new product mode.

## What labor must stop doing in answers

- Leading with “seal was in/out of the meter,” “component X ms,” “first-touch offline vs online” **as if that replaces TPS**  
- Presenting diag pump numbers (~50k) as “our TPS” without saying they are **not** the product default scoreboard  
- Inventing thirty alternative meanings of “transaction”

Diag ladders may still run **internally** to find levers. When talking to the principal: **TPS first, TPS only, then one next lever.**

## Next lever (still for TPS)

Raise **product** TPS toward SQLite’s band. Candidate: background runway preparer (still `todo`) — judged only by whether **TPS goes up** on the same peer recipe.

**Watermark opt-in so far:** failed that test ([WATERMARK_MADE_TPS_WORSE.md](WATERMARK_MADE_TPS_WORSE.md)) — ≈ default TPS, more disk.

## Non-claims

Not that ~12–14k is accepted as good enough. Not default-on watermark. Not that diag ~50k is the product number.
