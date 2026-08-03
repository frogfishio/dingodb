# Why smart Mode A X is unmeasured

Status: **labor answer** (not package accept)  
Card: `af57c3bd-3c83-4558-84cc-c8d84f5f721f`  
Date: 2026-08-03  

## Question

> Are we unable to measure smart-mode X on the SQLite-comparable bed because
> we haven’t completed the features yet?

## Answer

**No — not primarily.** We **can** measure Adaptive; FN-2 **did** measure Mode A
smart X (≈2.5k acked puts/s on APFS `/var/tmp`) via peer-pump `--awo-mode`.

What *was* missing before FN-2 was a **campaign we had not run**, not a blocked
feature:

| Blocker? | Status |
|----------|--------|
| Adaptive code path usable | **Yes** (T11 / Q1–Q2 labor) |
| PEER Mode A + Adaptive measured | **Yes — FN-2** ([FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md)) |
| `residiuum-testrig` peer-pump AWO flag | **Present** (`--awo-mode`) |
| `residiuum-perf --awo-mode adaptive` | **Present** (alternate) |

Incomplete residuals (`select_plan` not in collector, default-off, Q3/Q4,
package accept) affect **how good** smart mode is and whether it is product-
default — they do **not** make “unable to measure.”

## One line

```text
Mode A smart X  ≈  2470/s on FN-2 APFS bed (loses to Residiuum-off ~12.5k)
Unknown was “campaign not run” — now run; Scratch re-run still open
```
