# Seal-threshold survey — diagnosis + Scratch campaigns

Diagnostic only — not a published SLO (see `doc/reference/operations/BENCHMARK_DISCLOSURE.md`).
Use **within-volume ratios**, not absolute product MiB/s claims.

## What was wrong with the first run

The first campaign (internal disk, `/private/tmp/residiuum-survey-yRCsAz`) is a **real signal that segment threshold matters**, but it is **not clean proof**:

| Issue | Evidence |
|-------|----------|
| **Disk pressure** | Free space fell to ~7 GiB / 97% full during multi‑GiB pumps; absolute rates and later runs are contaminated |
| **Order not counterbalanced** | Configs not run as 64→4→4→64; order effects mixed with threshold effects |
| **CPU / RSS missing** | Original manifests have `peak_rss_bytes` / `peak_cpu_pct` = null — process sampling did not land (older binary or failed `ps` samples). Current `residiuum-testrig` **does** sample successfully |
| **Target size** | 2 GiB on-disk × 6 stores ≈ heavy local pressure |

So: **~2× 64 MiB vs 4 MiB was overstated under near-full disk**, not invented.

## Issues that are *not* product store bugs

1. **Footprint ~2.05×** is consistent and expected (encoding + segment layout overhead vs logical payload). That is **directory growth / footprint**, not measured physical-write amplification.
2. **Four shards not beating single-shard 64 MiB** is real under the current single-process `put_many` path (see multi-shard ladder below); not a silent regression, but also **not** multi-process capacity.
3. **CPU/RSS “deficiency”** is a harness observability gap on the first run, not a store failure — fixed for Scratch runs with current testrig.

---

## Campaign A — Scratch 1 GiB seal counterbalance (baseline retest)

**Host:** `Kazoo.local` arm64 Darwin 24.5  
**Volume:** `/Volumes/Scratch/TEST/` (~153 GiB free throughout; internal root not used)  
**Binary:** `target/release/residiuum-testrig`  
**Pump:** target **1 GiB** on-disk, payload **8 KiB**, durability **buffered**, **1** writer shard  
**Order (counterbalanced):** `64M → 4M → 4M → 64M`  
**Artifacts:**  
- Scratch: `/Volumes/Scratch/TEST/residiuum-survey-results-20260731/`  
- Workspace: `doc/wip/status/surveys/scratch-20260731/`

| Run | Seal | Logical MiB/s | Disk-growth MiB/s | Footprint | Keys | Elapsed | Peak RSS | Peak CPU% |
|-----|------|-------------:|------------------:|----------:|-----:|--------:|---------:|----------:|
| s01 | 64 MiB | 65.2 | 133.6 | 2.05× | 73728 | 8.83 s | 531 MiB | 36% |
| s02 | 4 MiB | 43.2 | 88.7 | 2.05× | 65536 | 11.85 s | 196 MiB | 47% |
| s03 | 4 MiB | 43.4 | 89.1 | 2.05× | 65536 | 11.81 s | 225 MiB | 47% |
| s04 | 64 MiB | 65.7 | 134.7 | 2.05× | 73728 | 8.76 s | 468 MiB | 39% |

**Medians (logical):** 4 MiB **43.3**; 64 MiB **65.5**; ratio **1.51×** (64/4).

---

## Campaign B — Scratch 2 GiB seal counterbalance (scale check)

Same host/volume/binary; target **2 GiB** on-disk; same counterbalance order.  
**Artifacts:**  
- Scratch: `/Volumes/Scratch/TEST/residiuum-survey-2g-20260731/`  
- Workspace: `doc/wip/status/surveys/scratch-2g-20260731/`

| Run | Seal | Logical MiB/s | Disk-growth MiB/s | Footprint | Keys | Elapsed | Peak RSS | Peak CPU% |
|-----|------|-------------:|------------------:|----------:|-----:|--------:|---------:|----------:|
| s01 | 64 MiB | 67.2 | 137.8 | 2.05× | 139264 | 16.18 s | 540 MiB | 37% |
| s02 | 4 MiB | 43.4 | 89.2 | 2.05× | 131072 | 23.58 s | 280 MiB | 47% |
| s03 | 4 MiB | 43.6 | 89.5 | 2.05× | 131072 | 23.50 s | 287 MiB | 48% |
| s04 | 64 MiB | 66.2 | 135.6 | 2.05× | 139264 | 16.44 s | 524 MiB | 44% |

**Medians (logical):** 4 MiB **43.5**; 64 MiB **66.7**; ratio **1.53×** (64/4).

### Interpretation (A + B)

