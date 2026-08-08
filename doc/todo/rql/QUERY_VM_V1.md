# Query VM v1 — instruction set + dispatch

Status: **2026-08-07** · Principal **rejected prior VM1 / P1c** · **VM1R labor closed** · Decision 0 remains OPEN · RQL-C1 **forbidden**
Profile id: **`residiuum-query-vm-v1`** · version byte **`1`** · durable magic **`QVM1`**
Runtime: `query_bytecode_v1/vm.rs` · `vm_exec.rs` · `qvm.rs` · `core_phases.rs` (`CoreFrame`)
Companion: [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md) ·
[RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md)
Historical (retired RQB1): [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) — **not** a live product carrier.

Opcode vocabulary (**RQL-VM0**) and intermediate Core/Full phase work (**VM2–VM4**,
including `CoreFrame`; fused `run_core_page` deleted DEL1) landed. **RQL-QVM1**
freezes durable `QVM1` encoding; product execute materializes QVM bytes before
run. **RQL-VM1R** unifies dispatch into one `run_vm` (Core + Full). **RQL-DQ1**
routes sql/json/mongo through portable → QVM. That does **not** close Decision 0:
opcode bodies remain Rust phase interpreters; prior VM1 / P1c claims stay rejected.

---

## Hard invariant

```text
All syntax → compiler intermediates → canonical QVM bytecode
                                      ↓
                              exactly one run_vm
                                      ↓
                          collection-qualified host API
```

Today (product): compile → **`encode_qvm` / `decode_qvm`** → **`run_vm`**.
**RQB1 is deleted** (Q0.A10): no `isa.rs`, no `from_isa_bytes`, no RQB1 encode/execute.
`VmProgram` holds ops + `VmPool` only (no plan/pipeline/project sidecars).

---

## Machine model (honest)

| Component | Role today | Residual |
|---|---|---|
| **`QVM1` bytes** | Durable executable form (ops + pool) | Full language not on op-118 Core wire |
| **Program** | Opcode vector + `VmPool` | — |
| **Dispatcher** | One `run_vm` (**VM1R labor closed**) | Bodies still Rust phases (IR residual) |
| **Dialects sql/json/mongo** | Portable → QVM (**DQ1 closed**) | Synthetic name-derived ids on `Collection` |
| **Host** | scan / index / get by `CollectionId` | — |
| **Foreign cache** | Keyed by `CollectionId` (R1) | Nested within-enrich scan residual |

---

## Opcode map (`u8`)

| Byte | Name | Meaning |
|---|---|---|
| `0x01` | `BindCollection` | Bind base collection by id |
| `0x10` | `Scan` | Host `list_keys` stream |
| `0x11` | `IndexEq` | Host equality-index probe (may fall back to Scan) |
| `0x20` | `Filter` | Kernel predicate over working set |
| `0x30` | `ProjectPaths` | Core path-project; optional group/agg payload in Project imm (Q2) |
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

- QVM1 / VM1R / DQ1 ≠ Decision 0 complete / C1 / pure micro-VM
- Prior VM1 / P1c board claims remain **rejected**
- **RQL-C1 must not be accepted**
- NEXT = residual IR honesty + principal Decision 0 review (see D0 inventory)

---

## Evidence

- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
- `doc/todo/rql/evidence/rql_vm2_core_phases.log`
- `doc/todo/rql/evidence/rql_vm3_materialize_split.log`
- `doc/todo/rql/evidence/rql_vm3b_filter_scan_split.log`
- `doc/todo/rql/evidence/rql_vm4_within_flatten.log`
- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
- `doc/todo/rql/evidence/rql_qvm1_durable_bytecode.log`
- `doc/todo/rql/evidence/rql_vm1r_one_run_vm.log`