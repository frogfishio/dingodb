# RQL — what is left to do

Status: **2026-08-05** · **X3 landed** · Decision 0 in force  
Detail: [QUERY_ISA_V1.md](./QUERY_ISA_V1.md) · [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md)

---

## Ordered programme

```text
Decision 0 → X1 → X2 → X2b → X2c → X2d → X3 ISA (done) → X4 ENR+SDA kernel eval → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X4** | Execute Core (then attach) by lowering ISA / plan into ENR+SDA kernel eval — not a parallel Rust algebra |

---

## Just shipped (X3)

- Durable ISA profile `residiuum-query-isa-v1` (`RQB1`)
- Core+full encode/decode; `QueryBytecodeV1.isa` stamped on lower
- `execute_isa_bytes` for Core through the same runtime
- Evidence: `doc/todo/rql/evidence/rql_x3_isa.log`

---

## One-line status

```text
NEXT labor  = RQL-X4 ENR+SDA kernel eval substrate
LANDED      = durable ISA carrier + unified runtime (emb + op 118)
HONESTY     = ISA is AST carrier; not yet residiuum-sda eval
```
