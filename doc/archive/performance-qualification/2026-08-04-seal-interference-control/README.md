# Seal Interference Control — 2026-08-04

Status: **measured** (measurement only — no sealing optimisation)  
Evidence: `artifacts/` under this directory (outside active `doc/todo/`).

Harness: `residiuum-testrig seal-interference-control`

## Recipe

```text
APFS · Real Full · payload=8 KiB · logical_data=256 MiB · concurrency=8
Buffered · AWO=Disabled · seed=42
Seal thresholds: 64 MiB (baseline) · 512 MiB · 1 GiB
```

## Evidence table

| Seal threshold | Ack TPS | Ack time | Pending seals @ ack | Sealed @ ack | Drain ns | Final seal ns | Catalog ns | Hydra ns | Chimera ns | Campaign TPS | Reopen exact |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 64.00 MiB | 24987 | 1.31 s | 2 | 2 | 1089012709 | 9824458 | 26607125 | 12104125 | 60277291 | 8097 | yes |
| 512.00 MiB | 82732 | 0.40 s | 0 | 0 | 291 | 298566542 | 180144625 | 176706667 | 2131519417 | 6983 | yes |
| 1.00 GiB | 82649 | 0.40 s | 0 | 0 | 167 | 294719875 | 177536000 | 177602708 | 1971406667 | 7128 | yes |

## Interpretation (decisive)

Ack throughput **jumps significantly** above ~22.6K when the seal threshold
exceeds the workload (**~83K** at 512 MiB / 1 GiB vs **~25K** at 64 MiB).

→ **Seal-pipeline interference is suppressing live writes.** Next optimisation
target is sealing / lifecycle — not yet the raw append path alone.

At last acknowledgement under 64 MiB: **2 pending seals inflight** and
**2 sealed segments** already exist; end-of-run `seal_active` spend is dominated
by **`drain_lifecycle` (~1.09 s)**. With no mid-run seals (512 MiB / 1 GiB),
drain is ~0 and the final seal cost shifts into **Chimera (~2.0–2.1 s)** plus
final active publish / catalog / Hydra.

## Corrected prior conclusions (ack/finalisation matrix)

- Old “~8–10K write TPS” was **campaign** TPS (ack + seal + close + reopen + verify), not acknowledgement.
- Acknowledged Real Full ~22.6K; skip-index live publish ~27K (~19%); Discard ~104K; mimic ~223K.
- Product ack path still has **~4.6× gap to Discard** and **~9.9× gap to mimic** — but the 64 MiB threshold control shows mid-run sealing was also capping ack.
- Do **not** treat prior “attack sealing because seal_active was 1.21 s” as proven without this control; this control now proves interference on the ack path.

## Honesty gaps closed in harness

1. `campaign_ops_per_sec` replaces ambiguous “lifecycle TPS” (includes reopen/verify).
2. Store reopen uses `scan_live_logical` coverage-aware ledger compare.
3. Raw mimic no longer claims `reopen_exact=true` (length + endpoints only).
4. Documented: skip-index does **not** disable Hydra/Chimera at seal.

## Not done here

No sealing optimisation, no AWO/watermark/controller changes.
