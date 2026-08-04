# CSE-3 — Compact + explicit recovery code

Status: **Stage 0 labor complete** (2026-08-04) — analysis/proof only.  
Depends: CSE-0 baseline, CSE-1 Compact FAIL, CSE-2R safety rollback.

## Stage gate

| Stage | Goal | Status |
|---|---|---|
| **0** | Strongest Materialized damage pattern + info bound + impossibility or coverage proof | **Complete** — [`CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md`](./CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md) |
| **1** | Codec selection (XOR / RS / …) **only after** principal A/B/C fork | Blocked |
| **2+** | Implementation against a named failure set | Not started |

## Stage 0 verdict (headline)

**P★:** Materialized Chimera guarantees recovery of the sealed **live** value set
\(V_S\) after **total loss of authoritative segment payloads**, if the `.cmr`
sidecar survives (`layout_direct`; F3/F4).

Matching P★ needs ≈**100%** independent redundancy for incompressible data.
**Reduced-overhead Compact ≡ Materialized under P★ is impossible.**

Principal must choose before Stage 1:

- **A** — Keep P★ (full-copy salvage; Compact ETQ-only)
- **B** — Weaken to a named smaller failure set, then select a code
- **C** — Hybrid (Compact hot path + full-copy salvage tier)

## Keep both implementations

```text
Product default: Materialized — safe, slow
Experimental:    Compact — fast, not yet equivalent
Target:          Compact + recovery only if Stage 0 fork allows a named set
```

## Non-claims

- Does **not** flip product default to Compact.
- Does **not** resume ETQ-2.
- Does **not** select XOR/RS in Stage 0.
- Does **not** accept Compact durability by demonstration alone.

## Evidence

- Stage 0: `doc/archive/performance-qualification/2026-08-04-cse3-stage0-recovery-bound/`
- Spine: [`CHIMERA_SALVAGE_EQUIVALENCE.md`](./CHIMERA_SALVAGE_EQUIVALENCE.md)
