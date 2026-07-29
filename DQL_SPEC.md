# Dingo Query Language (DQL)

Status: **Normative design v1.0-draft; shipped implementation is v0.1 subset**

Dialect identifier: `dql`

Audience: language, SDK, planner, server, and conformance implementers
Normative companions: [DINGO_PREDICATE_SPEC.md](DINGO_PREDICATE_SPEC.md),
[SDA_SPEC.md](SDA_SPEC.md), [SDA_PROFILE.md](SDA_PROFILE.md),
[crates/enr-core/ENR1.md](crates/enr-core/ENR1.md), and
[DX_SPEC.md](DX_SPEC.md)

Compatibility importer: [SQL_TO_DQL_SPEC.md](SQL_TO_DQL_SPEC.md)

That companion also defines SQL-ish+ (`sql+` / `sql-plus`) as an optional
directly executable frontend. DQL remains the compiled semantic authority.

## 1. Decision

DQL is DingoDB's official human query language.

It is:

- document-native;
- Heap-bound;
- read-only;
- deterministic;
- explicit about relationship cardinality;
- explicit about order, continuation, coverage, consistency, and budgets;
- compiled into a serializable query plan whose value operations lower to
  ENR and SDA.

DQL is not SQL, a write language, a constraint language, or a second
mathematical kernel.

DRE is the separate language for stored invariants. DQL and DRE share
`dingo-predicate-v1`, but they have different authority:

```text
DQL          asks what data to read and how to shape it
DRE          decides whether a proposed committed state is legal
```

A query cannot create, activate, weaken, or bypass a rule.

## 2. Language stack

```text
DQL source
    |
    v
canonical DqlPlanV1
    |
    +-- acquisition / indexes / frontiers / pages ---> query host
    |
    +-- match bags / cardinality / attach -----------> ENR
    |
    +-- predicates / projection ---------------------> SDA
```

The earlier v0.1 compiler lowers its supported `from` / `enrich` / `project`
subset directly to ENR+SDA text. That remains a valid implementation strategy
for the subset.

The complete language does **not** pretend that source acquisition, index
selection, budgets, coverage, sorting, or continuation are pure SDA. Those
operations belong to the host plan. The semantic expressions inside them
still lower to the same ENR/SDA kernels.

Comfort never redefines truth:

| Layer | Role |
|---|---|
| SDA | value, absence, carrier, and reduction semantics |
| ENR | match bags, cardinality, attach, and nested expansion |
| Dingo Predicate Profile | shared total predicate surface |
| DQL | official query surface and serializable host plan |
| foreign dialects | partial migration surfaces with declared holes |

## 3. V1 scope

DQL v1 supports:

- one root collection;
- zero or more root filters;
- named enrichment from other collections;
- explicit `exactly_one`, `optional`, or `many` cardinality;
- filters on enrichment candidates;
- nested enrichment inside a bounded attached bag/sequence;
- nested projection;
- deterministic scalar ordering;
- limit and authenticated keyset continuation;
- explicit coverage and consistency modes;
- explicit scan/materialization budgets;
- explain.

DQL v1 deliberately excludes:

- writes and DDL;
- rule declarations;
- arbitrary user functions;
- callbacks, recursion, and loops;
- implicit cross-Heap reads;
- unbounded recursive traversal;
- SQL-style flattening joins;
- offset pagination;
- aggregation and grouping;
- window functions;
- arbitrary computed projection expressions;
- text, vector, and geospatial clauses until their dedicated specifications
  are promoted from [FUTURE_ROADMAP.md](FUTURE_ROADMAP.md).

Excluded features are not parser accidents. They are outside the v1 language.
Pure SDA/ENR remains available where its host profile can safely express a
more advanced transformation.

## 4. Lexical rules

Source is UTF-8. Invalid UTF-8 is rejected.

ASCII keywords are case-insensitive. Identifiers and quoted field names are
case-sensitive.

```ebnf
identifier      = ( ALPHA | "_" ), { ALPHA | DIGIT | "_" } ;
unsigned        = DIGIT, { DIGIT } ;
string          = JSON string literal ;
comment         = "--", { any character except line ending } ;
```

Whitespace and comments separate tokens and otherwise have no meaning.

Bare identifiers are restricted to the grammar above. A field path may use
the bracket notation defined by `dingo-predicate-v1` for names that cannot be
written bare.

