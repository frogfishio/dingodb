# Odometer first — completed writes/s (SQLite-comparable)

Status: **labor evidence / communication rule** (not package accept / not SLO)  
Card: `7413cf09-f881-4e0c-aed4-5a216c6138bf`  
Date: 2026-08-03  

## Principal complaint (accepted)

> I ask how many completed end-to-end writes/s (like SQLite).  
> I get ops/s with lubricant and squint — engine RPM and gearbox ratio.  
> I asked how far the car is going.

**Fair.** Labor must answer the **odometer** first. Attribution (sync ratio,
seal share, cook %) is **secondary**, labeled as such.

## Primary metric (the car)

**Name:** completed end-to-end **acked puts/s**  
**Meaning:** one logical write presented, durability/ack boundary of the **named
mode** crossed, receipt returned — same idea as one SQLite insert that has
finished under that mode’s commit rules.

Prefer this over the word “TPS” (which invites transaction confusion). If the
principal says TPS, map it explicitly to **acked puts/s** under a named mode.

### The SQLite-comparable odometer (PEER Mode A)

Same bed, 8 KiB, QD=1, Scratch — Mode A = Residiuum per-put **Buffered** vs
SQLite **autocommit**:

| Engine | Completed acked writes/s | Source |
|--------|-------------------------:|--------|
| Residiuum | **~10 000** | Campaign F / F re-run / H ladder |
| SQLite | **~10 000** | same peer cells |

**Answer in one sentence:** On the fair 1:1 peer bed, Residiuum completes about
**the same number of end-to-end writes per second as SQLite — ballpark 10k/s.**

**Smart mode (Adaptive) on that same bed:** **X = unknown** — not measured.
Do not multiply 10k by T11’s Durable ~2×. See
[SMART_MODE_X_MODE_A.md](SMART_MODE_X_MODE_A.md).

That is the car distance for “like SQLite autocommit.” Not a marketing SLO;
diagnostic peer only (`README-PEER-SQL.md`).

### Same question, other named modes (still odometer — not RPM)

| Question | Completed acked writes/s | Notes |
|----------|-------------------------:|-------|
| Mode B SQLite txn-128 vs Residiuum put_many-128 | Residiuum ~**10k**; SQLite ~**18k** | Different amortize rules; not “same T” |
| Durable AWO T11 saturated (smoke) | ~**0.5k → ~1.1k** (Disabled → Static/Adaptive) | **Durable** barriers; not Mode A |
| Short Buffered no mid-seal micro (G.2) | ~**135k** | Same Buffered contract, **different bed** (short, no mid seals) — not the PEER long-peer odometer |

If you asked for SQLite-like 1:1, the answer is **~10k**, not 135k and not T11’s 2×.

## Secondary metrics (gearbox — only after odometer)

| Secondary | Role |
|-----------|------|
| `file_sync / logical_ack` | Why Durable thr moved (T11) |
| thr MiB/s | Bandwidth view of same acks |
| seal_rotate % / append % | Where wall time went |
| Discard vs Real | Diagnostic short-circuit vs media |
| “bands” 10k / 100–160k / 330k | Length/seal/cook context — **not** substitutes for Mode A odometer |

## Standing communication rule (labor)

1. **Lead** with: `acked_puts/s` + **mode** + **bed** + **payload** + vs SQLite if peer.  
2. **Then** (optional): mechanism / ratio / band explanation.  
3. **Never** lead with a ratio (“2×”, “k≈2”) when the question was absolute completed writes.  
4. If the bed is not PEER Mode A, say so in the **first** sentence (“not the SQLite 1:1 odometer”).

## Non-claims

Not a published product SLO. Not AWO/PQH accept. Does not retire Campaign H park.
