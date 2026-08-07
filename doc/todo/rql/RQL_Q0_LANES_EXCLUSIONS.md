# RQL-Q0 — Comparison lanes and exclusion / refusal freeze

Status: **labor complete · principal freeze pending**

Package: RQL-Q0 deliverables 4–5
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §2.3, §3, §7
Related: [RQL_Q0_ENV_MANIFEST.md](./RQL_Q0_ENV_MANIFEST.md), [RQL_Q0_CAPABILITY_MATRIX.md](./RQL_Q0_CAPABILITY_MATRIX.md)
Board task: Q0.4
Effective: 2026-08-07

---

## 1. Comparison lanes (frozen)

Gate-1 competitive cells must declare exactly one lane. **Never** score embedded
Residiuum against MongoDB TCP as a single undifferentiated contest.

### Lane E — Embedded

| Side | Engine | Transport |
|---|---|---|
| A | Residiuum embedded (`residiuum-sdk` / store in-process) | In-process API |
| B | Couchbase Lite **4.1.0** embedded | In-process library API |

Use for: local document query latency/throughput without network stack.

### Lane S — Local client/server

| Side | Engine | Transport |
|---|---|---|
| A | Residiuum server protocol (`residiuum serve` + client / op path under test) | Localhost framed RPC |
| B | MongoDB Community **8.2.12** | Localhost MongoDB wire |

Use for: protocol-inclusive operational comparison. Both sides use loopback only.

### Lane rules

1. Portfolio reporting labels every cell with `lane_id` ∈ {`embedded`,`local_client_server`}.
2. Geometric means across lanes are forbidden for Gate-1 pass/fail.
3. Warm cache, durability, and index maintenance posture must be matched **within**
   a lane (programme hard law: equivalent work only).
4. Optional future lanes (remote WAN, multi-node) are **out of Gate-1**.

---

## 2. Deliberate exclusions (stable refusal required)

From capability matrix Tier C + programme §2.3. Product paths must not silent-empty.

| Exclusion id | Construct | Refusal owner | Stable code / diagnostic |
|---|---|---|---|
| EX-FTS | Full-text search | parser / plan | `query_invalid` + detail `rql_construct_unsupported:full_text` |
| EX-VEC | Vector search | parser / plan | `rql_construct_unsupported:vector` |
| EX-GEO | Geospatial | parser / plan | `rql_construct_unsupported:geo` |
| EX-GRAPH | Recursive graph traversal | parser / plan | `rql_construct_unsupported:recursive_graph` |
| EX-CHANGE | Change streams / live queries | parser / plan | `rql_construct_unsupported:change_stream` |
| EX-SPILL | External-spill analytics pipelines | runtime policy | `rql_construct_unsupported:external_spill` |
| EX-WRITE-Q | Server-side write/update query pipelines | parser | `rql_construct_unsupported:write_pipeline` |
| EX-ML | Predictive / ML operators | parser | `rql_construct_unsupported:ml` |
| EX-OFFSET | SQL OFFSET / silent prefix discard | parser / sql+ | `sql_rql_construct_unsupported` (existing) / `rql_construct_unsupported:offset` |
| EX-DDA | `at rank` / ranked direct access (until promoted) | parser | Core+full refuse today; code `rql_construct_unsupported:at_rank` |
| EX-ACCESS-POL | sequential/direct/build access policies | parser | `rql_construct_unsupported:access_policy` |
| EX-RAW-SDA-AS-PRODUCT | Treating raw SDA dialect as product RQL path | API boundary | Not a query refuse — wrong API surface; document only |

**Naming policy:** prefer existing `ErrorCode` when the failure class matches
(`query_invalid`, `query_budget_required`, `resource_limit`, `coverage_incomplete`,
`consistency_violation`, `format_unsupported`). Structured detail strings carry
the `rql_construct_unsupported:*` or sql+ diagnostic family.

Existing sql+ diagnostics (do not rename without SPEC amend):

- `sql_rql_construct_unsupported`
- `sql_rql_statement_unsupported`
- `sql_rql_parse_error`

Existing Core full-language wire refusal: Full constructs on Core/op 118 path
must refuse (not partially execute). Owner: `refuse_full_language_on_core_wire`.

---

## 3. Profile-internal refusals (not Tier C, still stable)

| Situation | Code family | Owner |
|---|---|---|
| Malformed query / unknown operator | `query_invalid` | parser / plan |
| Missing required budget when policy demands | `query_budget_required` | resource |
| Hard budget exceeded | `resource_limit` | resource |
| Incomplete coverage; absence unprovable | `coverage_incomplete` | coverage |
| Consistency mode violation | `consistency_violation` | consistency / cursor |
| Tier A not yet implemented (blocker) | `query_invalid` or dedicated `rql_not_implemented:*` detail | compiler until SPEC+impl |
| Aggregates before SPEC amend | refuse with `rql_construct_unsupported:aggregate` (sql+ already refuses) | sql+ / parser |

**Law:** a hole must never become an empty complete page. Coverage refusals are
successful safety outcomes.

---

## 4. Document-native equivalents (not refusals)

These are allowed expression differences when the corpus marks
`document-native-equivalent`:

- Mongo filter/aggregation pipeline shape vs RQL source
- CBL SQL++ / QueryBuilder vs RQL source
- Parameter binding syntax (`$name` vs `?` vs driver bind maps)
- Field path spelling where corpus declares a mapping

They are **not** license to change result multisets, order, or coverage honesty.

---

## 5. Principal accept pack (Q0 bundle)

**Canonical pack:** [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md)

When reviewing Q0, accept or amend **all** of:

1. [RQL_Q0_ENV_MANIFEST.md](./RQL_Q0_ENV_MANIFEST.md) — pins + fingerprint
2. [RQL_Q0_CAPABILITY_MATRIX.md](./RQL_Q0_CAPABILITY_MATRIX.md) — classes + blockers
3. [RQL_Q0_RESULT_EQUIVALENCE.md](./RQL_Q0_RESULT_EQUIVALENCE.md) — equivalence
4. This file — lanes + exclusions + refusal codes
5. [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md) — sign-off + scoreboard propose

**Proposed scoreboard move after principal accept:** see accept pack §4.
**Do not** implement Q1 until principal fills accept pack §5.

**Open principal decision (from matrix):** confirm aggregates + computed/conditional
projection remain Tier A blockers (SPEC amend in Q2) rather than demotion to Tier C.

---

## 6. Exit (Q0.4)

- [x] Lanes E and S frozen with transport honesty
- [x] Tier C exclusions named with refusal owners/codes
- [x] Alignment with existing ErrorCode + sql+ diagnostics
- [x] Principal accept pack linked
- [ ] Principal accept of Q0 freeze
