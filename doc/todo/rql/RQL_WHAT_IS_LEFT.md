# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal rejected premature C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

---

## Are we done?

**No.** Named attach IR is **not** a finished bytecode machine.
**RQL-C1 must not be accepted.**

| Claim | Reality |
|---|---|
| Core path-project named IR | **IR1 labor closed** |
| Core order/sort-tuple named IR | **IR2 labor closed** |
| Core page/coverage named IR | **IR3 labor closed** |
| Enrich/within attach named IR | **IR4 labor closed** |
| One bytecode machine owns all query meaning | **False** — still Rust IR + scan glue |
| Ready for RQL-C1 | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT        = Principal Decision 0 / RQL-C1 gate only after residual honesty accepted
```

---

## Just shipped (IR4)

- `query_bytecode_v1/ir_attach.rs` (`residiuum-query-ir-attach-v1`)
- `execute_full_isa_with` runs attach via `CompiledAttachIr` only
- Evidence: `doc/todo/rql/evidence/rql_ir4_attach.log`

---

## Ordered residual

| # | Who | Package | Exit |
|---|---|---|---|
| **1–4** | Labor | **RQL-IR1 / IR2 / IR3 / IR4** | **labor closed** |
| **5** | Principal | **RQL-C1** | Only after IR residual accepted; **never before** |

---

## One-line status

```text
NEXT        = Principal Decision 0 / RQL-C1 gate (labor must not accept C1)
FORBIDDEN   = RQL-C1 accept (Decision 0 OPEN)
LANDED      = IR1–IR4 named IR phases (still Rust)
HONESTY     = see QUERY_IR_RESIDUAL.md — not a bytecode machine yet
```
