# SQL-ish+ and SQL to DQL cross-compiler specification

Status: **Normative design v1.0-draft**

Product surface: **SQL-ish+**

Canonical dialect identifier: `sql+`

Portable alias: `sql-plus`

Compiler profile: `dingo-sql-plus-to-dql-v1`

Target profile: `dql-plan-v1`

Audience: query-surface, migration-tool, dialect, compiler, SDK, and
conformance implementers

Normative companions: [DQL_SPEC.md](DQL_SPEC.md),
[DINGO_PREDICATE_SPEC.md](DINGO_PREDICATE_SPEC.md), and
[DRE_SPEC.md](DRE_SPEC.md)

Reference syntax: PostgreSQL 18 `SELECT`, used only to identify and refuse
syntax outside this profile:
<https://www.postgresql.org/docs/18/sql-select.html>

## 1. Purpose

SQL-ish+ is a first-class optional query surface for people who already know
SQL. It translates a deliberately bounded, read-only SQL `SELECT` profile into
DQL and executes the resulting DQL plan.

It serves three uses:

- write SQL-ish+ and execute it directly;
- inspect the generated DQL while learning DingoDB;
- translate an existing supported SQL query into durable DQL source.

It does not make DingoDB a SQL database and does not claim to implement
PostgreSQL, ISO SQL, relational bags, SQL transactions, or a relational
catalog.

The compiler has one governing rule:

> Emit equivalent DQL, emit a conditional translation with explicit
> obligations, or refuse. Never guess.

The existing `sql` dialect in `dingo-sdk` is a small SQL-ish-to-SDA filter
frontend. SQL-ish+ is its intended richer successor. This specification defines
the new compiler surfaces:

```text
sql+
sql-plus
```

and a logical API:

```text
compile_sql_to_dql(sql, bindings, evidence, options)
    -> SqlToDqlResult

execute_sql_plus(sql, bindings, evidence, options)
    -> SqlPlusResultStream
```

`sql+` and `sql-plus` are exact aliases. `sql+` is the product spelling;
`sql-plus` exists for shells, URLs, configuration formats, and registries where
`+` is inconvenient.

## 1.1 Authority

SQL-ish+ owns no execution semantics after compilation:

```text
SQL-ish+ source
      ↓
SQL-ish+ compiler
      ↓
canonical DQL source + DqlPlanV1 + mapping receipt
      ↓
ordinary DQL planner and executor
```

The DQL plan—not the SQL-ish+ source parser—is the runtime authority. Explain,
coverage, consistency, budgets, continuation, Heap isolation, and SDA
examination are the ordinary DQL mechanisms.

`SqlPlusResultStream` is the ordinary DQL result stream passed through the
documented SQL compatibility normalization in §11. It does not introduce a
second query executor.

Users may retain SQL-ish+ source, generated DQL, or both. Long-lived prepared
queries store the canonical DQL plan plus the source/compiler provenance
required to reproduce it.

## 2. Why translation is not mechanical syntax replacement

SQL and DQL differ materially:

| SQL | DQL |
|---|---|
| joins multiply and flatten rows | enrichment attaches named values/bags |
| join cardinality is implicit | cardinality is mandatory |
| SQL NULL uses three-valued logic | Null and absence are distinct; predicates are total |
| an inner join discards unmatched rows | `exactly_one` fails and `optional` preserves |
| result order is absent without `ORDER BY` | DQL always has deterministic key order |
| `OFFSET` selects by ordinal position | DQL uses authenticated keyset continuation |
| tables have catalogued columns/types | collections contain flexible documents |

A trustworthy compiler must account for every difference.

## 3. Translation classes

Every result has exactly one class:

```text
Exact
Conditional
Refused
```

### 3.1 Exact

For source SQL query `q`, target DQL plan `d`, source bindings `B`, and
admissible Heap state `S`:

```text
Class = Exact
    ⇒
NormalizeSqlRows(EvaluateSqlProfile(q, B, S))
      =
NormalizeDqlRows(EvaluateDql(d, B, S))
```

The normalizations in §11 account only for the declared SQL document view,
column labels, and JSON bridge. They do not excuse different membership or
multiplicity.

