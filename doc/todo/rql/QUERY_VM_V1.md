# Query VM v1 — instruction set + dispatch

Status: **2026-08-05** · Principal **rejected VM1 / P1c** · Decision 0 remains OPEN · RQL-C1 **forbidden**
Profile id: **`residiuum-query-vm-v1`** · version byte **`1`**
Runtime: `query_bytecode_v1/vm.rs` (opcodes) · `vm_exec.rs` (dispatch) · `core_phases.rs` (`CoreFrame`)
Companion: [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

Opcode vocabulary (**RQL-VM0**) and intermediate Core/Full phase work (**VM2–VM4**,
including `CoreFrame` / demoted `run_core_page`) landed. That does **not** close
Decision 0 and does **not** mean one finished machine: today there are still
`run_vm_core` and `run_vm_attach`, and `VmProgram` retains Core/pipeline/project
**sidecars**. Durable QVM bytecode is **mandatory** (QVM1) — not optional.
Principal rejected prior VM1 / P1c convergence claims.

---

## Hard invariant

```text
All syntax → compiler intermediates → canonical QVM bytecode
                                      ↓
                              exactly one run_vm
                                      ↓
                          collection-qualified host API
```

Today (`residiuum-query-isa-v1` / `RQB1`) remains a durable **AST carrier**.
Product execute lowers decoded plans into an in-memory `VmProgram` and runs
separate Core / attach loops. That façade is **not** yet canonical QVM
authority.

---

## Machine model (honest)

| Component | Role today | Residual |
|---|---|---|
| **Program** | Opcode vector + Core/pipeline/project sidecars | Eliminate sidecars (QVM1) |
| **Working set** | Ordered row bag `(key, json)` | — |
| **Dispatchers** | `run_vm_core` + `run_vm_attach` | One `run_vm` (VM1R) |
| **Host** | scan / index / get by `CollectionId` | — |
| **Foreign cache** | Keyed by `CollectionId` (R1) | — |

---

## Opcode map (`u8`)

| Byte | Name | Meaning |
|---|---|---|
| `0x01` | `BindCollection` | Bind base collection by id |
| `0x10` | `Scan` | Host `list_keys` stream |
| `0x11` | `IndexEq` | Host equality-index probe (may fall back to Scan) |
| `0x20` | `Filter` | Kernel predicate over working set |
| `0x30` | `ProjectPaths` | Core path-project |
| `0x40` | `Order` | Sort / sort-tuple |
| `0x50` | `Page` | Page size + coverage + cursor |
| `0x60` | `Enrich` | Enrich (root or nested inside Within…WithinEnd) |
| `0x61` | `Within` | Enter bag scope at path (shell imm) |
| `0x62` | `WithinEnd` | Leave within body; write elements back |
| `0x63` | `FilterAttach` | Post-attach filter (root or nested) |
| `0x64` | `ProjectBrace` | Brace `project { … }` |
| `0xFF` | `Halt` | Yield page |

Rust: `query_bytecode_v1::OpCode` / `VM_PROFILE` / `vm_exec`.

---

## Non-claims

- VM0–VM4 intermediate ≠ Decision 0 complete / C1
- Prior VM1 / P1c board claims are **rejected** by principal
- Flat Within stream ≠ durable QVM wire
- **RQL-C1 must not be accepted**
- NEXT = **QVM1** (mandatory durable bytecode) then **VM1R** (one `run_vm`)

---

## Evidence

- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
- `doc/todo/rql/evidence/rql_vm2_core_phases.log`
- `doc/todo/rql/evidence/rql_vm3_materialize_split.log`
- `doc/todo/rql/evidence/rql_vm3b_filter_scan_split.log`
- `doc/todo/rql/evidence/rql_vm4_within_flatten.log`
- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
