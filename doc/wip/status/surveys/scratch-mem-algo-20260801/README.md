# Memory / algorithm view of append_frame (2026-08-01)

## Two options

| Hypothesis | Evidence |
|------------|----------|
| **1. “It’s the CPU after all”** | **Partly.** Pure Blake is ~200–290k ops/s for 8 KiB; full put ~135k. Process CPU on full put is ~50–60% of **one** core, not 100%, and not machine-wide. |
| **2. “How we use memory / algorithm”** | **Yes as multi-pass payload traffic**, not as “RAM is full” or “millions of hidden syscalls.” |

## What append_frame does to memory (per 8 KiB put)

```text
caller body ──read──► BLAKE3 (hash arithmetic + sequential load)
caller body ──copy──► ActiveSegment Vec (prefix | env | body | suffix)
segment tail ──write_all──► OS page cache / disk  (~frame-sized)
write-through: drop RAM prefix (capacity kept)
```

Rough bytes streamed per put:

| Pass | Bytes |
|------|------:|
| Blake input read | 8 192 |
| memcpy into segment | 8 192 |
| write_all of frame | ~8 300+ |
| **Total class** | **~24 KiB / put** |

At ~135k ops/s → ~**3.3 GB/s** aggregate traffic class.  
That is **busy memory bandwidth**, not a joke, and not “DRAM is broken.”

**Not millions of syscalls on the cook path:** Blake is pure userland. Tail is ~1 seek + 1 write per put (~270k syscalls/s at 135k ops/s) — real, but Discard (0 writes) still only ~167k ops/s.

## Algorithm tweak tried

`encode_frame_into`: **hash-while-copy** (one sequential body pass instead of hash-then-extend).

Integrity unchanged (same BLAKE3). Format tests pass.  
**Result:** encode/full rates **within noise** of pre-tweak (~232k / ~135k).  

**Meaning:** the second full body *read* was not the main tax; **Blake work itself** (load + mix) dominates. Smarter algorithm still has to pay format-mandated integrity unless the wire contract changes.

## Smarter directions (honest ranking)

| Idea | Saves | Spec / risk |
|------|-------|-------------|
| Hash-while-copy (landed) | 1 payload read | Safe; small win in practice |
| Vectored write (prefix/env/body/suffix) without dual full buffer | Segment RAM + maybe cache | Needs careful write-through / crash |
| Batch many frames then one cook+write | Amortize fixed costs | Product API (txn/batch) |
| Weaken/skip Blake on put | Big | **Guarantee change — no silent** |
| SIMD Blake / hardware | Blake wall | Platform; still O(payload) |

## Bottom line

- Not option 1 alone (“CPU pegged”).  
- Option 2 is right as **algorithmic multi-pass over every payload** (integrity + dual-buffer + write), which *is* memory traffic + CPU arithmetic fused.  
- It is **not** mysterious syscalls or slow DRAM in the abstract.  
- Next product leverage is **amortize or restructure** the cook (batch/txn, write path shape), not micro-tweaking one memcpy.