Reserved words:

```text
after allow and as asc available budget bytes complete consistency
current desc documents enrich exact exactly_one explain false from
in incomplete limit many matching missing not null nulls optional
or order present project result_bytes true using where within
expect first last
```

Reserved words may appear as bracketed field segments but not as bare aliases,
collection names, or output names.

## 5. Grammar

The following EBNF is normative. `predicate`, `path`, and `literal` are imported
from `dingo-predicate-v1`. Within DQL predicates, `operand` is extended with
the parameter production below.

```ebnf
query             = [ "explain" ], from-clause, { pipeline-step },
                    [ project-clause ],
                    [ order-clause ],
                    [ after-clause ],
                    [ limit-clause ],
                    [ consistency-clause ],
                    [ coverage-clause ],
                    [ budget-clause ] ;

from-clause       = "from", source-ref, [ "as", identifier ] ;
source-ref        = identifier | string ;

pipeline-step     = where-clause | enrich-clause | within-clause ;
where-clause      = "where", predicate ;

enrich-clause     = "enrich", identifier,
                    "using", source-ref, [ "as", identifier ],
                    "matching", path, "=", path,
                    [ "where", predicate ],
                    "expect", cardinality ;

cardinality       = "exactly_one" | "optional" | "many" ;

within-clause     = "within", path, [ "as", identifier ], "{",
                      { nested-step },
                    "}" ;
nested-step       = where-clause | enrich-clause | within-clause ;

project-clause    = "project", "{", [ project-item,
                    { [ "," ], project-item }, [ "," ] ], "}" ;
project-item      = path
                  | identifier, ":", path
                  | identifier, "{", [ project-item,
                    { [ "," ], project-item }, [ "," ] ], "}" ;

order-clause      = "order", "by", order-term, { ",", order-term } ;
order-term        = path, [ "asc" | "desc" ],
                    [ "nulls", "first" | "last" ],
                    [ "missing", "first" | "last" ] ;

after-clause      = "after", string ;
limit-clause      = "limit", unsigned ;

consistency-clause = "consistency", ( "available" | "current" ) ;
coverage-clause   = "coverage", ( "complete" | "allow", "incomplete" ) ;

budget-clause     = "budget", "{",
                      budget-entry, { [ "," ], budget-entry }, [ "," ],
                    "}" ;
budget-entry      = "documents", ":", unsigned
                  | "bytes", ":", unsigned
                  | "result_bytes", ":", unsigned ;

parameter         = "$", identifier ;
```

Clauses after `project` are terminal options and occur in the order shown.
Every terminal clause appears at most once. Every budget key appears at most
once.

`from` is required. Empty projection blocks, zero limits, and zero budgets are
legal and have their literal meanings.

## 6. Bindings and scope

### 6.1 Heap

A DQL query executes under exactly one authenticated `HeapId`. Every
collection binding resolves inside that Heap. Quoted source references permit
any valid collection name, for example `from "2026/imports" as imports`.

DQL has no syntax for another Heap. A host that attempts to bind a source from
another Heap must fail before execution with `dql_heap_mismatch`.

### 6.2 Root scope

```text
from orders as order
```

binds:

- source name `orders`;
- current row alias `order`;
- implicit current row for unqualified paths.

Without `as`, the singular alias is not guessed. Only unqualified current-row
paths are available.

In a root predicate:

```text
where status = "paid"
```

`status` means the current root row's `status`.

### 6.3 Enrichment scope

```text
enrich customer using customers as candidate
  matching customer_id = candidate.id
  where candidate.active = true
  expect exactly_one
```

binds:

- `customer` as the output field;
- `customers` as the source collection;
- `candidate` as one candidate row;
- the current left row unchanged.

The left match path is resolved against the current row. The right match path
is resolved against the candidate row. A qualified candidate path MUST begin
with the declared candidate alias. An unqualified right path is also resolved
against the candidate for compatibility with v0.1.

An enrichment `where` predicate is evaluated against each candidate. Paths
qualified by the candidate alias refer to that candidate. Paths qualified by a
visible outer alias refer to the corresponding outer row.

An output name must not already exist on the current row. Replacement requires
an explicit future construct; v1 fails with `dql_output_conflict`.