- Threshold effect is **stable across 1 GiB and 2 GiB** targets on Scratch (~**1.5×**).
- Pairs match tightly — order bias is small.
- **64 MiB** holds more live segment RSS; **4 MiB** spends more CPU% on seal/rotate.
- Internal contaminated ratio (~2.16×) remains **overstated**; do not use it for product claims.

---

## Campaign C — Scratch multi-shard ladder (Axis B, single process)

**Target:** 1 GiB on-disk · seal **64 MiB** · payload 8 KiB · buffered · seed 20260731  
**Shards:** 1 / 2 / 4 via `--writer-shards` (single process, `put_many` path)  
**Artifacts:**  
- Scratch: `/Volumes/Scratch/TEST/residiuum-survey-multishard-20260731/`  
- Workspace: `doc/wip/status/surveys/scratch-multishard-20260731/`

| Run | Shards | Logical MiB/s | Disk-growth MiB/s | Keys | Elapsed | Peak RSS | Peak CPU% | Writer model |
|-----|-------:|-------------:|------------------:|-----:|--------:|---------:|----------:|--------------|
| ms01 | 1 | 65.3 | 133.7 | 73728 | 8.83 s | 480 MiB | 44% | single_active_segment |
| ms02 | 2 | 61.9 | 126.2 | 81920 | 10.34 s | 607 MiB | 42% | sharded_active_segments |
| ms04 | 4 | 57.8 | 117.8 | 98304 | 13.29 s | 721 MiB | 76% | sharded_active_segments |

### Interpretation (multi-shard)

- Under this harness, **more writer shards does not increase** single-store logical throughput; 2- and 4-shard runs are **slower** than 1 shard while RSS and (at 4) CPU rise.
- This is **not** a multi-process capacity result (Axis C). Do **not** claim “4 shards = more store capacity” from these numbers.
- Key counts differ slightly (shard packing / seal cadence); compare rates, not raw keys alone.
- Further work if product wants multi-core write scaling: multi-process stores and/or concurrent producers — separate from this ladder.

---

## Harness: free-space refuse floor (landed)

`residiuum-testrig` now refuses pumps when free space is below a floor (prevents silent near-full contamination):

| Piece | Location |
|-------|----------|
| `free_space_bytes` / `default_min_free_for_target` / `ensure_free_space` | `crates/residiuum-testrig/src/size.rs` |
| Checked at `run_pump` start | `crates/residiuum-testrig/src/pump.rs` |
| CLI `--min-free auto\|SIZE\|0` (default **auto** ≈ 2.5× target + 512 MiB) | `crates/residiuum-testrig/src/main.rs` (`pump` + `run`) |
| Multi-process children pass `--min-free 0` (parent already checked total) | `pump.rs` child spawn |

Smoke: `--min-free 999G` → exit 1 with clear refuse message. Unit tests for parse + default floor pass.

---

## Comparison to contaminated internal medians

```text
Internal (2 GiB target, not counterbalanced, disk pressure):
  4 MiB median logical  ≈ 38.6 MiB/s
  64 MiB median logical ≈ 83.5 MiB/s
  ratio ≈ 2.16×

Scratch 1 GiB counterbalance (free space OK):
  4 MiB median logical  ≈ 43.3 MiB/s
  64 MiB median logical ≈ 65.5 MiB/s
  ratio ≈ 1.51×

Scratch 2 GiB counterbalance (free space OK):
  4 MiB median logical  ≈ 43.5 MiB/s
  64 MiB median logical ≈ 66.7 MiB/s
  ratio ≈ 1.53×
```

---

## Where the “issues” actually are

| Area | Status |
|------|--------|
| Store / PQH code from prior packages | Accepted; not implicated by disk-pressure noise |
| **Survey methodology** | Contaminated first run fixed by Scratch + counterbalance |
| **Testrig process sampling** | Works on current binary |
| **Free-space floor** | **Done** — pumps refuse when volume is too tight |
| **Seal default / threshold** | Product-relevant: ~**1.5×** clean 64 vs 4 MiB on Scratch |
| **Multi-shard capacity claim** | **Closed for single-process Axis B**: no throughput win at 2/4 shards; multi-process capacity still a separate question |

## Bottom line

There **was** a real problem with the **measurement environment** (full internal disk + incomplete order + missing process samples), not a silent “everything is broken in the store.”  
On Scratch with free space:

1. Segment threshold still matters (~**1.5×** at both 1 GiB and 2 GiB targets).  
2. Single-process multi-shard ladder does **not** improve logical throughput.  
3. Testrig now **refuses** near-full disks so this class of contamination is harder to repeat.

**Do not** publish absolute product MB/s from these diagnostics without Benchmark Disclosure + controlled runner class.
