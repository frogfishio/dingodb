# Next measurement — acknowledgement/finalisation split

Status: **ready for development — sole active performance package**  
Date: 2026-08-04

## Objective

Determine whether `~9.5K ops/s` is a hot acknowledgement limit or a lifecycle
number dominated by finalisation. Change measurement only; do not optimise
storage or tune AWO.

## Required output

Record `workload_start`, `last_successful_ack`, `drain_complete`,
`seal_complete`, `close_complete`, `reopen_complete`, and
`verification_complete`.

Emit `acknowledged_write_ops_per_sec`, `ack_elapsed_ns`, stage durations,
`lifecycle_elapsed_ns`, and `lifecycle_ops_per_sec`. Remove or rename the
ambiguous existing `ops_per_sec`.

## Fail-closed correctness

A run is invalid unless every issued operation has exactly one acknowledgement;
drain and seal succeed; close and ordinary reopen succeed; a coverage-aware scan
is complete; and reopened `(key, body_hash)` equals the acknowledged ledger.
Never discard `seal_active()`. Any failure returns non-zero and no throughput
verdict.

Snapshot boundaries at the last acknowledgement and after seal. Report logical
and encoded bytes, write/sync counts and durations, rotations, index/derived
counts, CPU/wall time, and bytes by file role.

## First matrix

```text
APFS · payload=8 KiB · logical_data=256 MiB · concurrency=8
Buffered · AWO=Disabled · seed=42
```

Run real full, real with index/derived publication disabled, discard full, and
raw write mimic. Clean work directories after evidence is written.

## Acceptance

| Cell | Ack TPS | Ack time | Seal time | Lifecycle TPS | Reopen exact |
|---|---:|---:|---:|---:|---|

The executable must answer whether the primary loss occurs before the last
acknowledgement or during finalisation. Only then select another optimisation.
