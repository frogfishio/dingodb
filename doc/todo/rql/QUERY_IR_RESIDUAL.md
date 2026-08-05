# Query IR residual — what is still not a bytecode machine

Status: **2026-08-05 · RQL-VM0**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.** Principal rejected D0 closure — see [QUERY_VM_V1.md](./QUERY_VM_V1.md).

Named IR phases (project + order + page + attach orchestration) are still
**Rust IR residual**. Query VM **opcodes** are frozen (VM0) but **not dispatched**.
Not a finished bytecode machine.

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` | Durable AST carrier — **not** an opcode machine |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate |
| Core path-project | `ir_project.rs` | **Named IR (IR1)** — still Rust |
| Core order / sort-tuple | `ir_order.rs` | **Named IR (IR2)** — still Rust |
| Core page / coverage / cursor | `ir_page.rs` | **Named IR (IR3)** — still Rust |
| Enrich / within / brace project orchestration | `ir_attach.rs` | **Named IR (IR4)** — still Rust |
| Query VM opcodes | `vm.rs` | **Vocabulary frozen (VM0)** — **not dispatched** |
| Attach row helpers | `full_attach.rs` | Shared Rust helpers used by IR4 |
| Scan/index orchestration loop | `core_page.rs` | Host I/O orchestration (allowed host + residual glue) |
| Host scan / index / get | `HostCapabilities` | Allowed host boundary |

Detail: [QUERY_IR_PROJECT_V1.md](./QUERY_IR_PROJECT_V1.md) ·
[QUERY_IR_ORDER_V1.md](./QUERY_IR_ORDER_V1.md) ·
[QUERY_IR_PAGE_V1.md](./QUERY_IR_PAGE_V1.md) ·
[QUERY_IR_ATTACH_V1.md](./QUERY_IR_ATTACH_V1.md).

---

## Explicit non-claims

- IR1–IR4 ≠ Decision 0 closed
- Named IR ≠ opcode machine / Query VM
- VM0 opcode freeze ≠ dispatch machine (VM1)
- **RQL-C1 must not be accepted**
- NEXT labor = Query VM programme ([QUERY_VM_V1.md](./QUERY_VM_V1.md)) — **not** principal C1

---

## Evidence

- `doc/todo/rql/evidence/rql_ir4_attach.log`
- `doc/todo/rql/evidence/rql_d0r_harden.log`
- `doc/todo/rql/evidence/rql_p0b_private_api.log`
- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
