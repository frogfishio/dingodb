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

## Results

### A. With mid-run seals mis-binned into prep (first read)

`put_prep` looked like **~65%** until seal wall was timed separately.

### B. Corrected breakdown (seal timed as `seal_rotate`)

20k × 8 KiB, **64 MiB** seal (2 rotates mid-run):

| Phase | **% wall** | Notes |
|-------|----------:|-------|
| **seal_rotate** | **~65%** | n=2 full seal/rotate events |
| append_frame | ~19% | Blake + copy into segment |
| file_write | ~6% | per-put seek+write_all |
| encode_env / publish / prep | ~4% | hot path small once seals separated |
| other | ~6% | harness keys, etc. |
| Buffered rate | **~42k ops/s** | wall ~475 ms |

### C. No mid-run seal (seal threshold 512 MiB)

| Phase | **% wall** |
|-------|----------:|
| **append_frame** | **~53%** |
| file_write | ~17% |
| encode_env | ~6% |
| prep | ~3% |
| publish | ~3% |
| other | ~18% |
| **Buffered rate** | **~108k ops/s** (~843 logical MiB/s microbench) |

### D. Hygiene opts (still CSQ-safe)

| Change | Intent |
|--------|--------|
| Thread-local **CSPRNG pool** (`ids.rs`) | Amortize `getrandom` syscalls; entropy still OS-only |
| **Cached `now_ns`** (Instant interpolate, ~1 ms OS refresh) | Avoid `SystemTime::now()` every put |
| Memory put after opts | **~1.0M ops/s** (was ~0.58M) |

### E. Peer-A 256 MiB logical (Scratch)

| Seal threshold | ops/s | Logical MiB/s |
|---------------:|------:|-------------:|
| 64 MiB (Campaign F-like) | ~10.2k | ~80 |
| 512 MiB (fewer mid seals) | ~8.6k | ~67 |

Larger threshold **did not** help the long peer run (RSS/cache pressure from a multi-hundred-MiB active segment). Microbench without seals shows the **ceiling**; production Mode A needs **faster seals** and/or **write-through**, not only “never seal.”

## Process of elimination (updated)

1. **Not Blake / encode_env alone** for long runs with default seal.
2. **Not dual-index publish** (~1–3%).
3. **First “prep 65%” was seal amortized into prep** — fixed accounting.
4. **True long-run hog: segment seal/rotate** (~65% when it fires).
5. **True continuous Mode A hog (no seal): append_frame** (~53%), then file_write (~17%).
6. Alloc/arena still free money on the small %s, not the main bar.

## Next optimisations (ordered)

1. **Faster Buffered seal** (already rename-seal; next: stream/mmap finalize, less dual-copy, less derived work on seal).
2. **append_frame** micro (reserve, less realloc) for continuous put.
3. **Write-through active segment** so large seal thresholds do not thrash RAM on peer-scale runs.
4. Scratch/reuse for envelope temps (low %).

## Re-run

```sh
cargo build -p residiuum-testrig --release
target/release/residiuum-testrig phase-bench \
  -w /Volumes/Scratch/TEST/residiuum-mode-a-breakdown-YYYYMMDD \
  --ops 20000 --payload-size 8192
```