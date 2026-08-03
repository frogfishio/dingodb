# AWO-2 residual checklist

Status: **labor floor delivered — not package accept**  
Date: 2026-08-02  
Card: `19bfe3f9-3000-4b65-9c66-851ea1c53917`

## Hang root cause (this turn)

The first cooker worker loop parked inactive workers on an unbounded
`Condvar` wait gated by `pop_while`. When `active_cookers == 0` (or
shutdown raced with a non-empty queue and zero actives), **no worker
drained the queue** and `PersistentCookerPool::shutdown` blocked forever
on `join`. Combined with `cargo test --lib adaptive_write::` (rebuilds the
entire store unit-test binary), runs looked “hung.”

**Fix:** timed park for inactive workers; on shutdown force-activate all
permits and drain with non-blocking pop; `Drop` joins workers; ready-ring
push has a bounded spin. Pool tests live only in `awo_credit_bounds` (not
under heavy `--lib` store suite).

## Delivered

| Item | Evidence |
|---|---|
| `credits.rs` byte/entry ledger + `mutation_credit` | `awo_credit_bounds` credit tests |
| `queue.rs` bounded Mutex/Condvar + timeouts | unit + credit enqueue path |
| `ordered_ready.rs` BTreeMap ticket ring | order tests |
| `cooker.rs` persistent pool, permits, pure encode | frame equivalence + parallel order |
| `policy.rs` machine defaults, mode disabled | policy test |
| Failpoint names `awo.cook.before/after` | hit sites in `cook_item_frame` |
| No Store/FD touch from cookers | pure `cook_item_frame` only |

## Exit commands

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_credit_bounds -- --test-threads=1
cargo test -p residiuum-store --features legacy-raw-store --test awo_persist_before_publish -- --test-threads=1
bash scripts/verify-awo.sh
```

## Explicit residuals (not closed)

1. **Coordinator as sole mutation authority** — AWO-3 Static Arbiter.
2. **Replace diagnostic `put_many` per-batch thread::scope** with the
   persistent pool on the product path — AWO-3.
3. **Credit return on all product completion paths** — needs coordinator.
4. **Package accept** — principal only.

## Next package

**AWO-3** — StoreHost / HeapStore admission, mode default disabled.