### 6.4 Nested scope

```text
within items as item {
  enrich product using products as candidate
    matching item.product_id = candidate.id
    expect exactly_one
}
```

`within` requires its path to resolve to a sequence or bag. Each element becomes
the current row for the nested block. The block preserves carrier kind and
multiplicity, enriches each element independently, and replaces the field with
the resulting carrier.

Outer aliases remain readable. The element alias shadows no existing alias.
Nested depth is bounded by the host and recorded in the plan profile.

`within` over an absent path, Null, or a non-carrier value is a runtime
`dql_within_type` error. It is never interpreted as an empty bag.

### 6.5 Parameters

DQL permits a parameter wherever the shared predicate grammar permits an
operand:

```text
where age >= $minimum_age and country = $country
```

Parameters are supplied through a separate typed binding map before planning
or execution. They are values, never source fragments.

- every referenced parameter must be bound exactly once;
- unused bindings are rejected by default and may be permitted only by an
  explicit host compatibility option;
- v1 parameters are Null, Boolean, integer, decimal, string, or bytes;
- parameter names are case-sensitive;
- source text cannot interpolate collection names, aliases, paths, keywords,
  ordering, limits, budgets, or clauses through parameters;
- parameter values are included in execution and continuation identity;
- explain redacts parameter values by default and reports their types.

A missing binding is `dql_parameter_unbound`. A carrier or product parameter is
`dql_parameter_type`.

## 7. Root filtering

A root `where` keeps the current row exactly when the imported
`dingo-predicate-v1` predicate evaluates to `true`.

Pipeline order is semantic:

```text
from orders
where status = "paid"
enrich customer ...
where customer.country = "TH"
```

The first predicate runs before enrichment. The second runs after `customer`
has been attached.

The optimizer may reorder a predicate only after proving:

- all referenced bindings exist at the new position;
- cardinality failures and stable errors remain identical;
- coverage and resource behavior are not weakened;
- result ordering and multiplicity are unchanged.

## 8. Matching and cardinality

For current row `l`, candidate source `R`, candidate filter `F`, left path
`pL`, and right path `pR`:

```text
Match(l, R, pL, pR, F)
  = Bag{ r ∈ R |
      F(r, l)
      ∧ present(l.pL)
      ∧ present(r.pR)
      ∧ value(l.pL) =SDA value(r.pR)
    }
```

Properties:

- the primitive result is always a bag;
- source duplicates are preserved;
- absent keys never match, including absent-to-absent;
- explicit Null is present and therefore matches explicit Null;
- equality is SDA structural equality;
- encounter order has no meaning unless the source has a declared order.

Cardinality interpretation is mandatory:

| Surface | Match count | Result |
|---|---:|---|
| `exactly_one` | 0 | `Fail(t_enr_missing)` |
| | 1 | the value |
| | >1 | `Fail(t_enr_duplicate)` |
| `optional` | 0 | `None` |
| | 1 | `Some(value)` |
| | >1 | `Fail(t_enr_duplicate)` |
| `many` | any | original match bag |

A cardinality failure fails the query page; it does not silently discard the
root row. The error identifies the enrichment path and root key but excludes
document bodies by default.

An `optional` attachment logically stores an SDA optional carrier. A lossless
Dingo result preserves `None` versus `Some(Null)`. A JSON compatibility bridge
omits a field for `None` and emits JSON null for `Some(Null)`.

For paths that traverse a DQL-created optional attachment, DQL uses lifted
resolution:

```text
resolve(Some(v), remaining_path) = resolve(v, remaining_path)
resolve(None, remaining_path)    = Absent
```

This lifting applies only to optional carriers introduced by the typed DQL
plan; it does not reinterpret stored document values. It makes predicates and
projection such as `customer.name` meaningful after `expect optional` while
preserving `None` versus `Some(Null)` in the logical result.

## 9. Projection

Without `project`, the result is the complete current artefact after all
pipeline steps.

Projection creates a new product for every current row.

```text
project {
  id,
  customer {
    name,
    region: address.region
  },
  items {
    sku,
    quantity
  }
}
```

Rules:

- `id` copies the current row's `id` under `id`;
- `region: address.region` copies a path under output name `region`;
- `customer { ... }` projects a product or optional product;
- `items { ... }` maps projection over a sequence or bag while preserving its
  carrier and multiplicity;
