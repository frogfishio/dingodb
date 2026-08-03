# Smart-mode X on the SQLite-comparable bed

Status: **labor answer with FN-2 integers (not package accept)**  
Card: `be92b967-4b6c-48ef-a1a0-2ef73c65ca3f`  
Date: 2026-08-03  

## Question

Given Residiuum ≈ SQLite ≈ **~10 000** completed acked writes/s on **1:1 / Mode A**
(Scratch history), what is **X** for Residiuum in **“smart mode”** (Adaptive AWO)?

## Answer

**Measured (FN-2, APFS `/var/tmp`, Scratch not mounted):**

```text
X_smart_ModeA  ≈  2 470 acked puts/s
```

On the same run, Residiuum-off ≈ **12 600**/s and SQLite ≈ **29 200**/s.
Adaptive ≈ Static (~2.5k) and **loses** to Residiuum-off on Mode A QD=1
(`independent_admit_put+collection` — collection delay, no pile-up).

**Not** “CPU hammered harder than off”: Static/Adaptive peak CPU ~68–76% (same
band as off ~68%) while elapsed ~13 s vs off ~2.6 s. Delay tax, not hotter CPU.
See [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md).

**Feed label:** this X is **embedded-sync QD=1** (peer-pump waits ack before N+1).
Not the multi-user / server-async Adaptive odometer —
[EMBEDDED_SYNC_VS_SERVER_ASYNC.md](EMBEDDED_SYNC_VS_SERVER_ASYNC.md).

**Server-async Adaptive X (c=8, same bed):** ≈ **13 600**/s ≈ off — see
[FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md).

Full table + disclosure: [FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md).
Artifacts: `artifacts/firm-numbers-fn2-mode-a-apfs/`.

**Do not** invent `X ≈ 20 000` from T11’s ~2×. That 2× was **Durable** smoke
with outstanding pile-up — a different durability class and bed.

## Four-cell (this host, Mode A knobs)

| Cell | acked puts/s |
|------|-------------:|
| SQLite A | ~29 200 |
| Residiuum-off | ~12 600 |
| Residiuum Static | ~2 460 |
| Residiuum Adaptive (X) | ~**2 470** |

Scratch 2026-08-01 Mode A parity (~10k / ~10k) is a **different volume**; re-run
on Scratch for peer-ratio continuity. Directional signal (Adaptive ≪ off under
QD=1) is the firm smart-mode finding.

## Closest Adaptive absolute on other beds

| Bed | Smart (Adaptive) completed acked writes/s | Same as Mode A? |
|-----|------------------------------------------:|-----------------|
| T11 Durable saturated smoke | ~**1 100**/s (@ 16 KiB → thr~8.8 MiB/s) | **No** |
| Mode A QD=1 + Adaptive (FN-2) | ~**2 470**/s | **Yes (this measure)** |

Adaptive remains **default-off** in product.

## Optimize bound (FN-3 input)

Adaptive does **not** beat Residiuum-off on Mode A QD=1. Next residual is
collection delay under no pile-up — not random thr tuning. See FN-2 §5.
