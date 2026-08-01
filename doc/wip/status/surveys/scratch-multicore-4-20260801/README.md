# Multicore 4 — Axis B vs Axis C (Scratch, 2026-08-01)

**Diagnostic only.** Not a published SLO.  
Host: Scratch · payload 8 KiB · buffered · seal 64 MiB · seed 20260801.

## Question

Does “use 4 cores” change Residiuum write numbers?

## Axis B — one store, `--writer-shards 4` (single process)

Target on-disk **512 MiB**.

| Run | Shards | Batch | ops/s | “MiB/s” (pump) | Peak CPU% | Peak RSS |
|-----|-------:|------:|------:|---------------:|----------:|---------:|
| b1 | 1 | 1 | **10 171** | 162 | 64 | 453 MiB |
| b4 | **4** | 1 | **8 012** | 128 | **121** | 519 MiB |
| b1b | 1 | 128 | **10 604** | 169 | 69 | 357 MiB |
| b4b | **4** | 128 | **8 865** | 141 | 92 | 485 MiB |

**Read:** 4 shards **does not speed up** one store; it **slows** (~0.79–0.84×) while raising process CPU% above 100 (more than one core busy / contended). Same shape as Campaign C multi-shard ladder.

## Axis C — 4 independent store processes (true multi-process)

Fair harness: 4 parallel `pump`s, each **128 MiB** on-disk, **batch=1**, aggregate = sum(keys)/parent wall.

| Run | Stores | ops/s aggregate | vs 1×512 MiB |
|-----|-------:|----------------:|-------------:|
| c1 / b1 | 1 | ~10.2k | 1.0× |
| c4fair | **4** | **~14.6k** | **~1.44×** |

Per-child ~3.9k ops/s (short 128 MiB roots; more setup overhead than long single pump).  
**Not ~4×.** Independent stores scale better than shared-store shards, but this volume/path is not linear at 4.

## Peer-SQL note

`peer-pump` is still **single-threaded**. Multicore does not apply to Campaign F peer cells until the harness grows multi-writer / multi-process peer modes.

## Artifacts

`b1.json` `b4.json` `b1b.json` `b4b.json` `c4fair.json` (+ earlier `c4.json` harness default batch).
