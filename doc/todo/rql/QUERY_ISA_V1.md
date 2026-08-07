# Query ISA v1 — RETIRED historical note (was RQL-X3)

> **RETIRED (Q0.A10 / Q0.A12).** This document is **not** a live product carrier.
> Magic `RQB1`, module `query_bytecode_v1/isa.rs`, and all RQB1 encode/decode/import/execute
> APIs are **deleted**. Product durable authority is **`QVM1`** only — see
> [QUERY_VM_V1.md](./QUERY_VM_V1.md) · [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md).
> Architecture gate forbids restoring `isa.rs` / RQB1 product symbols.
> Preserve this file as historical evidence of the former RQL-X3 design; body text below is **not normative**.

Status: **RETIRED 2026-08-07** · original labor closed 2026-08-05 · board RQL-X3 `82113ba2`
Profile id (historical): **`residiuum-query-isa-v1`**
Companion (live): [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · runtime: **`qvm.rs` (QVM1)** — not `isa.rs`

~~Former freeze of durable binary carrier…~~ **Superseded:** durable carrier is QVM1.

---

## 1. Law

| Layer | Role |
|---|---|
| Syntax / logical plan | Compile inputs |
| **ISA bytes** (`residiuum-query-isa-v1`) | Durable program identity + exchange |
| Runtime (`query_bytecode_v1`) | Sole semantic executor |
| Host | scan / index / get only |

In-memory `RqlPlanV1` / pipeline structs are **decoded views of ISA bytes**.
Wire, hash, and Core execution bind to ISA bytes (see [`isa_hash`] /
`QueryBytecodeV1::isa_bytes`). **RQL-X5:** Core `execute_bytecode` decodes ISA
only — no independent executable `plan` field.

**Residual:** full-language still bypasses ISA until **RQL-X5b**. Page/order/
project/coverage remain Rust interpreters of **ISA-decoded** Core plans
(Decision 0 still open).

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
| `decode_isa` | ISA → `QueryIsaProgram` (reserved bits + size caps) |
| `decode_isa_canonical` | Decode + require canonical re-encode equality |
| `QueryBytecodeV1.isa` | Always stamped on Core lower |
| `execute_isa_bytes` | Canonical decode Core ISA → crate-private `execute_decoded_core` |
| `execute_decoded_core` | **crate-private** shared Core page after decode (X5c) |
| `execute_full_isa_with` | Canonical decode full ISA → Core + attach |
| `execute_plan` / attach helpers | **crate-private** (RQL-P0b) — not public ISA bypass |
| `isa_hash` | Domain-separated BLAKE3-256 over ISA bytes |

Full-language execute: `execute_rql_full*` compiles then
`encode_full_program` → `execute_full_isa_with` (RQL-X5b). Core wire stays on
`execute_isa_bytes` / `execute_core_rql`. `CompiledRqlFull` is not an
executable authority. Residual interpreters:
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md).

---

## 4. Honesty / residual

- Encoding is durable AST carrier; Core **and attach filter** eval is via
  [QUERY_KERNEL_SDA_V1.md](./QUERY_KERNEL_SDA_V1.md) (X4 / X4b).
- **RQL-D0R:** reserved top-level / budget flag bits rejected; product execute
  uses `decode_isa_canonical` (re-encode equality). Size caps:
  `ISA_MAX_TOTAL_BYTES` / `ISA_MAX_SECTION_BYTES`.
- Still **not** an opcode Query VM — see [QUERY_VM_V1.md](./QUERY_VM_V1.md).
- No claim that remote op 118 ships ISA bytes on the wire yet (packing residual).

---

## 5. Amendment

Amending magic / version / section semantics requires an RQL-0 note and board
card. Silent second carriers are a Decision 0 violation.
