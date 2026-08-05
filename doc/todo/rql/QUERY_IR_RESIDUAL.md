# Query IR residual — what is still not a bytecode machine

Status: **2026-08-05 · RQL-X5c**  
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**  
**RQL-C1 must not be accepted.**

This page is the honesty ledger for X5c. ISA sole input (X5/X5b) is **not**
the same as “one bytecode machine owns all query meaning.”

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
    │               execute_decoded_core ──► execute_plan (Rust)
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

There is no second Core executor. Full does **not** re-encode Core ISA before
the base page.

---

## Phase ledger — machine vs residual Rust

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` | Durable AST carrier — **not** an opcode machine |
| `where` / attach filters / candidate `where` | `kernel.rs` → SDA | Kernel substrate — **not** full query bytecode |
| Page / limit / cursor resume | `core_page.rs` | **Rust IR residual** |
| Order / sort-tuple | `core_page.rs` | **Rust IR residual** |
| Core `project` (path list) | `core_page.rs` | **Rust IR residual** |
| Coverage policy / holes | `core_page.rs` | **Rust IR residual** |
| Enrich / within attach | `full_attach.rs` | **Rust IR residual** |
| Brace `project { … }` | `full_attach.rs` | **Rust IR residual** |
| Host scan / index / get | `HostCapabilities` | Allowed host boundary |

Anything marked **Rust IR residual** still interprets decoded Rust structures.
That is why Decision 0 remains **OPEN**.

---

## Explicit non-claims

- X5 + X5b + X5c ≠ Decision 0 closed
- X5c ≠ RQL-C1
- Shared `execute_decoded_core` ≠ bytecode machine for order/page/project/coverage/enrich

---

## Evidence

- `doc/todo/rql/evidence/rql_x5c_dispatch.log`
- Arch gate requires this file + `execute_decoded_core` sharing
