# AWO-5 residual checklist

Status: **labor floor delivered — not package accept**  
Date: 2026-08-02  
Card: `2b57bc17-6c85-4bcf-bb05-017d1ece6dfd`

## Delivered

| Item | Evidence |
|---|---|
| EWMA `m',d'` /8, lower/upper ×3 | `estimator.rs` + unit/oracle tests |
| Warm 32 / stale 30s | `ServiceEstimator` + ManualClock |
| Payload power-of-two bucket | `payload_bucket` |
| Candidate set 1..1024 + queued | `selector::candidate_entry_counts` |
| Selection via pure `decide()` | `select_plan` (golden arithmetic authority) |
| Collection delay rules §11.5 | `collection_delay_ns` |
| Scale hysteresis + dwell | `ScaleController` + `ManualClock` |
| `AwoClock` (no wall sleep in tests) | `ManualClock` / `InstantClock` |
| Mode still disabled by default | policy test |
| Tests | `awo_adaptive_oracle` |

## Explicit residuals

1. ~~**Wire controller into product admit loop**~~ — **closed labor**: Adaptive `admit_put_batch` calls `select_plan`; cold → natural take-1; estimator observes batch service; scale evaluates simple cooker signals.
2. **Per-bucket multi-lane estimators** — single service estimator floor.
3. **controller_stability / falsification** suites (lying estimators).
4. **Richer utilisation telemetry** (not binary pending-based util).
5. **Package accept** — principal only.

## Exit command

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_adaptive_oracle -- --test-threads=1
```

## Next package

**AWO-6** — crash matrix + PQH qualification; or wire Adaptive mode to selector in product path.