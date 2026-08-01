# Core Storage Qualification

State: **ACTIVE — C0 labor floors delivered; principal accept open**

This program has two complementary authorities:

| Document | Authority |
|---|---|
| [CORE_STORAGE_QUALIFICATION_SPEC.md](CORE_STORAGE_QUALIFICATION_SPEC.md) | What storage must guarantee, which failures are in scope, and what constitutes qualification |
| [CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md](CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md) | Package order, artifacts, test infrastructure, and evidence delivery |

The specification wins on semantics. The implementation plan wins on package
execution within the order admitted by
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md) §6A.

## Live claim and command

- **Profile:** `residiuum-core-storage-v1` / level **A2**
  (post identity reset; Residiuum profile id only — see rebrand protocol reset).
- **Command:** `residiuum verify --profile residiuum-core-storage-v1 --level A2`
  — stand-in: `bash scripts/residiuum-verify-core-storage.sh`.
- **Registries:** `spec/verification/core-storage/`.
- **Scoreboard:** [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md).
- **Kanban:** Feature C0; tasks CSQ-0…CSQ-12 (+ CSQ-DOC). Live stages are
  board-owned — this README does not mirror columns.

## Package graph

```text
CSQ-0 registries
→ (CSQ-1 independent oracles ‖ CSQ-2 boundary instrumentation)
→ CSQ-3 format corpus
→ CSQ-4 state machine
→ CSQ-5 crash/filesystem campaign
→ CSQ-6 chunk/large-value
→ CSQ-7 damage/salvage
→ CSQ-8 derived/maintenance/backup/migration
→ CSQ-9 concurrency/resources
→ CSQ-10 mutation/fuzz
→ CSQ-11 compatibility/scale/soak
→ CSQ-12 evidence bundle / A2 evaluator
```

## Snapshot (2026-07-31)

Implementer labor floors for **CSQ-0…CSQ-12** are on the board at **`in_review`**.
Scoreboard package states remain **`active`** until principal acceptance.
The CSQ-12 evidence runner currently reports **`result=not_run`** with exact
residual A2 gates (predecessor accept, full boundary/platform/soak/mutation/
publication) — it does **not** claim A2.

**Next principal step:** accept CSQ labor handoffs and close residual A2 gates.
**Next blocked program:** APB-0 / HAR-1+ / APP-2+ wait on `CSQ-12 = accept`.