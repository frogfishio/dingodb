# Try it — watermark @ 64 MiB default (paired)

**Date:** 2026-08-03  
**Card:** `300dcd54` — Try watermark@64MiB — paired measure vs grow/SQLite  
**Status:** labor `in_review` — **not** package accept / **not** default-on  

Principal: “lets try it” after locking configurable capacity (default 64 MiB).

## Recipe

Mode A · `--concurrency 8` · 8 KiB · 256 MiB logical · seed 42 · APFS `/var/tmp` · `min-free 0`  
Binary: `target/release/residiuum-testrig` (post 64 MiB default).  
Artifacts: [`artifacts/try-wm-64mib-apfs/`](artifacts/try-wm-64mib-apfs/).

Disk during run: ~95% full / ~12 GiB free — **largely self-inflicted** (uncleaned peer work dirs; principal later cleaned). See [OWN_DISK_FILL_CLEANUP.md](OWN_DISK_FILL_CLEANUP.md).

## Results (honest E2E — seal in meter)

| Cell | ops/s | elapsed_ms | bytes_on_disk | Notes |
|------|------:|-----------:|--------------:|-------|
| SQLite A | **~29 100** | 1124 | ~273 MiB | Stronger than prior noisy ~13k bed |
| Residiuum grow | ~6 700 | 4914 | ~522 MiB | Product default |
| Watermark **64/64**, seal **64 MiB** | ~6 500 | 5009 | ~586 MiB | Default knobs; seal within capacity |
| Watermark **64/64**, seal **512 MiB** | ~6 800 | 4832 | ~586 MiB | Hybrid after 64 MiB runway |
| Watermark **512/64**, seal **512 MiB** | ~7 500 | 4355 | ~1.0 GiB | Legacy large reserve; slight lift |

## Verdict

```text
Tried it. Product watermark @ 64 MiB ≈ grow on this E2E meter (~6.5–6.8k).
512 MiB capacity ≈ +12% vs grow here (~7.5k) — still ≪ SQLite (~29k).
Space amp real: 64 MiB cells ~+60 MiB vs grow; 512 MiB cell ~2× disk.
Not a SQLite-band unlock. Not default-on evidence.
```

## Why no miracle

E2E peer-pump still pays seal/chimera and any mid-run first-touch once past runway. Put-path pre-touch **can** lift thr in diag setups; this product-flag E2E try does not. Next lever remains **background runway preparer** (off put path) — still a separate `todo`.

## Non-claims

Not that 64 MiB is wrong as a default capacity (space honesty still favors it). Not Scratch/Linux. Not that SQLite ~29k replaces every prior cell. Not AWO/PQH accept.
