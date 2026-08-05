# Query VM v1 — one bytecode, one semantic executor

Status: **charter / residual** · Decision 0 **OPEN** · RQL-C1 **forbidden**  
Authority: principal reject of premature D0/C1 (2026-08-05) ·
[RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

IR1–IR4 are **accepted intermediate labor**. They organized Rust interpreters of
ISA-decoded structures. They are **not** Decision 0 closed and **not** a finished
bytecode machine.

---

## Hard invariant

```text
All syntax → compiler intermediates → canonical Query ISA
                                      ↓
                              exactly one Query VM
                                      ↓
                          collection-qualified host API
```

---

## What “done” requires

1. **Real instruction set** covering scan, filter, project, order, page, enrich, within.
2. **One instruction-dispatch machine** for Core and Full RQL.
3. Rust plans / IR are **compiler intermediates only** — never execute query semantics.
4. **No public non-ISA** execution APIs (`execute_plan`, attach/project helpers, …).
5. Every collection operand bound to **immutable heap/collection identity**.
6. One **collection-qualified** host-capability interface (no Full→`HeapClient` bypass).
7. **Canonical ISA**: reject reserved bits; decode → re-encode equality; size limits.
8. Architectural test: every frontend reaches the **same** dispatch loop.
9. Delete old semantic executors after equivalence tests pass.

---

## Current honesty

| Layer today | Status |
|---|---|
| `residiuum-query-isa-v1` | Durable **serialized plan/AST carrier** — not opcode VM |
| `execute_plan` / `CompiledAttachIr::run` | Rust semantic interpreters |
| Public SDK re-exports | **P0b:** non-ISA execute helpers crate-private; ISA entries remain public |
| Host | Core: `HostCapabilities`; Full: `HeapClient` (P1b residual) |

---

## Non-claims

- Do **not** accept Decision 0 closure.
- Do **not** accept RQL-C1.
- Named IR ≠ Query VM.
