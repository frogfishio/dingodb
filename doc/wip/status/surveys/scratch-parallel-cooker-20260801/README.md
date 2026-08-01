# Parallel record cooker (Option C) — Scratch 2026-08-01

**Not** naive multi-store clones. **Yes** parallelising the full **record cook**
(item envelope + frame encode including BLAKE3), then ordered install + one tail write.

## API

```rust
store.set_cook_parallelism(4); // default 1 = serial
store.put_many(&items, DurabilityMode::Buffered)?;
```

- Single-shard `put_many` only (batch path).
- Workers cook independent frames; writer installs with `append_preencoded_frame`.
- Integrity preserved (real Blake). Diagnostic short-circuits unchanged.

## Results (20k × 8 KiB, batch=128, real Scratch, 512 MiB seal)

| Phase | ops/s | vs cook1 |
|-------|------:|---------:|
| Mode A single-put (batch=1) | ~135k | — |
| **put_many cook1** (serial cook) | **~184k** | 1.00× |
| **put_many cook2** | **~257k** | **~1.4×** |
| **put_many cook4** | **~326k** | **~1.8×** |

(8000-op ladder: cook1 184k · cook2 257k · cook4 326k.)

CPU samples on short cook4 phase still noisy; scaling is real in wall ops/s.

## Read

1. **Parallelising the whole cooker works** — not Blake-only micro-tasks.
2. **~1.8× with 4 workers** vs serial cook on the same batch path (not 4×: install + index + write remain serial).
3. Explains why **naive 4-store / 4-shard** was weak: wrong work unit and shared contention.
4. Mode A single-put still single-threaded cook; Option C pays when there is a **batch** to fan out.

## Artifacts

`phase-bench.txt` · `phase-bench.json`
