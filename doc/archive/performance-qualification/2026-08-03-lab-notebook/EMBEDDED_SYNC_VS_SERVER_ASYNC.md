# Embedded sync vs server async — feed shape (principal lock)

Status: **principal framing locked (labor) — not package accept / not new measure yet**  
Date: 2026-08-03  
Principal: FN-2 Mode A QD=1 peer-pump is artificial sabotage of the optimiser —
forcing one-by-one; mixing embedded sync with “real database” async feed.

## Verdict

**Agreed.** The empty waiting window / Static~2.5k story is not “AWO cannot
microbatch.” It is **how we fed the store**: a single synchronous client that
waits for ack before the next put (peer-pump Mode A). That is **embedded sync
hammering**, not `millions of users → web server → database`.

```text
FN-2 Mode A feed:   one client thread → put → wait ack → put → wait ack → …
Real DB feed:       many concurrent clients / async handlers → overlapping puts
```

AWO’s job is to coalesce **overlap**. We removed the overlap, then blamed the
optimiser for not batching. Apples and oranges.

## Two modes (name them)

| Mode | Feed | What it measures | FN-2 role |
|------|------|------------------|-----------|
| **1. Embedded sync** | Single thread, QD=1, wait-ack-before-next | One embedded user’s serial puts; ~single-core critical path | What we ran as “Mode A” |
| **2. Server async (“normal”)** | Concurrent / outstanding > 1 (web tier parallelism) | Real multi-client DB; AWO can microbatch | **Measured** — [FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md) |

Principal map:

1. **Embedded mode** — on fast disk we lose badly to SQLite (~12.6k vs ~29k).
   Looks CPU-bound; sync hammering also caps how much parallelism helps on that
   path. Mode A = this sync embedded shape.
2. **Normal mode** — we are a real database: async, many in-flight requests;
   optimiser + multicore have a job.

Do **not** sell FN-2 Adaptive/Static ~2.5k as “smart mode on a real DB.”
That cell is **smart mode + embedded-sync feed sabotage**.

## What stays true vs what was misread

| Claim | Status |
|-------|--------|
| peer-pump waits for ack before N+1 | **True** — client ([WHO_WAITS_FOR_ACK.md](WHO_WAITS_FOR_ACK.md)) |
| That starves collector / microbatch | **True** ([ZERO_IN_WAITING_WINDOW.md](ZERO_IN_WAITING_WINDOW.md)) |
| AWO is incapable of microbatch | **False** — T11 pile-up already showed it |
| Residiuum-off ~12.6k vs SQLite ~29k on APFS | **Still a real embedded-sync peer fact** (CPU/cook wall) |
| Adaptive loses to off on FN-2 | **True only under sync QD=1 feed** — not the server-async verdict |

## Mixing apples and oranges (stop)

| Wrong merge | Right split |
|-------------|-------------|
| “Mode A Adaptive X” = product smart-mode odometer | Split: **embedded-sync Adaptive** (FN-2, hostile to AWO) vs **server-async Adaptive** (not yet FN-measured on Mode A knobs) |
| “We can’t microbatch” | “**This feed** can’t form pile-up” |
| Multicore cook on Mode A QD=1 | Irrelevant to server-async; batch=1 serial client |

PEER-SQL Mode A remains a valid **embedded / autocommit-peer** contract. It is
the wrong exclusive bed for judging AWO as a multi-user DB optimiser.

## Next honest measure (when pulled)

**Server-async Mode A-shaped payloads** (8 KiB Buffered), outstanding/concurrency
> 1 (web-tier-like), same disclosure — **done (labor):**
[FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md)
(c=8 APFS: Adaptive≈off~13.6k; Static~13.1k; SQLite~29.7k; delay tax gone).

Until Scratch re-run, treat as APFS diagnostic only.

## Non-claims

Not package accept. Not “QD=1 illegal.” Not “drop PEER Mode A.”  
Not implemented this turn — framing + program split only.
