# Answer — MQ-style raw / own filesystem vs Residiuum

**Date:** 2026-08-03  
**Card:** `2b87f88d`  
**Ask:** “are you aware that some systems I think MQ Series have their own filesystems on raw hardware disks?”

## Short answer

**Yes.** Serious queue/DB engines often **own the media layout** (raw LUN, dedicated volume, or a filesystem they fully control and pre-shape) so the OS is not inventing extents under every append.  
Residiuum **today does not**. We write ordinary files under a host FS (`active.residiuum` / sealed segments). That is why APFS first-touch / grow-on-append showed up as ~10k vs pre-touch ~35–50k. Watermark is a **userspace** attempt to pre-shape those files — not a private disk FS.

## What that MQ-class pattern is (roughly)

Systems in that family (IBM MQ linear logs on dedicated volumes, classic DBMS on raw/ASM, etc.) typically:

- take a **block device or dedicated volume** they treat as theirs  
- lay out logs/pages with **their** allocator (fixed regions, preformatted, sequential)  
- avoid “surprise” host-FS sparse growth on the hot path  

Exact MQ product packaging varies by era/edition (linear vs circular logging, filesystem vs older raw-LV stories). The **idea** you are pointing at is real: **own allocation, don’t rent the filesystem’s growth tax.**

## Where Residiuum sits

| Layer | Residiuum now |
|-------|----------------|
| Byte owner | Host filesystem files under a store directory |
| Growth | Default `GrowOnAppend`; opt-in `Watermark` prealloc+zero runway |
| PQH / product safety | Spec **forbids** opening raw block devices for write in the qualification harness ([PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md)) |
| Salvage story | Node dirs stay plain `residiuum-store` trees — intentional |

So: aware of the pattern; **not** implementing “ResidiuumFS on `/dev/diskX`” as current labor.

## How this maps to the thr fight

```text
MQ-class raw / owned layout     →  allocation is product-owned; FS tax minimized by design
Residiuum watermark             →  still files; pre-touch capacity inside the file
Residiuum GrowOnAppend          →  files; host FS extends under each append (the ~10k wall)
```

Watermark ≈ “steal the useful bit of the raw-layout idea (pages exist before hot writes)” **without** taking on a custom filesystem product.

## What this is not asking us to do next

- Do not spin up a Residiuum filesystem on raw disks as the answer to Mode A thr unless principal opens that lane (huge scope: installer, permissions, salvage, CI, Windows/macOS).  
- Do decide whether, **on files**, default-on watermark/pre-touch is worth the space amp — that is the live decision from the last cards.

## Non-claims

Not that Residiuum must become MQ. Not that raw devices are required for SQLite parity. Not that watermark equals a private FS.
