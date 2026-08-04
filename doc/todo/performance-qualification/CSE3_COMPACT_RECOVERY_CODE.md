# CSE-3 — Compact + Recovery Shadow (Hybrid)

Status: **Stage 2k + 2l accepted** (2026-08-04) — CompactShadow is the
**fresh-store product default**. Sustained **~21–23K** 8 KiB life=ack is the
**controlled campaign** SoT (≈1.7–1.9× Materialized ~12.4K; not a hardware
floor). ~30.9K activate-path is candidate-only / not reproducible.
Legacy trees migrate via Step 8. Ops:
[`CSE3_COMPACTSHADOW_OPS_NOTES.md`](./CSE3_COMPACTSHADOW_OPS_NOTES.md).
Depends: CSE-0/1, CSE-2R safety rollback, Stage 0 P★ bound, Stage 1 accepted.

## Stage gate

| Stage | Goal | Status |
|---|---|---|
| **0** | P★ bound + reduced-overhead impossibility | **Complete** — [`CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md`](./CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md) |
| **1** | Principal **C — Hybrid** formalized: Compact + Recovery Shadow (+ lifecycle addendum) | **Principal-accepted** — [`CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`](./CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md) |
| **2** | Implement Shadow + CSE equivalence + lifecycle gates | **Complete through 2k** — [`CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md`](./CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md) |
| **3** | Perf gates; retire Materialized as fresh default | **Accepted** — campaign ~21–23K; Stage 2l accepted; migration residual non-blocking |

## Delivery sequence (Stage 2)

1. `.rsh` wire + atomic publication  
2. Streaming sequential writer  
3. Generation-exact salvage + tombstones  
4. `protected_frontier` + lag telemetry  
5. Compaction / retention / secure-delete / encryption / backup / scrub  
6. CSE F0–F5 suite  
7. ≥7 seg/s, non-growing backlog  
8. **Only then** product seal → Compact + Shadow  
9. Full-product throughput re-qualification  

Until Stage 2k, Materialized Chimera was the fresh-store safe path. Fresh
stores now default to CompactShadow; legacy Materialized trees migrate via
Step 8 (no silent open flip).

## Hybrid (normative)

```text
Compact Chimera → query acceleration; tiny, derived, disposable (~0.74% amp)
Recovery Shadow → full-copy salvage; recovery artifact (~100% of V_S); NOT disposable
```

- Preserves P★; does not defeat info theory.
- Win: sequential Shadow construction vs query-oriented Materialized Chimera persist.
- **Ack ≠ P★** — P★ only after Shadow atomic durable.
- Tombstones, compaction coverage, retention/secure delete, and
  `protected_frontier` are normative (Stage 1 Addendum A).
- Materialized remains product until Shadow passes CSE equivalence.

## Non-claims

- No product flip in Stage 1.
- No ETQ-2 resume.
- No Shadow code in Stage 1.
- No multi-media independence claim.

## Evidence

- Stage 0: `doc/archive/…/2026-08-04-cse3-stage0-recovery-bound/`
- Stage 1: `doc/archive/…/2026-08-04-cse3-stage1-hybrid-recovery-shadow/`
- Spine: [`CHIMERA_SALVAGE_EQUIVALENCE.md`](./CHIMERA_SALVAGE_EQUIVALENCE.md)
