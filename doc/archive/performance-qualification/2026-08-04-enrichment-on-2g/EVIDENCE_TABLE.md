# Evidence — 2 GiB enrichment-on

| Claim | Bound / oracle | Result |
|---|---|---|
| Ack TPS recorded | `acknowledged_write_ops_per_sec` | **47 428** |
| Completed-enrichment throughput | `enrichment.completed_enrichment_ops_per_sec` | **1.61** jobs/s |
| Backlog slope | OLS `enrichment.backlog_slope_per_sec` | **+4.14** jobs/s |
| Peak / final backlog | peak / final_at_ack | **24 / 24** |
| Auth drain time | `drain_elapsed_ns` | **29.3 ms** |
| Enrich drain time | `enrichment_drain_elapsed_ns` | **14.94 s** |
| Complete-lifecycle TPS | ack + auth seal + enrich drain | **12 429** |
| Exact reopen | `coverage_scan` | **PASS** |
| Index/query verify | point-get endpoints + mid key | **PASS** |
| Digests drained | Known / Pending | **33 / 0** |

```bash
cargo build -p residiuum-testrig --release
./target/release/residiuum-testrig ack-finalize \
  -w "$WORK" --cell real-full --target-bytes 2G \
  --payload-size 8192 --concurrency 8 --seed 42 \
  --seal-threshold 64M --min-free 512M --json-out
# enrichment on = omit --no-enrichment
```
