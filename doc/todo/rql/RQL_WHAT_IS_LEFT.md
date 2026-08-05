# RQL — what is left to do

Status: **2026-08-05** · **X2d landed** · Decision 0 in force  
Detail: [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) · [QUERY_RUNTIME_CONVERGENCE.md](./QUERY_RUNTIME_CONVERGENCE.md)

---

## Ordered programme

```text
Decision 0 → X1 → X2 → X2b → X2c → X2d (done) → X3 binary ISA / kernel lower → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X3** | Durable binary (or ENR+SDA-equivalent) bytecode encoding; stop treating Rust AST + text SDA as the product ISA |

---

## Just shipped (X2d)

- Deleted `query_exec_v1.rs` + `rql_full_v1.rs` shim modules
- Op **118** uses `HostCapabilities` + `execute_core_rql` / `explain_core_source`
- CI forbids shim files; requires `execute_core_rql` on server dispatch
- Evidence: `doc/todo/rql/evidence/rql_x2d_shim_delete.log`

---

## One-line status

```text
NEXT labor  = RQL-X3 durable bytecode ISA / kernel lower
LANDED      = one product runtime under query_bytecode_v1/ (emb + op 118)
HONESTY     = encoding still Rust plan/AST intermediate — not frozen binary ISA yet
```
