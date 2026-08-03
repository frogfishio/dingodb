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
- Enrichment-off control @ 64 MiB: **~65K** — authoritative finalize still
  dominates vs enrichment resource contention alone.
- Partial zero-scan (stream-hash + metadata publish, no frame scan): **~71K**
  ack @ 64 MiB, multi-rotate, exact reopen — still short of **≥74.7K**.

## Next developer instruction (freeze)

Run the 64 MiB control once with derived enrichment disabled during the
acknowledgement window. Then implement **zero-read** authoritative sealing:
maintain summary and hash state incrementally, append the precomputed summary
at rotation, return compact metadata only, and remove the 64 MiB
`sealed_bytes` transfer and writer-side rescan. Re-run the identical control
with enrichment off/on. Acceptance: **≥74.7K** ack TPS, multiple rotations,
exact reopen.

That cleanly distinguishes:

1. authoritative reread/rescan cost;
2. concurrent enrichment resource contention;
3. any remaining rotation/fsync cost.

**Do not return to AWO** or append-path tuning until this lane closes or the
principal waives the floor.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
- `doc/archive/performance-qualification/2026-08-04-zero-scan-auth-seal/`
