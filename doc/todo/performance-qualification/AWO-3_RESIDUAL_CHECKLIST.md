# AWO-3 residual checklist

Status: **labor floor delivered — not package accept**  
Date: 2026-08-02  
Card: `ab62782e-734b-43da-a000-98fa4102c5a4`

## Delivered (this labor)

| Item | Evidence |
|---|---|
| `AdaptiveWriteHandle` / status / drain | `adaptive_write/runtime.rs` |
| `AdaptiveWriteError` wire ids (`write_overloaded`, …) | `AdaptiveWriteError::as_str` |
| Eligibility classify put/delete | `classify_put` / `classify_delete` |
| `StoreHost::{create,open}_with_adaptive_write` | `heap/host.rs` |
| Mode **default disabled** | machine defaults + host attach |
| Lease fence `AdaptiveWriterActive` | `Store::awo_lease_active` + direct put/delete/put_many |
| Natural admit under lease | `admit_put` / `admit_delete` + `*_awo_owned` |
| Cooker warm on Static/Adaptive | status `cooker_threads` |
| Tests | `awo_static_admission` **6/6** |

## Explicit residuals (not closed)

1. **E6** heap-qualified active-writer layout (`active/<heap-id>/<shard>.residiuum`) — still open; product heap routing uses shared physical lock.
2. **Batch coalescing + cook→install** under Static mode — admits are natural-sync on this floor; cooker is warmed only.
3. **HeapStore routing** through AWO handle for collection put/delete — residual; façade still calls physical put under mutex (will hit lease if Static).
4. **Server error map / operation_id** for remote enablement — AWO-3 deep / server package.
5. **Adaptive controller** — AWO-5 (Adaptive mode currently same floor as Static).
6. **Package accept** — principal only.

## Exit command

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission -- --test-threads=1
```

## Next package

**AWO-3 deepen or AWO-4** — batch cook install under lease; pipeline depth ≤ 2; or HeapStore admit routing + E6.
