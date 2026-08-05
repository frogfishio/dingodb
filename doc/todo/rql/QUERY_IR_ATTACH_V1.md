# Query IR — enrich / within attach (RQL-IR4)

Status: **labor closed 2026-08-05** · board RQL-IR4 `e7077a45`  
Profile id: **`residiuum-query-ir-attach-v1`**  
Runtime: `query_bytecode_v1/ir_attach.rs`  
Companion: [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

Fourth **named** IR lowering slice. Ordered enrich / within / filter pipeline
and brace project run here; `execute_full_isa_with` calls
`CompiledAttachIr::run` (not an inline attach loop).

Attach row helpers (`attach_enrich_rows`, …) remain in `full_attach.rs` as
shared implementation used by the IR phase and unit tests.

---

## Honesty

| Claim | Reality |
|---|---|
| Named IR phase for enrich/within attach orchestration | **True** |
| Opcode / bytecode machine for attach | **False** — still Rust |
| Decision 0 closed / RQL-C1 | **Forbidden** |

---

## APIs

| API | Meaning |
|---|---|
| `CompiledAttachIr::lower` | Pipeline + optional brace project |
| `CompiledAttachIr::run` / `run_attach_pipeline` | Foreign load + attach + filter + project |

---

## Residual after IR4

Still residual (Decision 0 OPEN):

- attach helper implementations in `full_attach` (used by IR; not opcodes)
- scan/index orchestration body in `core_page` (host I/O loop)
- no finished bytecode machine

See [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md).
