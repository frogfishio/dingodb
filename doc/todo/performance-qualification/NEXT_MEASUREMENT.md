# Next measurement — acknowledgement/finalisation split

Status: **measured — evidence in**  
`doc/todo/performance-qualification/artifacts/ack-finalize-20260804-043624/`  
Date: 2026-08-04

Harness: `residiuum-testrig ack-finalize` / `ack-finalize-matrix`  
(`crates/residiuum-testrig/src/ack_finalize.rs`). Measurement only — no storage
or AWO changes.

## Objective

Determine whether `~9.5K ops/s` is a hot acknowledgement limit or a lifecycle
number dominated by finalisation. Change measurement only; do not optimise
storage or tune AWO.

## Required output

Record `workload_start`, `last_successful_ack`, `drain_complete`,
`seal_complete`, `close_complete`, `reopen_complete`, and
`verification_complete`.

Emit `acknowledged_write_ops_per_sec`, `ack_elapsed_ns`, stage durations,
`lifecycle_elapsed_ns`, and `lifecycle_ops_per_sec`. Do **not** emit ambiguous
`ops_per_sec`.

## Fail-closed correctness

A run is invalid unless every issued operation has exactly one acknowledgement;
drain and seal succeed; close and ordinary reopen succeed; a coverage-aware scan
is complete; and reopened `(key, body_hash)` equals the acknowledged ledger.
Never discard `seal_active()`. Any failure returns non-zero and no throughput
verdict.

**Discard exception:** Discard never writes put bytes to media, so ordinary
reopen cannot reconstruct the ledger. The cell still requires successful
`seal_active()` and reports `reopen_exact=false` honestly (not a Real verdict).

## First matrix

```text
APFS · payload=8 KiB · logical_data=256 MiB · concurrency=8
Buffered · AWO=Disabled · seed=42
```

Run real full, real with index/derived publication disabled, discard full, and
raw write mimic. Clean work directories after evidence is written.

## Evidence (2026-08-04 APFS `/tmp`)

| Cell | Ack TPS | Ack time | Seal time | Lifecycle TPS | Reopen exact |
|---|---:|---:|---:|---:|---|
| Real full | 22595 | 1.45 s | 1.21 s | 7902 | yes |
| Real, indexing disabled | 26978 | 1.21 s | 1.09 s | 8761 | yes |
| Discard | 104188 | 0.31 s | 0.02 s | 84489 | no |
| Raw mimic | 222839 | 0.15 s | 0.01 s | 211973 | yes |

### Branch reading

- **Lifecycle ~7.9K** matches the prior apparent crisis (~9.5K wall).
- **Ack ~22.6K** (Real full) is **not** the ~10K floor and **not** ~100K.
- Closest package branch: **ack near ~30K, lifecycle ~10K → attack
  sealing / index finalisation** (seal ≈ 1.21 s vs ack window 1.45 s).
- Discard ack ~104K and raw mimic ~223K show the live media/cook path still
  has headroom below the raw disk ceiling; that is secondary until seal cost
  is addressed.

Pause AWO-Q2, watermark experiments, controller tuning, and further theory
docs until a seal/finalisation package is chosen from this table.