- source order of output fields is preserved;
- duplicate output names are static errors;
- a leaf and a block cannot claim the same output path;
- projection never changes cardinality.

If a leaf path is absent, the projected field is absent. If it contains Null,
the output contains Null.

A block over an absent value is absent. A block over Null remains Null. A block
over a product projects that product. A block over an optional maps through the
optional. A block over a sequence or bag maps over its members. Any other value
fails with `dql_project_type`.

Arbitrary calculated fields are outside v1. Use a subsequent SDA reduction
when required.

## 10. Ordering

Without `order by`, root results use ascending immutable document key order.
Filesystem order, worker completion order, hash iteration order, and physical
segment order are never observable.

Every explicit ordering appends immutable document key ascending as an
unwritten final tie-breaker. A query therefore has a strict deterministic
order suitable for continuation.

V1 sort keys must be scalar:

```text
Bool | Integer | Decimal | String | Bytes | Null | Absent
```

Integer and decimal share one exact numeric family. Present non-null values
sort by this family rank:

```text
Bool < Number < String < Bytes
```

Within families:

- `false < true`;
- numbers use exact mathematical order;
- strings use Unicode scalar/code-point lexicographic order;
- bytes use unsigned lexicographic order.

`asc` or `desc` reverses only the present non-null value ordering. Null and
Absent placement is controlled independently:

- default `nulls last`;
- default `missing last`;
- if both occupy the same end, Null precedes Absent;
- explicit `first`/`last` moves that category to the selected end;
- incompatible duplicate placement directives are a static error.

A product, sequence, bag, set, or map encountered as a sort key fails with
`dql_sort_type`.

## 11. Limit and continuation

`limit n` caps returned root rows after filtering, enrichment, projection, and
ordering. It does not weaken cardinality validation for rows actually
considered.

DQL uses keyset continuation:

```text
after "<opaque authenticated token>"
```

Offset pagination is not part of v1.

A continuation token binds at least:

- Heap ID;
- canonical plan hash excluding `after`;
- parameter hash;
- source identities;
- effective ordering;
- last emitted ordering tuple and immutable document key;
- consistency and coverage modes;
- relevant partition/index frontiers;
- token format version and expiry policy;
- an authenticity tag.

A token from another query, Heap, principal, or incompatible frontier fails.
It never restarts silently.

`after` without an explicit or implicit deterministic ordering is impossible,
because document key ordering is always present.

## 12. Consistency and coverage

Defaults:

```text
consistency available
coverage complete
```

### 12.1 Consistency

`available` reads the currently published authoritative/index frontiers and
reports them.

`current` binds a required frontier at query admission and waits until every
participating source and required derived index has observed it, or fails with
a timeout/cancellation error. It does not mean linearizability outside the
query's qualified scope.

### 12.2 Coverage

`coverage complete` fails when missing partitions, offline required tiers,
known holes, budget exhaustion, or damaged required index state could change
membership or order.

`coverage allow incomplete` permits results with structured coverage evidence.
It never converts the evidence into a claim of completeness.

Every result page carries:

```text
query_id
plan_hash
heap_id
source_frontiers
index_frontiers
coverage
known_holes
consistency
ordering
continuation?
```

An empty result under incomplete coverage is not evidence that no matching
document exists.

## 13. Budgets

Example:

```text
budget {
  documents: 100000,
  bytes: 67108864,
  result_bytes: 16777216
}
```

Meanings:

- `documents`: maximum authoritative documents/candidates examined;
- `bytes`: maximum source payload bytes read;
- `result_bytes`: maximum memory/output bytes materialized for sorting and
  result construction.

Budgets are hard upper bounds. Reaching a bound:

- fails with `dql_budget_exhausted` under `coverage complete`;
- returns explicit incomplete coverage under `coverage allow incomplete`.

A server may impose tighter policy ceilings. It must report the effective
budget in explain and result evidence.

No budget authorizes silent truncation.

## 14. Explain

`explain` compiles and plans the query but does not enumerate result documents.

Structured explain includes:

