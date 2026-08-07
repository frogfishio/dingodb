# RQL-Q1 — Practical query corpus (human report)

Status: **Q1.2 Commerce + Messaging landed** (2026-08-07) · package **not accepted**  
Package: RQL-Q1 · Feature `019fda4c-11fd-7102-bd55-10a347802144`  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §4  
Machine corpus: [`spec/rql/qualification/corpus-v1/`](../../../spec/rql/qualification/corpus-v1/)

## 1. Delivery shape (fixed)

Programme + Q0 index require **exactly**:

1. Machine-readable corpus data (schema + versioned cases + fixtures)
2. **One** short human report (this file)

Not another multi-document Q0-style freeze family.

## 2. Version identity

| Field | Value |
|---|---|
| Format | `residiuum-rql-q1-corpus-v1` |
| Profile | `rql-gate1-practical-corpus-v1` |
| Corpus version | **`rql-q1-corpus-v0.2.0`** (was v0.1.0 scaffold) |
| Equivalence profile | `rql-q0-result-equivalence-v1` |
| Q0 freeze tip (authority) | `e1f5c670a99dc54da477c531c83bca4985199a42` |
| Live cases | **54** (commerce 33 + messaging 21); status `draft` |
| Generators | [`generators/`](../../../spec/rql/qualification/corpus-v1/generators/) + `tools/rql_q1/materialise_fixture.py` |

## 3. Case record contract

Every case must carry programme §4.2 fields. Normative JSON Schema:

- Wrapper: `spec/rql/qualification/corpus-v1/corpus-v1.schema.json`
- Case: `spec/rql/qualification/corpus-v1/corpus-case-v1.schema.json`

| Group | Fields |
|---|---|
| Identity | `case_id`, `tier` (A/B/C), `domain` (five required domains) |
| Intent | `plain_english_intent` |
| Fixture | `fixture.generator_id` + `fixture.seed` (+ optional params) |
| Expected | `expected.kind` ∈ literal / oracle_rule / stable_refusal / deferred_q2 |
| Order | `ordering_and_multiplicity` |
| Engines | `implementations.rql` + `.mongo` + `.cbl` (source, pipeline/find, sqlpp/builder, or refusal) |
| Indexes | `indexes.required` / `optional` |
| Classes | `selectivity_class`, `cardinality_class` |
| Variants | `variants.missing_null_type`, `variants.cursor_page` |
| Exclusion | `exclusion_or_refusal` |
| Floors | `family_tags` (see §4) |

Intention + expected result are **authority**. RQL / Mongo / CBL are implementations of that intention. Expected results must not depend on Residiuum optimiser choices.

## 4. Distribution floors (§4.3)

Floor measurement = **count of cases listing each `family_tags` entry** (overlap OK).

| Family tag | Floor | Count after Q1.2 |
|---|---:|---:|
| `selection_key_eq_range_compound` | 20 | **20** (OK) |
| `predicate_missing_null_type_nested_array` | 20 | 10 (below; Q1.3 fill) |
| `projection_computed_conditional` | 15 | 5 (below) |
| `order_topk_cursor` | 15 | 10 (below) |
| `enrichment_cardinality` | 15 | 5 (below) |
| `group_aggregate` | 15 | 6 (below) |
| `budget_coverage_damage_refusal` | 10 | 5 (below) |

`floor_policy.enforce_floors` remains **false** until Q1.4 / package exit.

## 5. Amendment process (principal-reviewed)

### 5.1 When an amendment is required

Any change that alters:

- case intention, expected result, oracle rule, ordering/multiplicity;
- Tier A inclusion or deliberate exclusion / refusal code;
- floor policy constants or measurement rule;
- RQL/Mongo/CBL forms in a way that changes comparable answer semantics;

requires a **versioned, principal-reviewed** amendment. Typos in notes alone may be PATCH with explicit log entry.

### 5.2 Procedure

1. Open labor against Feature RQL-Q1 (or a named amendment card).
2. Bump `corpus_version` (`rql-q1-corpus-vMAJOR.MINOR.PATCH`).
3. Append `amendment_log` with date, summary, case id adds/changes/archives, disposition `pending`.
4. Prefer **archive + new `case_id`** over in-place redefinition of a previously frozen id.
5. Run `bash scripts/verify-rql-q1-corpus.sh` (must stay green).
6. Principal sets disposition `accepted` / `accepted_with_amendments` / `rejected` on the log entry.
7. Only after principal package accept may scoreboard `RQL-Q1` move to `accept`.

### 5.3 Semver rules (also in corpus JSON)

- **MAJOR** — remove or redefine frozen case meaning; change floor policy.
- **MINOR** — add cases or non-breaking optional fields after principal accept.
- **PATCH** — notes, generator params that do not change intention/result.

### 5.4 Forbidden

- Post-hoc exclusion of a diverging cell after measurement (equivalence anti-escape).
- Silent edit of a frozen `case_id` without log + version bump.
- Claiming floors met while `enforce_floors` is false, or while cases remain `scaffold`/`draft` only.
- Treating APP-5 / full-v1 surface corpora as this Tier-A corpus.

## 6. Task plan

| Task | State (labor) | Deliverable |
|---|---|---|
| Q1.1 schema + versioning + amendment | `in_review` | Scaffold |
| Q1.2 Commerce + Messaging | **this delivery** → `in_review` | 54 draft cases + generators |
| Q1.3 Directory + Telemetry + Project | `todo` | remaining domains + floor fill |
| Q1.4 floors + comparator review | `todo` | enforce floors; equivalence review |

## 7. Q1.2 delivery notes

### Domains

| Domain | Cases | Collections (generators) |
|---|---:|---|
| Commerce | 33 | orders, products, customers, line_items, inventory |
| Messaging | 21 | conversations, messages, participants |

### Dogfood honesty

No in-tree Residiuum dogfood datasets for commerce/messaging were found (2026-08-07).
All Q1.2 cases use `dogfood.origin = invented_honest_label` with shapes aligned to programme
§4.1 and the existing app-core `orders` dialect. Real dogfood can replace generators under
a versioned amendment (archive + new case ids if meanings change).

### Expected results

- Optimiser-independent `oracle_rule` or `deferred_q2` / `stable_refusal` text on every case.
- RQL forms present as `source` where app-core surface allows; `pending` for Q0 blockers
  (enrich wire, aggregates, computed projection) with Mongo/CBL comparator forms retained.
- Native Residiuum surfaces (budget, coverage, consistency) marked
  `predeclared_native_diff` where Mongo/CBL lack equivalence.

### Generators

Documented under `spec/rql/qualification/corpus-v1/generators/`.
Executable materialiser:

```sh
python3 tools/rql_q1/materialise_fixture.py --generator commerce.orders_v1 --seed 1 --params '{"n_orders":32}'
```

Same seed ⇒ identical JSON (determinism smoke checked in labor).

## 8. Validation evidence

```sh
bash scripts/verify-rql-q1-corpus.sh
```

Expected: exit 0; live cases validate; floors reported with `enforce_floors=false`;
selection floor met (20/20); others below until Q1.3 bulk.

## 9. Residual

- Q1.3 Directory + Telemetry + Project management cases (raise floors toward package exit).
- Target total ~100–150 cases across five domains.
- Dogfood promotion when real datasets exist (≥2 domains programme law).
- Freeze canonical QVM hashes (many RQL `pending` until Q2).
- Turn on `enforce_floors` + comparator review (Q1.4).
- Principal package accept (scoreboard) — not labor `done`.
