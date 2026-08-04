# Enrichment Throughput Qualification (ETQ)

Status: **ETQ open**; **CSE blocks product Compact accept and ETQ-2 resume**.  
**ETQ-0 accepted** (root cause: full-payload Chimera).  
**ETQ-1 Compact Chimera** = **provisional performance-architecture accept only**.  
**ETQ-2 Single-Pass Decode** = **deferred** behind CSE.  
AWO: **paused**. Three-cell: **deprioritized**.  
Date: 2026-08-04

## Qualified Compact status (hard)

> Compact Chimera performance architecture accepted provisionally.
> Durability equivalence is unproven and blocks product/default acceptance.

### Not accepted (until CSE exits)

- Product default
- Durability-equivalent to materialized Chimera
- Safe to remove materialized format/reader
- Eligible for migration of existing data

Materialized Chimera encode/decode stays intact. ~3× TPS / ~0.74% amp remain
valid **performance** evidence only.

## Honest product numbers (performance campaign)

| Label | Value | Meaning |
|---|---:|---|
| Acknowledgement TPS | ~43.8K | Burst |
| Complete-lifecycle TPS | ~37.9K | ~3× vs materialized-Chimera era (~12.4K) |
| Logical payload | ~296 MiB/s | 37.9K × 8 KiB |
| Chimera derived / auth | ~0.74% | Was ~98% |
| Enrichment complete | ~4.93 seg/s | Still short of ≥7 |
| Backlog slope | +0.64 | Still slightly positive |

Evidence: `doc/archive/performance-qualification/2026-08-04-etq1-compact-chimera/`.

## Sequence (updated)

1. **CSE-0 → CSE-1 → (CSE-2 if needed)** —
   [CHIMERA_SALVAGE_EQUIVALENCE.md](./CHIMERA_SALVAGE_EQUIVALENCE.md)
2. Resume **ETQ-2** Single-Pass Decode only after Compact is viable
   ([ETQ2_SINGLE_PASS_DECODE.md](./ETQ2_SINGLE_PASS_DECODE.md))

Required: \(\mathrm{Recoverable}_{compact}(f)\supseteq\mathrm{Recoverable}_{materialized}(f)\).

## Evidence homes

- ETQ-0: `…/2026-08-04-etq0-enrichment-stage-breakdown/`
- ETQ-1: `…/2026-08-04-etq1-compact-chimera/`
- CSE / ETQ-2: see charters (dates TBD)
