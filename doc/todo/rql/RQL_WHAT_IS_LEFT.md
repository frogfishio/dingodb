# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** prior VM1, P1c, D0 closure, RQL-C1
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Durable **QVM1** bytes exist; product Core/Full execute materializes them;
`VmProgram` has no plan/pipeline/project sidecars; and **one `run_vm`** dispatches
Core + Full (**VM1R labor closed**). Decision 0 remains OPEN: `RQB1` is still the
public AST carrier, sql/json/mongo still → SDA, and IR helpers remain Rust.
**RQL-C1 must not be accepted.** Prior VM1 / P1c convergence claims stay rejected.

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA (Core/Full SDK) | **P0b labor closed** |
| Query VM opcode vocabulary | **VM0 accepted as design foundation** |
| Collection-qualified host API | **P1b labor closed** |
| Core opcodes drive phases / Within flatten | **VM2–VM4 accepted as intermediate** (VM3b labor closed) |
| Durable QVM wire (`QVM1` magic) | **QVM1 labor closed** — pool + ops; execute via materialize |
| One `run_vm` | **VM1R labor closed** — Core + Full enter the same loop |
| Every product frontend → same QVM | **P1c rejected** — dialect `rql`→SDA retired (R1); sql/json/mongo still → SDA |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = dialect→QVM (sql/json/mongo); then delete obsolete executors
```

---

## Hard acceptance invariant (principal)

```text
RQL / SQL-ish+ / JSON / Mongo / Builder / Wire
        → canonical QVM bytecode → one run_vm → HostCapabilities

Raw SDA → explicitly raw SDA APIs only (dialect `sda` / Collection::sda)
```

---

## Just shipped (RQL-VM1R)

- Deleted dual dispatchers `run_vm_core` / `run_vm_attach`
- One `run_vm` + `VmOutcome` (Core page + attach rows in one machine frame)
- Full path: `lower_full` → `materialize_qvm` → `run_vm` once (no Core re-entry)
- Evidence: `doc/todo/rql/evidence/rql_vm1r_one_run_vm.log`

---

## Ordered residual

| # | Package | Exit |
|---|---|---|
| **R1** | Retire dialect rql→SDA; cache-by-id; arch honesty | **labor closed** |
| **QVM1** | Durable QVM bytecode; eliminate plan/pipeline sidecars | **labor closed** |
| **VM1R** | One `run_vm` (repair rejected VM1) | **labor closed** |
| Dialects sql/json/mongo → QVM | Still dialect→SDA today | **residual** |
| Delete obsolete private executors / oracles | Drift risk | **residual** |
| Public wire cutover RQB1 → QVM1 | RQB1 still compile carrier | **residual** |
| **C1** | Principal only — **never** before invariant holds | |

---

## One-line status

```text
NEXT        = dialect→QVM (sql/json/mongo); then delete obsolete executors
FORBIDDEN   = Decision 0 close; RQL-C1 accept; claim prior VM1/P1c converged
LANDED      = D0R; P0b; P1b; VM0 vocab; VM2–VM4 intermediate; R1; QVM1; VM1R
HONESTY     = RQB1 still AST carrier; sql/json/mongo→SDA; IR Rust
```