### 3.2 Conditional

A conditional translation includes a finite set of machine-readable
obligations:

```text
Obligations = {
  right_key_unique(...),
  reference_total(...),
  field_scalar(...),
  ...
}
```

Its equivalence statement is:

```text
All(Obligations, S)
    ⇒
NormalizeSqlRows(EvaluateSqlProfile(q, B, S))
      =
NormalizeDqlRows(EvaluateDql(d, B, S))
```

Conditional output is not executable by default. It becomes executable only
when:

- active DRE artifacts prove every obligation at the bound frontier; or
- the caller explicitly accepts an assumption manifest for offline migration
  analysis.

Assumptions are never converted into database guarantees.

The interactive/runtime `sql+` surface executes only `Exact`, including a
formerly conditional translation whose obligations have been discharged by
active evidence. It never executes an unproven assumption manifest.

### 3.3 Refused

If equivalence cannot be stated inside the profile, compilation returns
`Refused` with stable diagnostics and no executable target.

## 4. Input profile

V1 accepts one `SELECT` statement and an optional trailing semicolon.

```ebnf
select          = "SELECT", [ "ALL" ], select-list,
                  "FROM", table-ref,
                  { join-clause },
                  [ "WHERE", sql-predicate ],
                  [ order-clause ],
                  [ limit-clause ],
                  [ ";" ] ;

select-list     = "*"
                | select-item, { ",", select-item } ;
select-item     = column-ref, [ [ "AS" ], identifier ] ;

table-ref       = sql-name, [ [ "AS" ], identifier ] ;
sql-name        = identifier | quoted-identifier ;

join-clause     = [ "INNER" ], "JOIN", table-ref,
                  "ON", equality
                | "LEFT", [ "OUTER" ], "JOIN", table-ref,
                  "ON", equality ;
equality        = column-ref, "=", column-ref ;

sql-predicate   = sql-or ;
sql-or          = sql-and, { "OR", sql-and } ;
sql-and         = sql-not, { "AND", sql-not } ;
sql-not         = [ "NOT" ], sql-primary ;
sql-primary     = "(", sql-predicate, ")"
                | sql-comparison
                | sql-null-test
                | sql-membership ;
sql-comparison  = sql-operand, compare-op, sql-operand ;
compare-op      = "=" | "<>" | "!=" | "<" | "<=" | ">" | ">=" ;
sql-null-test   = column-ref, "IS", [ "NOT" ], "NULL" ;
sql-membership  = column-ref, [ "NOT" ], "IN", "(",
                  sql-literal, { ",", sql-literal }, ")" ;
sql-operand     = column-ref | sql-literal | parameter ;

column-ref      = [ identifier, "." ], sql-name ;
sql-literal     = "NULL" | "TRUE" | "FALSE"
                | sql-integer | sql-decimal | sql-string ;
parameter       = ":", identifier ;

order-clause    = "ORDER", "BY", order-term, { ",", order-term } ;
order-term      = column-ref, [ "ASC" | "DESC" ],
                  [ "NULLS", "FIRST" | "LAST" ] ;
limit-clause    = "LIMIT", unsigned ;
```

Keywords are ASCII case-insensitive. Unquoted identifiers are folded to lower
case. Double-quoted identifiers preserve decoded spelling and escape `"` as
`""`. SQL strings use single quotes and escape `'` as `''`.

Numbers are exact integer or base-10 decimal literals. Exponents, binary
floating point, hexadecimal numbers, collations, casts, and implicit coercions
are outside v1.

## 5. Required compiler inputs

Compilation receives:

```text
SqlBindings {
  heap_id
  tables: Map<SqlName, CollectionIdentity>
  columns: Map<TableAlias, Map<SqlColumn, DingoPath>>
  document_view: "dingo-sql-document-view-v1"
}

TranslationEvidence {
  active_dre_artifacts
  collection_key_profiles
  unique_indexes_with_guarantee_scope
  source_frontiers
}

SqlToDqlOptions {
  mode: prove | emit_conditional
  coverage
  consistency
  budget?
}
```

