# Query IR — Core page / coverage (RQL-IR3)

Status: **labor closed 2026-08-05** · board RQL-IR3 `de24fab1`  
Profile id: **`residiuum-query-ir-page-v1`**  
Runtime: `query_bytecode_v1/ir_page.rs`  
Companion: [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

Third **named** IR lowering slice. Page-size clamping, coverage policy merge,
and cursor mint/decode live here; `execute_plan` calls them.

---

## Honesty

| Claim | Reality |
|---|---|
| Named IR phase for Core page/coverage | **True** |
| Opcode / bytecode machine for page | **False** — still Rust |
| Decision 0 closed / RQL-C1 | **Forbidden** |

---

## APIs

| API | Meaning |
|---|---|
| `CompiledPageIr::lower` | Plan page_size + coverage |
| `resolve_page_size` / `rows_needed` | Effective page clamp |
| `resolve_coverage_mode` / `finish_coverage` | Coverage merge + fail-closed |
| `mint_page_cursor` / `decode_after` | Multipage continuation packing |

---

## Residual after IR3

Still in `full_attach` / scan orchestration:

- enrich / within attach loops
- brace `project { … }`
- scan/index orchestration body in `core_page` (host I/O loop)

See [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md).
