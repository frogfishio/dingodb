# AWO three-way — T9 decisive finding (independent writes not connected)

Status: **labor complete (honesty freeze) — not package accept, not a code fix**  
Feature: Measure adaptive write batching `ac713f4d-…` · AWO product residual  
Date: 2026-08-03  
Evidence: T8 APFS smoke (`file_sync/ops = 1` all modes under independent singles)

---

## 1. Decisive result (one sentence)

> **The harness is correct. The automatic write optimiser is not yet connected to independent writes.**

T8 proved presentation fairness (`batch=1`, concurrent singles). Barrier math stayed
`file_sync == ops` for disabled **and** static **and** adaptive. That is not thr
noise — it is the product path.

---

## 2. Code path (product single-write)

`PhysicalStore` is `type PhysicalStore = Store` (`kernel/mod.rs`).  
Heap product put holds the **global physical mutex** for the whole admit+execute:

```text
HeapStore::put_if
  → physical.lock()                          // Arc<Mutex<PhysicalStore>>
  → AdaptiveWriteHandle::admit_put(...)      // if lease active
       → admit_natural(...)
            → put_subject_bytes_if_awo_owned  // synchronous natural put
  → WriteCompletion::wait()                  // already completed inline
  → drop(guard)                              // release mutex
```

Anchors:

| Step | Location |
|------|----------|
| Mutex + admit_put + wait | `crates/residiuum-store/src/heap/heap_store.rs` `put_if` |
| admit_put → admit_natural | `adaptive_write/runtime.rs` `admit_put` |
| Natural = sync op under credits | `adaptive_write/runtime.rs` `admit_natural` |

```rust
// admit_put (abbreviated)
self.admit_natural(store, |s| {
    s.put_subject_bytes_if_awo_owned(subject, value, mode, condition)
})

// admit_natural (abbreviated)
// reserve credit → op(store) immediately → release credit → Admitted(receipt)
```

Module floor is explicit (`runtime.rs` crate docs):

- **Static:** *single admits execute natural under the lease*
- **Batch path:** `admit_put_batch` only when the **caller** presents a multi-item slice

There is **no** enqueue → collect → multi-item install on the independent `admit_put` path today. No collection delay is applied before the natural put. The global mutex serializes concurrent heap puts so concurrent singles **cannot** pile up inside AWO while another put holds the store.

---

## 3. Why T8 measured exactly this

| Layer | What T8 did | Effect |
|-------|-------------|--------|
| Presentation | `--present-batch 1` always | No L-API `put_many(N)` for Disabled |
| Harness flush | `admit_put_batch(store, &[one])` on main thread | Still one item → one `put_many_awo_owned` → one Durable sync |
| Concurrent preparers | conc=4 prepares singles; install serial | Mutex/store still sees N=1 flushes |
| Adaptive `plan_batch_take` | Input length 1 | Natural vs Batch cannot choose k>1 |

So T8 cannot “see” collection that does not exist. Equal `file_sync/ops` across modes is the **expected** signature of natural-only independent writes.

---

## 4. Two layers (keep separate)

| Layer | Connected today? | What it does |
|-------|------------------|--------------|
| **L-API** `put_many` / `admit_put_batch([N])` | Yes | N appends, one tail sync (store batch API) |
| **L-AWO collection of independent puts** | **No** | Spec intent: queue + collection delay + batch plan from concurrent singles |

Prior wrong saturated design (`batch_size=8`) tested L-API. T7 v2 + T8 correctly test L-AWO and show it is **not wired**.

---

## 5. What “connected” would require (residual — do not freestyle here)

Normative direction lives in `ADAPTIVE_WRITE_OPTIMISER_SPEC.md` (natural vs batch,
`collection_delay`, queue). Product residual (names only; not this card’s implement):

1. **Independent admit path** that can **return without** holding the physical mutex through Durable sync (or a dedicated collector thread with store access rules).
2. **Collection** of eligible singles up to Static/Adaptive plan (respect deadlines).
3. **One install** via existing `put_many_awo_owned` / persist-before-publish.
4. **Heap lock story** reworked so concurrent independent writes can exist in the AWO queue (today’s global mutex makes that impossible).
5. Re-run T8 Shape B: expect Static `file_sync/ops ≪ 1`, Adaptive → Static under saturation; sparse Adaptive ≈ Disabled latency.

Until then: **do not** claim Adaptive/Static product wins on independent-write workloads.

---

## 6. Claim table

| Claim | Status |
|-------|--------|
| T8 harness presentation AWO-fair (`batch=1`) | **Yes** |
| Independent-write path is natural-only under lease | **Yes** (code + T8) |
| Global physical mutex serializes heap puts through natural admit | **Yes** |
| AWO collection of independent writes product-ready | **No** |
| T6 Scratch thr gap explained by multi-write barriers | **No** (batch=1 equal syncs) |
| Package accept / default-on AWO | **No** |

---

## 7. Related

| Path | Role |
|------|------|
| `AWO_THREE_WAY_T8_SINGLES_RUN.md` | Measurement that forced this conclusion |
| `AWO_THREE_WAY_T7_SPARSE_SATURATED.md` | Presentation law (independent singles) |
| `heap_store.rs` `put_if` | Product single-write mutex + admit_put |
| `adaptive_write/runtime.rs` | admit_put → admit_natural |
| `ADAPTIVE_WRITE_OPTIMISER_SPEC.md` | Intended collection / natural vs batch |
