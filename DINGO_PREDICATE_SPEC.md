# Dingo Predicate Profile

Status: **Normative design v1.0-draft**

Profile identifier: `dingo-predicate-v1`

Audience: DQL, DRE, SDK, compiler, and conformance implementers
Normative companions: [SDA_SPEC.md](SDA_SPEC.md) and
[SDA_PROFILE.md](SDA_PROFILE.md)

## 1. Purpose

DQL and DRE are separate languages:

- DQL retrieves and shapes data.
- DRE decides whether a proposed committed state is legal.

They deliberately share this small predicate profile so that equality,
comparison, presence, Boolean composition, paths, and literals do not acquire
two meanings.

This profile is surface syntax plus a total mathematical interpretation. It is
not a general expression language and is not Turing-complete.

## 2. Values

The predicate domain preserves Dingo/SDA distinctions:

```text
Absent  ≠  Null  ≠  value
```

`Absent` is the result of resolving a path that does not exist. It is not a
stored value. `Null` is a stored value.

V1 literals are:

```ebnf
literal       = "null" | "true" | "false" | integer | decimal | string ;
integer       = [ "-" ], digit, { digit } ;
decimal       = [ "-" ], digit, { digit }, ".", digit, { digit } ;
string        = '"', { json-character | json-escape }, '"' ;
```

Integers are signed arbitrary-precision mathematical integers subject to the
host's declared byte bound. Decimals are signed base-10 values represented as
an unscaled integer and a scale. Binary floating point is not used.

Source spelling is normalized canonically:

- `-0` becomes `0`;
- decimal trailing zeroes are removed;
- decimal negative zero becomes zero;
- strings are decoded from JSON escapes and retained as Unicode scalar
  sequences;
- no ambient locale or Unicode normalization is applied.

## 3. Identifiers and paths

ASCII keywords are case-insensitive. Identifiers are case-sensitive.

```ebnf
identifier    = ( ALPHA | "_" ), { ALPHA | DIGIT | "_" } ;
path          = path-segment, { ".", path-segment | bracket-segment } ;
path-segment  = identifier ;
bracket-segment = "[", string, "]" ;
```

Examples:

```text
status
address.city
profile["postal-code"]
```

A bracket segment denotes one literal map key. V1 paths never traverse every
member of a sequence implicitly. Languages using this profile must introduce
an explicit bounded construct such as DRE `each`.

Canonical paths encode every segment as its decoded string value. Therefore:

```text
profile.name = profile["name"]
```

## 4. Grammar

```ebnf
predicate       = or-expression ;
or-expression   = and-expression, { "or", and-expression } ;
and-expression  = not-expression, { "and", not-expression } ;
not-expression  = [ "not" ], primary ;

primary         = "(", predicate, ")"
                | "true"
                | "false"
                | comparison
                | membership
                | presence
                | null-test
                | string-test ;

comparison      = operand, compare-op, operand ;
compare-op      = "=" | "!=" | "<" | "<=" | ">" | ">=" ;

membership      = operand, [ "not" ], "in", literal-list ;
literal-list    = "[", [ literal, { ",", literal } ], "]" ;

presence        = "present", "(", path, ")"
                | "missing", "(", path, ")" ;

null-test       = path, "is", [ "not" ], "null" ;

string-test     = "starts_with", "(", path, ",", string, ")"
                | "contains", "(", path, ",", literal, ")" ;

operand         = path | literal ;
```

Reserved words cannot be used as bare identifiers. They may be used through a
bracket segment.

Operator precedence, from strongest to weakest, is:

1. parentheses and primitive predicates;
2. `not`;
3. `and`;
4. `or`.

Comparisons do not chain. `a < b < c` is a static error.

## 5. Resolution

For document `d`, `resolve(d, p)` returns exactly one of:

```text
Absent
Present(Null)
Present(v)
```

Resolution rules:

1. start at `d`;
2. consume path segments from left to right;
3. if the current value is not a product/map, return `Absent`;
4. if the named field is missing, return `Absent`;
5. otherwise continue with the stored value;
6. after the final segment, return `Present(value)`.

Explicit `Null` at an intermediate segment cannot contain a child and therefore
resolves a longer path to `Absent`.

## 6. Total predicate semantics

Every well-formed predicate returns exactly one Boolean. Heterogeneous document
data does not make predicate evaluation throw.

### 6.1 Presence and Null

```text
present(p)       ⇔ resolve(p) = Present(v), including v = Null
missing(p)       ⇔ resolve(p) = Absent
p is null        ⇔ resolve(p) = Present(Null)
p is not null    ⇔ resolve(p) = Present(v) ∧ v ≠ Null
```

Consequently, `p is not null` is false when `p` is absent.

### 6.2 Equality

Equality between two present values uses SDA structural equality.

```text
Absent = anything     ⇔ false
anything = Absent     ⇔ false
Present(a) = Present(b) ⇔ a =SDA b
```

