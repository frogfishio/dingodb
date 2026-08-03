# Fast disk → Residiuum CPU wall (Mode A)

Status: **labor confirmation (self_check) — not package accept / not SLO**  
Date: 2026-08-03  
Principal: *“we are taking a massive CPU hit on /var/tmp i.e. it looked like
10K/s because the disk was the bottleneck. Fast disk → cpu is the bottleneck
for us.”*

## Verdict

**Yes — with one precision.**

The Scratch ~10k Mode A **parity** was mostly **SQLite waiting on disk** lining
up with **Residiuum already burning CPU**. On fast APFS `/var/tmp`, the disk
gets out of the way and Residiuum stays on a **CPU / cook wall** (~12.5k) while
SQLite runs away (~29k).

```text
Scratch exFAT:  SQLite IO-bound (~16% CPU) ≈ Residiuum CPU-bound (~77% CPU)  →  both ~10k
APFS /var/tmp:  disk cheap → SQLite ~29k (CPU ~78%); Residiuum ~12.5k (CPU ~68%)
```

So: **fast disk did not unlock a Residiuum thr cliff** — it **revealed** that our
Mode A ceiling was already CPU-shaped. SQLite’s 10k→30k is the disk escaping;
our ~1.26× move is the CPU staying put.

## Numbers (same Mode A knobs)

| Engine | Scratch ops/s | Scratch CPU | APFS ops/s | APFS CPU | Δ thr |
|--------|-------------:|------------:|-----------:|---------:|------:|
| SQLite | 9 458 | **16.5%** | 29 205 | **78.1%** | **~3.1×** |
| Residiuum-off | 9 924 | **77.4%** | 12 553 | **68.5%** | **~1.26×** |

Sources: Scratch peer JSON 2026-08-01; FN-2
`artifacts/firm-numbers-fn2-mode-a-apfs/`. Bed story:
[SQLITE_10K_TO_30K.md](SQLITE_10K_TO_30K.md).

## How to say it without lying

| Soft claim | Honest claim |
|------------|--------------|
| “We were disk-bound at 10k” | **SQLite** was disk-bound; **we** were already ~CPU-bound at the same ops/s |
| “Fast disk → CPU is our bottleneck” | **Yes** — APFS shows Residiuum Mode A thr barely moves while CPU stays high |
| “10k was fake” | **10k was real on Scratch** — but it was a **coincidence of two walls**, not our honest max on a fast peer bed |

## What the CPU wall is (already parked)

Mode A wall inventory (Scratch surveys / PARKED ladder): data cooking
(Blake + encode/frame), append path, seal policy — **not** “need a faster USB
disk.” Parallel cook helps batch-rich shapes; QD=1 Mode A still pays per-put
cook. See `doc/wip/status/surveys/PARKED-write-path-wall-20260801.md`.

FN-2 Adaptive ~2.5k is a **different** tax (QD=1 collection delay) — not this
CPU-wall story.

## Optimize implication (ties FN-3)

On a fast bed, beating SQLite Mode A means **cutting Residiuum CPU per acked
put** (cook/frame/index/seal shape under Buffered QD=1) — not more AWO
collection delay, and not assuming Scratch 10k parity was our physics floor.

## Non-claims

Not package accept. Not “disk never matters.” Not Scratch PEER accept of APFS
ratios. Scratch remains the reportable PEER volume when mounted.
