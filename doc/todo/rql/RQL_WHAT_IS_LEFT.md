# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Core opcodes own Scan/Filter/Order/Page/Project bodies (**VM3**), and
product frontends share one Query VM funnel (**P1c**), but key-stream Scan still
applies `where` early for APP-6 page early-stop (`filtered_during_scan`).
**RQL-C1 must not be accepted. Decision 0 must not be closed.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **P0b labor closed** |
| Query VM opcode set + dispatch | **VM0+VM1 labor closed** |
| Collection-qualified host API | **P1b labor closed** |
| Core opcodes drive phases | **VM2 labor closed** — `CoreFrame` |
| Every product frontend → same dispatch loop | **P1c labor closed** |
| Scan→Project opcode-owned bodies | **VM3 labor closed** — working bag |
| Key-stream Filter fully separate from Scan | **Partial** — early-stop honesty |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = optional key-stream Filter separation; C1 remains principal-only
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
| **VM3** | Split materialize into opcode-owned bodies | **labor closed** |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (VM3)

- `CoreFrame` working bag: Scan loads, Filter applies where, Order sorts,
  Page resumes/truncates, ProjectPaths projects + page artefact
- `run_core_page` demoted to CoreFrame orchestrator
- Key-stream honesty: Scan may apply `where` for APP-6 early-stop
  (`filtered_during_scan`); Filter confirms
- Evidence: `doc/todo/rql/evidence/rql_vm3_materialize_split.log`

---

## One-line status

```text
NEXT        = optional key-stream Filter separation
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4; D0R; P0b; VM0–VM3; P1b; P1c
HONESTY     = key-stream Scan may apply where early (filtered_during_scan)
```
