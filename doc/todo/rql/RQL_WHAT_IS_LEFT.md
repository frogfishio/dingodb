# RQL — what is left to do

Status: **2026-08-05** · **X2 foundation landed** · Decision 0 in force  
Authority: [CRITICAL_PATH.md](../../../CRITICAL_PATH.md)  
Detail: [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md) · [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §0

---

## Ordered programme

```text
Decision 0 → X1 freeze → X2 foundation (done) → X2b port/delete → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X2b** | Port Core (+ then full) semantics into `query_bytecode_v1`; prove equivalence; delete `query_exec_v1` / `execute_rql_full`; tighten CI; unify op 118 |
| **2** | **Human** | Review X2 foundation | Confirm single entry + host trait + CI gate |

---

## Just shipped (X2 foundation)

- `crates/residiuum-sdk/src/query_bytecode_v1.rs` — `HostCapabilities`, `QueryBytecodeV1`, `execute_core_rql` / `execute_bytecode`
- Embedded `CollectionClient.rql` + builder `run` route through that entry
- CI: `scripts/check_query_runtime_architecture.sh` (wired in `.github/workflows/ci.yml`)
- Evidence: `doc/todo/rql/evidence/rql_x2_foundation.log`

**Honesty:** Core page algebra still physically lives in frozen `query_exec_v1` (delegated). That is migration, not a second product entry.

---

## Hard freeze (still)

| Item | Rule |
|---|---|
| Feature growth on `query_exec_v1` / `rql_full_v1` | Forbidden |
| New `pub fn execute_*` outside allowlist | CI fail |
| Host | scan / index / get only |

---

## One-line status

```text
NEXT labor  = RQL-X2b port semantics + delete frozen executors
LANDED      = query_bytecode_v1 product entry + CI anti-executor
NOT next    = S1, D1, façade features
```
