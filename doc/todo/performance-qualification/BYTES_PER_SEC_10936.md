# What is ~10 936 TPS in bytes/sec?

**Date:** 2026-08-03  
**Card:** `7cf2faee`  
**Source:** `artifacts/coalesce100k-apfs/coalesce100k.json`  
**Scoreboard lock:** TPS (`ops_per_sec`) stays primary; this is a unit translation only.

## Yes — already in the peer JSON

| Field | Value | Meaning |
|-------|------:|---------|
| `ops_per_sec` | **10 936** | acked puts/s (TPS) |
| `logical_bytes` | 268 435 456 | keys × 8 KiB = 256 MiB payload |
| `bytes_on_disk` | 547 157 640 | Residiuum store footprint after run (~522 MiB) |
| `elapsed_ms` | 2 996 | wall for the put timer |
| `mb_per_sec` | **~85.4** | **logical** MiB/s (`logical_bytes / 1024² / secs`) |
| `mb_per_sec_disk` | **~174.2** | **on-disk** MiB/s (`bytes_on_disk / 1024² / secs`) |

Raw SI:

| Meter | Bytes/s |
|-------|--------:|
| Logical payload | **~89.6 MB/s** (~85.4 MiB/s) |
| On-disk footprint / elapsed | **~183 MB/s** (~174 MiB/s) |

Also: `ops_per_sec × payload_size` ≈ **89.6 MB/s** logical (same as `mb_per_sec` in decimal).

## Caveats

- Field name is `mb_per_sec` but the harness divides by **1024²** → report as **MiB/s**.  
- **Logical** = user payload only. **Disk** includes frames/index/overhead (~2.0× amp here: 547 MiB disk / 256 MiB logical).  
- Disk MiB/s is **not** a pure `write(2)` syscall meter — it is end-of-run store size / wall. Coalesce may still write the same durable bytes; TPS is the locked scoreboard.

## Paired cells (same artifact dir)

| Cell | TPS | logical MiB/s | disk MiB/s |
|------|----:|--------------:|-----------:|
| Real | ~8 885 | ~69.4 | ~141.5 |
| Coalesce100k | ~10 936 | ~85.4 | ~174.2 |
| Discard | ~128 583 | ~1005 | ~0 (no real write) |
