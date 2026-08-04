# CSE-3 Stage 2k — CompactShadow default flip (accepted)

Status: **principal-accepted product default** (2026-08-04)  
CompactShadow is Residiuum proper for **fresh** stores. Flip gate stays closed.

## Product performance figure (SoT)

> **~21–23K sustained 8 KiB writes/sec** with Compact Chimera, full Recovery
> Shadow, normal lifecycle, exact recovery, and no growing deferred-work debt.
>
> ≈ **164–180 MiB/s** logical payload · ≈ **1.7–1.9×** Materialized ~12.4K ·
> **ack ≈ lifecycle** (sustainable).

| Label | TPS | Role |
|---|---|---|
| **Product (2k default)** | **~21–23K** | Official sustained figure |
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

Stage **2l** — narrow A/B (same binary): fresh-default vs manually activated
CompactShadow; exclude init/reopen from the ack window; compare stage timings.
Until that runs, any “reopen/dual-attach cost” explanation is a **hypothesis**.

## Residual

- Pre-2k Materialized deployments still need Step 8 migrate.
- Kanban `done` for individual cards is human accept (labor stops at `in_review`).
