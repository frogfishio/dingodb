# RQL — what is left to do

Status: **2026-08-05** · **Decision 0 OPEN** · Principal rejected premature C1  
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [QUERY_ISA_V1.md](./QUERY_ISA_V1.md)

---

## Are we done?

**No.** Prior “labor DONE / Decision 0 closed” SoT was **false** and is withdrawn.
**RQL-C1 must not be accepted.**

| Claim | Reality |
|---|---|
| Core ISA sole executable input | **X5 labor closed** — private envelope; decode drives exec |
| Full RQL on same ISA runtime | **False** — still bypasses until **X5b** |
| One bytecode machine owns all query meaning | **False** — page/order/project/coverage/enrich still Rust interpreters of decoded structures |
| Ready for RQL-C1 | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT labor  = RQL-X5b full-language execute from ISA
```

---

## Blocking findings (principal) — status

1. ISA does not control Core execution — **addressed in X5** (decode-only path + mismatch test).
2. Full RQL bypasses ISA — **open → X5b**.
3. Most Core semantics still Rust plan interpreter — **open → X5c** (honest residual).
4. Arch check filename-only — **partially addressed** (behavioral decode/private-field gate).
5. Tests weak on ISA identity — **partially addressed** (non-empty mismatch + corrupt ISA).

---

## Just shipped (X5 — Core only)

- `QueryBytecodeV1` holds private ISA bytes only
- `execute_bytecode` → `execute_isa_bytes` → `decode_isa` → page exec
- Evidence: `doc/todo/rql/evidence/rql_x5_isa_sole.log`

---

## Ordered residual

| # | Who | Package | Exit |
|---|---|---|---|
| **1** | Labor | **RQL-X5** | **labor closed** — evidence `rql_x5_isa_sole.log` |
| **2** | **Labor** | **RQL-X5b** | Full-language execute only via ISA encode→decode; non-empty E2E |
| **3** | Labor | **RQL-X5c** | One dispatch honesty; order/project/page/coverage IR residual |
| **4** | Principal | **RQL-C1** | Only after X5+; **never before** |

---

## One-line status

```text
NEXT labor  = RQL-X5b execute full-language from ISA
FORBIDDEN   = RQL-C1 accept (Decision 0 OPEN)
LANDED      = X5 Core ISA sole input (private envelope + decode)
HONESTY     = scaffolding + X5 ≠ full convergence
```
