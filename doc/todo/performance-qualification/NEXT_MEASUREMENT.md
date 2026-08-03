# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane = architectural accept, performance gate open**  
Date: 2026-08-04

## Wording (hard)

Chimera / derived enrichment no longer causes **queue backpressure** on the
put path (`max_pending_seals` counts authoritative finalize only).

That does **not** prove enrichment is free of **CPU / disk / cache
interference** while writes continue. Those are separate residuals.

## Settled facts

- Old ~8–10K was **campaign** TPS, not ack.
- Real Full ack without mid-run seals: **~82–83K** (~80% of Discard ~104K).
- 64 MiB seals with Chimera **on the seal lane**: **~25K** ack (queue + work).
- Seal Fast Lane (derived off the lane): **~50K** ack @ 64 MiB — arch OK,
  **≥74.7K gate failed**. Keep board **`in_review`** (not labor-`done`).
- Enrichment-off control @ 64 MiB: **~46K** — authoritative finalize dominates
  vs enrichment resource contention alone (`authoritative_finalisation_dominant`).
- Partial zero-scan (stream-hash + metadata publish, no frame scan): **~71K**
  ack @ 64 MiB, multi-rotate, exact reopen — still short of **≥74.7K**.
- True zero-read attempts (resident prefix move; write-tail rolling BLAKE3):
  **~44–66K** — regress vs stream-hash; **not** enabled on the hot path.
  Evidence: `doc/archive/performance-qualification/2026-08-04-zero-read-auth-seal/`.

## Next developer instruction (freeze)

The ≥74.7K gate is still open. Zero-read hashing on the put path and keeping
the full prefix resident both lose to stream-hash finalize. Remaining work is
to close the ~71K→74.7K gap **without** reintroducing those regressions
(rotation/fsync/start_active cost, seal-worker CPU interference, or a waived
floor). Do **not** return to AWO or append-path tuning until this lane closes
or the principal waives.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
- `doc/archive/performance-qualification/2026-08-04-zero-scan-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-zero-read-auth-seal/`
