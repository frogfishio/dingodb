# Query IR residual — what is still not a finished bytecode machine

Status: **2026-08-05 · RQL-R1**
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**
**RQL-C1 must not be accepted.** Principal rejected VM1, P1c, and D0 closure —
see [QUERY_VM_V1.md](./QUERY_VM_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md).

Named IR phases remain **Rust IR residual**. Intermediate VM phase work exists.
Honest residual: RQB1 is still an AST carrier; `VmProgram` sidecars remain;
two dispatch loops remain; sql/json/mongo dialects still compile to SDA;
durable QVM encoding is **mandatory** (QVM1).

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` (`RQB1`) | AST carrier — **not** final QVM authority |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate |
| Core path-project / order / page | `ir_project.rs` / `ir_order.rs` / `ir_page.rs` | **Named IR** — still Rust |
| Enrich / within helpers | `full_attach.rs` / `ir_attach.rs` | Called from attach opcodes |
| Query VM opcodes | `vm.rs` | **Vocabulary (VM0)** |
| Dispatch | `vm_exec.rs` (`run_vm_core` + `run_vm_attach`) | **Rejected as “one machine”** — VM1R residual |
| Core opcode phases | `core_phases.rs` | Intermediate (VM2–VM4) |
| Host by `CollectionId` | `HostCapabilities` | **P1b closed** |
| Foreign cache by id | `vm_exec` / `full_attach` | **R1 closed** |
| Dialect id `rql` → SDA | `dialects` | **Retired (R1 refuse)** |
| sql/json/mongo → SDA | `dialects` | **Residual** (must become QVM) |
| Durable QVM wire | — | **Mandatory residual (QVM1)** |

---

## Explicit non-claims

- Intermediate VM2–VM4 ≠ Decision 0 closed
- Prior VM1 / P1c claims ≠ accepted
- **RQL-C1 must not be accepted**
- NEXT labor = **QVM1** then **VM1R** — **not** principal C1

---

## Evidence

- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
- prior VM2–VM4 evidence logs under `doc/todo/rql/evidence/`
