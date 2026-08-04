# CSE-3 Stage 2 step 9 — product-API CompactShadow campaign (qual store)

Labor evidence (2026-08-04). Mode armed only via prepare/activate/reopen
(no `CSE3_STEP7_DUAL_STREAM`, no test-only dual attach).

## Result (2 GiB / 64 MiB seals, Buffered puts)

| Metric | Value |
|---|---|
| Ack TPS (puts only) | **23278** |
| Lifecycle TPS (puts + seals + enrichment) | 11845 |
| Shadow finalize | 76.90 seg/s |
| Compact amp | 0.75% |
| Shadow amp | 100.1% |
| Gates | frontier, verified `.rsh`, P★ recovery, reopen, continue, locator-only |

Ack is in the ~28K band, established independently of Step 7 harness numbers.
Lifecycle is lower because product enrichment (Hydra/Chimera) remains sync on
`seal_active`.

## Bugs fixed to unblock scale

- `segment_seq` under-count on resume: active files omit seq in the filename;
  resume/seal now bump `segment_seq` so the next mint cannot overwrite a
  just-sealed CompactShadow segment.

## Posture

Release/default `Store::create` remains **Materialized**. Qual-store activation
is the only CompactShadow product path until principal flips the default.
