# Real /tmp disk bed — 2026-08-01

**Diagnostic only — not a published SLO.**  
**Parent (parked campaign):** [../PARKED-write-path-wall-20260801.md](../PARKED-write-path-wall-20260801.md)

Answers: “~330k × 8 KiB looked like multi‑GiB/s; T3 can’t do that — what happens on real APFS `/tmp` with ≥1 GiB?”

## Bed

| | |
|--|--|
| Volume | macOS APFS `/tmp` (system data volume) |
| Payload | 8 KiB |
| Budget | ≤5 GiB peak (cleaned between runs) |
| Binary | `residiuum-testrig` release |

## A) phase-bench — 1 GiB logical per phase (131 072 × 8 KiB)

Work: `/tmp/residiuum-real-tmp-20260801`. Seal for cook phases raised above payload so parallel install does not trip mid-batch rotate.

| Phase | ops/s | MiB/s (logical) |
|-------|------:|----------------:|
| raw `write_all` payload (no fsync) | ~209k | ~1630 |
| Buffered Mode A (batch=1, real file) | **~81k** | **~636** |
| Disk detached Buffered (Discard) | ~140k | ~1095 |
| **put_many cook1** | **~131k** | **~1025** |
| put_many cook2 | ~158k | ~1231 |
| put_many cook4 | ~116k | ~908 |

Read:

- **Not** Scratch 20k-op micro (~330k / ~2.5 GiB/s).
- Real-file vs Discard: Real ≈ **58%** of Discard → **I/O/page-cache matters** at 1 GiB.
- cook4 **does not win** vs cook1 here (0.89×); path is no longer pure cook-CPU bound.
- Logical ~0.6–1.2 GiB/s class on this SSD/cache — still **not** free media forever.

## B) peer-pump Mode B — 2 GiB logical, multi-seal

| Run | seal | cook | ops/s | logical MiB/s | disk | wall |
|-----|------|------|------:|--------------:|-----:|-----:|
| cook1 | 64 MiB (many seals) | 1 | **~10.2k** | **~80** | 4.10 GiB | 25.7 s |
| cook4 | 64 MiB | 4 | **fail** | — | — | mid-batch: *segment rotated mid parallel cook install* |
| cook4 | 4 GiB (no mid rotate) | 4 | ~7.9k | ~61 | 4.10 GiB | 33.3 s |

Read:

- **Multi-seal + ~2 GiB** lands near **~10k puts/s / ~80 MiB/s** logical on this box — same order as PEER-SQL Mode A long peer, **not** 300k.
- Parallel cook still **cannot rotate seal mid `put_many` batch** (product gap / known diagnostic limit).
- On-disk ≈ **2× logical** in this bed (~4.1 GiB for 2 GiB payload).

## vs earlier Scratch micro (20k ops, USB Scratch, seal 512 MiB)

| | Scratch micro cook4 | /tmp 1 GiB phase cook4 | /tmp 2 GiB peer multi-seal cook1 |
|--|--------------------:|-----------------------:|----------------------------------:|
| ops/s | ~330k | ~116k | ~10k |
| logical MiB/s | ~2576 | ~908 | ~80 |

Same software; **load length + seal + media** change the answer.

## Artifacts

- `phase-bench.txt`
- `peer-b-cook1.log`
- `peer-b-cook4.log` (fail)
- `peer-b-cook4-bigseal.log`

Harness notes: cook seal auto-sized in phase-bench; peer optional `RESIDIUUM_COOK_PARALLELISM`.