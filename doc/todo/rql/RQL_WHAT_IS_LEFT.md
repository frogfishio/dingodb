# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Product frontends share one Query VM dispatch funnel (**P1c**), but
`run_core_page` still fuses Scan→Project materialize for APP-6 equivalence.
**RQL-C1 must not be accepted. Decision 0 must not be closed.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **P0b labor closed** |
| Query VM opcode set + dispatch | **VM0+VM1 labor closed** |
| Collection-qualified host API | **P1b labor closed** |
| Core opcodes drive phases (not fused execute_plan entry) | **VM2 labor closed** — `CoreFrame` |
| Every product frontend → same dispatch loop | **P1c labor closed** |
| Scan→Project fully split into independent opcode bodies | **Partial** — `run_core_page` residual |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = optional further materialize split (`run_core_page`); C1 remains principal-only
```

---

## Hard acceptance invariant (principal)

```text
All syntax → compiler intermediates → canonical Query ISA
                                      ↓
                              exactly one Query VM
                                      ↓
                          collection-qualified host API
```

---

## Ordered residual (mandatory labor)

| # | Package | Exit |
|---|---|---|
| **D0R** | SoT + enrich/within `using_id` bind + ISA reserved/canonical | **labor closed** |
| **P0b** | Privatize public non-ISA execute/project/attach APIs | **labor closed** |
| **VM0** | Define Query VM instruction set | **labor closed** |
| **VM1** | One instruction-dispatch machine (Core + Full) | **labor closed** |
| **P1b** | Unify host behind collection-qualified `HostCapabilities` | **labor closed** |
| **VM2** | Core opcodes → `CoreFrame` phases; demote `execute_plan` | **labor closed** |
| **P1c** | Arch test: every frontend → same dispatch loop | **labor closed** |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (P1c)

- Frontends funnel: `CollectionClient::rql` / builder `run` / view-bound /
  op 118 / Full ISA → `execute_decoded_core` → `run_vm_core` (+ attach VM)
- Deleted dead `core_page::execute_rql` VM bypass
- Arch gate enforces frontend → shared dispatch
- Evidence: `doc/todo/rql/evidence/rql_p1c_frontend_dispatch.log`

---

## One-line status

```text
NEXT        = optional materialize split (run_core_page)
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4; D0R; P0b; VM0–VM2; P1b; P1c
HONESTY     = CoreFrame phases; run_core_page still fused Scan→Project
```
