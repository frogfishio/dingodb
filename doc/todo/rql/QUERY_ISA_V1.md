# Query ISA v1 — durable program encoding (RQL-X3)

Status: **labor closed 2026-08-05** · board RQL-X3 `82113ba2`  
Profile id: **`residiuum-query-isa-v1`**  
Companion: [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · runtime crate `query_bytecode_v1/isa.rs`

This freezes the **durable binary carrier** for compiled query programs so the
product ISA is not “whatever Rust structs are in memory today.”

---

## 1. Law

| Layer | Role |
|---|---|
| Syntax / logical plan | Compile inputs |
| **ISA bytes** (`residiuum-query-isa-v1`) | Durable program identity + exchange |
| Runtime (`query_bytecode_v1`) | Sole semantic executor |
| Host | scan / index / get only |

In-memory `RqlPlanV1` / pipeline structs are **views**. Wire, hash, and
persistence should bind to ISA bytes (see [`isa_hash`]).

---

## 2. Binary layout

Little-endian. Version byte `1`.

```text
magic[4] = "RQB1"
version  = u8(1)
flags    = u8
  bit0 = has_budget
  bit1 = has_full_pipeline
core_len = u32
core     = rql-plan-encoding-v1 body (RqlPlanV1::canonical_bytes)
[budget] = flag byte + optional u64 fields (documents / bytes / result_bytes)
[full]   = u32 len + canonical JSON section (pipeline + brace project)
```

Core body reuses the already-frozen plan encoding profile
(`rql-plan-encoding-v1`). Full section uses sorted-key JSON with predicate
canonical JSON for filters / candidate `where`.

---

## 3. Product APIs

| API | Meaning |
|---|---|
| `encode_core_program` / `encode_full_program` | Lower plan (+ attach) → ISA |
| `decode_isa` | ISA → `QueryIsaProgram` |
| `QueryBytecodeV1.isa` | Always stamped on Core lower |
| `execute_isa_bytes` | Decode Core ISA → same `execute_bytecode` path |
| `isa_hash` | Domain-separated BLAKE3-256 over ISA bytes |

Full-language ISA is encode/decode proven; Core wire execute stays on
`execute_isa_bytes` / `execute_core_rql`. Full execute still enters via
`execute_rql_full*` after compile (same runtime module).

---

## 4. Honesty / residual

- Encoding is **ENR+SDA-equivalent AST carrier**, not yet evaluation inside
  `residiuum-sda`. Kernel eval as shared substrate is **RQL-X4**.
- No claim that remote op 118 ships ISA bytes on the wire yet (packing residual).

---

## 5. Amendment

Amending magic / version / section semantics requires an RQL-0 note and board
card. Silent second carriers are a Decision 0 violation.
