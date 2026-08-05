# Query IR — Core path-project (RQL-IR1)

Status: **labor closed 2026-08-05** · board RQL-IR1 `237db0b6`  
Profile id: **`residiuum-query-ir-project-v1`**  
Runtime: `query_bytecode_v1/ir_project.rs`  
Companion: [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

First **named** IR lowering slice after X5c. Application Core path-project
meaning lives in this module; `execute_plan` calls it (no private inline
`project_doc`).

---

## Honesty

| Claim | Reality |
|---|---|
| Named IR phase for Core project | **True** |
| Opcode / bytecode machine for project | **False** — still Rust |
| Decision 0 closed / RQL-C1 | **Forbidden** |

---

## APIs

| API | Meaning |
|---|---|
| `CompiledProjectIr::lower` | Plan project list → IR |
| `CompiledProjectIr::apply` / `apply_project_paths` | Project one document |
| `execute_plan` | Uses `apply_project_paths` on matched rows |

---

## Residual after IR1

Still anonymous Rust in `core_page` / `full_attach`:

- page / limit / cursor
- order / sort-tuple
- coverage
- enrich / within / brace project

See [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md).
