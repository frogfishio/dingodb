# Query IR residual — what is still not a finished bytecode machine

Status: **2026-08-05 · RQL-P1c**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.** Principal rejected D0 closure — see [QUERY_VM_V1.md](./QUERY_VM_V1.md).

Named IR phases remain **Rust IR residual**. Query VM **dispatch** (VM1) +
**CoreFrame phases** (VM2) + collection-qualified host (P1b) + **frontend funnel
arch-tested onto one loop (P1c)** exist, but `run_core_page` still fuses
Scan→Project materialize for APP-6 equivalence. Not Decision 0 closed.

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` | Durable AST carrier — lowers into QVM |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate |
| Core path-project | `ir_project.rs` | **Named IR (IR1)** — still Rust |
| Core order / sort-tuple | `ir_order.rs` | **Named IR (IR2)** — still Rust |
| Core page / coverage / cursor | `ir_page.rs` | **Named IR (IR3)** — still Rust |
| Enrich / within / brace helpers | `ir_attach.rs` / `full_attach.rs` | **Named IR (IR4)** — called from VM attach ops |
| Query VM opcodes | `vm.rs` | **Vocabulary frozen (VM0)** |
| Query VM dispatch | `vm_exec.rs` | **Dispatch loop (VM1)** |
| Core opcode phases | `core_phases.rs` `CoreFrame` | **VM2** — IndexEq real; Scan→Project via `run_core_page` |
| Scan/index materialize | `core_phases.rs` `run_core_page` | **Fused Scan→Project** residual |
| Host scan / index / get | `HostCapabilities` by `CollectionId` | **P1b labor closed** |
| Product frontends → one loop | SDK rql/builder/view + op 118 + Full | **P1c labor closed** |

Detail: [QUERY_IR_PROJECT_V1.md](./QUERY_IR_PROJECT_V1.md) ·
[QUERY_IR_ORDER_V1.md](./QUERY_IR_ORDER_V1.md) ·
[QUERY_IR_PAGE_V1.md](./QUERY_IR_PAGE_V1.md) ·
[QUERY_IR_ATTACH_V1.md](./QUERY_IR_ATTACH_V1.md).

---

## Explicit non-claims

- IR1–IR4 ≠ Decision 0 closed
- Named IR ≠ finished opcode-granular machine
- VM1 / P1b / VM2 / P1c ≠ Decision 0 closed / C1
- `run_core_page` fused materialize ≠ fully split opcode bodies
- **RQL-C1 must not be accepted**
- NEXT labor = optional further materialize split — **not** principal C1

---

## Evidence

- `doc/todo/rql/evidence/rql_ir4_attach.log`
- `doc/todo/rql/evidence/rql_d0r_harden.log`
- `doc/todo/rql/evidence/rql_p0b_private_api.log`
- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
- `doc/todo/rql/evidence/rql_vm1_dispatch.log`
- `doc/todo/rql/evidence/rql_p1b_host_by_id.log`
- `doc/todo/rql/evidence/rql_vm2_core_phases.log`
- `doc/todo/rql/evidence/rql_p1c_frontend_dispatch.log`
