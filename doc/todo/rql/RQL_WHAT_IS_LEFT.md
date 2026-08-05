# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** VM1, P1c, D0 closure, RQL-C1
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Intermediate labor (VM2–VM4 phase work, P0b, P1b, D0R) is useful, but the
architecture is **not converged**. Principal **rejects** claims that there is
already one instruction-dispatch machine (VM1) or that every frontend enters it
(P1c). **Durable QVM bytecode is mandatory** (not optional). **RQL-C1 must not
be accepted.** Decision 0 remains OPEN.

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA (Core/Full SDK) | **P0b labor closed** |
| Query VM opcode vocabulary | **VM0 accepted as design foundation** |
| Collection-qualified host API | **P1b labor closed** |
| Core opcodes drive phases / Within flatten | **VM2–VM4 accepted as intermediate** (VM3b labor closed) |
| One `run_vm` / no semantic sidecars | **Rejected / residual** — see QVM1 + VM1R |
| Every product frontend → same QVM | **P1c rejected** — dialect `rql`→SDA retired (R1); sql/json/mongo still → SDA |
| Durable QVM wire as sole executable | **Mandatory residual (QVM1)** |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = QVM1 (durable bytecode) → VM1R (one run_vm); then dialect→QVM
```

---

## Hard acceptance invariant (principal)

```text
RQL / SQL-ish+ / JSON / Mongo / Builder / Wire
        → canonical QVM bytecode → one run_vm → HostCapabilities

Raw SDA → explicitly raw SDA APIs only (dialect `sda` / Collection::sda)
```

---

## Just shipped (RQL-R1)

- Dialect id `rql` **refuses** on `compile_dialect` / `find_dialect` (no parallel RQL→SDA)
- Legacy `dialects/rql` compiler is **test-only**
- Foreign doc cache keyed by `CollectionId` (names diagnostic only)
- Arch gate enumerates dialect/SDA surfaces; SoT marks VM1/P1c **rejected**
- Evidence: `doc/todo/rql/evidence/rql_r1_dialect_cache_arch.log`

---

## Ordered residual

| # | Package | Exit |
|---|---|---|
| **R1** | Retire dialect rql→SDA; cache-by-id; arch honesty | **labor closed** |
| **QVM1** | Durable QVM bytecode; eliminate plan/pipeline sidecars | **todo** |
| **VM1R** | One `run_vm` (repair rejected VM1) | **todo** |
| Dialects sql/json/mongo → QVM | Still dialect→SDA today | **residual** |
| Delete obsolete private executors / oracles | Drift risk | **residual** |
| Bytecode mutation tests | Fail-closed on opcode edit | **residual** |
| **C1** | Principal only — **never** before invariant holds | |

---

## One-line status

```text
NEXT        = QVM1 durable bytecode (mandatory); then VM1R one run_vm
FORBIDDEN   = Decision 0 close; RQL-C1 accept; claim VM1/P1c converged
LANDED      = D0R; P0b; P1b; VM0 vocab; VM2–VM4 intermediate; R1
HONESTY     = RQB1 still AST carrier; two dispatch loops; sql/json/mongo→SDA; IR Rust
```
