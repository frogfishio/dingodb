# Evidence — ETQ-0 enrichment stage breakdown

| Claim | Oracle | Result |
|---|---|---|
| Per-segment stage timings recorded | `enrichment.stage_breakdown` n=32 | PASS |
| Dominant stage identified | ranked mean wall | **Chimera construct+persist** |
| Chimera capacity vs 5.8 seg/s | `service_capacity.chimera` | **2.56 FAIL** |
| Chimera capacity vs 7.0 seg/s | same | **2.56 FAIL** |
| BLAKE3 / Hydra / read+decode vs 7.0 | capacities | **PASS** (≥9) |
| Catalog apply vs floors | capacity ≫7 | **PASS** |
| Bytes read/written disclosed | mean bytes | ~64 MiB / ~63 MiB |
| CPU vs wall disclosed | `cpu_vs_wall_ratio` | ~0.72 |

```bash
cargo build -p residiuum-testrig --release
./target/release/residiuum-testrig ack-finalize \
  -w "$WORK" --cell real-full --target-bytes 2G \
  --payload-size 8192 --concurrency 8 --seed 42 \
  --seal-threshold 64M --min-free 512M --json-out
# omit --no-enrichment
```
