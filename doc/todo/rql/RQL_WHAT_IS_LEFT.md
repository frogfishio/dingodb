# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** Public non-ISA execute helpers are now crate-private, but there is still
**no** one Query VM opcode machine.
**RQL-C1 must not be accepted. Decision 0 must not be closed.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **P0b labor closed** (helpers `pub(crate)`) |
| ISA = executable Query VM | **False** — still serialized plan + Rust interpreters |
| Collection operands bound by immutable id | **Partial** — D0R harden for enrich/within |
| Canonical ISA (reserved bits + re-encode) | **Partial** — D0R harden |
| One collection-qualified host API | **False** — Core vs full still diverge |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = Query VM programme (RQL-VM0 instruction set)
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
| **VM0** | Charter real Query VM instruction set | [QUERY_VM_V1.md](./QUERY_VM_V1.md) |
| **VM1** | One instruction-dispatch machine (Core + Full) | single loop |
| **VM2** | Plans/IR compile-only; delete semantic executors after equivalence | |
| **P1b** | Unify host behind collection-qualified `HostCapabilities` | |
| **P1c** | Arch test: every frontend → same dispatch loop | |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (P0b)

- `execute_plan` / `execute_decoded_core` / attach+project+order+page helpers → `pub(crate)`
- SDK `lib.rs` re-exports only ISA/compile/explain sanctioned entries
- Integration tests routed through `execute_isa_bytes` / `execute_rql_full`
- Evidence: `doc/todo/rql/evidence/rql_p0b_private_api.log`

---

## One-line status

```text
NEXT        = RQL-VM0 (Query VM instruction set)
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4; D0R harden; P0b public ISA-only surface
HONESTY     = still not one Query VM — see QUERY_VM_V1.md
```
