# Firm numbers — concurrent (server-async) Mode A feed

Status: **labor measure (self_check) — not package accept / not Scratch PEER**  
Date: 2026-08-03  
Depends on: [EMBEDDED_SYNC_VS_SERVER_ASYNC.md](EMBEDDED_SYNC_VS_SERVER_ASYNC.md)

## What we ran

Same Mode A knobs as FN-2 (8 KiB, 256 MiB logical, Buffered, seed 42, APFS
`/var/tmp`), but **`--concurrency 8`**: eight client threads, each still
per-thread QD=1. Overlap = eight in-flight presents — web-tier-like feed so
AWO can microbatch.

Harness: `peer-pump --concurrency N` (`feed_shape=server_async_concurrent`).

## Numbers (acked puts/s)

| Cell | Embedded sync FN-2 (c=1) | **Concurrent c=8** |
|------|-------------------------:|-------------------:|
| SQLite A | ~29 200 | ~**29 700** |
| Residiuum-off | ~12 600 | ~**13 200** |
| Residiuum Static | ~**2 460** | ~**13 100** |
| Residiuum Adaptive | ~**2 470** | ~**13 600** |

```text
Adaptive X (server-async feed)  ≈  13 600 acked puts/s   (this bed)
≈ off; ≫ embedded-sync Adaptive (~2.5k)
≪ SQLite (~30k) — CPU/cook wall remains
```

## Verdict

1. **Principal was right about feed sabotage.** Concurrent feed removes the
   ~2.5k delay tax; Static/Adaptive return to the ~off band.
2. **AWO does not beat Residiuum-off** on this concurrent Buffered Mode A cell
   (Adaptive ≈ off, slight noise). Microbatch is no longer self-owning; it also
   is not a free thr win vs natural concurrent puts here.
3. **Still lose to SQLite ~2.2×** — same CPU wall as embedded-sync off vs SQLite.
4. Do **not** quote FN-2 Adaptive ~2.5k as server-async smart mode.

**Multicore follow-up:** concurrent × cook1/4/8 — **no thr lift** (still
~13–14k). See [FIRM_NUMBERS_CONCURRENT_MULTICORE.md](FIRM_NUMBERS_CONCURRENT_MULTICORE.md).

## Recipe

```bash
cargo build -p residiuum-testrig --release
BIN=target/release/residiuum-testrig
C=8
for MODE in disabled static adaptive; do
  $BIN peer-pump -w /var/tmp/ra-$MODE --engine residiuum --mode A \
    --awo-mode $MODE --concurrency $C \
    --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 --json-out
done
$BIN peer-pump -w /var/tmp/sa --engine sqlite --mode A --concurrency $C \
  --target-bytes 256M --payload-size 8192 --seed 42 --min-free 0 --json-out
```

Artifacts: `artifacts/firm-numbers-concurrent-apfs/`.

## Non-claims

Not Scratch. Not package accept / default-on. Not “Adaptive wins multi-user.”
Not that embedded-sync Mode A is invalid — it remains the autocommit peer; it
is the wrong exclusive bed for AWO thr claims.
