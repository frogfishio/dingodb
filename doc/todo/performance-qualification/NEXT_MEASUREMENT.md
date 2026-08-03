# Next measurement — acknowledgement / seal line

Status: **Zero-Scan Auth Seal in tree; ~71K ack @ 64 MiB (gate 74.7K not met)**  
Date: 2026-08-04

## Settled facts

- Old ~8–10K was **campaign** TPS, not ack.
- Real Full ack without mid-run seals: **~82–83K** (~80% of Discard ~104K).
- 64 MiB seals with Chimera on the seal lane: **~25K** ack.
- Seal Fast Lane (derived off the lane): **~50K** ack @ 64 MiB.
- Enrichment-off control @ 64 MiB: **~65K** — authoritative finalize still
  dominates (not enrichment contention alone).
- Zero-Scan (stream-hash + metadata publish, no frame scan): **~71K** ack @
  64 MiB, multi-rotate, coverage-exact reopen — **still short of ≥74.7K**.

## Active residual

Auth worker still performs one sequential read of the pending prefix to hash.
Hot-path rolling BLAKE3 meets “zero-read” but drops high-threshold ack to
~50K and is rejected for the gate. Next: background rolling hash that does
not charge the put mutex / ack path (or otherwise close the ~5% gap).

Pause AWO and append tuning until the 74.7K floor is met or explicitly waived.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
- `doc/archive/performance-qualification/2026-08-04-zero-scan-auth-seal/`
