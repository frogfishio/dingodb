# Evidence table — CSE-3 Stage 2 step 7

| Gate | Bound | Result |
|---|---|---|
| Shadow publication | ≥7 seg/s | **FAIL** (2 GiB: 3.69; best 256 MiB: 6.33) |
| Backlog slope after warm-up | ≤0 | **PASS** (0.000) |
| `protected_frontier` gap-free | no gaps | **PASS** |
| Lifecycle near ack | close | **OPEN** (~27% of ack @ 2 GiB; expected with ~100% amp post-drain) |
| Shadow amp | ≈100% + framing | **PASS** (100.9%) |
| Compact amp | ≤5% | **PASS** (0.75%) |
| Recovery after auth+Compact delete | ok | **PASS** |
| Verified `.rsh` for every protected | ok | **PASS** |

## Recipe

```bash
CSE3_STEP7_TARGET_BYTES=2147483648 \
CSE3_STEP7_WORK=/tmp/cse3-step7-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step7_shadow_perf step7_smoke_candidate_harness -- --nocapture
```

## Artifacts

- `campaign-2g.log`
- `campaign-256m-best.log`
- Unit: `cargo test -p residiuum-store --lib recovery_shadow`
- Step 6 regression: `cse3_stage2_shadow_f0_f5` 15/15
