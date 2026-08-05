# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Labor closed through **VM3b** (Filter owns where; Scan does not), but Decision 0
remains OPEN: IR phases are still Rust, nested Within remains on immediates, and
no durable QVM wire encoding. **RQL-C1 must not be accepted.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **P0b labor closed** |
| Query VM opcode set + dispatch | **VM0+VM1 labor closed** |
| Collection-qualified host API | **P1b labor closed** |
| Core opcodes drive phases | **VM2 labor closed** — `CoreFrame` |
| Every product frontend → same dispatch loop | **P1c labor closed** |
| Scan→Project opcode-owned bodies | **VM3 labor closed** |
| Key-stream Filter separate from Scan | **VM3b labor closed** — `PendingKeys` |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = optional nested Within flatten / QVM wire; C1 remains principal-only
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
| **VM1** | One instruction-dispatch machine (Core + Full) | **labor closed** |
| **P1b** | Unify host behind collection-qualified `HostCapabilities` | **labor closed** |
| **VM2** | Core opcodes → `CoreFrame` phases; demote `execute_plan` | **labor closed** |
| **P1c** | Arch test: every frontend → same dispatch loop | **labor closed** |
| **VM3** | Split materialize into opcode-owned bodies | **labor closed** |
| **VM3b** | Key-stream Filter separate from Scan | **labor closed** |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (VM3b)

- Scan establishes `PendingKeys` (Index / Stream / Materialized) — **no** `where`
- Filter owns `get` + `where` + APP-6 early-stop for key-stream
- Evidence: `doc/todo/rql/evidence/rql_vm3b_filter_scan_split.log`

---

## One-line status

```text
NEXT        = optional nested Within flatten / QVM wire encoding
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4; D0R; P0b; VM0–VM3b; P1b; P1c
HONESTY     = IR still Rust; nested Within on imm; no durable QVM wire
```
