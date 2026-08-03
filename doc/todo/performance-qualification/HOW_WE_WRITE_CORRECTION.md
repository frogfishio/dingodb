# “HOW we write makes all the difference” — correction

**Short answer: not quite.** Paying for the write path matters a lot. **How big / how coalesced** each `write_all` is does **not**.

## What you might mean vs what the numbers say

| Claim | Verdict | Evidence |
|-------|---------|----------|
| “Write **size** / buffering (64 KiB pages) is the lever” | **No** | Coalesce64k ≈ Real (~10–11k) |
| “Whether we **do** segment-tail `write_all` dominates Real thr” | **Yes** | Discard ~129k ≈ 13× Real |
| “Our real-write **path** is more expensive than SQLite’s for Mode A” | **Yes (gap exists)** | SQLite ~30k vs Residiuum Real ~13k — both writing durable bytes; we pay more per ack |
| “Which physical disk (T3 vs APFS) is the Residiuum lever” | **No** | Residiuum stayed ~10–14k across beds; SQLite jumped |

So if “HOW” means **coalesce / page size / adaptive disk pager** → **false**.

If “HOW” means **the whole Residiuum write path** (hashed frames, per-put tail write, index publish, Buffered `write_all` cost) vs **SQLite’s cheaper autocommit write** → **that gap is real**, but we have **not** isolated which sub-step inside “write path” is the SQLite delta — only that:

1. Skipping `write_all` entirely (Discard) removes most of *our* Real wall.
2. Changing write chunk size does not.
3. Cook alone (Discard) is already above SQLite — so closing to SQLite is about **cheaper durable bytes**, not faster Blake.

## One line

**Not:** how we *shape* writes to disk.  
**Yes:** that we *spend* on the write path (and more than SQLite for this peer).  
**Next:** bisect *inside* that path (syscall count, copy, seek, index-coupled flush) — not another 64 KiB buffer.

See [UNDERSTAND_THE_NUMBERS.md](UNDERSTAND_THE_NUMBERS.md), [FIRM_NUMBERS_DIAG_COALESCE.md](FIRM_NUMBERS_DIAG_COALESCE.md).
