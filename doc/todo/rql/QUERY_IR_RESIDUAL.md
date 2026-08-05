# Query IR residual — what is still not a finished bytecode machine

Status: **2026-08-05 · RQL-VM1R**
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**
**RQL-C1 must not be accepted.** Principal rejected prior VM1, P1c, and D0 closure —
see [QUERY_VM_V1.md](./QUERY_VM_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md).

Named IR phases remain **Rust IR residual**. Intermediate VM phase work exists.
**QVM1** durable encoding + `VmPool` landed. **VM1R** unified dispatch into one
`run_vm`. Honest residual: `RQB1` is still the public AST carrier; sql/json/mongo
dialects still compile to SDA.

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | `isa.rs` (`RQB1`) | AST carrier — lowers into QVM at execute |
| Durable QVM wire | `qvm.rs` (`QVM1`) | **QVM1 labor closed** |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate |
| Core path-project / order / page | `ir_project.rs` / `ir_order.rs` / `ir_page.rs` | **Named IR** — still Rust |
| Enrich / within helpers | `full_attach.rs` / `ir_attach.rs` | Called from attach opcodes |
| Query VM opcodes | `vm.rs` | **Vocabulary (VM0)** |
| Dispatch | `vm_exec.rs` (`run_vm`) | **VM1R labor closed** (prior VM1 claim rejected) |
| Core opcode phases | `core_phases.rs` | Intermediate (VM2–VM4) |
| Host by `CollectionId` | `HostCapabilities` | **P1b closed** |
| Foreign cache by id | `vm_exec` / `full_attach` | **R1 closed** |
| Dialect id `rql` → SDA | `dialects` | **Retired (R1 refuse)** |
| sql/json/mongo → SDA | `dialects` | **Residual** (must become QVM) |

---

## Explicit non-claims

- Intermediate VM2–VM4 ≠ Decision 0 closed
- QVM1 / VM1R ≠ Decision 0 closed / every frontend → QVM
- Prior VM1 / P1c claims ≠ accepted
- **RQL-C1 must not be accepted**
- NEXT labor = dialect→QVM — **not** principal C1

---

## Evidence

- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
- `doc/todo/rql/evidence/rql_qvm1_durable_bytecode.log`
- `doc/todo/rql/evidence/rql_vm1r_one_run_vm.log`
- prior VM2–VM4 evidence logs under `doc/todo/rql/evidence/`

---

## Evidence

- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
- `doc/todo/rql/evidence/rql_qvm1_durable_bytecode.log`
- prior VM2–VM4 evidence logs under `doc/todo/rql/evidence/`
