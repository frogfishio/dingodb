# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane = architectural accept, paired-median perf gate open**  
Date: 2026-08-04

## Wording (hard)

Chimera / derived enrichment no longer causes **queue backpressure** on the
put path (`max_pending_seals` counts authoritative finalize only).

That does **not** prove enrichment is free of **CPU / disk / cache
interference** while writes continue. Those are separate residuals.

## Gate correction (principal)

The frozen **≥74.7K** absolute floor was `0.90 × ~83K` from an older
high-threshold control. Contemporary high-threshold control on this bed is
**~77–78K**, so the intended accept gate is:

\[
\frac{\operatorname{median}(TPS_{64MiB})}
{\operatorname{median}(TPS_{control})} \ge 0.90
\]

with enrichment **off**, ≥6 reps each, alternating, multi-rotate + exact
reopen on the 64 MiB cell. Do not fetishize “zero read”; fastest correct path
wins (currently background stream-hash finalisation).

## Settled facts

- Old ~8–10K was **campaign** TPS, not ack.
- Real Full ack without mid-run seals: historically **~82–83K**; contemporary
  paired control median **~78.5K** (enrichment off, 512 MiB threshold).
- 64 MiB seals with Chimera **on the seal lane**: **~25K** ack (queue + work).
- Seal Fast Lane (derived off the lane): **~50K** ack @ 64 MiB — arch OK;
  keep board **`in_review`** (not labor-`done`).
- Partial zero-scan (stream-hash + metadata publish): prior peaks **~71K**;
  **paired median campaign** (enrichment off): median **~68.9K** vs control
  median **~78.5K** → ratio **0.878** (**FAIL** ≥0.90).
  Evidence: `doc/archive/performance-qualification/2026-08-04-paired-median-gate/`.
- True zero-read attempts (resident prefix move; write-tail rolling BLAKE3):
  **~44–66K** — regress vs stream-hash; **not** enabled on the hot path.
  Evidence: `doc/archive/performance-qualification/2026-08-04-zero-read-auth-seal/`.

## Next developer instruction (freeze)

Paired-median gate still open after the final measurement-only package.
Stream-hash remains the hot path; do **not** reintroduce resident-prefix or
write-tail hashing. Remaining options: close the ~2–3 pp median gap without
regressions, principal waiver of the 90% paired floor, or accept arch-only
and measure enrichment resource interference separately. Do **not** return to
AWO or append-path tuning until this lane closes or the principal waives.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
- `doc/archive/performance-qualification/2026-08-04-zero-scan-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-zero-read-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-paired-median-gate/`
