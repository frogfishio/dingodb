# CSE — Chimera Salvage Equivalence

Status: **frozen next sequence** (principal 2026-08-04).  
Blocks: product/default Compact Chimera acceptance; removal/migration of
materialized Chimera; resume of **ETQ-2** until Compact is proven viable.

## Qualified Compact status (hard)

> Compact Chimera performance architecture accepted provisionally.
> Durability equivalence is **disproven** by CSE-1 (inequality FAIL) and
> blocks product/default acceptance until CSE-2 restores parity.

The ~3× complete-lifecycle TPS gain (~12.4K → ~37.9K) and ~0.74% Chimera amp
remain **valid performance evidence**. They are **not** permission to alter
Residiuum’s damage-tolerance story.

### Explicit non-claims (until CSE exits)

- Compact is **not** the product default acceptance.
- Compact is **not** durability-equivalent to materialized Chimera.
- Materialized format/reader must **not** be removed.
- Existing data must **not** be migrated to Compact-only.

Materialized Chimera encode/decode (`build_materialized_layout` / v1 reader)
stays intact throughout.

## Required inequality

\[
\operatorname{Recoverable}_{\mathrm{compact}}(f)
\supseteq
\operatorname{Recoverable}_{\mathrm{materialized}}(f)
\]

for every frozen failure \(f\) in the CSE-0 set.

## Sequence

| Package | Goal |
|---|---|
| **CSE-0** | Materialized Chimera recovery baseline — **labor complete** 2026-08-04; archive `…/2026-08-04-cse0-materialized-recovery-baseline/`. |
| **CSE-1** | Compact equivalence campaign — **labor complete** 2026-08-04; inequality **FAIL**; archive `…/2026-08-04-cse1-compact-equivalence/`. |
| **CSE-2** | Minimum parity — **required** (Compact loses recoverability vs Materialized). **Next.** |
| **ETQ-2** | Resume Single-Pass Enrichment Decode **after** Compact is proven viable (or CSE-2 restores parity). |

## CSE-0 — Materialized recovery baseline

Status: **labor complete** (2026-08-04). Evidence:
`doc/archive/performance-qualification/2026-08-04-cse0-materialized-recovery-baseline/`.

- Frozen \(F\): F0 control, F1 wipe Chimera, F2 corrupt `.cmr`, F3 XOR auth body
  `t`, F4 delete sealed segment, F5 F3+wipe Chimera.
- Channels: `auth` (`Store::get`), `chimera` (`get_via_chimera`, index-gated),
  `layout_direct` (Materialized `.cmr` resolve).
- Headline: F3 Materialized **does** expand ChimeraGet for damaged `t`; F4
  product channels empty (index needs segment) but **format** still recovers
  all keys from embedded `.cmr`.
- Test: `crates/residiuum-store/tests/cse0_materialized_chimera_recovery.rs`.

## CSE-1 — Compact equivalence

Status: **labor complete** (2026-08-04). Evidence:
`doc/archive/performance-qualification/2026-08-04-cse1-compact-equivalence/`.

- Same \(F\) against Compact seal layouts.
- Comparison: \(\operatorname{Recoverable}_{\mathrm{compact}}\) **does not**
  contain \(\operatorname{Recoverable}_{\mathrm{materialized}}\) on all cells.
- Gaps: F0/F3/F4 `layout_direct` (no embedded payloads); F3 `chimera` (no
  salvage expansion for damaged `t`). Auth matches Materialized on every cell.
- Test: `crates/residiuum-store/tests/cse1_compact_chimera_equivalence.rs`.
- **CSE-2 required.**

## CSE-2 — Minimum parity (required)

- Compact regresses recoverability vs Materialized (CSE-1 FAIL).
- Smallest change to restore the inequality (may include retaining selective
  materialized payloads for damaged classes — never silently drop salvage).

## Evidence homes

- CSE-0: `doc/archive/…/2026-08-04-cse0-materialized-recovery-baseline/`
- CSE-1: `doc/archive/…/2026-08-04-cse1-compact-equivalence/`
- CSE-2: `doc/archive/…/YYYY-MM-DD-cse2-minimum-parity/`

## Relation to ETQ

ETQ performance work may continue to **measure** Compact as an experiment, but
**ETQ-2 Single-Pass Decode** waits until CSE clears Compact viability (or
explicit principal waiver). AWO remains paused.
