# CSE-2R — Safety rollback (2026-08-04)

Status: **labor complete / principal-reclassified**.  
**Not** Compact minimum parity. Charter:
[`CHIMERA_SALVAGE_EQUIVALENCE.md`](../../todo/performance-qualification/CHIMERA_SALVAGE_EQUIVALENCE.md).

> **CSE safety rollback:** Materialized Chimera restored while compact recovery
> remains unresolved.

## What happened

Prior labor switched product seal/enrichment Chimera back to
`build_materialized_layout` and reported `equivalence_holds=true`. That equality
is

\[
\operatorname{Product}_{new} = \operatorname{Product}_{old}
\]

(Materialized again) — **not**

\[
\operatorname{Recoverable}_{\mathrm{compact}} \supseteq
\operatorname{Recoverable}_{\mathrm{materialized}}.
\]

No Compact parity / recovery-code mechanism was implemented.

## Consequences (honest)

| Item | Status |
|---|---|
| Product safety | Restored (Materialized embed) |
| Compact safety | Still failed / unproven (CSE-1) |
| Product Chimera amp | ≈98% class (Materialized) |
| Sustainable product TPS | Likely near ~12.4K class |
| 37.9K / ~0.74% amp | Experimental Compact measurement only |
| ETQ-2 | **Paused** |

## Mechanism kept

| Path | Builder |
|---|---|
| Product seal / rebuild / live-projection | `build_materialized_layout` |
| Experimental Compact (ETQ) | `build_compact_layout` |

## Evidence

| Artifact | Path |
|---|---|
| Rollback JSON | `rollback.json` |
| Failure table | `FAILURE_TABLE.md` |
| Guard test | `crates/residiuum-store/tests/cse2r_safety_rollback.rs` |
| Run log | `run.log` |

## Next

**CSE-3** — Compact + explicit recovery code
([`CSE3_COMPACT_RECOVERY_CODE.md`](../../todo/performance-qualification/CSE3_COMPACT_RECOVERY_CODE.md)).
