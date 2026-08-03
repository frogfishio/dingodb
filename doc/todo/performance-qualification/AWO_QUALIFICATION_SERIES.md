# AWO-Q — Post-T11 qualification series

Status: **AWO-Q1 principal `done` (correctness scope) — next pull AWO-Q2**  
Entry: T11 evidence freeze principal `done` (`b7986427`)  
Date: 2026-08-03 (Q1 package principal closed; Q2 promoted to `todo`)  

## 0. Entry honesty

T11 smoke is a **mechanism floor**, not completion:

| Frozen (T11) | Not claimed |
|--------------|-------------|
| Saturated ≈2 logical acks / sync and ≈2× thr | Product floors / default-on |
| Sparse: no batching; 11–20% smoke thr penalty | Sparse product latency |
| Independent collection connected (labor) | Genuine multi-thread admit path |
| Adaptive thr ≈ Static on one smoke cell | Adaptive **decision quality** |
| Unchunked `append_count == logical_ack_count == 24` | Chunked workloads |

**Metric law (carry forward from T11):**

```text
file_sync / logical_acknowledged_operations
logical acknowledgements / sync
```

Do not report barrier amortization as `file_sync/append` or “writes/sync”.

**Authority:** this series does not override `MASTER_DELIVERY_PLAN.md` gate order.
It is a measurement / AWO product residual lane after the T11 freeze.

---

## 1. Packages (pull order)

### AWO-Q1 — Genuine multithreaded admission (**first pull**)

**Problem:** T10/T11 saturated path maps `conc>1` to **serial admit** with
`outstanding = max(out, conc)`. Barrier amortization was shown under a
**serial-outstanding** harness, not under real concurrent independent callers.

**Prove:**

1. **Distinct producer threads** (or processes) — not a single thread simulating depth.
2. **Deterministic operation/completion ledger** — each logical put has a unique id;
   exactly-once completion visible to the harness (no lost, duplicated, or double-acked
   operations).
3. **Caller waits without holding the store mutex** — `WriteCompletion::wait` (or
   equivalent) outside `physical.lock()` for the whole durable path.
4. **No lost, duplicated, or double-delivered acknowledgements** under concurrent admit.
5. **Reopen integrity** — valid + reopen digests green after concurrent admit (T10 residual).
6. **Optional thr cell** — re-measure Off/Static/Adaptive only after correctness;
   still smoke unless upgraded to diagnostic class.

**Non-goals for Q1:** adaptive controller quality, sustained 120s qualification,
sparse product floors, default-on.

**Likely touch points (implementer, not pre-committed):**

- PQH `real_store` / presentation pin (`conc`, `outstanding`, serial admit map)
- Heap `put_if` / `admit_put` mutex scope
- Collection queue under concurrent producers
- Unit / store-driver tests for concurrent admit + ledger

**Evidence exit (labor → in_review):** named evidence note + tests green + honest
claim table. Principal `done` only after review.

#### Q1 implementer pull slices (board sub-cards)

| Slice | Pull order | Outcome |
|-------|------------|---------|
| **Q1.1** Concurrent harness + ledger | **First code pull** | Drop serial-admit map for AWO batch=1; real multi-thread/process producers; deterministic ledger of logical puts ↔ completions. **Brief:** `AWO_Q1_1_IMPLEMENTER_BRIEF.md` (anchors in `store_driver/real.rs`) |
| **Q1.2** Wait-outside-mutex product path | After 1.1 | Heap/store admit path: wait not under physical mutex for full Durable; concurrent admits can pile collection |
| **Q1.3** Correctness + evidence freeze | After 1.2 | No lost/dup/double-ack; reopen integrity; claim table; optional thr smoke only after green |

Package **AWO-Q1** closes when 1.1–1.3 are principal-`done`.

---

### AWO-Q2 — Adaptive decision quality

**Problem:** T11 saturated cell shows Adaptive thr ≈ Static. That is **not** proof
that Adaptive chooses correctly under varying load (sparse vs saturated, credit
pressure, collection delay).

**Prove (when pulled):** Adaptive diverges from Static where the controller should
prefer different batch/delay plans; sparse Adaptive does not explode latency
beyond documented smoke bounds without a product claim.

**Depends on:** Q1 correctness path (concurrent admit must be real, or Adaptive
cannot be stressed honestly).

---

### AWO-Q3 — Sustained / diagnostic qualification

**Problem:** T11 is `max_cells=1` smoke. Product ranking and floors need PQH
controlled class (reps, budgets, disclosure) per `PERFORMANCE_QUALIFICATION_HARNESS_SPEC`.

**Prove (when pulled):** diagnostic-class three-way matrix under independent singles
with logical-ack metrics; no product claim without disclosure chain.

**Depends on:** Q1 (and preferably Q2 if Adaptive is ranked).

---

### AWO-Q4 — Sparse latency product bound

**Problem:** Sparse 11–20% thr penalty is a **smoke observation**. Product language
needs either a bound + campaign or an explicit residual that Adaptive is opt-in
for sparse writers.

**Prove (when pulled):** documented product posture (bound, opt-in, or residual)
with measure evidence — not a re-quote of T11 smoke alone.

**Depends on:** Q1 path real; Q3 if claiming diagnostic floors.

---

## 2. Explicit series non-claims

- AWO package accept / default-on  
- Cluster / search / archive pull-forward  
- Replacing CSQ-12 / Heap app-ready critical path  
- Treating T11 thr×2 as qualification accept  

---

## 3. Board mapping

Feature: `d0ae3c06-ca55-4cea-8282-8ee89278d849` — **AWO-Q — Post-T11 qualification series**  
Principal approved series 2026-08-03 (“keep going”).

| Card | Id | Stage | Notes |
|------|-----|--------|-------|
| AWO-Q0 series contract | `cfb1a4d3-…` | **`done`** | Principal accept |
| AWO-Q1 package umbrella | `b6e2a138-…` | **`done`** | Principal accept 2026-08-03 (correctness scope) |
| **AWO-Q1.1** harness + ledger | `fab1a943-…` | **`done`** | Principal accept |
| AWO-Q1.2 wait-outside-mutex | `37a4c1a2-…` | **`done`** | Principal accept |
| AWO-Q1.3 correctness + freeze | `ba312673-…` | **`done`** | Principal accept; concurrent rotate + exact scan set |
| **AWO-Q2** adaptive decision quality | `0a043642-…` | **`todo`** | **Next pull** — Adaptive load-curve / ceiling vs Static |
| AWO-Q3 sustained qualification | `ce3e8a1c-…` | `backlog` | After Q2 preferred |
| AWO-Q4 sparse latency product bound | `c827e21a-…` | `backlog` | After Q1 path real; Q3 if floors |

---

## 4. Related

| Path | Role |
|------|------|
| `AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md` | Entry freeze (both sides) |
| `AWO_THREE_WAY_T10_HARNESS_RERUN.md` | Serial-outstanding residual |
| `AWO_INDEPENDENT_COLLECTION_CONNECT.md` | Collection connect labor |
| `AWO_Q1_PACKAGE_CLOSEOUT.md` | Q1 package labor freeze + principal accept sequence |
| `AWO_Q1_1_HARNESS.md` / `AWO_Q1_2_FACADE.md` / `AWO_Q1_3_REOPEN.md` | Slice evidence |
| `ADAPTIVE_WRITE_OPTIMISER_SPEC.md` | Normative AWO behaviour |
| `PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md` | Measure semantics |