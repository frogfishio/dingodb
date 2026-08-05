# Query runtime convergence — one bytecode, one runtime

Status: **principal Decision 0 · 2026-08-05**  
Board: RQL-0D `d06c909f` · Feature `1a8a3e05`  
Authority: [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) §0 · [DIALECTS.md](../../SDA/DIALECTS.md)

This charter freezes the **architectural violation** of parallel semantic
executors and names the convergence work. It is not product accept.

---

## Doctrine (end state)

```text
RQL source ─┐
SQL+ ───────┼─► canonical logical plan ─► one query bytecode ─► one runtime
Rust builder ┘                                      │
                                                    └─► host capabilities
                                                        scan / index / get
```

| Allowed | Forbidden |
|---|---|
| Multiple syntaxes | Multiple semantic executors |
| Multiple compiler stages | Host-owned filter / join / project / order / cardinality / page / missing-null / coverage meanings |
| Multiple physical access strategies (scan vs index) | A second production runtime beside the bytecode machine |

The storage host supplies **data access primitives** only. Query meaning lives
in the **one bytecode machine**.

**Honesty:** ENR+SDA today compiles to **text** and evaluates in
`residiuum-sda`. That is the intended kernel lineage, but it is **not yet** a
frozen query bytecode with a host-capability boundary. Freezing that boundary
is the first convergence deliverable.

---

## Violation in tree (do not grow)

| Surface | Role today | Status under Decision 0 |
|---|---|---|
| `query_exec_v1` (APP-6 / op 118 Core) | Production-shaped **semantic** page executor | **Feature freeze** |
| `execute_rql_full` (`rql-full-v1`) | Local attach / project **semantic** façade | **Feature freeze** |
| Dialect `rql` → ENR+SDA text → `residiuum-sda` | Doctrine-aligned subset path | Keep as lineage; not a second product story |
| Test-only reference interpreter | Oracle | Allowed **only** in tests; never a product path |

---

## Convergence sequence

1. **Freeze** all feature development in `query_exec_v1` and `execute_rql_full`
   (bugfix / evidence honesty only).
2. **Define** the single executable bytecode and its host-capability boundary
   (scan / index / get — nothing semantic).
3. **Lower** `RqlPlanV1`, full RQL, SQL+, and the Rust builder into that bytecode.
4. **Route** embedded execution and op **118** through the same runtime.
5. **Port** every useful feature from the two Rust executors into the bytecode
   machine (not by growing those executors).
6. **Prove** old/new result equivalence during migration.
7. **Delete** the production Rust executors and their semantic implementations.
8. **Add** a CI architecture test that fails if another semantic executor appears.

Until steps 1–2 are frozen ([QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md)),
**no additional RQL feature work** (S1, D1, wire parity, within-index, …)
proceeds.

### Step status (2026-08-05)

| Step | State |
|---|---|
| 1 Freeze `query_exec_v1` / `execute_rql_full` features | **done** (Decision 0 + module banners) |
| 2 Define bytecode + host boundary | **done** — [QUERY_BYTECODE_V1.md](./QUERY_BYTECODE_V1.md) |
| 3 Lower + single product entry (Core) | **foundation done** — `query_bytecode_v1` + `CollectionClient.rql` / builder `run` |
| 4 Route op 118 through same runtime | **residual** (wire still server-local Core compile; emb entry unified) |
| 5–7 Port Core / equivalence / shrink frozen Core executor | **X2b done** — Core in `query_bytecode_v1/core_page`; `query_exec_v1` shim |
| 5–8 Port full attach; delete façades; unify op 118; CI bytecode-only | **RQL-X2c** |

---

## Non-claims

- This charter does not accept APP-6 / APP-7 / APB-7 or full RQL-v1.
- Phase 3 attach labor remains useful **evidence / port inventory**, not a
  second product runtime.
- `in_review` ≠ architecture frozen ≠ package accept.
