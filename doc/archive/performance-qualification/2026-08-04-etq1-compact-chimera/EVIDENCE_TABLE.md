# Evidence — ETQ-1 Compact Chimera Persistence

| Claim | Oracle | Result |
|---|---|---|
| Compact layout version 2 | `CHIMERA_LAYOUT_VERSION=2`, tag 6 `SegmentFrame` | PASS |
| Derived Chimera ≤5% auth | on-disk `.cmr` / sealed segments | **PASS** (~0.74%) |
| Locators resolve exact bodies | `get_via_chimera` + store seal tests | PASS |
| Fail-closed missing/mismatch | `segment_frame_fail_closed_*` | PASS |
| Rebuild from segments/index | `chimera_rebuild_after_wipe` | PASS |
| Materialized non-default | seal uses `build_compact_layout` | PASS |
| Chimera stage ≥7 seg/s | `service_capacity.chimera` | **PASS** (63.0) |
| Enrichment ≥7 seg/s | `completed_enrichment_ops_per_sec` | **FAIL** (4.93) |
| Backlog slope ≤0 | OLS during ack | **FAIL** (+0.64) |
| Lifecycle ≈ ack TPS | 37.9K vs 43.8K | PASS (~87%) |
| Reopen exact + index query | campaign JSON | PASS |

```bash
cargo build -p residiuum-testrig --release
./target/release/residiuum-testrig ack-finalize \
  -w "$WORK" --cell real-full --target-bytes 2G \
  --payload-size 8192 --concurrency 8 --seed 42 \
  --seal-threshold 64M --min-free 512M --json-out
# omit --no-enrichment
```
