# Query VM v1 — instruction set + dispatch

Status: **VM0–VM3b + P1c labor closed 2026-08-05** · board RQL-VM3b `5f907b28`  
Profile id: **`residiuum-query-vm-v1`** · version byte **`1`**  
Runtime: `query_bytecode_v1/vm.rs` (opcodes) · `vm_exec.rs` (dispatch) · `core_phases.rs` (`CoreFrame`)  
Companion: [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

Opcode vocabulary (**RQL-VM0**), one dispatch machine (**RQL-VM1**), Core
opcode → `CoreFrame` phases (**RQL-VM2/VM3**), and key-stream Filter separate
from Scan (**RQL-VM3b**). Does **not** close Decision 0 / accept RQL-C1.
Honest residual: nested Within on immediates; IR helpers remain Rust; no
durable QVM wire encoding.

---

## Hard invariant

```text
All syntax → compiler intermediates → canonical Query ISA / QVM program
                                      ↓
                              exactly one Query VM (opcode dispatch)
                                      ↓
                          collection-qualified host API
```

Today (`residiuum-query-isa-v1` / `RQB1`) remains a durable **AST carrier**.
Product execute lowers decoded plans into a QVM program and runs
`run_vm_core` / `run_vm_attach`. Core opcodes call `CoreFrame` phase helpers
with a working bag (**RQL-VM3**); Scan leaves `PendingKeys` and Filter owns
`where` (**RQL-VM3b**). `execute_plan` / `run_core_page` are thin orchestrators
only. Host Full attach uses collection-qualified `HostCapabilities` (**RQL-P1b**).

---

## Machine model

| Component | Role |
|---|---|
| **Program** | Sequence of [`OpCode`] + typed immediates (`VmProgram`) |
| **Working set** | Ordered row bag `(key, json)` under transformation |
| **Frame** | Bound collection id(s); nested within still carried on Within imm |
| **Host** | scan / index / get only — **by immutable collection id** (P1b) |
| **Kernel** | Predicate eval via `residiuum-query-kernel-sda-v1` |

Rules:

1. One dispatch loop interprets opcodes (Core via `run_vm_core`; Full attach via `run_vm_attach`).
2. `RqlPlanV1` / IR structs remain **compiler intermediates**; product entry is ISA → lower → VM. Nested Within flatten / QVM wire remain residuals after VM3b.
3. Every collection operand is an immutable `CollectionId` (name is diagnostic).
4. Unknown opcode bytes and reserved immediates **refuse**.

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
| `0x60` | `Enrich` | Root enrich (foreign id + match + cardinality + output) |
| `0x61` | `Within` | Enter bag scope at path |
| `0x62` | `WithinEnd` | Leave within body |
| `0x63` | `FilterAttach` | Post-attach filter |
| `0x64` | `ProjectBrace` | Brace `project { … }` |
| `0xFF` | `Halt` | Yield page |

Reserved ranges: `0x00`, `0x02–0x0F` unused bind, `0x12–0x1F`, `0x21–0x2F`,
`0x31–0x3F`, `0x41–0x4F`, `0x51–0x5F`, `0x65–0xEF`, `0xF0–0xFE`.

Rust: `query_bytecode_v1::OpCode` / `VM_PROFILE` / `vm_exec`.

---

## Canonical Core lowering sketch

```text
BindCollection(base_id)
IndexEq | Scan
Filter
Order
Page
ProjectPaths
Halt
```

## Canonical Full lowering sketch

```text
… Core prefix …
Enrich(using_id, …)* | Within … WithinEnd | FilterAttach*
ProjectBrace?
Halt
```

Nested enrich inside `Within`…`WithinEnd` remains on the `Within` immediate
until a later slice expands nested ops onto the flat stream.

---

## Relationship to `residiuum-query-isa-v1`

| Artifact | Role now (after VM3b) | Residual |
|---|---|---|
| `RQB1` ISA bytes | Durable AST carrier + execute authority; lower → QVM | Compile input / exchange |
| `OpCode` / `VmProgram` / `CoreFrame` | **Dispatched** product path with working bag | Nested Within flatten |
| `PendingKeys` | Scan→Filter handoff (no where in Scan) | — |
| `execute_plan` / `run_core_page` | Thin `CoreFrame` orchestrators (crate-private) | Prefer VM entry only |
| Attach helpers | Called from Full attach opcode bodies | Expand nested Within later |

In-memory typed `VmProgram` is the machine form; a durable QVM wire encoding
may ship later (magic/version distinct from `RQB1`).

---

## Non-claims

- VM3b / P1c ≠ Decision 0 complete / C1
- Nested Within on imm ≠ flat opcode stream
- **RQL-C1 must not be accepted**
- NEXT labor = optional nested Within flatten / QVM wire
- Named IR phases ≠ finished Query VM alone

---

## Evidence

- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
- `doc/todo/rql/evidence/rql_vm1_dispatch.log`
- `doc/todo/rql/evidence/rql_vm2_core_phases.log`
- `doc/todo/rql/evidence/rql_p1c_frontend_dispatch.log`
- `doc/todo/rql/evidence/rql_vm3_materialize_split.log`
- `doc/todo/rql/evidence/rql_vm3b_filter_scan_split.log`