- dialect and semantic profile versions;
- canonical source hash and plan hash;
- source and alias bindings;
- canonical predicate ASTs and SDA lowering;
- ENR match/cardinality/attach lowering;
- inferred Heap scope;
- chosen and rejected indexes;
- scan and cardinality estimates with their evidence age;
- pushdown decisions;
- ordering and continuation plan;
- source and index frontiers;
- consistency and coverage requirements;
- requested, policy, and effective budgets;
- unsupported or fallback operations;
- whether absence and completeness can be proven.

Human explain is a presentation of the same structured artifact.

## 15. Canonical plan

Compilation produces `DqlPlanV1`:

```text
DqlPlanV1 {
  profile: "dql-plan-v1"
  heap_binding
  root: Source
  steps: Seq<Filter | Enrich | Within>
  projection: Optional<Project>
  order: Seq<OrderTerm>        // includes implicit key tie-break
  after: Optional<Token>
  limit: Optional<UInt>
  consistency: Available | Current
  coverage: Complete | AllowIncomplete
  budget: Optional<Budget>
  explain: Bool
  predicate_profile: "dingo-predicate-v1"
  enr_profile
  sda_profile
}
```

Every `Enrich` contains:

```text
Enrich {
  output_path
  source_identity
  candidate_alias?
  left_path
  right_path
  candidate_predicate
  cardinality
}
```

The canonical encoding:

- uses immutable collection identities after binding, not mutable display
  names;
- preserves semantic step order;
- stores normalized predicates and paths;
- inserts all defaults;
- includes the implicit key tie-break;
- excludes comments and insignificant source formatting;
- is deterministically serialized;
- is hashed with a domain-separated cryptographic hash.

The logical plan above is normative. This language specification does not
assign persistent binary field numbers. No implementation may persist,
exchange, or authenticate a purported `dql-plan-v1` until a companion plan
encoding profile fixes its canonical bytes. Compiler and executor development
may use an ephemeral in-process representation in the meantime.

Two sources with the same canonical plan have the same DQL semantics under the
same bindings, parameters, profile versions, and input state.

## 16. Compilation

A conforming compiler:

1. validates UTF-8 and resource ceilings;
2. lexes and parses the complete source;
3. resolves keywords, aliases, paths, and output namespaces;
4. imports and normalizes `dingo-predicate-v1` predicates;
5. checks clause order and uniqueness;
6. checks nested scopes and projection conflicts;
7. binds source names to immutable identities in the authenticated Heap;
8. inserts defaults and the key-order tie-break;
9. lowers filters/projection to SDA forms;
10. lowers match/cardinality/attach/within to ENR forms;
11. emits and validates canonical `DqlPlanV1`;
12. plans physical execution without changing logical meaning.

Compilation is total:

```text
Compile(source, bindings, profiles)
    = Plan
    | finite ordered diagnostics
```

No unsupported syntax is ignored. No cardinality, coverage, budget, or
consistency requirement is weakened.

## 17. Optimizer laws

Physical plans may use scans, Hydra indexes, hash probes, batch probes, remote
partition requests, caches, or future specialized indexes.

An optimization is legal only if it preserves:

- result values;
- multiplicity and carriers;
- cardinality failures;
- deterministic order;
- coverage evidence;
- consistency;
- resource-bound semantics;
- Heap noninterference;
- stable error class where the specification requires one.

Indexes are accelerators, not authorities. Every valid v1 filter has an
authoritative scan interpretation. If a scan is unavailable under the budget,
the query fails or reports incomplete coverage according to §12–§13.

## 18. Stable errors

Initial stable families:

```text
dql_lex_error
dql_parse_error
dql_reserved_identifier
dql_duplicate_clause
dql_clause_order
dql_source_unknown
dql_alias_conflict
dql_path_invalid
dql_output_conflict
dql_projection_conflict
dql_predicate_invalid
dql_parameter_unbound
dql_parameter_type
dql_cardinality_missing
dql_cardinality_duplicate
dql_within_type
dql_project_type
dql_sort_type
dql_budget_invalid
dql_budget_exhausted
dql_coverage_incomplete
dql_current_unavailable
dql_continuation_invalid
dql_continuation_expired
dql_heap_mismatch
dql_profile_unsupported
dql_limit_exceeded
```

Compile errors carry source spans. Runtime errors carry plan hash, safe
operator path, and safe bounded witnesses. Document bodies and secret values
are excluded by default.

## 19. Security

