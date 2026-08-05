# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Opcode vocabulary is frozen (**RQL-VM0**), but there is still **no**
dispatch machine executing those opcodes.
**RQL-C1 must not be accepted. Decision 0 must not be closed.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **P0b labor closed** |
| Query VM opcode set defined | **VM0 labor closed** — [QUERY_VM_V1.md](./QUERY_VM_V1.md) |
| One opcode dispatch machine | **False** — residual **RQL-VM1** |
| Collection-qualified host API | **False** — residual **RQL-P1b** |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = RQL-VM1 one Query VM dispatch machine
```

---

## Hard acceptance invariant (principal)

```text
All syntax → compiler intermediates → canonical Query ISA
                                      ↓
                              exactly one Query VM
                                      ↓
                          collection-qualified host API
```

---

## Ordered residual (mandatory labor)

| # | Package | Exit |
|---|---|---|
| **D0R** | SoT + enrich/within `using_id` bind + ISA reserved/canonical | **labor closed** |
| **P0b** | Privatize public non-ISA execute/project/attach APIs | **labor closed** |
| **VM0** | Define Query VM instruction set | **labor closed** |
| **VM1** | One instruction-dispatch machine (Core + Full) | single loop |
| **VM2** | Plans/IR compile-only; delete semantic executors after equivalence | |
| **P1b** | Unify host behind collection-qualified `HostCapabilities` | |
| **P1c** | Arch test: every frontend → same dispatch loop | |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (VM0)

- `query_bytecode_v1/vm.rs` — `residiuum-query-vm-v1` / `OpCode` vocabulary
- [QUERY_VM_V1.md](./QUERY_VM_V1.md) — machine model + Core/Full lowering sketches
- No dispatch yet (honest residual → VM1)
- Evidence: `doc/todo/rql/evidence/rql_vm0_opcodes.log`

---

## One-line status

```text
NEXT        = RQL-VM1 (one opcode dispatch machine)
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4; D0R; P0b; VM0 opcode freeze
HONESTY     = opcodes defined, not yet executed — see QUERY_VM_V1.md
```
