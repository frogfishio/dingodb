# RQL-Q2.2c — `pkg_array_predicate_surface`

Status: **labor complete** (2026-08-08)  
Authority: Q2.1 audit rank 3; [RESIDIUUM_PREDICATE_SPEC.md](../../reference/query/RESIDIUUM_PREDICATE_SPEC.md) §2 / §6.5; RQL_SPEC grammar note

## Gap (pre-fix)

Seven Tier-A cases failed **syntax** at compile:

| Case | Source |
|---|---|
| `commerce.orders.array_tags_empty` | `tags = []` |
| `messaging.messages.has_attachments` | `attachments != []` |
| `directory.entries.tags_empty` | `tags = []` |
| `directory.entries.tags_contains_vip` | `tags contains "vip"` |
| `telemetry.events.tags_empty` | `tags = []` |
| `project_management.tasks.blocked_by_empty` | `blocked_by = []` |
| `project_management.projects.labels_empty` | `labels = []` |

Function-form `contains(tags, "sku")` already executed (not in residual set).

## Delivered

1. **Spec freeze** — array-literal production; infix `path contains literal` ≡ function form; empty-array equality is ordinary SDA structural `=` / `!=` (absent ≠ empty).
2. **Parse** — `rql_app_core` accepts `[]` / non-empty array literals as operands and literals; infix `contains` after a path.
3. **Execute path** — existing `Predicate::Cmp` + kernel SDA `Seq[]` lowering; existing `Predicate::Contains` bag/string semantics (no second executor).
4. **Tests** — unit compile tests in `rql_app_core`; kernel oracle empty-array + contains.
5. **Re-audit** — execute_ok **129**/147 (was 122); gap **16** (was 23); `pkg_array_predicate_surface` **0** residual.

## Non-claims

- Not Q2 package accept / not 100% Tier A
- Not Q3 result-correctness oracle
- Not Decision 0 / RQL-C1
- Object/map literals still out of V1 surface

## Evidence

- `doc/reference/query/RESIDIUUM_PREDICATE_SPEC.md`
- `doc/wip/query/RQL_SPEC.md` (array predicate pointer + reserved words)
- `crates/residiuum-sdk/src/rql_app_core.rs`
- `crates/residiuum-sdk/src/query_bytecode_v1/kernel.rs`
- `spec/rql/qualification/corpus-v1/q2_1_capability_audit.json`
