# AWO-3 residual checklist

Status: **labor floor + batch/heap deepen — not package accept**  
Date: 2026-08-02  
Card: `ab62782e-734b-43da-a000-98fa4102c5a4`

## Delivered

| Item | Evidence |
|---|---|
| `AdaptiveWriteHandle` / status / drain | `adaptive_write/runtime.rs` |
| `AdaptiveWriteError` wire ids | `AdaptiveWriteError::as_str` |
| Eligibility classify put/delete | `classify_put` / `classify_delete` |
| `StoreHost::{create,open}_with_adaptive_write` | `heap/host.rs` |
| Mode **default disabled** | machine defaults + host attach |
| Lease fence `AdaptiveWriterActive` | `Store::awo_lease_active` |
| Natural admit under lease | `admit_put` / `admit_delete` + `*_awo_owned` |
| **Batch admit under lease** | `admit_put_batch` → `put_many_awo_owned` (persist-before-publish; sets cook_parallelism from cooker pool) |
| **HeapStore AWO routing** | `open_heap` clones handle; `put_if`/`delete_if` admit when lease active |
| Cooker warm on Static/Adaptive | status `cooker_threads` |
| Tests | `awo_static_admission` (batch + lease + disabled) |

## Explicit residuals (not closed)

1. **E6** heap-qualified active-writer layout — still open.
2. **Persistent cooker owns encode for batch** — product batch still uses store parallel cook path (cooker pool warmed; not yet sole cook authority).
3. **HeapStore integration test with live HeapCap** — ceremony-heavy; wiring present, mint residual.
4. **Server error map / operation_id** for remote enablement.
5. **Adaptive controller** — AWO-5.
6. **AWO-4** pipeline depth ≤ 2 / no third reservation.
7. **Package accept** — principal only.

## Exit command

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission -- --test-threads=1
```

## Next package

**AWO-4** — ordered overlap depth ≤ 2, seal-safe shutdown; or cooker-as-sole-encode authority.