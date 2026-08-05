# Query IR residual — what is still not a bytecode machine

Status: **2026-08-05 · RQL-IR2**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.**

Named IR phases (project + order) are **not** “one bytecode machine owns all
query meaning.”

---

## One-dispatch story (landed)

```text
decode_isa → execute_decoded_core → execute_plan
                 ├─ kernel where (SDA)
                 ├─ ir_project (IR1)
                 ├─ ir_order (IR2)
                 └─ page / coverage (Rust residual in core_page)
```

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` | Durable AST carrier — **not** an opcode machine |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate |
| Core path-project | `ir_project.rs` | **Named IR (IR1)** — still Rust |
| Core order / sort-tuple | `ir_order.rs` | **Named IR (IR2)** — still Rust |
| Page / limit / cursor mint | `core_page.rs` | **Rust IR residual** |
| Coverage policy / holes | `core_page.rs` | **Rust IR residual** |
| Enrich / within / brace project | `full_attach.rs` | **Rust IR residual** |
| Host scan / index / get | `HostCapabilities` | Allowed host boundary |

Detail: [QUERY_IR_PROJECT_V1.md](./QUERY_IR_PROJECT_V1.md) · [QUERY_IR_ORDER_V1.md](./QUERY_IR_ORDER_V1.md).

---

## Explicit non-claims

- IR1 + IR2 ≠ Decision 0 closed
- Named IR ≠ opcode machine
- **RQL-C1 must not be accepted**

---

## Evidence

- `doc/todo/rql/evidence/rql_ir2_order.log`
