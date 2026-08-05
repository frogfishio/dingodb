# RQL — what is left to do

Status: **2026-08-05** · Decision 0 **labor programme closed through X4b**  
Detail: [QUERY_KERNEL_SDA_V1.md](./QUERY_KERNEL_SDA_V1.md) · [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)

---

## Are we done?

| Question | Answer |
|---|---|
| **Decision 0 convergence labor (0D→X1→X2→X2b→X2c→X2d→X3→X4→X4b)?** | **Yes — labor closed.** One runtime, durable ISA, Core+attach filters via `residiuum-sda`. Board: no `todo`/`doing` on this spine. |
| **Product accept (APP-6 / APP-7 / APB-7 / “query qualified”)?** | **No.** That is **RQL-C1** — principal accept, not labor-`done`. |
| **Full RQL-v1 / S1 / D1 / wire ISA packing / `$key` in where?** | **No** — optional residuals or gated features; not blocking the Decision 0 architecture claim. |

```text
DONE (labor) = Decision 0 architecture: one plan → one ISA → one runtime → SDA kernel
NOT done     = package accept, feature growth (S1/D1), wire packing polish
```

---

## Do next (only if principal wants more)

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Principal** | **RQL-C1** | Scoreboard APP-6/APP-7/APB-7 → `accept` (or explicit waiver) |
| **2** | Labor (optional) | Wire / `$key` | Op 118 ISA packing; `$key` in `where` kernel lower |

---

## Landed sequence

`0D` → `X1` → `X2` → `X2b` → `X2c` → `X2d` → `X3` → `X4` → `X4b` — all `in_review` awaiting principal.

---

## One-line status

```text
Verdict     = Decision 0 labor DONE; product accept NOT done
NEXT        = principal RQL-C1 (or stop / waive)
OPTIONAL    = wire ISA packing, $key-in-where
```