SQL table and column names are never interpolated into DQL without binding.
All names resolve to immutable Heap-local identities and canonical Dingo
paths.

The compiler has no network resolver and does not consult an ambient SQL
catalog.

## 6. SQL document view

The source semantics are defined over a logical SQL view of Dingo documents:

- one live document is one SQL row;
- `_key` is a non-null key column;
- a present scalar field is the corresponding SQL scalar;
- a missing field becomes SQL NULL;
- stored Dingo Null becomes SQL NULL;
- products, bags, sequences, sets, maps, and bytes are not SQL scalars in v1;
- decimal/integer values remain exact;
- no implicit text/number/Boolean conversion occurs.

Because missing and stored Null collapse in this compatibility view, SQL cannot
round-trip that distinction. The translation receipt records:

```text
null_model: missing_and_null_collapse
```

A caller requiring the distinction must use DQL directly.

## 7. Three-valued predicate translation

SQL predicates evaluate to:

```text
TRUE | FALSE | UNKNOWN
```

`WHERE` retains only TRUE. DQL predicates are total Booleans. Therefore the
compiler does not translate `NOT`, `AND`, or `OR` textually. It computes two
Boolean DQL predicates for each SQL predicate `p`:

```text
T(p)  -- SQL p is TRUE
F(p)  -- SQL p is FALSE
```

UNKNOWN is the state where both are false.

For non-null scalar operands:

```text
Known(x) ≜ present(x) and x is not null

T(x = y)  ≜ Known(x) and Known(y) and x = y
F(x = y)  ≜ Known(x) and Known(y) and x != y

T(x <> y) ≜ F(x = y)
F(x <> y) ≜ T(x = y)
```

Ordering comparisons use the same pattern with the inverse relation.

Null tests use the SQL document view:

```text
T(x IS NULL)     ≜ missing(x) or x is null
F(x IS NULL)     ≜ x is not null

T(x IS NOT NULL) ≜ x is not null
F(x IS NOT NULL) ≜ missing(x) or x is null
```

Composition:

```text
T(NOT p)   ≜ F(p)
F(NOT p)   ≜ T(p)

T(p AND q) ≜ T(p) and T(q)
F(p AND q) ≜ F(p) or F(q)

T(p OR q)  ≜ T(p) or T(q)
F(p OR q)  ≜ F(p) and F(q)
```

`IN` is expanded according to SQL three-valued equality. `NOT IN` is translated
as `NOT(IN(...))`, not as a DQL `not in` shortcut. In particular, a NULL member
can make a non-matching `IN` result UNKNOWN.

The generated DQL `where` uses only `T(source_predicate)`.

## 8. FROM and joins

### 8.1 Root

```sql
FROM orders AS o
```

becomes:

```text
from orders as o
```

The SQL query must contain exactly one root table.

### 8.2 Join evidence

SQL does not state whether a join yields zero, one, or many right rows. DQL
requires that fact. Every join therefore needs:

```text
JoinEvidence {
  output_name
  right_key_unique
  relationship_total
  comparison_profile
}
```

Evidence may come from active DRE references/uniqueness or an assumption
manifest. The compiler never infers uniqueness from observed data, an index
that is not a constraint, naming conventions, or sampled statistics.

### 8.3 Inner many-to-one join

Given a unique right key, an inner join:

```sql
JOIN customers AS c ON o.customer_id = c.id
```

becomes:

```text
enrich c using customers as candidate
  matching o.customer_id = candidate.id
  expect optional

where present(c.id)
```

This preserves SQL's behavior of dropping unmatched left rows while rejecting
duplicate right matches. DQL's lifted optional-path resolution makes `c.id`
absent for `None` and present for `Some(customer)`.

If an active DRE proves a total exactly-one reference, the compiler may instead
emit `expect exactly_one` and omit the presence filter; the receipt records the
proof artifact.

### 8.4 Left many-to-one join

With a unique right key:

```sql
LEFT JOIN customers AS c ON o.customer_id = c.id
```

becomes:

```text
enrich c using customers as candidate
  matching o.customer_id = candidate.id
  expect optional
```

The JSON normalization maps the missing optional attachment to SQL NULLs for
projected right columns.

