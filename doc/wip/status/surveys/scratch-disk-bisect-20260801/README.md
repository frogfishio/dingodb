# Disk detach bisection (Scratch, 2026-08-01)

**Diagnostic only.** Goal: answer “is Mode A micro wall still the disk (small writes / caches)?” by measuring the **full Buffered put pipeline with disk detached**.

## Method

| Step | What is measured |
|------|------------------|
| Blake / encode | pure CPU format path |
| Memory put | store index path, **no** segment tail write |
| **Buffered + Discard** | full Buffered put; `durable_len` advances; **no `write_all`** |
| **Buffered + DevNull** | full path; `write_all` to reused `/dev/null` (syscall/VFS, no media) |
| **Buffered + Real** | full path; real active file under work volume |
| raw → null vs raw → file | pure `write_all` without Residiuum |

Store API (diagnostic only, never product default):

- `DiagnosticIoSink::{Real, Discard, DevNull}`
- `Store::set_diagnostic_io_sink`

Micro uses **512 MiB seal** so mid-run seals do not dominate.

## Scratch result (20k × 8 KiB)

| Phase | ops/s | wall_ms | Notes |
|-------|------:|--------:|-------|
| blake3 | ~246k | 81 | |
| encode_frame_into | ~235k | 85 | |
| raw → `/dev/null` | ~3.7M | 5 | pure syscall |
| raw → Scratch file | ~960k | 21 | page cache + media |
| **Memory put** | **~981k** | 20 | |
| **Buffered Discard** | **~167k** | 120 | disk fully detached |
| **Buffered DevNull** | **~159k** | 126 | ≈ Discard |
| **Buffered Real** | **~135k** | 149 | real file |

**Ratios (v2):** Real/Discard ≈ **0.81** · DevNull/Discard ≈ **0.95** · Discard/Memory ≈ **0.16**

## Read

1. **Memory → Discard (~6× slowdown)** is almost all **store-side** work (append Blake+copy ~64% of Discard wall, index, etc.). Disk is **not** in this gap.
2. **Discard ≈ DevNull** → adding a real `write_all` syscall without media barely moves the needle.
3. **Discard → Real (~19% slower)** → real file / page cache adds **some** cost (~24 ms write sum vs ~0.3 ms Discard), but **not** the main story. Real is still the same order as Discard (~135k vs ~167k).
4. Therefore on this micro: **it is still not “the disk hates small writes” as the primary limiter.** Primary limiter is **pre-disk store pipeline** (append/encode/index). Disk is a **secondary ~20%** tax on wall when comparing Discard vs Real.

Long multi-seal peers remain a separate story (`seal_rotate`); this ladder turns seals off.

## Artifacts

- `phase-bench-v2.txt` / `phase-bench-v2.json` — fair DevNull (reused handle)
- `phase-bench.txt` — first cut (DevNull open-per-put inflated write time; ignore for DevNull)

## Re-run

```sh
cargo build -p residiuum-testrig --release
./target/release/residiuum-testrig phase-bench \
  -w /Volumes/Scratch/TEST/residiuum-disk-bisect-YYYYMMDD \
  --ops 20000 --payload-size 8192
```
