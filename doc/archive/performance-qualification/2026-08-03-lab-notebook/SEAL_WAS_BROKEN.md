# Answer — what “the seal was broken” means

**Date:** 2026-08-03  
**Card:** `482e8a11`  
**Ask:** “what do you mean the seal was broken?”

## Not this

It does **not** mean:

- Durable puts were lying / CSQ receipts were fake mid-run  
- The store silently corrupted user data and called it verified  
- Product `--segment-growth watermark` couldn’t seal  

“Broken seal” here = **end-of-run `seal_active()` failed on the diagnostic watermark path**, and the harness **hid** that failure.

## What “seal” is in this test

Peer-pump Mode A does many Buffered puts, then at the end calls:

```text
store.seal_active()   // rotate active segment → sealed file + indexes (chimera, …)
```

That seal work is **inside** the timed window (`t0` … `elapsed`). So a successful seal costs real wall time (rewrite/rename + derived indexes). A failed seal that is ignored costs almost nothing.

## What went wrong (diag `realpreallocwm` only)

1. **Setup bug.** After create, diag watermark prealloc **bulk-zeroed from file offset 0**, including the bytes that held the on-disk **segment descriptor**.  
   In-memory state was fine, so puts kept succeeding. On disk, offset 0 was zeros.

2. **Seal reads the file.** End-of-run seal (write-through path) does `fs::read(active)` and scans for a contiguous verified prefix starting at 0. Zeros at the start → scan finds nothing useful →  

   `Err(CorruptMeta("pending segment empty or unreadable"))`

3. **Harness swallowed it.** Peer-pump does `let _ = store.seal_active();` — error discarded. No sealed segment file, no chimera build, timer stops “early.”

That is the “broken seal”: **seal attempted, failed, failure ignored → thr inflated.**

## Why product looked “worse”

Product watermark **protected** the durable prefix (zeroed *ahead* of live bytes). Seal **succeeded**, paid rename + chimera, and landed ~6–8k on the same recipe while the broken diag path printed ~32k.

After fixing diag zeroing + seal truncate-on-prealloc, honest diag ≈ product ≈ Real grow ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md)).

## One picture

```text
puts (Buffered) ──► [timer still running] ──► seal_active() ──► chimera/indexes
                         ▲
                         │
              broken diag: seal Err(_) ignored here
              → skip expensive tail → fake high ops/s
```

## Evidence

| Proof | What it showed |
|-------|----------------|
| Probe before fix | diag `seal_active => Err(...empty or unreadable)`; product `Ok(())` |
| Diag store tree | no `segments/*.residiuum`; fat `active` left behind |
| Product store tree | sealed segment + new active + chimera |
| `peer.rs` | `let _ = …seal_active()` at end of concurrent pump |
| Fix | zero ahead of `durable_len`; seal truncates prealloc before summary (`store.rs` + `tests/wm_seal_probe.rs`) |

## Non-claims

Not that all historical seals were broken. Not that product durability labels were wrong. Not that watermark is useless — only that the **~32k meter reading** was dishonest.
