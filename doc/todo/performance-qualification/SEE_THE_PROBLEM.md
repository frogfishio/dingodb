# See the problem now?

**Date:** 2026-08-03  
**Card:** `33644355`  
**Ask:** “see the problem now?”

## Yes.

```text
Same byte sizes / same write counts on this disk  →  ~129k put-shaped ops/s  (~1 GiB/s)
Residiuum peer Real / coalesce                 →  ~9–11k TPS               (~70–85 MiB/s)
```

**~12× gap.** The bed is not slow at accepting this write shape. Residiuum is.

## What we ruled out (again, with teeth)

| Suspect | Status |
|---------|--------|
| “Disk can’t do ~85 MiB/s” | **Dead** — mimic does ~1 GiB/s with the same per-op sizes |
| “Need bigger write chunks (100 KiB coalesce)” | **Dead as the main lever** — modest TPS bump only |
| “Index publish timer is the wall” | **Dead as the sole story** — publish ~11 ms; skip-index still only ~30k |
| “Just add watermark / prealloc and TPS follows” | **Not shown** — shape ok, peer TPS not ≫ grow |

## What the problem actually is

**Product path cost between “bytes ready” and “acked put”** — not the abstract ability of APFS to absorb 8440 B appends.

Already pointed (bisect ladder): Real write path / first-touch·growth ≫ cook alone (Discard ~128k ≈ mimic). Prealloc and coalesce did not close the gap to mimic. So the remaining wall sits in **how Residiuum performs Real appends** (and everything coupled to them on the put path), not in “we chose the wrong chunk size.”

## One line

```text
The disk is fine. Our put path is ~12× slower than the same-sized writes.
That is the problem.
```

Scoreboard stays **TPS**. Mimic is a ceiling check, not a product claim. See [WRITE_MIMIC.md](WRITE_MIMIC.md).

**Where the 12× sits:** [WHERE_IS_THE_12X.md](WHERE_IS_THE_12X.md) (~4× grow × ~3× index/derived).
