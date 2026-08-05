# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal **rejected** D0 closure + RQL-C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md)

---

## Are we done?

**No.** IR1–IR4 organized Rust interpreters; they did **not** deliver one bytecode
machine / one semantic executor.
**RQL-C1 must not be accepted. Decision 0 must not be closed.**

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated ISA | **False** — residual (RQL-P0b) |
| ISA = executable Query VM | **False** — still serialized plan + Rust interpreters |
| Collection operands bound by immutable id | **Partial** — D0R harden for enrich/within |
| Canonical ISA (reserved bits + re-encode) | **Partial** — D0R harden |
| One collection-qualified host API | **False** — Core vs full still diverge |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = Query VM programme (mandatory implementation)
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
| **D0R** | SoT + enrich/within `using_id` bind + ISA reserved/canonical | this harden slice |
| **P0b** | Privatize public non-ISA execute/project/attach APIs | crate-private helpers; ISA entries only |
| **VM0** | Charter real Query VM instruction set | [QUERY_VM_V1.md](./QUERY_VM_V1.md) |
| **VM1** | One instruction-dispatch machine (Core + Full) | single loop |
| **VM2** | Plans/IR compile-only; delete semantic executors after equivalence | |
| **P1b** | Unify host behind collection-qualified `HostCapabilities` | |
| **P1c** | Arch test: every frontend → same dispatch loop | |
| **C1** | Principal only — **never** before invariant holds | |

---

## Just shipped (D0R)

- Principal reject of D0/C1 recorded; NEXT is Query VM work
- Enrich/within open-by-name verifies encoded `using_id`
- ISA rejects reserved flag/budget bits; execute paths require canonical re-encode
- Evidence: `doc/todo/rql/evidence/rql_d0r_harden.log`

---

## One-line status

```text
NEXT        = RQL-P0b / RQL-VM0 (mandatory labor)
FORBIDDEN   = Decision 0 close; RQL-C1 accept
LANDED      = IR1–IR4 intermediate; D0R identity/canonical harden
HONESTY     = still not one Query VM — see QUERY_VM_V1.md
```
