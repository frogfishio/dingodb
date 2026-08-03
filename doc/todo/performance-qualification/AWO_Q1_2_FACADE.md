# AWO-Q1.2 — Wait-outside-mutex under concurrent admit (labor evidence)

Status: **labor complete (in_review) — not package accept**  
Card: `37a4c1a2-0d0f-43e7-9484-17a33d54585f`  
Date: 2026-08-03  
Parent: AWO-Q1 (`b6e2a138`) · Series: `AWO_QUALIFICATION_SERIES.md`

## 1. Product path (already present)

`HeapStore::put_if` admits under the physical mutex and waits for Durable
collection install **outside** the lock (`heap_store.rs` ~L290–322). That is
the product wiring; Q1.1’s direct-handle harness only proved the collector.

## 2. What this labor adds

| Change | Location |
|--------|----------|
| Concurrent façade test: N threads × `HeapStore::put_collection` | `tests/awo_static_admission.rs` `heap_store_facade_concurrent_put_collection_wait_outside_mutex` |
| Asserts all keys readable via façade | same |
| Asserts Durable `file_sync < ops` (collector amortization requires pile-up) | same |

Direct-handle residual remains covered by `independent_puts_collect_amortize_file_sync`.

## 3. Verify

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission \
  heap_store_facade_concurrent -- --test-threads=1
```

## 4. Explicit non-claims

- No thr ranking / three-way re-run (Q1.3)
- No package accept / default-on
- No Adaptive decision quality (Q2)
