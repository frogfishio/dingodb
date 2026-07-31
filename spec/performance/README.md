# Performance qualification registries (PQH-0)

Profile: `residiuum-performance-qualification-v1`

| File | Role |
|---|---|
| profiles-v1.json | Qualification profile |
| layers-v1.json | L0–L6 experiment ladder |
| stages-v1.json | Latency accounting stages |
| metrics-v1.json | Metric ids, units, required flags |
| verdicts-v1.json | Closed bottleneck verdict set |
| validity-v1.json | Run validity classifications |
| omission-reasons-v1.json | Matrix cell omission reasons |
| matrix-v1.json | Axes + seed required cells |
| schemas/ | Manifest/result/comparison JSON Schemas |
| fixtures/ | Accepted/rejected vectors |

Verify: `bash scripts/verify-performance-registry.sh`
