# CSE — Chimera Salvage Equivalence

Status: **frozen sequence** (principal correction 2026-08-04).  
Blocks: product/default Compact Chimera acceptance; removal/migration of
Materialized Chimera; resume of **ETQ-2** until Compact+recovery proves
salvage equivalence (not a Materialized rollback).

## Dual implementation (keep both)

```text
Product default: Materialized — safe, slow (~98% Chimera amp; ~12.4K TPS class)
Experimental:    Compact SegmentFrame — fast (~0.74% amp; ~37.9K experimental), not equivalent
Target:          Compact + explicit recovery code — fast and equally safe
```

## Qualified Compact status (hard)

> Compact Chimera performance architecture accepted provisionally as **ETQ
> measurement only**. CSE-1 proved Compact SegmentFrame fails salvage
> equivalence. The 2026-08-04 product restore of Materialized embeds is a
> **CSE safety rollback**, not Compact parity. Compact recovery remains
> unresolved. **ETQ-2 stays paused.**

The ~3× complete-lifecycle TPS gain (~12.4K → ~37.9K) and ~0.74% Chimera amp
remain **valid experimental performance evidence**. They are **not** product
performance and **not** permission to alter Residiuum’s damage-tolerance story.

### Explicit non-claims

- Compact SegmentFrame is **not** the product durability default (CSE-1 FAIL).
- CSE-2R safety rollback is **not** Compact minimum parity — no recovery
  mechanism was implemented; `Product_new = Product_old` (Materialized).
- Materialized format/reader must **not** be removed.
- Existing data must **not** be migrated to Compact-only.
- 37.9K TPS is **experimental**, not sustainable product throughput.

## Required inequality (still open for Compact)

\[
\operatorname{Recoverable}_{\mathrm{compact(+recovery)}}(f)
\supseteq
\operatorname{Recoverable}_{\mathrm{materialized}}(f)
\]

for every frozen failure \(f\) in the CSE-0 set. Satisfying this by switching
the product path back to Materialized does **not** count.

## Sequence

| Package | Goal |
|---|---|
| **CSE-0** | Materialized recovery baseline — **labor complete**; archive `…/cse0-materialized-recovery-baseline/`. |
| **CSE-1** | Compact equivalence campaign — **labor complete**; inequality **FAIL**; archive `…/cse1-compact-equivalence/`. |
| **CSE-2R** | **Safety rollback** (not parity) — Materialized restored on product seal; Compact unresolved; archive `…/cse2r-safety-rollback/`. |
| **CSE-3** | Compact + recovery — **Stage 0 complete** (P★ bound; reduced-overhead ≡ Materialized **impossible**); Stage 1 blocked on principal A/B/C; archive `…/cse3-stage0-recovery-bound/`. |
| **ETQ-2** | **Paused** until Compact+recovery clears salvage (principal). |

## CSE-0 — Materialized recovery baseline

Status: **labor complete** (2026-08-04). Evidence:
`doc/archive/performance-qualification/2026-08-04-cse0-materialized-recovery-baseline/`.

- Frozen \(F\): F0 control, F1 wipe Chimera, F2 corrupt `.cmr`, F3 XOR auth body
  `t`, F4 delete sealed segment, F5 F3+wipe Chimera.
- Channels: `auth`, `chimera`, `layout_direct`.
- Test: `crates/residiuum-store/tests/cse0_materialized_chimera_recovery.rs`.

## CSE-1 — Compact equivalence

Status: **labor complete** (2026-08-04). Evidence:
`doc/archive/performance-qualification/2026-08-04-cse1-compact-equivalence/`.

- Compact fails ⊇ on F0/F3/F4 `layout_direct` and F3 `chimera` (no embedded
  salvage for damaged `t`).
- Test: `crates/residiuum-store/tests/cse1_compact_chimera_equivalence.rs`.

## CSE-2R — Safety rollback (NOT minimum parity)

Status: **labor complete / reclassified** (principal 2026-08-04). Evidence:
`doc/archive/performance-qualification/2026-08-04-cse2r-safety-rollback/`.

> **CSE safety rollback:** Materialized Chimera restored while compact recovery
> remains unresolved.

- Mechanism: product seal/enrichment writes `build_materialized_layout` again.
- Result: product safety restored; Compact safety still failed/unproven.
- Consequences: product Chimera amp ≈98%; sustainable product throughput likely
  near ~12.4K TPS; 37.9K remains experimental.
- **Do not** call this Compact parity — no parity/recovery code was implemented.
- Test (product Materialized restore guard): `cse2r_safety_rollback.rs`.

## CSE-3 — Compact + explicit recovery code

Charter: [`CSE3_COMPACT_RECOVERY_CODE.md`](./CSE3_COMPACT_RECOVERY_CODE.md).  
Stage 0: [`CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md`](./CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md).

**Stage 0 complete:** strongest Materialized format pattern **P★** = recover
sealed live set \(V_S\) after total authoritative-segment payload loss with
`.cmr` intact. Info lower bound = full live length \(L\). Reduced-overhead
Compact equivalence to P★ is **impossible**. Stage 1 codec selection waits on
principal **A / B / C** (keep P★ / weaken named set / hybrid).

## Evidence homes

- CSE-0: `doc/archive/…/2026-08-04-cse0-materialized-recovery-baseline/`
- CSE-1: `doc/archive/…/2026-08-04-cse1-compact-equivalence/`
- CSE-2R: `doc/archive/…/2026-08-04-cse2r-safety-rollback/`
- CSE-3 Stage 0: `doc/archive/…/2026-08-04-cse3-stage0-recovery-bound/`

## Relation to ETQ

ETQ may **measure** Compact experimentally. **ETQ-2 Single-Pass Decode stays
paused** until Compact+recovery clears salvage (or explicit principal waiver).
AWO remains paused.
