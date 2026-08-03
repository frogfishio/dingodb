# Why smart Mode A X is unmeasured

Status: **labor answer** (not package accept)  
Card: `af57c3bd-3c83-4558-84cc-c8d84f5f721f`  
Date: 2026-08-03  

## Question

> Are we unable to measure smart-mode X on the SQLite-comparable bed because
> we haven’t completed the features yet?

## Answer

**No — not primarily.**

We **can** measure Adaptive today. T11 already ran `awo_mode=adaptive` and
reported completed-write rates (on a **Durable** smoke bed, ~1.1k/s — different
odometer).

What is missing for **Mode A smart X** is a **campaign we have not run**, not a
blocked feature:

| Blocker? | Status |
|----------|--------|
| Adaptive code path usable | **Yes** (T11 / Q1–Q2 labor) |
| PEER Mode A + Adaptive measured | **No — not run** |
| `residiuum-testrig` peer-pump AWO flag | **Absent** (harness plumb gap) |
| `residiuum-perf --awo-mode adaptive` | **Present** (alternate way to take X) |

Incomplete residuals (`select_plan` not in collector, default-off, Q3/Q4,
package accept) affect **how good** smart mode is and whether it is product-
default — they do **not** make “unable to measure.”

## One line

```text
Unknown X  =  we haven’t taken the Mode A + Adaptive odometer reading yet
           ≠  Adaptive unfinished so measurement impossible
```
