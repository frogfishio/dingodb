# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal rejected premature C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

---

## Are we done?

**No.** ISA sole input + one-dispatch honesty are **not** a finished bytecode machine.
**RQL-C1 must not be accepted.**

| Claim | Reality |
|---|---|
| Core ISA sole executable input | **X5 labor closed** |
| Full RQL on same ISA runtime | **X5b labor closed** |
| One post-decode Core dispatch | **X5c labor closed** — `execute_decoded_core` |
| One bytecode machine owns all query meaning | **False** — see [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) |
| Ready for RQL-C1 | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT        = Principal gate (C1 forbidden until IR residual programme accepted)
```

---

## Blocking findings (principal) — status

1. ISA does not control Core execution — **addressed in X5**.
2. Full RQL bypasses ISA — **addressed in X5b**.
3. Most Core semantics still Rust plan interpreter — **documented in X5c**; **still residual**.
4. Arch check filename-only — **addressed** (behavioral ISA + dispatch + IR doc gate).
5. Tests weak on ISA identity — **addressed** (Core mismatch + full E2E + corrupt ISA).

---

## Just shipped (X5c — one-dispatch honesty)

- Shared `execute_decoded_core` for Core + full base page (no Core re-encode)
- IR residual ledger: [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)
- Evidence: `doc/todo/rql/evidence/rql_x5c_dispatch.log`

---

## Ordered residual

| # | Who | Package | Exit |
|---|---|---|---|
| **1–3** | Labor | **RQL-X5 / X5b / X5c** | **labor closed** |
| **4** | Principal | **RQL-C1** | Only after IR residual accepted; **never before** |
| **5** | Labor (when awarded) | IR lowering slices | Order/page/project/coverage/enrich → real machine |

---

## One-line status

```text
NEXT        = principal Decision 0 / C1 gate (C1 forbidden)
FORBIDDEN   = RQL-C1 accept (Decision 0 OPEN)
LANDED      = X5 + X5b ISA sole input; X5c one-dispatch + IR ledger
HONESTY     = see QUERY_IR_RESIDUAL.md — not a bytecode machine yet
```
