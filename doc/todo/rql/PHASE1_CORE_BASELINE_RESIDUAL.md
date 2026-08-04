# RQL Phase 1 — Application Core baseline residual (labor)

Status: **labor 2026-08-04** · package `APB-7` / `APP-6` / `APP-7` **active / not accept**  
Authority: [PATH_TO_FULL_RQL.md](./PATH_TO_FULL_RQL.md) ·  
[APB7_DUAL_BACKEND_SUITE.md](../application-baseline/APB7_DUAL_BACKEND_SUITE.md) ·  
[NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md)

Board card: RQL PATH **T1** `0a5c700a` (Query spine Feature `1a8a3e05`).

This document records **labor evidence for Phase 1 closeout readiness**.
It does **not** authorize scoreboard package accept.

---

## 1. What Phase 1 needs

| Need | Labor status |
|---|---|
| Comprehensive Core **compile** corpus | **expanded** — `spec/app/v1/rql_app_core_corpus_v1.json` accept **15** / reject **17** |
| Compile → **execute** + multipage oracles | **added** — `crates/residiuum-sdk/tests/app_core_execute_corpus.rs` |
| APB-7 dual-pack + slice gates | **re-run green** (embedded) — see §2 |
| Dual backend / op **118** | **re-run green** — `apb7_query_from_remote_collection_plane` 1/1 (after `FindWirePage` host-test adapt) |
| Scoreboard `APB-7` / `APP-6` / `APP-7` → **accept** | **principal only** |

---

## 2. Gate commands (disk-safe)

```bash
export TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-sdk --lib rql_app_core -- --test-threads=1
cargo test -p residiuum-sdk --test app5_rql_app_core -- --test-threads=1
cargo test -p residiuum-sdk --test app4_predicate_plan -- --test-threads=1
cargo test -p residiuum-sdk --test app_core_execute_corpus -- --test-threads=1
cargo test -p residiuum-sdk --test app6_page_executor -- --test-threads=1
cargo test -p residiuum-sdk --test app6_field_order_multipage -- --test-threads=1
cargo test -p residiuum-sdk --test apb7_query_dual_pack -- --test-threads=1
cargo test -p residiuum-sdk --test apb7_multipage_oracle_matrix -- --test-threads=1
# … remaining apb7_* as in APB7_DUAL_BACKEND_SUITE.md
```

Evidence log (this turn): `doc/todo/rql/evidence/phase1_core_gate.log`

---

## 3. Accept residuals (principal checklist)

Still **forbidden** for labor to mark package accept while any row is open:

| Residual | Owner |
|---|---|
| HAR-4 package accept / ceremony residual | principal + HAR lane |
| Range / multi-field index pushdown multipage | APB-7 residual |
| ReadView / SI multipage under pin | APB-6 / APB-7 T5 |
| Heap-confined durable cursor secrets | APB-7 T10 residual |
| Principal scoreboard gate for APP-6 / APP-7 / APB-7 | **principal** |

---

## 4. Explicit non-claims

- Expanded corpus + execute oracle ≠ APB-7 package accept.
- Op 118 active ≠ product “query baseline qualified.”
- Full RQL-v1 (`enrich` / `within` / …) stays **backlog** (`89a80e77`) until Phase 1 accept.
- Store packaging **0.2.2** does not qualify RQL.

---

## 5. Next pull after principal Phase 1 accept

1. Promote Phase 2 expressiveness / SQL-ish refuse matrix (board T2 `b4ebdaf9`).
2. Only then pull full-language card `89a80e77`.
