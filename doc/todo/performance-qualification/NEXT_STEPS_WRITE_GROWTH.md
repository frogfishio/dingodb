# Next steps (post write-path / prealloc findings)

**Date:** 2026-08-03 · Direction for principal — not package accept / not a product ship plan  
**Principal steer (locked):** Prealloc space is **not** a morality debate. People buy specialist hardware for ~5% wins; reserved runway is cheap. Capacity is **configurable** (default **64 MiB**, not fixed ½ GiB; large hosts may use multi‑GiB / 10 GiB). Extension **must not** tax transactions — background watcher ahead of head. Grow-on-append-as-virtue is rejected. See [PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md](PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md), [PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md](PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md).

**Where we are:** append/growth was the Real wall; sparse pre-size is placebo; put-path pre-touch can lift thr (~35–50k diag); product watermark E2E **not yet a proven win** after seal/odometer honesty fix (prior ~32k diag **withdrawn**). Overwrite/Discard still ~100k+.

## Do next (ordered)

### 0. Background runway preparer (principal-shaped next)

Move zero/extend **off the put path**: watcher keeps N MiB (or seal-chunk) prepared ahead of `durable_len`; optionally prepare segment N+1. Puts only consume runway; fail closed if exhausted.

**Pass:** put-path thr ≫ GrowOnAppend with E2E seal in meter; space amp disclosed.  
**This is the preferred shape** — not “debate whether to prealloc.”

### 1. Confirm prealloc-fill is not a timing cheat

Pre-touch ran **before** the odometer. Re-run with either:

- touch cost **inside** the timed window, or  
- amortize: touch only the next seal-sized chunk as you go (background / ahead-of-write) ← **same as §0**

**Pass:** still ≫ Real (~2×+) with honest accounting.  
**Fail:** win was “pay allocation offline.”

### 2. Product-shaped allocation spike (still diagnostic)

Replace “touch every 1 MiB of 512 MiB at create” with something closer to shippable:

| Candidate | Why |
|-----------|-----|
| ~~macOS `F_PREALLOCATE` alone~~ | **≈ Real** ([GEMINI_PREALLOC_PLATFORM_REVIEW.md](GEMINI_PREALLOC_PLATFORM_REVIEW.md)) |
| ~~`F_PREALLOCATE` + bulk zero~~ | **~48–51k pump** — works; setup heavy ([PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md)) |
| ~~Seal-sized ahead-of-write zero (diag)~~ | Prior ~32k **withdrawn** (seal fail + ignored error); product flag ~6k E2E on noisy disk — re-pair quiet |
| Chunk-size sweep / match seal threshold | Tune 16/64/128 MiB |
| **Background prepare segment N+1 / runway** | **Principal steer — next** |
| Linux `fallocate` | Unmeasured |
| Product design note | Now evidence-backed enough to draft |

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

**Update 2026-08-03:** seal-sized watermark is an **opt-in product API**
([PRODUCT_SEGMENT_WATERMARK.md](PRODUCT_SEGMENT_WATERMARK.md)) — still not
default-on. Honest paired measure ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md))
shows watermark ≈ Real after correcting a diag seal-fail cheat; remaining work is
sticky config, default-on gate, and cleaner re-pair on a host with disk headroom.

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
