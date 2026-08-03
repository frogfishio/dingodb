# Sustained rotation — evidence table

2 GiB · 64 MiB seals · enrichment off · concurrency 8 · seed 42

| Metric | Value |
|---|---:|
| Ack TPS | 47759 |
| Sealed @ last ack | 32 |
| Rotations timed | 32 |
| Reopen exact | yes |
| Ack wall ms | 5489 |

| Rotation stage | ns | % ack wall |
|---|---:|---:|
| flush | 10416 | 0.00 |
| rename_pending | 3009334 | 0.05 |
| start_active | 2769998 | 0.05 |
| backpressure_wait | 2081 | 0.00 |
| auth_publish | 4060671 | 0.07 |
| catalog_apply | 763593795 | 13.91 |
| **total stages** | 773446295 | **14.09** |
