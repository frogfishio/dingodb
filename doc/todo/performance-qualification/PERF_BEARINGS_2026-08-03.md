# Performance bearings — where we are vs the bigger truth

Status: **labor evidence** (not package accept / not product floors)  
Card: `1e88c4c3-7b8b-4e9b-bea5-a5804f6c2fc9`  
Date: 2026-08-03  
Sources: scoreboard header, T11 freeze, AWO-Q0…Q2, PEER-SQL Campaign F/G, MASTER §20

## Verdict (one paragraph)

We are **measurably closer to the performance bigger truth** than before AWO
measurement, but we are **not** at product performance qualification or M2
“SQLite replacement” honesty. The bigger truth we were chasing was *why*
Residiuum looks slow and *what* actually moves durable write throughput — not a
marketing bench. That causal picture is now partially frozen; the remaining gap
is sustained/product claims and the Mode-B txn amortization story.

## 1. What “bigger truth” means here

| Layer | Question | Status |
|-------|----------|--------|
| **Mechanism** | Do we amortize durability barriers under pile-up? | **Yes — T11 freeze** (principal `done`) |
| **Correctness under load** | Concurrent independent admit without lost/dup ack? | **Yes — AWO-Q1** (principal `done`) |
| **Controller honesty** | Does Adaptive decide intelligently vs Static? | **Partial — AWO-Q2 labor `in_review`** |
| **Peer reality** | Fair Residiuum vs SQLite same bed? | **Mode A ≈ parity; Mode B SQLite ~1.8×** (PEER-SQL) |
| **Product floors** | Controlled PQH qualification + floors? | **No** — PQH-0…11 `active` / board `in_review`, not principal accept |
| **M2 gate** | Careful outsider replaces SQLite+files? | **No** — still M1 spine / residual; `qualified=false` |

## 2. Performance-wise: current facts only

### AWO mechanism (T11 — first positive signal)

Saturated independent singles (APFS smoke, presentation-fair):

| Mode | ~thr MiB/s | logical acks / sync |
|------|-----------:|--------------------:|
| Disabled | ~4.2 | 1 |
| Static | ~9.0 | 2 |
| Adaptive | ~8.8 | 2 |

Sparse: **no** batching benefit; **11–20%** smoke thr penalty vs Disabled.

**Metric law:** `file_sync / logical_acknowledged_operations` (not file_sync/append).

### AWO-Q progress

| Package | Board | Meaning |
|---------|-------|---------|
| Q0 series contract | `done` | Entry honesty |
| Q1 concurrent admit | `done` | Real multi-thread path + ledger + reopen |
| Q2 decision quality | `in_review` | Cold Natural-1; warm can Batch; envelope ≤3× Static smoke; detach hang fixed |
| Q3 sustained diagnostic | `backlog` | Not started |
| Q4 sparse product bound | `backlog` | Not started |

Q2 explicit residual: IndependentCollector flush is still delay/max-entries —
`select_plan` is **not** wired into collector; Adaptive vs Static *decision*
divergence is proven on multi-item `admit_put_batch`, not on every concurrent
single path.

### PEER-SQL (same-bed Scratch)

| Mode | Residiuum / SQLite | Read |
|------|-------------------:|------|
| A (autocommit / per-put Buffered) | ~0.98–1.05× | Fair peer ≈ parity |
| B (txn-128 / put_many 128) | ~0.55× | SQLite wins on amortized commit — expected without product txn API |

Instrumentation (Campaign G): long Mode A with default seal is often
**seal-bound**; continuous no-rotate path is **append-bound** (Blake+copy), not
a mystery “disk idle + CPU idle” story.

### PQH lane

PQH-0…11 labor largely complete and board `in_review`; scoreboard still
`active` / **not** principal accept. No controlled-host 120s qualification
product baseline; no new quantitative floors from AWO smoke.

## 3. Are we closer?

**Yes, on mechanism + honesty.** Before T9–T11 we could not honestly claim AWO
helped independent writes (collection was disconnected; harness could fake
batching). Now:

1. Collection is connected for independent admits.
2. Saturated thr×2 ↔ sync/2 is a **frozen causal signal**.
3. Concurrent admit correctness (Q1) and Adaptive decision cells (Q2 labor) sit
   on top of that floor.
4. PEER-SQL Mode A parity kills “Residiuum is mysteriously 10× slower at
   autocommit” as a default narrative.

**No, on product / M2 bigger truth.** Still open:

- AWO default-off; no package accept; no product floors
- Sparse latency product posture (Q4)
- Sustained PQH diagnostic (Q3) + principal PQH accept
- Mode B gap needs a named batch/txn product story (or explicit “SQLite still
  better for amortized commit” docs) — MASTER M2, not this smoke lane
- Heap `qualified = false`; query spine / APB packages active without accept

## 4. Next honest pulls (performance lane)

1. Principal accept or reject **AWO-Q2** (`0a043642`) from `AWO_Q2_DECISION_QUALITY.md`
2. Promote **AWO-Q3** (sustained diagnostic) when ready — not smoke alone
3. Keep **PQH principal accept** as hygiene — do not invent floors from T11
4. Do **not** treat board `in_review` AWO-0…7 cards as AWO product accept

## 5. Non-claims

- Not production-ready performance
- Not AWO package accept / default-on
- Not PQH qualification accept
- Not M2 SQLite-replacement gate
- Hang fix (collector join-after-detach) is correctness hygiene, not a thr claim
