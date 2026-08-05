# Query IR — Core order / sort-tuple (RQL-IR2)

Status: **labor closed 2026-08-05** · board RQL-IR2 `5351eb88`  
Profile id: **`residiuum-query-ir-order-v1`**  
Runtime: `query_bytecode_v1/ir_order.rs`  
Companion: [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md)

Second **named** IR lowering slice. Application Core `order by` compare and
sort-tuple resume live in this module; `execute_plan` calls it.

---

## Honesty

| Claim | Reality |
|---|---|
| Named IR phase for Core order | **True** |
| Opcode / bytecode machine for order | **False** — still Rust |
| Decision 0 closed / RQL-C1 | **Forbidden** |

---

## APIs

| API | Meaning |
|---|---|
| `CompiledOrderIr::lower` | Plan order → IR |
| `compare_rows` / `build_sort_tuple` | Sort + cursor tuple |
| `retain_after_sort_tuple` / `cmp_sort_tuples` | Multipage field-order resume |
| `key_from_sort_tuple` | Key-stream resume helper |
| `execute_plan` | Uses the above; no private compare helpers |

---

## Residual after IR2

Still in `core_page` / `full_attach`:

- page / limit / cursor mint packing
- coverage policy / holes
- enrich / within / brace project

See [QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md).
