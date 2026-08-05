# Query IR residual — what is still not a bytecode machine

Status: **2026-08-05 · RQL-IR1**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.**

ISA sole input (X5/X5b) + one-dispatch (X5c) + named Core project IR (IR1) are
**not** “one bytecode machine owns all query meaning.”

---

## One-dispatch story (landed)

```text
source / builder
    │
    ▼
encode_*_program  ──► ISA bytes
    │
    ▼
decode_isa  ──► QueryIsaProgram { core, budget, full? }
    │
    ├─ Core only ──► execute_isa_bytes
    │                      │
    │                      ▼
    │               execute_decoded_core ──► execute_plan
    │                                            ├─ kernel where (SDA)
    │                                            ├─ ir_project (IR1)
    │                                            └─ page/order/coverage (Rust residual)
    │
    └─ Full ───────► execute_full_isa_with
                           │
                           ├─ execute_decoded_core (same Core entry)
                           └─ Rust attach loop over decoded pipeline/project
```

| Entry | Role |
|---|---|
| `execute_core_rql` / `execute_bytecode` / `execute_isa_bytes` | Core product path |
| `execute_rql_full*` / `execute_full_isa_with` | Full-language path |
| `execute_decoded_core` | **Shared** post-decode Core page (X5c) |
| `ir_project` | Named Core path-project phase (IR1) |

---

## Phase ledger — machine vs residual Rust

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` | Durable AST carrier — **not** an opcode machine |
| `where` / attach filters / candidate `where` | `kernel.rs` → SDA | Kernel substrate — **not** full query bytecode |
| Core path-project | `ir_project.rs` | **Named IR phase (IR1)** — still Rust, not opcodes |
| Page / limit / cursor resume | `core_page.rs` | **Rust IR residual** |
| Order / sort-tuple | `core_page.rs` | **Rust IR residual** |
| Coverage policy / holes | `core_page.rs` | **Rust IR residual** |
| Enrich / within attach | `full_attach.rs` | **Rust IR residual** |
| Brace `project { … }` | `full_attach.rs` | **Rust IR residual** |
| Host scan / index / get | `HostCapabilities` | Allowed host boundary |

Anything marked **Rust IR residual** (or named IR still in Rust) still interprets
decoded structures. That is why Decision 0 remains **OPEN**.

Detail for IR1: [QUERY_IR_PROJECT_V1.md](./QUERY_IR_PROJECT_V1.md).

---

## Explicit non-claims

- X5 + X5b + X5c + IR1 ≠ Decision 0 closed
- Named `ir_project` ≠ opcode machine
- **RQL-C1 must not be accepted**

---

## Evidence

- `doc/todo/rql/evidence/rql_ir1_project.log`
- Arch gate requires `ir_project.rs` + `core_page` calling `apply_project_paths`
