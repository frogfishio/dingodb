# Query kernel SDA v1 — ENR+SDA eval substrate (RQL-X4)

Status: **labor closed 2026-08-05** · board RQL-X4 `401f7ef5`  
Profile id: **`residiuum-query-kernel-sda-v1`**  
Runtime: `query_bytecode_v1/kernel.rs` · Companion: [QUERY_ISA_V1.md](./QUERY_ISA_V1.md)

Core `where` meaning evaluates through **`residiuum-sda`** boolean programs.
Host remains scan / index / get only.

---

## 1. Law

| Layer | Role |
|---|---|
| RQL / plan / ISA | Compile + durable carrier |
| **Kernel** (`residiuum-query-kernel-sda-v1`) | Predicate → SDA text → `Program::run_json` |
| Host | Data access only |

[`Predicate::eval`](../../../crates/residiuum-sdk/src/predicate.rs) remains the
**equivalence oracle** in tests. Product Core page execution uses the kernel.

---

## 2. Lowering (Residiuum Absent/Null honesty)

Uses `getPath` / `mapOpt` / `Some` / `None` so:

- **Absent = x** → false (`None ≠ Some(…)`)
- **Absent ≠ x** → false (`mapOpt(None, …) ≠ Some(true)`)
- **Present null** vs **missing** distinguished (`Some(null)` vs `None`)

Params are substituted as SDA literals at compile-where time (bind-once).

---

## 3. Product APIs

| API | Meaning |
|---|---|
| `compile_where` / `lower_predicate` | Plan `where` → SDA |
| `CompiledKernelWhere::eval_doc` | Bool against one document |
| Core `execute_plan` | Compiles where once; filters via kernel |

---

## 4. Residual (RQL-X4b)

- Full-language attach / nested `where` / enrich match still uses Rust
  `Predicate::eval` inside `full_attach` — port to the same kernel next.
- `$key` in `where` refused by kernel (order tie-break only today).
- Wire does not yet ship SDA source / ISA+kernel stamps separately.

---

## 5. Amendment

Semantic changes to Absent/Null lowering require RQL-0 note + board card.
