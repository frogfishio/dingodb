# RQL — what is left to do

Status: **2026-08-07** · **Decision 0 OPEN** · Principal **rejected** prior VM1, P1c, D0 closure, RQL-C1
Detail: [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) ·
[QUERY_IR_RESIDUAL.md](./QUERY_IR_RESIDUAL.md) · [QUERY_VM_V1.md](./QUERY_VM_V1.md) ·
[RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md) (**D0.1**) ·
[RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md) (**D0.2 principal checklist**)

---

## Are we done?

**No.** Public product bytecode is **QVM1** (typed opcode immediates + identity
pool; no `RqlPlanV1` sidecar on `VmPool`). sql/json/mongo builtins compile to
portable Filter → QVM; raw SDA is restricted to dialect `sda` /
`compile_sda_source` / `Collection::sda`. One `run_vm` dispatches Core + Full.
Decision 0 remains OPEN: IR helpers are still Rust phase bodies, and principal
has not accepted C1. **RQL-C1 must not be accepted.** Prior VM1 / P1c
convergence claims stay rejected.

| Claim | Reality |
|---|---|
| IR1–IR4 named phases | **Accepted as intermediate labor** |
| Public execute only via validated bytecode (Core/Full SDK) | **P0b + WIRE1 labor closed** (QVM1 public) |
| Query VM opcode vocabulary | **VM0 accepted as design foundation** |
| Collection-qualified host API | **P1b labor closed** |
| Core opcodes drive phases / Within flatten | **VM2–VM4 accepted as intermediate** (VM3b labor closed) |
| Durable QVM wire (`QVM1` magic) + typed operands | **QVM1 labor closed** — no plan sidecar |
| One `run_vm` | **VM1R labor closed** — Core + Full enter the same loop |
| Dialects sql/json/mongo → QVM | **DQ1 labor closed** (portable → QVM; not SDA) |
| Every product frontend → same QVM | **P1c rejected** as prior claim; dialect path now QVM (R1+DQ1) |
| Ready for RQL-C1 / Decision 0 close | **Forbidden** |

```text
Verdict     = Decision 0 OPEN; RQL-C1 must NOT be accepted
NEXT        = Q0 ACCEPT done; claim Q1.1 corpus schema; Decision 0 still OPEN
D0 close bar= A1–A8 product QVM path (A9 micro-op purity NOT required — Q0.A7)
```

---

## Hard acceptance invariant (principal)

```text
RQL / SQL-ish+ / JSON / Mongo / Builder / Wire
        → canonical QVM bytecode → one run_vm → HostCapabilities

Raw SDA → explicitly raw SDA APIs only (dialect `sda` / Collection::sda)
```

---

## Just shipped (typed QVM + WIRE1 + DEL1 + correctness)

- Architecture gate forbids deleted `run_core_page` / `execute_plan`
- `VmPool` holds plan_hash / coverage / consistency only (no `RqlPlanV1`)
- Core opcodes carry typed immediates (`Where`, `Order`, `Page`, `Project`, …)
- `QueryBytecodeV1` stores **QVM1** bytes; `isa_hash` = `qvm_hash`
- QVM verifier: single terminal Halt, Core prefix grammar, Within balance
- `op_count` bounded (`QVM_MAX_OPS` + remaining-byte check)
- `order_by` / `force_scan` compiled into portable dialect → QVM
- `compile_json_value` → `CompiledPortable` (not SDA)
- Custom `QueryDialect` is portable-only; raw SDA is explicit surface only
- Filter is sole where authority (`IndexEq` is force_scan only)
- Cursor identity = `qvm_hash` of complete canonical QVM bytes (not wire plan_hash)
- Full product path: compile → `encode_qvm` → `execute_full_qvm_with` (**RQB1 deleted** Q0.A10; QVM1 only)
- Canonical decode: `encode(decode(bytes)) == bytes`
- Public QVM API is byte-oriented (`validate_qvm` / `qvm_hash`); `encode`/`decode` crate-private
- `run_attach_pipeline` deleted; dead Compiled*Ir wrappers removed
- Evidence path: architecture gate + focused corpus

---

## Ordered residual

| # | Package | Exit |
|---|---|---|
| **R1** | Retire dialect rql→SDA; cache-by-id; arch honesty | **labor closed** |
| **QVM1** | Durable QVM bytecode; eliminate plan/pipeline sidecars | **labor closed** |
| **VM1R** | One `run_vm` (repair rejected VM1) | **labor closed** |
| **DQ1** | Dialects sql/json/mongo → portable → QVM | **labor closed** |
| **WIRE1** | Public QVM1 wire (store/hash/execute) | **labor closed** |
| **DEL1** | Delete obsolete private executors | **labor closed** (gate forbids) |
| **D0.1** | Residual IR honesty inventory | **labor closed** → board `done` (IR = tech-debt; Q0.A7) |
| **D0.2** | Principal close readiness checklist | **labor closed**; A9 out_of_scope_for_D0_close (Q0.A7) |
| **C1** | Principal only — **never** before invariant holds | |

---

## One-line status

```text
NEXT        = Q0 package accept (RQL_Q0_PRINCIPAL_ACCEPT §5); labor hold active
FORBIDDEN   = Decision 0 close; RQL-C1 accept; claim prior VM1/P1c converged; claim Q1 under hold
LANDED      = D0R; P0b; P1b; VM0 vocab; VM2–VM4 intermediate; R1; QVM1; VM1R; DQ1; WIRE1; DEL1; D0.1; D0.2; Q0 pack; Q0.6 hold
HONESTY     = IR residual + open A9–A11; Decision 0 OPEN; Q1 [BLOCKED:Q0] — see RQL_LABOR_HOLD.md
```