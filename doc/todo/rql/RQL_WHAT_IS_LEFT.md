# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal rejected premature C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

---

## Are we done?

**No.** Named Core project IR is **not** a finished bytecode machine.
**RQL-C1 must not be accepted.**

| Claim | Reality |
|---|---|
| Core ISA sole executable input | **X5 labor closed** |
| Full RQL on same ISA runtime | **X5b labor closed** |
| One post-decode Core dispatch | **X5c labor closed** |
| Core path-project named IR | **IR1 labor closed** — still Rust |
| One bytecode machine owns all query meaning | **False** — see [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) |
| Ready for RQL-C1 | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = RQL-IR2 Core order/sort-tuple named IR (when on todo)
```

---

## Blocking findings (principal) — status

1. ISA does not control Core execution — **addressed in X5**.
2. Full RQL bypasses ISA — **addressed in X5b**.
3. Most Core semantics still Rust plan interpreter — **IR1 started** (project only); page/order/coverage/enrich remain.
4. Arch check filename-only — **addressed**.
5. Tests weak on ISA identity — **addressed**.

---

## Just shipped (IR1 — Core path-project IR)

- `query_bytecode_v1/ir_project.rs` (`residiuum-query-ir-project-v1`)
- `execute_plan` uses `apply_project_paths` only
- Evidence: `doc/todo/rql/evidence/rql_ir1_project.log`

---

## Ordered residual

| # | Who | Package | Exit |
|---|---|---|---|
| **1** | Labor | **RQL-IR1** | **labor closed** |
| **2** | **Labor** | **RQL-IR2** | Core order/sort-tuple named IR phase |
| **3** | Labor | IR page / coverage / enrich slices | Further residual |
| **4** | Principal | **RQL-C1** | Only after IR residual accepted; **never before** |

---

## One-line status

```text
NEXT labor  = RQL-IR2 Core order named IR
FORBIDDEN   = RQL-C1 accept (Decision 0 OPEN)
LANDED      = X5/X5b/X5c + IR1 path-project named IR
HONESTY     = see QUERY_IR_RESIDUAL.md — not a bytecode machine yet
```
