# RQL — what is left to do

Status: **2026-08-05** · **X2b Core port landed** · Decision 0 in force  
Detail: [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)

---

## Ordered programme

```text
Decision 0 → X1 → X2 foundation → X2b Core port (done) → X2c full+delete → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X2c** | Port `rql_full_v1` attach into bytecode runtime; delete `query_exec_v1` shim + production `execute_rql_full`; unify op 118; tighten CI allowlist to bytecode-only |

---

## Just shipped (X2b)

- Core page semantics live under `query_bytecode_v1/core_page.rs`
- `query_exec_v1` is a **re-export shim only** (CI enforces no local `pub fn execute_*`)
- Product entry remains `execute_core_rql` / `execute_bytecode`
- Evidence: `doc/todo/rql/evidence/rql_x2b_core_port.log`

---

## One-line status

```text
NEXT labor  = RQL-X2c port full attach + delete frozen façades
LANDED      = Core semantics owned by query_bytecode_v1
STILL THERE = rql_full_v1 façade + query_exec_v1 shim (compat)
```
