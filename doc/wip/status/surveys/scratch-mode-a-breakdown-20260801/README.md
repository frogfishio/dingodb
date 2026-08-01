# Mode A put-path breakdown (instrumentation)

**Diagnostic only.** Scratch run 2026-08-01 · `phase-bench --ops 20000 --payload-size 8192`.

## Goal

Higher-resolution timers on the Buffered **single-put (Mode A)** path so we can
eliminate hogs before fine optimisations (arena, dual-index, etc.).

## Probe extensions (this work)

`BoundaryProbe` / `Store` now time (when `enable_boundary_probe()`):

| Phase | What |
|-------|------|
| **put_prep** | `ensure_active` + `maybe_auto_seal` + item/event id mint + envelope subject setup |
| **encode_envelope** | `encode_item_envelope` CBOR |
| **append_encoded_frame** | `segment.append` / `encode_frame_into` (Blake + copy into segment buffer) |
| **publish_visibility** | dual-index `apply_durable_event` |
| **put_post** | collection note + rate-limited derived checkpoint touch |
| **file_write** | `write_segment_tail` `write_all` |
| **file_sync** | Durable only (`n=0` on Buffered) |

Harness: `residiuum-testrig phase-bench` prints MODE_A breakdown with **% of wall**.

## Results (Scratch, 20k × 8 KiB Buffered batch=1)

| Phase | sum_ms | mean µs/op | **% wall** |
|-------|-------:|-----------:|----------:|
| **put_prep** | **340.5** | **17.0** | **~65%** |
| append_frame | 92.6 | 4.6 | ~18% |
| file_write | 47.7 | 2.4 | ~9% |
| encode_envelope | 10.7 | 0.5 | ~2% |
| publish_index | 5.2 | 0.3 | ~1% |
| put_post | 0.5 | ~0 | ~0% |
| file_sync | 0 | — | 0% |
| other (harness key format, Instant, …) | ~30 | — | ~6% |
| **wall** | **527** | — | 100% |
| **accounted** | **497** | — | **~94%** |

Buffered rate this run: **~38k ops/s** (~296 logical MiB/s microbench; peer-A 256 MiB was lower due to longer run/seal/RSS).

## Process of elimination

1. **Not Blake-bound** — pure Blake ~265k ops/s; encode_env only 2% wall.
2. **Not dual-index publish alone** — publish ~1% wall (locator-first already cheap here).
3. **Not allocator temps alone** — encode_env is small; arena is still useful but not the 65% hog.
4. **Not fsync** — Buffered file_sync n=0.
5. **Biggest hog: put_prep (~65%)** — mostly **`maybe_auto_seal` / ensure_active / id path** before any frame encode. Next resolution step: split prep into seal-check vs ensure vs id-mint (or sample seal hit rate).
6. **Second: append_frame (~18%)** — frame Blake+copy into segment buffer; worth micro-opts after prep.
7. **Third: file_write (~9%)** — per-put seek+write_all on Mode A; Mode B amortizes this.

## Next optimisations (ordered by this data)

1. **Cheapen / skip work in put_prep** when under seal threshold (hot path: seal check must be O(1) and branch-predictable; avoid heavy work every put).
2. **append_frame** micro (reuse, reserve segment to seal size).
3. **file_write** batching only if Mode A product allows (else leave to Atomics/Mode B).
4. **Scratch/reuse** for envelope temps (2% — still free money, not the main story).

## Re-run

```sh
cargo build -p residiuum-testrig --release
target/release/residiuum-testrig phase-bench \
  -w /Volumes/Scratch/TEST/residiuum-mode-a-breakdown-YYYYMMDD \
  --ops 20000 --payload-size 8192
```
