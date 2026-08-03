# Smart-mode X on the SQLite-comparable bed

Status: **labor answer** (not package accept)  
Card: `be92b967-4b6c-48ef-a1a0-2ef73c65ca3f`  
Date: 2026-08-03  

## Question

Given Residiuum ≈ SQLite ≈ **~10 000** completed acked writes/s on **1:1 / Mode A**,  
what is **X** for Residiuum in **“smart mode”** (Adaptive AWO)?

## Answer

**Unknown — not measured on that bed.**

There is **no** PEER Mode A (SQLite-comparable 1:1, Buffered long peer) campaign
with `awo_mode=adaptive` that reports completed acked writes/s.

So for the same odometer definition as the ~10k figure:

```text
X_smart_ModeA  =  (no data)
```

Do **not** invent `X ≈ 20 000` from T11’s ~2×. That 2× was **Durable** smoke
(~0.5k → ~1.1k acked writes/s), a different durability class and bed — not Mode A.

## Closest Adaptive absolute we do have (different question)

| Bed | Smart (Adaptive) completed acked writes/s | Same as Mode A 10k? |
|-----|------------------------------------------:|---------------------|
| T11 Durable saturated smoke | ~**1 100**/s (@ 16 KiB cell → thr~8.8 MiB/s) | **No** |
| PEER Mode A + Adaptive | — | **Not run** |

Adaptive remains **default-off** in product.

## Residual (next measure if principal wants X)

Run PEER Mode A knobs with Residiuum Adaptive on, same Scratch bed, report
**acked puts/s** only — then X exists. Until then, answer stays **unknown**.

**Why unknown:** campaign not run (peer-pump also lacks AWO plumb) — **not**
because Adaptive is incomplete. See
[WHY_SMART_MODE_A_UNMEASURED.md](WHY_SMART_MODE_A_UNMEASURED.md).

**Program to get firm X:** [FIRM_NUMBERS_GOALS.md](FIRM_NUMBERS_GOALS.md)
(FN-1 harness → FN-2 four-cell measure → FN-3 optimize bound).
