# Query IR residual — what is still not a bytecode machine

Status: **2026-08-05 · RQL-IR3**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.**

Named Core IR phases (project + order + page) are **not** a finished bytecode
machine.

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` | Durable AST carrier — **not** an opcode machine |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate |
| Core path-project | `ir_project.rs` | **Named IR (IR1)** — still Rust |
| Core order / sort-tuple | `ir_order.rs` | **Named IR (IR2)** — still Rust |
| Core page / coverage / cursor | `ir_page.rs` | **Named IR (IR3)** — still Rust |
| Scan/index orchestration loop | `core_page.rs` | Host I/O orchestration (allowed host + residual glue) |
| Enrich / within / brace project | `full_attach.rs` | **Rust IR residual** |
| Host scan / index / get | `HostCapabilities` | Allowed host boundary |

Detail: [QUERY_IR_PROJECT_V1.md](./QUERY_IR_PROJECT_V1.md) ·
[QUERY_IR_ORDER_V1.md](./QUERY_IR_ORDER_V1.md) ·
[QUERY_IR_PAGE_V1.md](./QUERY_IR_PAGE_V1.md).

---

## Explicit non-claims

- IR1–IR3 ≠ Decision 0 closed
- Named IR ≠ opcode machine
- **RQL-C1 must not be accepted**

---

## Evidence

- `doc/todo/rql/evidence/rql_ir3_page.log`
