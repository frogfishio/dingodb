# CSE-3 Stage 2k — CompactShadow default flip (accepted)

Status: **principal-accepted product default** (2026-08-04)  
CompactShadow is Residiuum proper for **fresh** stores. Flip gate stays closed.

## Product performance figure (SoT)

> **~21–23K sustained 8 KiB writes/sec** with Compact Chimera, full Recovery
> Shadow, normal lifecycle, exact recovery, and no growing deferred-work debt.
>
> ≈ **164–180 MiB/s** logical payload · ≈ **1.7–1.9×** Materialized ~12.4K ·
> **ack ≈ lifecycle** (sustainable).

This band is the **controlled campaign result**, not a universal
hardware-independent floor (Stage 2l saw absolute arms ~13K–~27K under load).

| Label | TPS | Role |
|---|---|---|
| **Product (2k default)** | **~21–23K** | Official sustained campaign figure |
| Materialized predecessor | ~12.4K | Equally protected prior product |
| Activate-path candidate (2i) | ~30.9K | Historical candidate only — **not** product |

Do **not** quote 30.9K as the product number.

## Ceremony (delivered)

1. Fresh `Store::create` → CompactShadow + dual-stream.
2. Missing `recovery/mode.v1` ⇒ Materialized on open (no silent migrate).
3. Legacy: prepare → activate; Materialized retained through activate.
4. Rollback non-destructive.
5. Suites green (F0–F5 15/15, RSHD0004 16/16, pair 4/4, segid 8/8).
6. Step 9 without manual activation — gates_pass; life=ack ≈21–23K.
7. Ops: [`CSE3_COMPACTSHADOW_OPS_NOTES.md`](./CSE3_COMPACTSHADOW_OPS_NOTES.md).

## Follow-up (does not reopen flip)

Stage **2l** — **principal-accepted / closed**. Fresh CompactShadow is the
correct product path; ~30.9K not reproducible; activate reopen tax tied to
retained Materialized media. See
[`CSE3_STAGE2_STEP2L_TPS_AB.md`](./CSE3_STAGE2_STEP2L_TPS_AB.md).

## Residual

- Pre-2k Materialized deployments still need Step 8 migrate.
- **Non-blocking:** activated legacy stores retain Materialized recovery media
  for rollback and may reopen more slowly than native CompactShadow stores.
  Later (optional): mark retained Materialized as **rollback-only** so ordinary
  open/query ignore them; do **not** optimize immediately.
- Kanban `done` for individual cards is human accept (labor stops at `in_review`).
- No more sealing or Chimera work is required for correctness from Stages 2k/2l.