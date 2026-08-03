# Why SQLite went ~10K → ~30K

Status: **labor answer (self_check) — not package accept / not new PEER campaign**  
Date: 2026-08-03  
Question: how did SQLite Mode A go from ~10 000 to ~30 000 acked puts/s?

## One line

```text
It didn’t. We changed the bed: Scratch exFAT (Samsung T3) → internal APFS /var/tmp.
```

Same host (`Kazoo.local`), same Mode A knobs (8 KiB, QD=1, 256 MiB logical,
WAL + `synchronous=NORMAL`), same peer-pump shape. **SQLite software did not
suddenly get 3× faster** — the volume did.

## Evidence table

| Field | Scratch PEER 2026-08-01 | FN-2 2026-08-03 |
|-------|-------------------------|-----------------|
| Path | `/Volumes/Scratch/TEST/...` | `/var/tmp/...` |
| Volume | Samsung T3 **exFAT** (`disk16s1`) | Internal **APFS** SSD (`/System/Volumes/Data`) |
| SQLite Mode A | **9 458**/s | **29 205**/s (~**3.1×**) |
| Residiuum Mode A off | **9 924**/s | **12 553**/s (~**1.26×**) |
| SQLite peak CPU | **16.5%** (IO-bound) | **78.1%** (CPU-bound) |
| Residiuum peak CPU | 77.4% | 68.5% |
| elapsed (SQLite) | 3.46 s | 1.12 s |
| keys / payload | 32768 × 8192 | same |

Sources: `doc/wip/status/surveys/scratch-sqlite-peer-20260801/sqlite-A.json`,
`artifacts/firm-numbers-fn2-mode-a-apfs/sqlite-A.json`, T6 Scratch = exFAT
Samsung T3 (`AWO_THREE_WAY_T6_INTERACTIVE.md`).

## Why the asymmetry matters

If SQLite had a code/build leap, Residiuum would not be the control. Both engines
moved to the faster volume; **only SQLite scaled ~3×**. Residiuum stays near
Blake + frame + dual-index + seal work even when the disk is cheap — so it only
picked up ~26%. That is the PEER-SQL “same bed” lesson: absolute SQLite numbers
are **volume-dominated**; Residiuum/SQLite **ratios** only mean something on one
named volume.

CPU shift seals it: Scratch SQLite sat at ~16% CPU waiting on exFAT/external I/O;
APFS `/var/tmp` lets WAL autocommit run until the process is mostly CPU.

**Residiuum reading (principal):** Scratch ~10k parity looked shared because
SQLite was disk-bound while we were **already CPU-bound** (~77% CPU at ~10k).
Fast disk → SQLite ~3×; we ~1.26× still ~CPU-hot. Confirmed:
[FAST_DISK_CPU_WALL.md](FAST_DISK_CPU_WALL.md).

## What did *not* cause 10K → 30K

| Hypothesis | Status |
|------------|--------|
| SQLite knobs changed (journal / sync) | **No** — both WAL + NORMAL |
| peer-pump Mode A recipe changed | **No** — same 8 KiB / QD=1 / 256 MiB |
| AWO / Adaptive | **N/A** — SQLite has no AWO |
| Different machine | **No** — both `Kazoo.local` |
| Marketing “SQLite got faster” | **Reject** — bed swap |

## How to read odometers going forward

| Number | Bed | Use |
|--------|-----|-----|
| ~10k / ~10k Residiuum≈SQLite | Scratch exFAT | PEER-SQL reportable Mode A parity |
| ~12.5k Residiuum / ~29k SQLite | APFS `/var/tmp` | FN-2 diagnostic only; **not** Scratch ratios |
| Adaptive ~2.5k | APFS FN-2 | Mode A QD=1 collection tax (separate finding) |

**Rule:** never stitch Scratch ~10k SQLite to APFS ~30k SQLite as a product
improvement. Re-mount Scratch and re-run if you need continuity with Campaign F.

## Non-claims

Not a new Scratch campaign. Not an SLO. Not “prefer `/var/tmp` for PEER.”
Scratch remains the PEER-SQL reportable volume when mounted.
