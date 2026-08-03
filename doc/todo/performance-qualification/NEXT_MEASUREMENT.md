# Next measurement — acknowledgement / seal line

Status: **Seal Fast Lane = architectural accept; paired-median perf gate open**  
Date: 2026-08-04

## Wording (hard)

Chimera / derived enrichment no longer causes **queue backpressure** on the
put path (`max_pending_seals` counts authoritative finalize only).

That does **not** prove enrichment is free of **CPU / disk / cache
interference** while writes continue. Those are separate residuals.

## Gate (principal)

\[
\frac{\operatorname{median}(TPS_{64MiB})}
{\operatorname{median}(TPS_{control})} \ge 0.90
\]

Enrichment **off**, ≥6 reps each, alternating, multi-rotate + exact reopen on
the 64 MiB cell. Stale absolute floor 74.7K (`0.90×83K`) is retired.

## Architectural lock — whole-segment BLAKE3

**Derived, not authoritative.** Sealed authority is
`{durable prefix ‖ segment-summary frame}`; frame CRC/body hashes detect
corruption. Whole-segment BLAKE3 lives in tier placement / segment catalog and
is filled by enrichment after `SealDone` (`CONTENT_HASH_PENDING` until then).
Hot path: `meta_publish_plan` (no pending read on the auth worker).

Evidence: `doc/archive/performance-qualification/2026-08-04-defer-segment-blake3/`.

## Settled facts

- Old ~8–10K was **campaign** TPS, not ack.
- Contemporary high-threshold control (enrichment off, 512 MiB): median
  **~74–78K** across campaigns (machine noise).
- Seal Fast Lane (derived off the lane): arch OK; keep board **`in_review`**.
- Stream-hash auth finalize: paired median ratio **0.878** (FAIL).
- Meta-publish (BLAKE3 deferred): paired median ratio **0.869** (FAIL) — did
  **not** close the ~2 pp gap. Residual is non-hash rotate/publish cost.
- Put-path / write-tail rolling BLAKE3: measured regressions; stay **off**.

## Next developer instruction (freeze)

Paired-median gate still open after deferring derived BLAKE3. Do **not** put
hashing back on the auth or put path. Remaining options: profile
truncate/append-summary/rename/`start_active`/fsync overlap, principal waiver
of the 90% paired floor, or accept arch-only and measure enrichment resource
interference separately. Do **not** return to AWO / append-path tuning until
this lane closes or the principal waives.

## Archives

- `doc/archive/performance-qualification/2026-08-04-ack-finalize-split/`
- `doc/archive/performance-qualification/2026-08-04-seal-interference-control/`
- `doc/archive/performance-qualification/2026-08-04-seal-fast-lane/`
- `doc/archive/performance-qualification/2026-08-04-zero-scan-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-zero-read-auth-seal/`
- `doc/archive/performance-qualification/2026-08-04-paired-median-gate/`
- `doc/archive/performance-qualification/2026-08-04-defer-segment-blake3/`
