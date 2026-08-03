# Zero-Scan Authoritative Seal — evidence

Date: 2026-08-04  
Recipe: APFS · Real Full · payload=8 KiB · logical=256 MiB · concurrency=8 ·
Buffered · AWO=Disabled · seed=42 · seal_threshold=64 MiB (unless noted)

## Control: enrichment off @ 64 MiB

| Metric | Value |
|---|---:|
| Ack TPS | ~64.6K |
| Sealed @ last ack | 3 |
| Enrichment backlog | 0 |
| Reopen exact | yes |

Interpretation: **authoritative finalisation still dominates** (not ~83K).
Concurrent enrichment is not the primary residual after Seal Fast Lane.

See `enrichment-off-final/`.

## High-threshold control (same bed)

| Seal threshold | Ack TPS |
|---|---:|
| 512 MiB (no mid-run seal) | ~81.8K |

## Zero-scan measure (enrichment on, isolated)

| Metric | Value | Gate |
|---|---:|---|
| Ack TPS | **~71.4K** | FAIL (≥ 74.7K) |
| Sealed @ last ack | ≥ 3 | PASS |
| Reopen exact | yes | PASS |

Best observed in this campaign: **~71.4K** (prior run peak ~71.4K).
Still short of the 10%-of-83K floor.

## What changed in code

- `FinalizeSealMeta`: stream-hash pending prefix + append precomputed summary;
  **no frame scan**, no 64 MiB `Vec` returned to the writer.
- Writer rotate stays O(1); catalog apply is constant-time
  (`note_sealed_summary`); durable catalog persist deferred to drain.
- Enrichment: `set_enrichment_enabled`, min-gap isolation on enrich worker.
- Hot-path rolling BLAKE3 was tried and **rejected** for the gate: it drops
  high-threshold ack from ~82K to ~50K. Stream-hash on the auth worker is the
  retained trade-off (one sequential read, zero scan).

## Residual

Gap **~71K → 74.7K** (~5%). Auth worker sequential read of pending still
overlaps live append I/O. True zero-read requires rolling hash off the ack
critical path (background hasher without per-put copies) — not yet meeting
the floor when done naively on the writer.
