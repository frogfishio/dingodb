# AWO-6 residual checklist

Status: **labor floor (inventory + wired cells) — not package accept; not G8**  
Date: 2026-08-02  
Card: `5a8f98d0-5f89-42b3-8f70-b12f2eeccc2f`

## Delivered

| Item | Evidence |
|---|---|
| Closed failpoint inventory partition (11 names) | `awo_crash_matrix` |
| Wired put_many Error cells (8) hit-proof + no publish | `each_wired_error_failpoint_aborts_without_index_publish` |
| Deferred reserve/cook cells named, unvisited on put_many | `deferred_cells_named_but_not_required_on_put_many` |
| Short-write poison cell | `short_write_cell_poisons_writer` |
| Adaptive admit uses `select_plan` when cold → natural 1 | `awo_adaptive_oracle` adaptive_mode test |

## Explicit non-claims

- **Not** multi-process kill/reopen campaign for every cell  
- **Not** PQH L3/L4 controlled qualification (E3 partial) — smoke never marks G8  
- **Not** Verus/TLA deepen beyond AWO-0 skeleton  
- **Not** package accept  

## Exit command

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_crash_matrix -- --test-threads=1
cargo test -p residiuum-store --features legacy-raw-store --test awo_adaptive_oracle -- --test-threads=1
```

## Next package

**AWO-6 deepen** multi-process cells + PQH when E3; or **AWO-7** productisation (principal for default-on).
