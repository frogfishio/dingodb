# Evidence — Derived Catalog Checkpointing

| Claim | Bound | Oracle | Result |
|---|---|---|---|
| `catalog_apply` share of ack wall | &lt; 1% | ack-finalize `rotation_stages.pct_of_ack_wall.catalog_apply` | **0.005%** PASS |
| Reopen before pending checkpoint | exact gets | `reopen_exact_before_pending_catalog_checkpoint` | PASS |
| Reopen after catalog delete | exact gets | `reopen_exact_after_deleting_derived_catalogs` | PASS |
| Rotation catalog cost flat | 1024-avg &lt; 8× 32-avg | `catalog_apply_cost_flat_across_segment_counts` | PASS |
| Sustained 2 GiB @ 64 MiB | reopen exact; stages recorded | `sustained-2g-64m.json` | PASS (ack TPS 57 640) |

Commands:

```bash
cargo test -p residiuum-store --features legacy-raw-store --test derived_catalog_checkpoint
cargo build -p residiuum-testrig --release
./target/release/residiuum-testrig ack-finalize \
  --work "$WORK" --cell real-full --target-bytes 2G \
  --seal-threshold 64M --no-enrichment --json-out
```
