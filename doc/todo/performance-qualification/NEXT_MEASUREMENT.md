# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane in tree; residual = authoritative finalize re-read**  
Date: 2026-08-04

## Settled facts

- Old ~8–10K was **campaign** TPS, not ack.
- Real Full ack without mid-run seals: **~82–83K** (~80% of Discard ~104K).
- 64 MiB seals with Chimera on the seal lane: **~25K** ack.
- Seal Fast Lane (derived off the lane): **~50K** ack @ 64 MiB with rotations +
  coverage-exact reopen — **not yet** within 10% of ~83K.

## Active residual

Authoritative worker still fully reads pending segment bytes to publish. That
contends with live appends. Next package: append-summary / no-full-re-read
finalize. Pause AWO and append tuning until then.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
