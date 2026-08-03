# Next steps (post write-path / prealloc findings)

**Date:** 2026-08-03 · Direction for principal — not package accept / not a product ship plan  
**Where we are:** append/growth was the Real wall; sparse pre-size is placebo; **physical page pre-touch ~37k (~4×)** on concurrent Mode A; overwrite/Discard still ~100k+.

## Do next (ordered)

### 1. Confirm prealloc-fill is not a timing cheat

Pre-touch ran **before** the odometer. Re-run with either:

- touch cost **inside** the timed window, or  
- amortize: touch only the next seal-sized chunk as you go (background / ahead-of-write)

**Pass:** still ≫ Real (~2×+) with honest accounting.  
**Fail:** win was “pay allocation offline.”

### 2. Product-shaped allocation spike (still diagnostic)

Replace “touch every 1 MiB of 512 MiB at create” with something closer to shippable:

| Candidate | Why |
|-----------|-----|
| ~~macOS `F_PREALLOCATE`~~ | **Tried on APFS: ≈ Real (~9.5k)** — does not reproduce touch win ([GEMINI_PREALLOC_PLATFORM_REVIEW.md](GEMINI_PREALLOC_PLATFORM_REVIEW.md)) |
| Linux `fallocate` / `posix_fallocate` | Unmeasured; may differ from APFS |
| Seal-sized extents + ahead-of-write zero/touch | Bound waste; may match what touch actually bought |
| Double-buffer: prepare segment N+1 while writing N | Hide setup latency |

Same peer recipe: Mode A · c=8 · APFS · vs Real · vs SQLite A.  
**Goal:** see if ~37k is reachable without a crude page-poke.

### 3. Re-measure the SQLite gap under the same feed

With Real ~10k and prealloc-fill ~37k on **c=8**, republish one table:

| Cell | ops/s |
|------|------:|
| SQLite A c=8 | ~30k (prior) |
| Residiuum Real c=8 | ~10k |
| Residiuum prealloc-fill c=8 | ~37k |
| Residiuum Discard c=8 | ~120k |

**Question:** is “beat SQLite on Mode A concurrent” now an allocation problem, not a Blake/AWO problem? (Likely yes on this bed — lock it with one paired run.)

### 4. Scratch PEER continuity (when mounted)

Repeat **Real vs prealloc-fill vs SQLite** on `/Volumes/Scratch` (exFAT).  
We already know Residiuum Real ≈ flat across beds; **does allocated-pre-touch still win on Scratch?** If yes, story is OS page alloc, not APFS-only.

### 5. Only then design product prealloc

If (1)–(4) hold: write a small design note (not a feature merge):

- when to allocate (create / rotate / watermark)  
- how much (seal threshold vs fixed)  
- failure modes (disk full, sparse fallback)  
- durability/CSQ: prealloc must not weaken Buffered/Durable labels  
- disclosure: space amplification vs thr

**Do not** merge diagnostic `RealPreallocFill` into product paths.

### 6. Parallel residual (do not confuse with #1–5)

| Residual | Status | Action |
|----------|--------|--------|
| AWO QD=1 ~2.5k | Understood (feed tax) | Use c>1 for thr claims |
| AWO ≉ beat off on c=8 | Open | Separate from write-growth; don’t block on it for Mode A odometer |
| ~37k → ~100k (vs overwrite) | Open | Unique page dirtying / amplification; only after product alloc shape exists |
| On-disk ~2× logical | Open | Bytes/ack vs SQLite; secondary |

## Explicitly not next

- Another 64 KiB coalesce spike (already flat)  
- More multicore cook on Mode A batch=1 (already flat)  
- Treating Discard ~120k as a product target  
- Shipping `set_len` “prealloc” without allocation proof  

## One-sentence program

**Prove honest physical preallocation closes Real→SQLite on concurrent Mode A; then design seal-sized product alloc; leave AWO and the 37k→100k band as separate lanes.**

## Evidence trail

| Doc | Finding |
|-----|---------|
| [WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md) | Wall = append/growth |
| [PREALLOC_SPIKE.md](PREALLOC_SPIKE.md) | Sparse no; page-touch ~4× |
| [FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md) | c=8 removes AWO delay tax |
| [UNDERSTAND_THE_NUMBERS.md](UNDERSTAND_THE_NUMBERS.md) | Full stitch |
| [HOW_WE_WRITE_CORRECTION.md](HOW_WE_WRITE_CORRECTION.md) | Not write *size* |
