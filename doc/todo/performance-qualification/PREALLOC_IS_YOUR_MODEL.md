# Prealloc = your linked-list-of-zeroed-chunks model

**Date:** 2026-08-03  
**Ask:** “that is what I thought we meant under prealloc — zero N MB, push data in, keep the index where we stopped, drop another set like a linked list on disk”

## Yes. Same idea.

| Your words | Residiuum name |
|------------|----------------|
| Zero N MiB runway | Watermark / preparer: capacity + bulk-zero ahead of head |
| Push data in | Puts write frames at advancing `durable_len` |
| Index where we stopped | `durable_len` (+ locators in the primary index) |
| Drop another set when full | Seal active → new active segment (segment chain on disk) |

```text
[ segment 0: zeroed N MiB | data……| head ] → seal
[ segment 1: zeroed N MiB | data……| head ] → seal
…
```

That **is** prealloc as you meant it. Not smash-at-offset-0.

## What we already have vs what still fails TPS

- **Segment chain / head / seal** — already product.  
- **Reserve length (`set_len` / capacity)** — opt-in watermark.  
- **Zero N MiB before puts use it** — shipped as **same-fd full-capacity zero** at create / policy-set / rotate (+ same-fd warm). Puts fail closed; no put-path bulk-zero. See [SAMEFD_FULLZERO.md](SAMEFD_FULLZERO.md).  
- **TPS** — same-fd full zero edged grow@512 (~9.4k vs ~8.6k) but lost to grow@1g (~8.2k vs ~10.7k). **Not default-on.** Offline-diag ~35–50k is not product thr.

## One line

```text
Prealloc = zero a chunk on the writer fd, write forward, remember head, seal and open the next chunk.
Shape is in-tree. TPS did not clear ≫ grow on this peer bed — leave default grow.
```
