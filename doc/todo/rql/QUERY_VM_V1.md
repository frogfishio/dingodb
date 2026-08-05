# Query VM v1 — instruction set + dispatch

Status: **2026-08-05** · Principal **rejected VM1 / P1c** · Decision 0 remains OPEN · RQL-C1 **forbidden**
Profile id: **`residiuum-query-vm-v1`** · version byte **`1`** · durable magic **`QVM1`**
Runtime: `query_bytecode_v1/vm.rs` · `vm_exec.rs` · `qvm.rs` · `core_phases.rs` (`CoreFrame`)
Companion: [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

Opcode vocabulary (**RQL-VM0**) and intermediate Core/Full phase work (**VM2–VM4**,
including `CoreFrame` / demoted `run_core_page`) landed. **RQL-QVM1** freezes a
durable `QVM1` encoding of the opcode stream + constant pool; product execute
materializes QVM bytes before run. That does **not** close Decision 0: two
dispatch loops remain (`run_vm_core` / `run_vm_attach` → **VM1R**), and `RQB1`
remains the public AST carrier that lowers into QVM. Principal rejected prior
VM1 / P1c convergence claims.

---

## Hard invariant

```text
All syntax → compiler intermediates → canonical QVM bytecode
                                      ↓
                              exactly one run_vm
                                      ↓
                          collection-qualified host API
```

Today: `RQB1` decode → lower → **`encode_qvm` / `decode_qvm`** → `run_vm_*`.
`VmProgram` holds ops + `VmPool` only (no plan/pipeline/project sidecars).

---

## Machine model (honest)

| Component | Role today | Residual |
|---|---|---|
| **`QVM1` bytes** | Durable executable form (ops + pool) | Public wire still often `RQB1` |
| **Program** | Opcode vector + `VmPool` (Core plan) | — |
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

Rust: `query_bytecode_v1::OpCode` / `VM_PROFILE` / `qvm` / `vm_exec`.

---

## Non-claims

- QVM1 ≠ Decision 0 complete / C1
- QVM1 ≠ one `run_vm` (that is VM1R)
- Prior VM1 / P1c board claims remain **rejected**
- **RQL-C1 must not be accepted**
- NEXT = **VM1R** then dialect→QVM

---

## Evidence

- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
- `doc/todo/rql/evidence/rql_vm2_core_phases.log`
- `doc/todo/rql/evidence/rql_vm3_materialize_split.log`
- `doc/todo/rql/evidence/rql_vm3b_filter_scan_split.log`
- `doc/todo/rql/evidence/rql_vm4_within_flatten.log`
- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
- `doc/todo/rql/evidence/rql_qvm1_durable_bytecode.log`
