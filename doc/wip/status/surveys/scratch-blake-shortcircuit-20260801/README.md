# Blake short-circuit (Scratch, 2026-08-01)

**Diagnostic only.** Isolate BLAKE3 body hash vs rest of append_frame.

## Method

- `residiuum_format::set_diagnostic_skip_body_hash(true)` / `Store::set_diagnostic_skip_blake(true)`
- Still: frame prefix/env/body **memcpy**, suffix CRCs, real-disk `write_all`
- Body-hash field forced to zeros → frames **fail verify** (not product)

## Result (20k × 8 KiB)

| Phase | ops/s | ≈ µs/op | vs full |
|-------|------:|--------:|--------:|
| full Buffered | **~139k** | **7.2** | 1.0× |
| **no-blake** (copy+write, no Blake) | **~264k** | **3.8** | **~1.9×** |
| no-append (skip entire cook+write) | **~532k** | **1.9** | ~3.8× |
| pure Blake alone | ~300k | 3.3 | — |

### Wall split (approx. from µs/op deltas)

| Piece | µs/op | % of full wall |
|-------|------:|---------------:|
| **Blake body hash** | **~3.4** | **~47%** |
| memcpy + framing + file write (after Blake) | ~1.9 | ~27% |
| prep + env encode + index (rest) | ~1.9 | ~26% |

## Read

**Yes — it is Blake**, as the largest single cook cost (~half of Mode A micro wall at 8 KiB).

Not “only Blake”: after removing it, you still pay ~half the remaining path (copy + write + prep).  
**How to handle it if product agrees:** integrity cost is O(payload); options are faster hash path, amortize over batches, or a **named** weaker mode — never silent skip in product paths.

## Artifacts

`phase-bench.txt` · `phase-bench.json`
