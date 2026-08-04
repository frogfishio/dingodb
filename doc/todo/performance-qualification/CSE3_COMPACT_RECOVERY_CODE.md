# CSE-3 — Compact + Recovery Shadow (Hybrid)

Status: **Stage 2 active through step 5** (2026-08-04) — 2a invariants
confirmed; lifecycle dual-run landed. **Materialized remains product until
step 8.** Next: CSE F0–F5 (step 6).  
Depends: CSE-0/1, CSE-2R safety rollback, Stage 0 P★ bound, Stage 1 accepted.

## Stage gate

| Stage | Goal | Status |
|---|---|---|
| **0** | P★ bound + reduced-overhead impossibility | **Complete** — [`CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md`](./CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md) |
| **1** | Principal **C — Hybrid** formalized: Compact + Recovery Shadow (+ lifecycle addendum) | **Principal-accepted** — [`CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`](./CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md) |
| **2** | Implement Shadow + CSE equivalence + lifecycle gates | **Active** — [`CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md`](./CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md) |
| **3** | Perf gates (≥7 seg/s, backlog ≤0, lifecycle ≈ ack); retire Materialized default | Blocked on Stage 2 step 8+ |

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

Until step 8, Materialized Chimera is the safe product path.

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
