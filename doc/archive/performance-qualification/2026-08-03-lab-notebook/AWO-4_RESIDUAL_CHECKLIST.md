# AWO-4 residual checklist

Status: **labor floor delivered — not package accept**  
Date: 2026-08-02  
Card: `3c4dbaf2-9fb1-49c5-8c71-702e5617da6c`

## Delivered

| Item | Evidence |
|---|---|
| `PipelineCoordinator` depth limit (1..=4, default 2) | `adaptive_write/coordinator.rs` |
| No third unresolved reservation | `try_begin_reservation` → `DepthExceeded` |
| Cook → Install phase ledger | `note_cook_complete` / `note_install_complete` |
| Seal fence | `begin_seal` / `end_seal` |
| Bounded shutdown | `begin_shutdown` + `wait_empty` + drain timeout |
| Wired into Static/Adaptive runtime | `AdaptiveWriteHandle::pipeline`, status.pipeline |
| `admit_put_batch` takes one pipeline slot | refuse when depth full |
| `drain_writes` begins pipeline shutdown | status.shutting_down |
| Tests | `awo_shutdown` |

## Explicit residuals

1. **True async overlap** — write A while cooker pool cooks reserved B as separate stages (today sync batch marks cook+install around one `put_many_awo_owned`).
2. **Per-shard coordinators** multi-writer.
3. **Seal integration** with `Store::seal` calling begin/end_seal automatically.
4. **PQH stage-overlap evidence** hooks.
5. **Package accept** — principal only.

## Exit command

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_shutdown -- --test-threads=1
```

## Next package

**AWO-5** — adaptive controller (EWMA, candidates, hysteresis); or deepen AWO-4 true cook/install overlap.
