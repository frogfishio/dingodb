# Query IR residual — what is still not a finished bytecode machine

Status: **2026-08-07 · aligned to [RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md)**
Authority: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · Decision 0 **OPEN**
**RQL-C1 must not be accepted.** Principal rejected prior VM1, P1c, and D0 closure —
see [QUERY_VM_V1.md](./QUERY_VM_V1.md) · [RQL_WHAT_IS_LEFT.md](./RQL_WHAT_IS_LEFT.md).

Named IR phases remain **Rust IR residual**. Intermediate VM phase work exists.
**QVM1** durable encoding + `VmPool` landed. **VM1R** unified dispatch into one
`run_vm`. **DQ1** landed: sql/json/mongo → portable → QVM (not SDA text).
Honest residual: opcode bodies are still large Rust phase interpreters of typed
immediates — not a pure stack micro-VM. Full cite table: **D0.1 inventory**.

---

## Phase ledger

| Phase | Location | Status |
|---|---|---|
| ISA encode/decode carrier | ~~`isa.rs` (`RQB1`)~~ | **Deleted (Q0.A10)** — retired; [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) historical only |
| Durable QVM wire | `qvm.rs` (`QVM1`) | **QVM1 labor closed** (public authority) |
| `where` / attach filters | `kernel.rs` → SDA | Kernel substrate (not second executor) |
| Core path-project / order / page | `ir_project.rs` / `ir_order.rs` / `ir_page.rs` | **Named IR** — still Rust |
| Enrich / within helpers | `full_attach.rs` / `vm_exec` | Called from attach opcodes; `ir_attach` stamp only |
| Query VM opcodes | `vm.rs` | **Vocabulary (VM0)** |
| Dispatch | `vm_exec.rs` (`run_vm`) | **VM1R labor closed** (prior VM1 claim rejected) |
| Core opcode phases | `core_phases.rs` | Intermediate (VM2–VM4) |
| Host by `CollectionId` | `HostCapabilities` | **P1b closed** |
| Foreign cache by id | `vm_exec` / `full_attach` | **R1 closed** |
| Dialect id `rql` → SDA | `dialects` | **Retired (R1 refuse)** |
| sql/json/mongo → portable → QVM | `dialects` + `Collection::find_portable_with` | **DQ1 labor closed** |
| Fused orchestrators | `run_core_page` / `execute_plan` / `run_attach_pipeline` | **Deleted (DEL1)** |

---

## Explicit non-claims

- Intermediate VM2–VM4 ≠ Decision 0 closed
- QVM1 / VM1R / DQ1 ≠ Decision 0 closed / pure micro-VM
- Prior VM1 / P1c claims ≠ accepted
- **RQL-C1 must not be accepted**
- NEXT labor = residual IR honesty inventory + principal review — **not** false C1

---

## Evidence

- Full inventory: [RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md)
- `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`
- `doc/todo/rql/evidence/rql_qvm1_durable_bytecode.log`
- `doc/todo/rql/evidence/rql_vm1r_one_run_vm.log`
- prior VM2–VM4 / DQ1 / WIRE1 / DEL1 evidence logs under `doc/todo/rql/evidence/`
- `bash scripts/check_query_runtime_architecture.sh`
