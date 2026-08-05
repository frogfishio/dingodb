# RQL — what is left to do

Status: **2026-08-05** · **X4 Core kernel landed** · Decision 0 in force  
Detail: [QUERY_KERNEL_SDA_V1.md](./QUERY_KERNEL_SDA_V1.md) · [QUERY_ISA_V1.md](./QUERY_ISA_V1.md)

---

## Ordered programme

```text
… → X3 ISA → X4 Core SDA kernel (done) → X4b full_attach kernel → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Labor** | **RQL-X4b** | Port `full_attach` filters / candidate `where` onto the same SDA kernel (no parallel `Predicate::eval` product path) |

---

## Just shipped (X4)

- Core `where` → SDA via `residiuum-query-kernel-sda-v1`
- `execute_plan` filters through kernel; oracle tests vs `Predicate::eval`
- Evidence: `doc/todo/rql/evidence/rql_x4_kernel.log`

---

## One-line status

```text
NEXT labor  = RQL-X4b full_attach on SDA kernel
LANDED      = Core where meaning via residiuum-sda (+ ISA + unified runtime)
HONESTY     = attach still uses Predicate::eval internally
```
