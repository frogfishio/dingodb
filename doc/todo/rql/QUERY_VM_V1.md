# Query VM v1 — instruction set (RQL-VM0)

Status: **labor closed 2026-08-05** · board RQL-VM0 `577b4a0b`  
Profile id: **`residiuum-query-vm-v1`** · version byte **`1`**  
Runtime stamp: `query_bytecode_v1/vm.rs`  
Companion: [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md)

This freezes the **opcode vocabulary** for one Query VM covering Core and Full
RQL. It does **not** implement the dispatch loop (**RQL-VM1**) and does **not**
close Decision 0 / accept RQL-C1.

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
VM0 defines the **machine** those carriers must lower into. Product execute
still uses Rust interpreters until VM1 lands.

---

## Machine model

| Component | Role |
|---|---|
| **Program** | Sequence of [`OpCode`] + immediates + const pool |
| **Working set** | Ordered row bag `(key, json)` under transformation |
| **Frame** | Bound collection id(s); nested within stack |
| **Host** | scan / index / get only — **by immutable collection id** (P1b) |
| **Kernel** | Predicate eval via `residiuum-query-kernel-sda-v1` |

Rules:

1. One dispatch loop interprets all opcodes (Core and Full share it).
2. `RqlPlanV1` / IR structs are **compiler intermediates only** — never product
   execute entry after VM1 equivalence.
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

Rust: `query_bytecode_v1::OpCode` / `VM_PROFILE`.

---

## Canonical Core lowering sketch

```text
BindCollection(base_id)
IndexEq? | Scan
Filter(where)
Order(terms)?
Page(page_size, coverage)
ProjectPaths(paths)?
Halt
```

## Canonical Full lowering sketch

```text
… Core prefix …
Enrich(using_id, …)* | Within … WithinEnd | FilterAttach*
ProjectBrace?
Halt
```

Nested enrich inside `Within`…`WithinEnd` uses the same `Enrich` opcode with
frame-relative left paths.

---

## Relationship to `residiuum-query-isa-v1`

| Artifact | Role now | Role after VM1 |
|---|---|---|
| `RQB1` ISA bytes | Durable AST carrier + execute authority | Compile input / exchange; lower → QVM ops |
| `OpCode` program | **Defined (VM0)**; not executed | Sole semantic executor input |
| `execute_plan` / attach IR | Crate-private Rust interpreters | Delete after equivalence (VM2) |

A future durable **QVM wire encoding** (magic/version distinct from `RQB1`) may
ship with VM1; until then, in-memory `Vec<Instruction>` is the machine form.

---

## Non-claims

- VM0 ≠ dispatch machine (that is **RQL-VM1**)
- VM0 ≠ Decision 0 closed
- **RQL-C1 must not be accepted**
- Named IR phases ≠ Query VM

---

## Evidence

- `doc/todo/rql/evidence/rql_vm0_opcodes.log`
