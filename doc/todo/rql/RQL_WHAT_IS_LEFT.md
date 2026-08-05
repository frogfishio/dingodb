# RQL — what is left to do

Status: **2026-08-05** · **X4b landed** · Decision 0 in force  
Detail: [QUERY_KERNEL_SDA_V1.md](./QUERY_KERNEL_SDA_V1.md) · [QUERY_ISA_V1.md](./QUERY_ISA_V1.md)

---

## Ordered programme

```text
… → X3 ISA → X4 Core kernel → X4b attach kernel (done) → …
```

---

## Do next

| # | Who | Package | What “done” means |
|---|---|---|---|
| **1** | **Principal / labor** | **RQL-C1** (or waiver) | Core product accept residuals for APP-6 / APP-7 / APB-7 scoreboard |
| **2** | Labor (optional) | Wire / `$key` residuals | Op 118 ISA packing; `$key` in `where` kernel lower |

---

## Just shipped (X4b)

- `full_attach` `filter_rows` + enrich `candidate_where` use SDA kernel
- No product `Predicate::eval` under `query_bytecode_v1/` (test oracle only)
- Evidence: `doc/todo/rql/evidence/rql_x4b_attach_kernel.log`

---

## One-line status

```text
NEXT        = RQL-C1 accept (principal) / optional wire+$key residuals
LANDED      = one runtime + ISA + SDA kernel for Core and attach filters
HONESTY     = Decision 0 predicate meaning via residiuum-sda
```
