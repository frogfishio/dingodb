# Firm numbers — concurrent feed + multicore cook

Status: **labor measure (self_check) — not package accept**  
Date: 2026-08-03  
Depends on: concurrent feed ([FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md));
CPU wall ([FAST_DISK_CPU_WALL.md](FAST_DISK_CPU_WALL.md))

## Hypothesis

Concurrent Mode A (~13k vs SQLite ~30k) is a **CPU wall**. More cook cores
should raise Residiuum thr.

## What we changed

1. **Harness:** same as concurrent — `--concurrency 8`, Mode A, 8 KiB, 256 MiB,
   APFS `/var/tmp`, seed 42.
2. **Cook:** `RESIDIUUM_COOK_PARALLELISM=1|4|8`.
3. **Store:** AWO collector flush was bytes-keyed **serial** cook; labor wired
   UTF-8 batches with `cook_parallelism > 1` into the existing str parallel-cook
   path so Adaptive/Static *could* use multicore on microbatches.

## Numbers (acked puts/s)

| Cell | cook1 | cook4 | cook8 |
|------|------:|------:|------:|
| SQLite A c=8 | **~26 800** (one control) | — | — |
| Residiuum-off | **~14 100** | ~13 100 | ~13 600 |
| Residiuum Static | ~13 600 | ~13 200 | ~13 000 |
| Residiuum Adaptive | ~13 500 | ~12 800 | ~12 900 |

## Verdict

**Multicore cook does not break the concurrent Mode A ceiling.**

- Best Residiuum ≈ **~14k** (off cook1) vs SQLite ≈ **~27–30k**.
- cook4/8 are **flat or slightly worse** (thread overhead / still-serial
  append+index under the store lock).
- Off never fans cook (still one key per `put_many` under the mutex).
- Adaptive/Static can fan cook on collected batches after the wire-up — still
  no thr win on this long peer.

So: we have hit a wall that **more Blake workers do not move** on this feed.
Likely remaining serial work: store mutex / ordered install / index / seal —
not “forgot to set cook=8.”

**Principal follow-on:** wall is past chew → **how we write**. We are **not**
SQLite 4 KiB pages; we append variable hashed frames. See
[SQLITE_PAGES_VS_RESIDIUUM_FRAMES.md](SQLITE_PAGES_VS_RESIDIUUM_FRAMES.md).

Matches PARKED lesson: parallel cook helps **short batch-rich micros**; long
peer concurrent Mode A is a different wall.

## Recipe

```bash
C=8
for N in 1 4 8; do
  for MODE in disabled static adaptive; do
    RESIDIUUM_COOK_PARALLELISM=$N target/release/residiuum-testrig peer-pump \
      -w /var/tmp/ra --engine residiuum --mode A --awo-mode $MODE \
      --concurrency $C --target-bytes 256M --payload-size 8192 --seed 42 \
      --min-free 0 --json-out
  done
done
```

Artifacts: `artifacts/firm-numbers-concurrent-multicore-apfs/`.

## Non-claims

Not package accept. Not “cores never help.” Not Scratch. Not AWO default-on.
