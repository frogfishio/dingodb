# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Query VM dispatch (**VM1**) and collection-qualified host (**P1b**) exist,
but Core opcode bodies still call fused `execute_plan` (**VM2**), and not every
frontend is arch-tested onto the same loop (**P1c**).
**RQL-C1 must not be accepted. Decision 0 must not be closed.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **P0b labor closed** |
| Query VM opcode set defined | **VM0 labor closed** — [QUERY_VM_V1.md](./QUERY_VM_V1.md) |
| One opcode dispatch machine | **VM1 labor closed** — `vm_exec.rs` |
| Collection-qualified host API | **P1b labor closed** — `HostCapabilities` by `CollectionId` |
| Plans/IR compile-only (no semantic executors) | **False** — residual **RQL-VM2** |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = RQL-VM2 (delete fused executors after equivalence) then P1c
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
| **VM2** | Plans/IR compile-only; delete semantic executors after equivalence | |
| **P1c** | Arch test: every frontend → same dispatch loop | |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (P1b)

- `HostCapabilities::{list_keys,get_json,lookup_index_keys}` take `CollectionId`
- `HeapClient` implements collection-qualified host (`open_collection_by_id`)
- Full attach / VM attach foreign loads use host-by-id (no name-only scan bypass)
- Evidence: `doc/todo/rql/evidence/rql_p1b_host_by_id.log`

---

## One-line status

```text
NEXT        = RQL-VM2 (split/delete fused Core/attach executors) then P1c
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4; D0R; P0b; VM0; VM1; P1b collection-qualified host
HONESTY     = host by id; execute_plan fused body residual → VM2
```