`!=` is true only when both operands are present and their values are not
equal:

```text
Absent != anything      ⇔ false
anything != Absent      ⇔ false
Present(a) != Present(b) ⇔ ¬(a =SDA b)
```

This deliberate rule prevents missing fields from accidentally satisfying
negative business predicates. Use `missing(p)` explicitly when absence should
match.

### 6.3 Ordering

`<`, `<=`, `>`, and `>=` are defined only for present operands in the same
ordered family:

- integer with integer;
- decimal with decimal;
- integer with decimal, after exact base-10 promotion;
- string with string, by Unicode scalar/code-point lexicographic order.

All other combinations evaluate to `false`, including Null, bytes, products,
sequences, and mismatched families.

### 6.4 Membership

`x in [a, b, ...]` is the finite disjunction of equality comparisons.

An absent operand is not a member. An empty list never matches.

`x not in L` is true only when `x` is present and no member equals it. It is
false when `x` is absent.

### 6.5 String and containment tests

`starts_with(p, s)` is true only when `p` resolves to a string whose Unicode
scalar sequence starts with `s`.

`contains(p, x)` is:

- substring containment when both resolved `p` and `x` are strings;
- element membership by SDA equality when `p` is a sequence or bag;
- false for absence, Null, or unsupported types.

No case folding, locale behavior, normalization, stemming, regular expression,
or fuzzy behavior is implied.

### 6.6 Boolean operators

All primitive predicates are total, so Boolean operators use ordinary
two-valued logic:

```text
not p
p and q
p or q
```

Evaluation order is not observable. Implementations may short-circuit, but
canonical meaning is independent of source or execution order.

## 7. Parameters and aliases

This profile defines path and literal predicates. A host language may add
statically bound aliases and parameters:

```text
order.status
$minimum_age
```

If it does:

- aliases must resolve statically;
- parameter types and values must be supplied before execution;
- parameters are data, never source fragments;
- absent parameters are a bind error, not `Absent`;
- parameter serialization is included in query identity;
- DRE v1 MUST NOT accept runtime parameters in active rules.

Parameter surface syntax is reserved for DQL. It is not otherwise defined by
this profile.

## 8. Canonical AST

All surface forms normalize to:

```text
Predicate =
    True
  | False
  | Present(Path)
  | Missing(Path)
  | IsNull(Path)
  | IsNotNull(Path)
  | Eq(Operand, Operand)
  | Ne(Operand, Operand)
  | Lt(Operand, Operand)
  | Lte(Operand, Operand)
  | Gt(Operand, Operand)
  | Gte(Operand, Operand)
  | In(Operand, Seq<Literal>)
  | NotIn(Operand, Seq<Literal>)
  | StartsWith(Path, String)
  | Contains(Path, Literal)
  | Not(Predicate)
  | And(Seq<Predicate>)
  | Or(Seq<Predicate>)
```

Canonicalization:

- flattens nested `And` and `Or`;
- preserves source order within them for diagnostics;
- removes redundant parentheses;
- canonicalizes paths and literals;
- does not reorder operands;
- does not apply algebraic rewrites that could change violation attribution.

## 9. Lowering obligation

Let `P` be an accepted canonical predicate and `d` an admissible document.

```text
EvaluatePredicate(P, d) : Bool
```

A compiler lowering `P` to SDA program `s` must satisfy:

```text
∀ P, d • EvaluateSDA(Compile(P), d) = EvaluatePredicate(P, d)
```

The lowering must implement the total rules in §6 explicitly. Raw SDA failures
must not leak out as a different predicate result.

## 10. Stable errors

Compilation uses these stable families:

```text
predicate_lex_error
predicate_parse_error
predicate_reserved_identifier
predicate_path_invalid
predicate_literal_invalid
predicate_comparison_chained
predicate_parameter_unbound
predicate_parameter_forbidden
predicate_limit_exceeded
predicate_semantics_unsupported
```

Every error includes a source span in Unicode scalar offsets and, where
available, line and column.

## 11. Resource limits

The accepting host declares hard limits for:

- source bytes;
- tokens;
- path segments and segment bytes;
- AST nodes;
- nesting depth;
- literal-list members;
- decoded string bytes.

Exceeding a limit is a compile error. No implementation may silently truncate a
predicate.

## 12. Conformance

A conforming implementation supplies golden tests for:

- Null versus absence in all comparison forms;
- heterogeneous ordering;
- exact decimal/integer comparison;
- Unicode code-point ordering;
- escaped and bracketed paths;
- empty and duplicate membership lists;
- Boolean precedence;
- canonical AST stability;
- parser rejection and stable errors;
- native evaluator equivalence with compiled SDA;
- architecture-independent results;
- configured resource ceilings.

The conformance corpus is versioned by `dingo-predicate-v1`. A semantic change
requires a new profile identifier.
