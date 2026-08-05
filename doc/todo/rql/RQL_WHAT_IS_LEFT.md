# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal rejected premature C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

---

## Are we done?

**No.** Named page IR is **not** a finished bytecode machine.
**RQL-C1 must not be accepted.**

| Claim | Reality |
|---|---|
| Core path-project named IR | **IR1 labor closed** |
| Core order/sort-tuple named IR | **IR2 labor closed** |
| Core page/coverage named IR | **IR3 labor closed** |
| One bytecode machine owns all query meaning | **False** — enrich/within residual |
| Ready for RQL-C1 | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = RQL-IR4 enrich/within named IR (when on todo)
```

---

## Just shipped (IR3)

- `query_bytecode_v1/ir_page.rs` (`residiuum-query-ir-page-v1`)
- `execute_plan` uses page-size / coverage / cursor APIs from IR module only
- Evidence: `doc/todo/rql/evidence/rql_ir3_page.log`

---

## Ordered residual

| # | Who | Package | Exit |
|---|---|---|---|
| **1–3** | Labor | **RQL-IR1 / IR2 / IR3** | **labor closed** |
| **4** | **Labor** | **RQL-IR4** | enrich/within named IR phase |
| **5** | Principal | **RQL-C1** | Only after IR residual accepted; **never before** |

---

## One-line status

```text
NEXT labor  = RQL-IR4 enrich/within named IR
FORBIDDEN   = RQL-C1 accept (Decision 0 OPEN)
LANDED      = IR1 project + IR2 order + IR3 page named IR
HONESTY     = see QUERY_IR_RESIDUAL.md — not a bytecode machine yet
```
