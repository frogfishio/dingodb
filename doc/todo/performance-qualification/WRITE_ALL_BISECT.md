# Write-path bisect: what’s expensive inside Discard’s 13×?

**Date:** 2026-08-03  
**Bed:** APFS `/var/tmp` · Mode A · Residiuum-off · `--concurrency 8` · 8 KiB · 256 MiB · seed 42 · seal 512 MiB  
**Question:** Discard ≫ Real — is it `write_all` as a syscall, seek, page-cache copy, or **growing the file**?

## Ladder

| `diag_io` | What it does | ops/s | disk |
|-----------|--------------|------:|-----:|
| **discard** | no OS I/O | **119 019** | ~0 |
| **seekonly** | `seek(durable_len)` only | **128 748** | ~0 |
| **devnull** | `write_all` → `/dev/null` | **124 786** | ~0 |
| **realoverwrite** | `seek(0)` + `write_all` (file stays tiny) | **96 238** | ~0 |
| **realnoseek** | append `write_all`, no seek | **9 759** | ~522 MiB |
| **real** | seek + append `write_all` | **10 115** | ~522 MiB |

Artifacts: [`artifacts/firm-numbers-write-all-bisect-apfs/`](artifacts/firm-numbers-write-all-bisect-apfs/).

## Verdict (decisive)

```text
Discard ≈ SeekOnly ≈ DevNull  ≈ 120–129k   ← seek free; write syscall free
RealOverwrite                 ≈  96k       ← writing into a real fd is cheap if file does not grow
Real ≈ RealNoSeek             ≈  10k       ← APPEND / FILE GROWTH is the wall (~10× vs overwrite)
```

1. **Not seek** — SeekOnly ≥ Discard; RealNoSeek ≈ Real.
2. **Not “write_all” as an abstract syscall** — DevNull ≈ Discard.
3. **Not “touching a regular file” in general** — RealOverwrite stays near the cook band (~96k).
4. **Yes: extending / appending the active segment** — only the growing-file cells fall to ~10k.

So Discard’s ~13× is almost entirely **append-growth cost** on the active segment (new pages / extent allocation / page-cache for a file that balloons to ~522 MiB), not hashing and not seek and not write-size coalesce.

## What this is not

- Not a product fix. Overwrite destroys durability; it is a bisect only.
- Not “APFS is slow” as media — Buffered overwrite is fast on the same volume.
- Not proof of the SQLite gap yet — SQLite also grows a DB file; why *their* growth is cheaper per Mode A ack is the next question (page reuse, WAL shape, fewer bytes/op, etc.).

## Next bisects (optional)

- ~~Preallocate (`fallocate` / `truncate` to final size) then append~~ → done: see [PREALLOC_SPIKE.md](PREALLOC_SPIKE.md) (sparse no; page-touch ~4×).
- Fewer / larger appends at the product layer (already tried 64 KiB coalesce ≈ Real — so growth still happens, just in bigger chunks; coalesce does not avoid extending the file).
- Compare bytes written per ack vs SQLite (Residiuum on-disk ~522 MiB for 256 MiB logical — ~2× amplification).

## How to re-run

```sh
BIN=target/release/residiuum-testrig
for d in discard seekonly devnull realoverwrite realnoseek real; do
  $BIN peer-pump -w /var/tmp/bisect-$d --engine residiuum --mode A \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 \
    --seal-threshold 512M --concurrency 8 --diag-io $d --json-out
done
```

Diagnostic only — not a product SLO.
