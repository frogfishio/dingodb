# RQL — what is left to do

Status: **2026-08-05** · **X2c full-attach port landed** · Decision 0 in force  
Detail: [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)

---

## Ordered programme

```text
Decision 0 → X1 → X2 → X2b Core → X2c full attach (done) → X2d delete shims + op118 → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X2d** | Delete `query_exec_v1` + `rql_full_v1` shim modules (callers → bytecode); unify op **118** through same runtime; CI/docs honesty |

---

## Just shipped (X2c)

- Full attach lives in `query_bytecode_v1/full_attach.rs`
- `rql_full_v1` is re-export shim only (same pattern as `query_exec_v1`)
- CI allowlist: **only** `query_bytecode_v1/` may define `pub fn execute_*`
- Evidence: `doc/todo/rql/evidence/rql_x2c_full_port.log`

---

## One-line status

```text
NEXT labor  = RQL-X2d delete shims + unify op 118
LANDED      = Core + full attach owned by query_bytecode_v1/
STILL THERE = thin compat shims (no local execute_* bodies)
```
