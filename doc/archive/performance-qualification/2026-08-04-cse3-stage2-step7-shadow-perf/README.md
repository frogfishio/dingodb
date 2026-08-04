# CSE-3 Stage 2 step 7 — Shadow+Compact perf (2026-08-04)

Status: **labor complete / performance gate OPEN** — harness + optimizations landed;
2 GiB/64 MiB campaign **does not yet clear ≥7 seg/s** on this host.  
**No product flip.** Step 8 remains blocked.

## Candidate config (harness only)

```text
Authoritative segments + Compact Chimera + Recovery Shadow − Materialized Chimera
```

## What landed

- `recovery_shadow::qualify` harness + `tests/cse3_stage2_step7_shadow_perf.rs`
- Stage timings: decode / encrypt / encode / write / sync / rename / dir sync /
  frontier / bytes / CPU / wall
- **RSHD0002** wire: put `record_hash = blake3(tag‖key‖gen‖body_hash)` so encode
  reuses scan-time body digests; **RSHD0001** still readable
- Atomic publish timing + `sync_data` option for large Shadow temps
- Step 6 matrix still **15/15** after wire bump

## 2 GiB / 64 MiB campaign (release)

Source: `campaign-2g.log`

| Metric | Result |
|---|---:|
| Shadow publication | **3.69 seg/s** (FAIL vs ≥7) |
| Backlog slope | **0.000** (PASS) |
| Compact amp | **0.75%** (PASS ≤5%) |
| Shadow amp | **100.9%** (PASS ≤130%) |
| Frontier gap-free | **PASS** |
| Verified `.rsh` | **PASS** |
| Recovery after auth+Compact delete | **PASS** |
| Ack ops/s | ~68.0K |
| Lifecycle ops/s | ~18.5K (~27% of ack) |

### Stage medians (2 GiB)

| Stage | Median | Range |
|---|---:|---:|
| source_read_decode | 38.3 ms | 35.3–55.4 |
| encode | 51.1 ms | 41.8–76.5 |
| sequential_write | 54.3 ms | 15.4–100.8 |
| file_sync | 42.1 ms | 23.1–234.5 |
| frontier_publish | 12.7 ms | 9.1–19.8 |
| dir_sync | 4.0 ms | 1.3–10.3 |
| wall (incl. Compact) | 223 ms | 149–452 |

Best quiet-disk 256 MiB cell (`campaign-256m-best.log`): **~6.33 seg/s**
(still short of 7; encode≈49 ms + write≈47 ms + sync≈32 ms dominate).

## Residual

≥7 seg/s @ 64 MiB with ~100% Shadow amplification is **blake3 + durable write/sync
bound** on this host. Further gains need a principal-approved wire/IO change
beyond RSHD0002, or a different seal class — **not** parallel workers and
**not** a product flip by assumption.

Step 8 blocked until the same campaign clears performance **and** P★ recovery.