### 8.5 Refused joins

V1 refuses:

- a right side not proven or declared unique;
- one-to-many or many-to-many joins, because SQL flattens/multiplies while DQL
  attaches a bag;
- RIGHT, FULL, CROSS, NATURAL, LATERAL, and comma joins;
- `USING`;
- non-equality or composite join predicates;
- joins to subqueries, functions, CTEs, or values;
- joins that escape the authenticated Heap.

A future shape-migration tool may deliberately convert one-to-many flat rows
into nested DQL bags, but that is not an equivalence-preserving compiler.

## 9. Projection

Single-root `SELECT *` omits DQL `project`.

`SELECT *` with any join is refused because SQL column collision and flattening
cannot be inferred safely.

Column projections:

```sql
SELECT o.id, c.name AS customer_name
```

become:

```text
project {
  id: o.id,
  customer_name: c.name
}
```

Rules:

- every selected item is a bound scalar column reference;
- output labels use explicit SQL aliases or the source column name;
- duplicate output labels are refused;
- unqualified ambiguous columns are refused;
- expressions, functions, arithmetic, casts, row constructors, and wildcard
  qualifiers are refused;
- optional right-side values normalize to SQL NULL in the compatibility
  result, while native DQL retains absence.

## 10. Ordering and limit

SQL `ORDER BY` scalar column references lower to DQL `order by`.

Because SQL dialects disagree about default Null placement, a nullable order
term must state `NULLS FIRST` or `NULLS LAST`. Otherwise compilation is
conditional on a supplied source-dialect default.

The generated DQL applies the same placement to Dingo Null and missing values
because both are SQL NULL in the compatibility view:

```text
ORDER BY x ASC NULLS LAST
    ->
order by x asc nulls last missing last
```

DQL adds immutable document key as a final tie-breaker. This refines an SQL
order that otherwise permits ties. It does not change the SQL multiset, but it
does make `LIMIT` deterministic. The receipt records the refinement.

`LIMIT n` lowers directly. `OFFSET`, `FETCH ... WITH TIES`, and `LIMIT ALL` are
refused. Applications use the DQL continuation returned by execution.

Without SQL `ORDER BY`, the generated DQL uses its defined key order and the
receipt records:

```text
order_refinement: source_order_unspecified_target_key_order
```

## 11. Result normalization

For equivalence testing:

```text
NormalizeSqlRows
```

- retains SQL row multiplicity;
- labels columns by the SQL select list;
- converts SQL NULL to a distinguished compatibility Null.

```text
NormalizeDqlRows
```

- converts a missing projected field or `None` optional right row to
  compatibility Null;
- converts stored Null/`Some(Null)` to compatibility Null;
- retains exact scalar values and row multiplicity;
- ignores deterministic ordering only when source SQL has no `ORDER BY`;
- otherwise compares ordered rows.

No normalization flattens an attached bag or discards duplicate rows.

## 12. Canonical output

```text
SqlToDqlResult =
  Exact {
    source_profile
    source_hash
    dql_source
    canonical_dql_plan
    binding_hash
    evidence_hashes
    mapping_receipt
  }
| Conditional {
    dql_source
    canonical_dql_plan
    obligations
    mapping_receipt
  }
| Refused {
    diagnostics
  }
```

The mapping receipt lists:

- every source construct and target node;
- identifier and path bindings;
- T/F predicate lowering;
- Null/absence collapse;
- join evidence and cardinality;
- ordering refinements;
- ignored lexical trivia;
- all assumptions;
- compiler/profile versions.

Every directly executed SQL-ish+ query exposes this receipt through `explain`
and SDA examination. A user can always ask:

```text
show dql
show obligations
show mapping
```

These are SDK/console actions over the compilation result, not additional SQL
grammar.

## 13. Stable diagnostics

```text
sql_dql_lex_error
sql_dql_parse_error
sql_dql_statement_unsupported
sql_dql_construct_unsupported
sql_dql_identifier_unbound
sql_dql_column_ambiguous
sql_dql_column_non_scalar
sql_dql_projection_conflict
sql_dql_expression_unsupported
sql_dql_join_shape_unsupported
sql_dql_join_uniqueness_unproven
sql_dql_join_scope_invalid
sql_dql_null_order_unspecified
sql_dql_parameter_unbound
sql_dql_dql_target_unsupported
sql_dql_limit_exceeded
```

