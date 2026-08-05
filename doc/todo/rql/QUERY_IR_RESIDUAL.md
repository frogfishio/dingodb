# Query IR residual — what is still not a finished bytecode machine

Status: **2026-08-05 · RQL-P1b**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.** Principal rejected D0 closure — see [QUERY_VM_V1.md](./QUERY_VM_V1.md).

Named IR phases (project + order + page + attach helpers) are still
**Rust IR residual**. Query VM **dispatch** exists (VM1) and host ops are
**collection-qualified** (P1b), but Core pipeline opcodes still call fused
`execute_plan` — not yet opcode-granular semantics without those interpreters.
Not Decision 0 closed.

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
| Query VM dispatch | `vm_exec.rs` | **Dispatch loop (VM1)** — Core fused body residual |
| Scan/index orchestration loop | `core_page.rs` `execute_plan` | **Fused Core body** until VM2 |
| Host scan / index / get | `HostCapabilities` by `CollectionId` | **P1b labor closed** |

Detail: [QUERY_IR_PROJECT_V1.md](./QUERY_IR_PROJECT_V1.md) ·
[QUERY_IR_ORDER_V1.md](./QUERY_IR_ORDER_V1.md) ·
[QUERY_IR_PAGE_V1.md](./QUERY_IR_PAGE_V1.md) ·
[QUERY_IR_ATTACH_V1.md](./QUERY_IR_ATTACH_V1.md).

---

## Explicit non-claims

- IR1–IR4 ≠ Decision 0 closed
- Named IR ≠ finished opcode-granular machine
- VM1 / P1b ≠ Decision 0 closed / C1
- Core fused `execute_plan` body ≠ VM2 complete
- **RQL-C1 must not be accepted**
- NEXT labor = **RQL-VM2** then **P1c** — **not** principal C1

---

## Evidence

- `doc/todo/rql/evidence/rql_ir4_attach.log`
- `doc/todo/rql/evidence/rql_d0r_harden.log`
- `doc/todo/rql/evidence/rql_p0b_private_api.log`
- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
- `doc/todo/rql/evidence/rql_vm1_dispatch.log`
- `doc/todo/rql/evidence/rql_p1b_host_by_id.log`
