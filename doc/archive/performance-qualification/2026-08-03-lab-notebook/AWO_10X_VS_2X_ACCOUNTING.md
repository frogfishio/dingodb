# Accounting — 10× bucket aim vs T11 ~2×

Status: **labor evidence** (not package accept / not floors)  
Card: `6a918f34-fee0-4924-b664-3aff5d6ed4f6`  
Date: 2026-08-03  
Audience: principal aim check — “I see ~10× when we bucket, but AWO only showed ~2×; where is the other ~8×?”

## 0. Verdict (yes, we see where you’re coming from)

Your **aim** for Adaptive is right:

```text
hammer 1:1 → optimiser decides bucketing is cheaper → auto-bucket → run
```

The **discrepancy is real in the numbers**, but it is mostly **cross-band accounting**,
not “AWO found only 2× of a true 10× Durable win and lost 8×.”

- **~10K → ~120–140K** lives in the **Buffered / short / seal-sensitive** world
  (PEER + Campaign G/H three-band rule). **Not “disk off”** — see
  `AWO_120K_NOT_DISK_OFF.md` (Buffered still writes; `file_sync=0` means no fsync).
- **T11 N → ~2N** lives in the **Durable / sync-bound / k≈2 collection** world
  (AWO three-way smoke).

Those are not the same multiplier applied to the same bottleneck.

## 1. Name the two stories with their beds

### Story A — “bucket climbs to ~120–140K” (your earlier tests)

Campaign H three-band rule (`TEST_RESULTS.md`):

| Band | Ballpark | What it is |
|------|----------|------------|
| **~10k** | PEER Mode A / multi-seal long peer | Adoption floor; Residiuum ≈ SQLite same bed |
| **~100–160k** | Short `/tmp` or no-mid-seal phase-bench | Cook/append path with seals avoided |
| **~330k** | Short Scratch cook micro | Cook-parallel fantasy; not media capacity |

Campaign G.2 Scratch micro (20k × 8 KiB, **512 MiB seal → 0 mid seals**):

| Cell | ops/s |
|------|------:|
| Buffered put batch=1 after write-through | **~135k** |
| file_sync count | **0** (Buffered micro) |

Long PEER Mode A (same knobs, **64 MiB seals mid-run**) stays **~10k** even after
write-through — rate does **not** jump to 135k on the long peer.

So “10K → 120K” is largely: **leave the multi-seal / long-peer band** and enter a
**short Buffered cook band**, not “flip Adaptive on under identical Durable load.”

### Story B — T11 first positive AWO signal (principal freeze)

Saturated independent singles, **Durable**, presentation-fair `batch_size=1`:

| Mode | ~thr | logical acks / sync |
|------|-----:|--------------------:|
| Disabled | ~4.2 MiB/s (~0.5k ops/s @ 8 KiB) | 1 |
| Static / Adaptive | ~9 MiB/s (~1.1k ops/s @ 8 KiB) | **2** |

Causal freeze: thr×2 ↔ sync/2. Collection depth on that smoke cell was **k≈2**,
not a 128-wide bucket.

Absolute rate is **~50× below** the PEER 10k band because this path pays
**Durable barriers**, not Buffered `file_sync=0` micros.

## 2. Factor the “missing ~8×”

Do **not** divide 12÷2 and call the quotient an AWO bug. Split the product:

```text
(your remembered 10K→~120K)
  ≈  leave multi-seal long peer (~10k)
  ×  enter short/no-seal Buffered cook band (~100–160k)
  ≈  ~12× across bands

(T11 AWO)
  ≈  Durable sync-bound cell
  ×  collection depth k≈2
  ≈  ~2× from barrier amortization only
```

| Factor | Explains part of 10K→120K? | Explains T11 2×? | Status vs Adaptive aim |
|--------|---------------------------:|-----------------:|------------------------|
| **Durability class** (Buffered vs Durable) | Dominates absolute TPS | Sets T11’s low base | Adaptive today measured on **Durable**; 120k beds are **Buffered** |
| **Seal / rotate wall** | Yes — G: ~42k→~108k with seals off | No (T11 not seal story) | Not AWO’s job; seal policy residual |
| **Collection depth k** | Only if Durable+sync-bound | **Yes — k≈2 ⇒ ≤2×** | Aim wants deep auto-buckets; T11 only proved shallow k |
| **Cook/Blake/append** | Yes in short band; parallel cook ~1.8× micro | Secondary on T11 | Separate cooker residual (Campaign H parked) |
| **L-API `put_many(N)` presentation** | Explicit buckets | Forbidden in T11 (fairness) | Adaptive must *form* buckets from 1:1; T11 proves shallow form works |

**Identity for sync-bound Durable:**

```text
max_sync_amortize_gain ≲ k   (logical acks per Durable barrier)
```

With **k≈2**, **~2× is the correct ceiling** for that cell. You are not “missing 8×
of sync amortization” there — you never formed a depth-10 bucket on that smoke.

To get **~10× from sync alone** on a Durable sync-dominated path you need roughly
**k≈10** (and still be sync-bound). That is a **deeper collection / plan** residual
(Q2 collector `select_plan`, Q3 sustained), not a T11 accounting error.

## 3. Map to the Adaptive aim (1:1 → auto-bucket)

| Aim step | Where we are |
|----------|----------------|
| Accept 1:1 hammer | Independent `admit_put` + collection connected (T10/T11) |
| Decide bucketing is cheaper | Q2 labor: cold Natural-1; warm can Batch on multi-item present; collector flush still delay/max-entries (**`select_plan` not wired**) |
| Auto-bucket depth that approaches explicit ceiling | **Shallow today (k≈2 on T11)**; Static multi-item present is the explicit ceiling in Q2 tests |
| Match remembered 120k band | **Wrong target for Durable T11.** 120k ≈ Buffered short/no-seal cook band; PEER long stays ~10k |

Also honesty from PEER Mode B: Residiuum `put_many` batch=128 on the **long** peer
stayed ~10k (≈ Mode A). Explicit L-API batching alone did **not** buy the 12× on
that bed — seals/cook still dominate. So “bucket = 12×” was never a single knob.

## 4. What “missing 8×” should mean going forward

Treat it as **three open accounting lines**, not one:

1. **Depth gap:** raise Durable collection **k** under pile-up (aim k≫2) and re-measure
   `logical_acks/sync` + thr — expect gain **≲ k** while sync-bound.
2. **Class gap:** if the product goal is “feel like 120k bucket benches,” say so as
   **Buffered / seal-policy / cook** work — do not expect Adaptive Durable smoke to
   invent that band.
3. **Peer gap:** Mode B SQLite txn still leads ~1.8× on amortized commit — product
   txn/batch API story (MASTER M2), orthogonal to T11’s 2×.

## 5. Non-claims

- Not “AWO is only 2× forever”
- Not “120k is a product floor”
- Not package accept / default-on
- Not that PEER 10k and T11 ~1k Durable are comparable TPS without labeling durability

## 6. Sources

- `TEST_RESULTS.md` — Campaign F/G/H; three-band rule  
- `AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md` — thr×2 ↔ sync/2, k≈2  
- `AWO_Q2_DECISION_QUALITY.md` — decision cells; collector `select_plan` residual  
- `PERF_BEARINGS_2026-08-03.md` — prior bearings  