- All bindings are authorized before acquisition.
- The compiler cannot synthesize a cross-Heap binding.
- Query text and parameters are separate; parameters cannot inject syntax.
- Continuation tokens are opaque and authenticated.
- Explain is capability-gated because index names, estimates, and plan details
  may reveal metadata.
- Diagnostics redact values by default.
- Budgets and cancellation apply across local, remote, nested, and fallback
  work.
- User-defined executable code is absent from v1.

## 20. Conformance

The v1 conformance suite must cover:

### 20.1 Parsing and canonicalization

- every production and reserved word;
- comments and whitespace;
- malformed UTF-8 and escapes;
- clause order and duplicates;
- canonical plan equality across formatting variants;
- parser resource ceilings.

### 20.2 Predicate equivalence

- all `dingo-predicate-v1` cases;
- native, SDA-lowered, indexed, and scanned equivalence;
- Null and absence;
- heterogeneous documents.

### 20.3 Enrichment

- missing, one, and duplicate matches for every cardinality;
- absent-to-absent does not match;
- explicit Null-to-Null does match;
- duplicate preservation for `many`;
- candidate filters and outer aliases;
- nested `within` with sequence and bag carriers;
- output conflicts.

### 20.4 Projection and ordering

- absent, Null, optional, product, sequence, and bag projection;
- deterministic field order;
- every scalar sort family;
- mixed families;
- Null and missing placement;
- immutable-key tie-breaking.

### 20.5 Paging, coverage, and budgets

- no duplicates or omissions across unchanged paged state;
- token tampering and query/Heap mismatch;
- partition/index frontier changes;
- complete versus allowed-incomplete behavior;
- budget exhaustion at every boundary;
- cancellation;
- empty results under incomplete coverage.

### 20.6 Equivalence and failure injection

- v0.1 subset equals direct ENR+SDA programs;
- scan and every applicable index produce equivalent logical results;
- crash/retry does not mutate data;
- damaged indexes fall back or report holes;
- unavailable sources never become empty sources;
- cross-Heap binding attempts fail before execution.

## 21. Implementation status and migration

The shipped `dql` compiler currently supports:

```text
from
enrich ... using ... matching ... expect ...
project { ... }
```

It directly emits ENR1/SDA text. It does not yet implement the complete v1
grammar, canonical host plan, filters, nested `within`, ordering, continuation,
coverage, consistency, budgets, or explain.

Therefore:

- current binaries identify their accepted surface as `dql-source-v0.1`;
- the future complete plan identifies itself as `dql-plan-v1`;
- accepting a v1-only construct on a v0.1 runtime fails closed;
- the v0.1 parser must not be described as complete DQL;
- existing v0.1 programs are intended to remain valid v1 programs, except that
  ambiguous alias behavior must be rejected rather than guessed.

Implementation order:

1. freeze canonical AST and shared predicate parser;
2. add root `where`;
3. introduce `DqlPlanV1` host execution;
4. add candidate filters and strict alias resolution;
5. add nested `within`;
6. freeze projection behavior;
7. add deterministic order and limit;
8. add continuation;
9. add coverage, consistency, and budgets;
10. add explain and full conformance.

## 22. Example

```text
from orders as order

where order.status = "paid"
  and present(order.customer_id)

enrich customer using customers as candidate
  matching order.customer_id = candidate.id
  where candidate.active = true
  expect exactly_one

enrich items using order_items as candidate
  matching order.id = candidate.order_id
  expect many

within items as item {
  enrich product using products as candidate
    matching item.product_id = candidate.id
    expect exactly_one
}

project {
  id,
  created_at,
  customer {
    name
  },
  items {
    quantity,
    product {
      name
    }
  }
}

order by created_at desc nulls last missing last
limit 100
consistency available
coverage complete
budget {
  documents: 100000,
  bytes: 67108864,
  result_bytes: 16777216
}
```

Its meaning is:

1. acquire live `orders` within the authenticated Heap;
2. retain paid orders with a present customer ID;
3. attach exactly one active customer or fail;
4. attach the bag of matching items;
5. attach exactly one product to every item or fail;
6. produce the declared nested shape;
7. sort deterministically with immutable key tie-breaking;
8. return at most 100 rows;
9. refuse to present incomplete coverage as complete;
10. obey the declared and server-effective resource bounds.

That is the DQL v1 contract.
