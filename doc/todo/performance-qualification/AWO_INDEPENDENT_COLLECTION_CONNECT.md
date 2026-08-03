# AWO — Connect collection to independent admits

Status: **labor floor delivered — not package accept**  
Date: 2026-08-03  
Card: `15b01097-0126-40a0-b6a1-a759d54844a2`  
Depends on: T9 decisive finding

## Decisive prior

T8/T9: harness fair; independent path was **natural-only** under global physical mutex.

## Delivered (this labor)

| Item | Evidence |
|------|----------|
| Independent put queue + collector thread | `adaptive_write/collection.rs` |
| `admit_put` enqueues unconditional Buffered/Durable when physical bound | `runtime.rs` `try_collect_put` |
| Multi-item install | `Store::put_many_subject_bytes_awo_owned` + collector |
| `bind_physical` from host attach | `heap/host.rs` |
| Heap wait **outside** mutex | `heap_store.rs` `put_if` |
| Concurrent amortization test | `independent_puts_collect_amortize_file_sync` (file_sync < ops) |

## Explicit residuals

1. PQH harness still flushes `admit_put_batch([1])` on main thread — re-run T8 Shape B after harness uses concurrent `admit_put` + wait outside lock (or Arc store).
2. Multi-shard collection install is sequential (not parallel put_many).
3. Adaptive plan sizing on collection drain (today Static-style drain after delay / pile-up).
4. Credit ledger timing vs install (released at enqueue for collection path).
5. Package accept — principal only.

## Exit

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission -- --test-threads=1
# includes independent_puts_collect_amortize_file_sync
```