Diagnostics include SQL source spans and never emit partial executable DQL
after a refusal.

## 14. Explicit refusals

V1 refuses:

- INSERT, UPDATE, DELETE, MERGE, DDL, and transaction statements;
- WITH/CTEs and subqueries;
- DISTINCT and DISTINCT ON;
- GROUP BY, HAVING, aggregates, windows, and set operators;
- functions, arithmetic, CASE, casts, collations, and custom operators;
- LIKE, SIMILAR TO, regex, BETWEEN, EXISTS, and quantified comparisons;
- arrays, rows, JSON operators, and vendor-specific values;
- implicit type coercion;
- locking clauses;
- tablesample;
- OFFSET and positional ordering;
- any trailing or second statement.

Refusal is a successful safety outcome.

## 15. Security and isolation

- The compiler is pure after bindings/evidence are supplied.
- It performs no network, filesystem, catalog, or service lookup.
- Every source collection and DRE artifact is bound to the same Heap.
- SQL parameters remain separate values and become DQL parameters.
- Source text cannot supply capabilities.
- Mapping receipts exclude secret values.
- Limits apply to source bytes, tokens, AST nodes, joins, projection items,
  predicate nodes, and emitted target bytes.

## 16. Conformance

Conformance requires:

- grammar accept/refuse corpus;
- quoted/unquoted identifier cases;
- exact decimal cases;
- exhaustive SQL three-valued truth tables;
- NULL and missing-field equivalence;
- `IN`/`NOT IN` with NULL members;
- single-table projection/filter/order/limit equivalence;
- inner and left many-to-one joins with missing and duplicate candidates;
- DRE-proven and assumption-only obligations;
- one-to-many refusal;
- ordering and tie refinement;
- malformed/tampered evidence;
- cross-Heap refusal;
- generated DQL parse and canonical-plan validation;
- differential execution against a reference evaluator for the defined SQL
  document view.

The compiler is conforming only for queries classified `Exact` or correctly
`Conditional`/`Refused`. Acceptance count is not a quality metric.

## 17. Implementation sequence

1. Freeze SQL lexer, AST, limits, and source spans.
2. Implement the SQL document-view reference evaluator.
3. Implement T/F three-valued predicate lowering.
4. Translate single-source projection, filter, order, and limit.
5. Bind immutable Heap-local collection/path identities.
6. Add evidence verification.
7. Add proven inner/left many-to-one joins.
8. Emit canonical mapping receipts.
9. Differentially test source evaluation against generated DQL.
10. Expose `sql+` and `sql-plus` as direct executable query dialects.
11. Deprecate the legacy `sql` mimicry path under the migration policy in §18.

No implementation step may silently widen the accepted SQL profile.

## 18. Replacement and compatibility policy

The legacy dialect and SQL-ish+ have different semantics and target types.
The identifier `sql` must not silently change meaning inside one compatibility
major.

Migration phases:

### Phase A — introduce

```text
sql            -> existing SQL-ish-to-SDA implementation
sql-legacy-v1  -> explicit alias of existing implementation
sql+           -> SQL-ish+ to DQL
sql-plus       -> alias of sql+
```

The SDK warns that `sql` is legacy and recommends `sql+`.

### Phase B — default transition

In the next declared SDK/query compatibility major:

```text
sql            -> sql+
sql-plus       -> sql+
sql-legacy-v1  -> retained explicit legacy behavior
```

The compatibility-major boundary, release notes, and explain output make the
change visible. Stored prepared queries retain their original explicit profile
and never reinterpret automatically.

### Phase C — eventual removal

`sql-legacy-v1` may be removed only under the published compatibility policy.
Its removal does not affect DQL plans previously generated by SQL-ish+.

At the end state, users can:

- use DQL directly;
- use SQL-ish+ permanently;
- inspect or export the DQL generated from SQL-ish+;
- move from SQL-ish+ to DQL incrementally without changing execution meaning.
