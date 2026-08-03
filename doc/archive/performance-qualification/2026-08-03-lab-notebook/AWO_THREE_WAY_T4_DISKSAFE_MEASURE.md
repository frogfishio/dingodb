# Measure AWO three-way — T4 first numbers (disk-safe slice)

Status: **labor complete (self_check) — DISK-SAFE FIRST SLICE only**  
Card: `f937e384-8216-448c-8bfd-aa51983f8625`  
Date: 2026-08-02  
Feature: Measure adaptive write batching (three-way fair run)

## Disk constraint (honesty)

A full T2 **diagnostic** campaign is **not runnable on this host right now**:

| Factor | Cost |
|--------|------|
| Diagnostic floor | **2 GiB logical bytes / cell** + 30s min |
| Campaign plan | 5 repetitions × 2 processes |
| Aborted mc1 diagnostic leftover | **~12–15 GiB** before cleanup (nearly filled the volume) |

Available free space after cleanup ≈ **32 GiB**. Full diagnostic × 3 modes would re-fill the disk.

**Therefore T4 this turn is a disk-safe smoke-class first slice**, not the T2 diagnostic freeze.

## Host

| Field | Value |
|-------|--------|
| OS | macOS 15.5 (Darwin 24.5.0 arm64) |
| CPU | Apple M4 · 10 cores |
| Binary | `target/release/residiuum-perf` (`--features store-driver`) |

## Slice knobs

| Knob | Value |
|------|--------|
| Class | **`smoke`** (op-cap; tiny disk) |
| Seed | `42` |
| max_cells | `1` (primary matrix cell after counterbalance) |
| Primary cell | `L4-durable-s16384-c1-o8-43` (payload 16 KiB, durable) |
| Modes | disabled · static · adaptive |
| Multiproc workers | `--no-spawn-workers` (in-process slots) |
| Store retention | **deleted after each mode**; only JSON evidence kept |

## Results (proxy metrics — DIAGNOSTIC/SMOKE ONLY)

**Do not** treat as product qualification, G8, or bottleneck marketing.  
Throughput fields are harness **`throughput_bytes_per_sec_proxy`** (smoke e2e proxy).

| Mode | validity | reopen | ack (med) | e2e proxy (ms, med) | thr proxy (MiB/s, med) | n runs |
|------|----------|--------|-----------|---------------------|------------------------|--------|
| disabled | valid | ok | 24 | ~106.3 | ~3.53 | 6 |
| static | valid | ok | 24 | ~96.0 | ~3.91 | 6 |
| adaptive | valid | ok | 24 | ~96.3 | ~3.90 | 6 |

Machine table: `artifacts/awo-three-way-t4-disksafe/summary.json`  
Campaign JSON (no stores): `artifacts/awo-three-way-t4-disksafe/campaigns/{disabled,static,adaptive}/`

### Reading the numbers (honest)

- Smoke is **op-capped** (~24 ops); absolute MiB/s is a **proxy under tiny work**, not a sustained window.
- Static and adaptive are **similar** and slightly faster than disabled on this micro-cell — **not** a claim that AWO wins; noise and smoke-scale dominate.
- Latency p50/p99 not emitted as separate fields on this result shape; e2e proxy used.

## Disk hygiene applied this turn

1. Killed leftover release `run`/`worker` processes from aborted diagnostic attempt.  
2. Removed `/tmp/awo-three-way-t4-seed42-mc1` (~12+ GiB).  
3. Each mode: run → copy JSON evidence → **rm -rf work dir**.  
4. Free space after: **~32 GiB** (was ~11 GiB before cleanup).

## Explicit residuals / non-claims

- **Not** T2 full matrix (18 comparison cells / max_cells=64).  
- **Not** diagnostic floors (2 GiB/cell).  
- **Not** qualification / controlled runner / product baseline.  
- **Not** ranking modes for product decisions.  
- Full diagnostic three-way needs a host with **ample free disk** (tens of GiB) or a future harness **disk-budget class**.

## Next

- **T5** — Honesty pass on these results (and residual catalog).  
- **T6** — Interactive re-run / stop condition (include disk free check).  
- Optional later: disk-budgeted diagnostic on larger volume.
