# SDA SPEC (core + standalone profile)

SDA = **Structured Data Algebra**.

SDA is a small, formal, portable language for transforming structured data.

SDA is **not** a database/query language (SQL/Mongo) and not a scripting runtime.
Its purpose is to give hosts a *closed, testable algebra* for reshaping, validating,
and normalizing structured data without hidden coercions.

### What SDA is for (Normative)

SDA exists to make common “tree in → tree out” transformations precise and portable (JSON/CBOR/MsgPack/Protobuf after parsing).
It is designed to be embedded into other languages and runtimes so that:

- hosts can standardize data semantics over tree carriers (absence vs `Null`, duplicates, ordering)
- tooling can reason about transformations (parsing, normalization boundaries, failure modes)
- implementations can be conformance-tested against stable behavior

### How SDA is used (Normative)

A host uses SDA in one of two ways:

1. **Standalone execution**: the host provides an input tree value and an SDA program; the program
   evaluates to a value (or `Fail(code,msg)`).
2. **Embedded execution**: a larger language treats SDA as an embedded expression form (e.g.
   delimited by host syntax such as `⟪ ... ⟫`), evaluates it against a host value, and receives
   the resulting value.

In both modes, SDA is pure: it cannot perform ambient IO. Any external interaction is done by
host-provided inputs and host-installed functions.
It is designed to be:

- **algebraically explicit** (no hidden coercions)
- **shape- and cardinality-honest** (absence/duplicates are first-class)
- **host-agnostic** (works over any “tree” representation)
- **deterministic** (no ambient IO; host provides data + caps)
- **conformance-testable** (stable errors, stable semantics)

SDA is intended to be embeddable in other languages. This document specifies:

1. the **semantic core** of SDA
2. the **embedding / transformation model** used when SDA is applied to host values
3. the **standalone profile** used by the repository's CLI and JSON bridge

It does **not** specify orchestration, external effects, retries, source lookup,
or workflow policy. Those belong to outer layers such as Axiom, enrichment, and shaping.

---

## Section layering doctrine

This specification distinguishes three levels inside SDA itself, plus an outer boundary:

1. **Core algebra**: the mathematically closed, pure semantics of SDA.
2. **Embedding / transformation model**: how the pure algebra is supplied with inputs,
   composed into stages, and consumed by a host while remaining pure.
3. **Standalone profile / extensions**: surface syntax choices, host-facing helpers,
   and convenience features layered on top of the core and embedding model.

Outside this document's semantic scope:

4. **Orchestration and effects**: HTTP, file access, retries, source lookup, shaping,
   and workflow control. These may consume SDA results but do not redefine SDA meaning.

Rule:

- Extensions may add surface area or host integration points.
- Extensions may **not** change the meaning of the core algebra.
- In particular, extensions may not weaken determinism, carrier semantics, normalization rules,
  eliminator semantics, or stable failure boundaries defined by the core.

Section role guide:

- **Core algebra**:
  - §1 Core data model
  - §2 Variables, scopes, and programs
  - §3 Carriers
  - §4 Absence, optionality, and errors
  - §5 Selectors
  - §6 The Three Eliminators
  - §7 Normalization
  - §8 Set / Bag / Seq operators
  - §9 Comprehensions
  - §10 Pipe operator
- **Embedding / transformation model**:
  - §10 Pipe operator
- **Mixed boundary sections**:
  - §11 Functions
  - §12 Error codes
  - §14 Conformance requirements
- **Standalone profile / extension-facing sections**:
  - §0 lexical and notation conventions
  - §13 Worked examples
  - §15 Non-goals
  - Appendix A grammar sketch
  - Appendix B embedding examples

Unless a section explicitly says otherwise, the semantic content of the core algebra is normative.

---

## 0.0 Normative vs informative

This specification uses two kinds of requirements:

- **Normative**: required for conformance (MUST/SHALL).
- **Informative**: rationale, examples, and non-binding guidance.

Unless a section is explicitly marked **Informative**, it should be treated as **Normative**.

### 0.1 Observational model (Normative)

SDA is a pure, deterministic transformation language.

- The core semantics has three layers:
  - **static validity**: whether a program uses only lawful SDA forms
  - **core evaluation**: the result of running a statically valid, closed SDA program on an input value
  - **profile / host invocation**: how a standalone tool or embedding host reports non-core invocation issues
- A statically valid, closed SDA program evaluates to either a **value** or a `Fail(code, msg)` for SDA-defined failure conditions.
- The observable result of the **core algebra** is therefore a value, including `Ok/Fail` wrappers when present.
- There is no ambient IO. Any host interaction is via host-provided input values and host-installed functions.
- Order is observable only for `Seq`. `Set`, `Bag`, `Map`, and `Prod` are compared extensionally (see §1.2).

Boundary clarification:

- A **core semantic failure** is part of SDA meaning and must preserve the same condition whether detected statically or dynamically.
- A **profile or invocation diagnostic** is not part of the core algebra unless this specification assigns it a stable SDA tag.
- Examples of profile or invocation diagnostics may include unsupported host extensions, unavailable host-installed helpers,
  malformed CLI invocation, or other non-core integration faults.

Core boundary rule:

- A host MAY reject statically invalid SDA before evaluation.
- A host MAY also defer some checks until evaluation if it cannot decide them earlier.
- It MUST NOT change the underlying core condition by reinterpreting an invalid or failing form as a different operation.

## 0.2 Comments and lexing

- Line comments begin with `;;` and continue to the end of the line.
- SDA is whitespace-insensitive except where whitespace is required to separate tokens.
- String literals support at least the following escape sequences: `\"`, `\\`, `\n`, `\t`.
- In the standalone profile, reserved keywords and built-in constructor names are case-insensitive at the lexical level.
  Examples: `null`, `Null`, `NULL`, `some`, and `Some` denote the same reserved forms.
  Hosts MAY choose a stricter surface, but if they do, that stricter rule is a host profile choice rather than core SDA semantics.

## 0.3 Notation

- Unicode operators: `∈ ∪ ∩ \ ∧ ∨ ¬ ≠ ≤ ≥ → ↦ ⟨ ⟩ ⊆ ∣ ⊎ ⊖ •`
- ASCII fallbacks:
  - `∈` -> `in`
  - `∧` -> `and`
  - `∨` -> `or`
  - `¬` -> `not`
  - `≠` -> `!=`
  - `→` -> `->`   ;; exact synonym
  - `↦` -> `=>`   ;; exact synonym
  - `⟨x⟩` -> `<x>`
  - `∣` -> `|`
  - `⊎` -> `bunion`
  - `⊖` -> `bdiff`
  - `•` -> `_`
- The following ASCII operator spellings are reserved keywords and not identifiers:
  - `->`, `=>`
  - `union`, `inter`, `diff`
  - `bunion`, `bdiff`

Conformance note (Normative):
For the following token pairs, the Unicode and ASCII spellings are exact synonyms and MUST parse to the same AST nodes:
- `→` and `->` (map entry / BagKV entry / binding constructor sugar where supported)
- `↦` and `=>` (lambda)
No other Unicode/ASCII equivalence is implied by this section unless explicitly listed.

Unless otherwise stated, examples show Unicode first and ASCII in comments.

---

## 1. Core data model

SDA manipulates **tree values**. A host provides input values (parsed from JSON/CBOR/etc.) and consumes output
values. SDA itself has no ambient IO.

### 1.1 Value kinds

SDA defines the following value kinds:

- **Null**: an explicit null data value. It is not the same as absence (see §4).
- **Bool**: `true | false`
- **Num**: exact numeric values (host chooses representation; must preserve equality)
- **Str**: Unicode string
- **Bytes**: raw bytes
- **Seq**: ordered sequence of values
- **Set**: mathematical set (unique elements, no order)
- **Bag**: multiset (elements may repeat; no order)
- **Map**: key/value mapping (unique keys)
- **Prod**: product/record (named fields; statically known in some embeddings)

Standalone SDA treats **Prod** as a record-like map with a *known shape* when the
host declares it as such.

### 1.2 Equality

`=` compares values for equality.

Requirements:
- For `Null`, equality is by kind: `Null = Null`.
- For `Bool`, `Num`, `Str`, and `Bytes`, equality is by value.
- Equality must be **reflexive**, **symmetric**, **transitive**.
- For `Set`, equality is extensional (same members).
- For `Bag`, equality is extensional with multiplicity.
- For `Seq`, equality is positional.
- For `Map`, equality is extensional (same bindings).
- For `Prod`, equality is extensional over field labels and values.
- For `BagKV`, equality is extensional with multiplicity over bindings.
- For `Bind(k, v)`, equality is pointwise: `Bind(k1, v1) = Bind(k2, v2)` iff `k1 = k2` and `v1 = v2`.
- For `Some(v)`, equality is pointwise over the wrapped value.
- `None = None`.
- For `Ok(v)`, equality is pointwise over the wrapped value.
- For `Fail(code, msg)`, equality is pointwise over `code` and `msg`.

Function equality:

- Function values are not part of the comparable data algebra.
- Standalone SDA MUST NOT treat two function values as equal merely because they are textually identical.
- If a function value reaches a position requiring equality (for example set membership or set construction),
  the host must reject that usage or otherwise make it unreachable in the standalone core.

When a host cannot provide a meaningful equality for a value (e.g. opaque handle),
it must not allow that value in positions requiring equality (e.g. Set membership).

---

## 2. Variables, scopes, and programs

An SDA program is a sequence of statements.

- Variables are immutable by default; rebinding is allowed via `let`.
- Scope is lexical.

### 2.1 Grammar sketch (informal)

- Statement:
  - `let IDENT = Expr ;`
  - `Expr ;` (expression statement)
- Expression:
  - literals, identifiers
  - function calls `f(a, b)`

SDA is intentionally small; hosts may restrict or extend the statement layer.

### 2.2 Lambda expressions

SDA supports minimal lambda expressions for use with combinators.

- Syntax:
  - Unicode: `x ↦ Expr`
  - ASCII:  `x => Expr`
- Only single-parameter lambdas are supported in v1.
- Lambdas are pure expressions and follow lexical scope.
- The bound variable is scoped within the lambda body.
- Lambdas are primarily intended for use with combinators such as `mapRes`, `bindRes`, `mapOpt`, `bindOpt`.

### 2.3 Identifier and call semantics

- A bare identifier denotes the value bound to that name in the current environment.
- If no such binding exists, evaluation yields `Fail(t_sda_unbound_name, "unbound name")`.
- A call expression evaluates its callee and arguments explicitly.
- If the evaluated callee is not callable, evaluation yields `Fail(t_sda_not_callable, "not callable")`.
- In standalone SDA v1, lambdas are single-parameter callables.
- If a callable is invoked with the wrong number of arguments, evaluation yields `Fail(t_sda_arity_mismatch, "arity mismatch")`.
- Hosts may install additional callable names, but they must preserve the same call boundary semantics.

---

## 3. Carriers (Seq / Set / Bag / Map / Prod)

SDA distinguishes carriers by algebra.

### 3.1 Seq

Ordered, may contain duplicates.

- Literal: `Seq[ a, b, c ]`

### 3.2 Set

Unordered, unique elements.

- Literal: `Set{ a, b, c }`
- Membership: `x ∈ S`  (ASCII: `x in S`)

### 3.3 Bag

Unordered, duplicates allowed.

- Literal: `Bag{ a, b, a }`

### 3.3.1 BagKV (bag of key/value pairs)

SDA also supports a keyed bag carrier used to model unnormalized inputs such as
HTTP headers and JSON with duplicate keys.

- Literal: `BagKV{ k -> v, k2 -> v2, k -> v3 }`  ;; duplicates allowed

### 3.3.1.1 Bind (first-class binding value)

SDA defines a first-class binding value:

- `Bind(k, v)` produces a binding from key `k` to value `v`.
- In the standalone profile, `k -> v` is guaranteed only in literal entry positions such as `Map{...}` and `BagKV{...}`.
- A host MAY additionally support `k -> v` as general expression sugar for `Bind(k, v)`, but if it does, that is a documented host extension unless and until the core grammar adopts it explicitly.

Bindings are normal values: they can appear inside `Seq`, `Set`, or `Bag`.
`BagKV{ ... }` is a bag whose elements are `Bind(k,v)` values.

`BagKV` is distinct from `Map`: it permits duplicate keys until explicitly
normalized (see §7).

### 3.4 Map

Unique-key mapping.

- Literal: `Map{ k -> v, k2 -> v2 }`
- In standalone SDA, Map literal keys MUST be `Str` literals.
- Bare identifiers are NOT allowed as Map keys.
- Hosts MAY extend the key domain (e.g. symbols) but MUST document the surface syntax.
- If a host extends keys beyond `Str`, `keys(map)` MUST return a `Set` of the extended key values. 

### 3.5 Prod (records)

Prod is a product with named fields.

Core algebra intent:

- `Prod` is a distinct carrier kind, not merely a `Map` with extra conventions.
- `Prod` exists so total projection can have lawful meaning.
- The field set of a `Prod` is treated as known for the purpose of total projection.

- Literal (standalone form): `Prod{ name: "steve", age: 11 }`

Standalone SDA treats a `Prod{ ... }` literal as introducing a known product shape locally.

Standalone clarification:

- In standalone SDA, `Prod` is a runtime carrier kind with distinct lawful eliminators.
- The phrase "known shape" means that total projection is defined on `Prod` and undefined on `Map`.
- This does **not** imply whole-program schema inference, effectful reflection, or host-specific structural typing in the core language.

Hosts may also construct `Prod` values from external schemas or typed embeddings, but they must
preserve the same core meaning: `Prod` supports total projection; `Map` does not.

A host may choose to represent `Prod` separately from `Map` (recommended), but representational
choice must not erase the semantic distinction.

---

## 4. Absence, optionality, and errors

SDA separates:

- **absence** (a key is not present)
- **null** (a value that is explicitly null)
- **failure** (an assertion violation)

### 4.0.1 Null vs absence (Normative)

`Null` is a **data value** (e.g. JSON `null`). It is not a synonym for absence.

- **Absence** means there is no binding for a key/field in a carrier.
- **Null** means a binding exists and its value is `Null`.
- Optional extraction (`?`) and required extraction (`!`) report absence based on bindings, not on whether the stored value is `Null`.

Example:

- If `m` is `Map{ "x" -> Null }`, then `m<"x">? = Some(Null)` and `m<"x">! = Ok(Null)`.
- If `m` is `Map{ }`, then `m<"x">? = None` and `m<"x">! = Fail(t_sda_missing_key, "missing key")`.

SDA provides two result wrappers:

- `Opt[T]`: `Some(v)` or `None`
- `Res[T]`: `Ok(v)` or `Fail(code, msg)`

Standalone syntax:

- `Some(v)`, `None`
- `Ok(v)`, `Fail(code, msg)`

Stable error codes are strings (or symbols if the host supports them).

---

## 5. Selectors

A **selector** identifies a field/key by label.

- Selector token: `name` (identifier) or `"name"` (string)
- A bare identifier selector (e.g. `email`) is shorthand for the string `"email"`.
- This shorthand applies only to selectors, not to Map literals.
- Multi-selector literal: `{a b c}` (compile-time only, see §5.2)

### 5.1 Selector addressing operator

SDA uses angle brackets to avoid dot access.

- Unicode: `x⟨k⟩` (ASCII: `x<k>`)

Angle brackets do not imply success; success depends on the eliminator suffix.

### 5.2 Static selectors

The multi-selector literal `{a b c}` is **static-only**.

- It is valid only where the grammar requires a static selector set.
- Attempting to store/pass it as a runtime value is an error:
  - `t_sda_selector_not_static`

Duplicates in a selector set are illegal:
- `{a a}` -> `t_sda_duplicate_label_in_selector`

---

## 6. The Three Eliminators (core)

SDA defines exactly three lawful eliminators. They differ only in tolerance.

### 6.1 Total eliminator (product projection)

Form:

- `x⟨k⟩` (no suffix)

Semantics:

- **Allowed only on Prod** (or host-declared schema-known product-like values).
- The name "total eliminator" means total on its lawful domain of product projection; it does not
  mean lawful on every SDA value.
- Core meaning:
  - if `x` is a `Prod` and `k` is a field of `x`, return its value
  - if `x` is a `Prod` and `k` is not a field of `x`, fail with `t_sda_unknown_field`
  - if `x` is not a `Prod`, the total eliminator is not lawful on that value shape
- Standalone and host realization:
  - when shape is statically known, a host SHOULD reject an impossible total projection before evaluation
  - when the host cannot reject earlier, it MUST preserve the same language boundary and not reinterpret
    total projection as Map or BagKV access
  - if evaluated against a non-`Prod`, the host MUST reject it or surface `t_sda_wrong_shape`

Type behavior:

- `Prod{name:T,...}⟨name⟩ : T`

Note: The total eliminator `x⟨k⟩` is legal only on `Prod` (schema-known). Optional (`?`) and required (`!`) eliminators are the only ones legal on `Map` or `BagKV`.
No extension may redefine bare `x⟨k⟩` as total projection on `Map`.

### 6.2 Optional eliminator (partial extraction)

Form:

- `x⟨k⟩?`

Semantics:

- On `Map`:
  - missing -> `None`
  - present -> `Some(v)`
- On `Bag` (key/value bags; see §6.4):
  - 0 matches -> `None`
  - 1 match -> `Some(v)`
  - >1 matches -> `None` (non-strict) or `Fail(t_sda_duplicate_key, ...)` (strict variant)

This spec defines the **non-strict** default for `?` on bags.

### 6.3 Required eliminator (assertive extraction)

Form:

- `x⟨k⟩!`

Semantics:

- On `Map`:
  - missing -> `Fail(t_sda_missing_key, "missing key")`
  - present -> `Ok(v)`
- On `Bag` (key/value bags):
  - 0 matches -> `Fail(t_sda_missing_key, "missing key")`
  - 1 match -> `Ok(v)`
  - >1 matches -> `Fail(t_sda_duplicate_key, "duplicate key")`

### 6.4 Key/value Bag vs element Bag

SDA supports two bag styles:

- `Bag[V]`: a bag of values
- `BagKV`: a bag of bindings `k -> v` (syntactic sugar for a bag of pairs `(k, v)`)

Eliminators `⟨k⟩?` and `⟨k⟩!` apply only to **BagKV**.

`BagKV` is a *carrier*; `Bind(k,v)` is a *value*.
If you have a `Bag` of `Bind` values, convert it explicitly via `asBagKV(...)` (see §11.1).

If applied to `Bag[V]`, fail with:
- `t_sda_wrong_shape`

---

### 6.5 Totality and failure table (Normative)

The following table is normative. It defines which operations are total and which may fail.

Operation                Input kind(s)                     Output        Total  Failure codes
----------------------  --------------------------------  ------------  -----  -----------------------------
x⟨k⟩                    Prod (schema-known)               value         No     t_sda_unknown_field,
                                                                           t_sda_wrong_shape
x⟨k⟩?                   Map, BagKV                        Opt[value]    Yes    t_sda_wrong_shape
x⟨k⟩!                   Map, BagKV                        Res[value]    No     t_sda_wrong_shape,
                                                                           t_sda_missing_key,
                                                                           t_sda_duplicate_key
normalizeUnique(bagkv)  BagKV                             Res[Map]      No     t_sda_wrong_shape,
                                                                           t_sda_duplicate_key
normalizeFirst(bagkv)   BagKV                             Map           Yes    t_sda_wrong_shape
normalizeLast(bagkv)    BagKV                             Map           Yes    t_sda_wrong_shape
asBagKV(bag)            Bag                              Res[BagKV]     No     t_sda_wrong_shape

Notes:
- “Total” means total on the operation's lawful input domain as defined by the core algebra.
- The name "total eliminator" refers to unwrapped projection on lawful `Prod` inputs, not to universal
  totality across all SDA values.
- An implementation MAY reject statically invalid uses earlier, but it must preserve the same core
  semantic condition if evaluation is attempted.
- In particular, impossible total projection on a known `Prod` shape and runtime total projection
  of a missing `Prod` field name describe the same core condition: `t_sda_unknown_field`.
- Applying total projection to a non-`Prod` value is a `t_sda_wrong_shape` condition.

## 7. Normalization (algebra changes)

Normalization changes a BagKV into a Map with an explicit policy.

### 7.1 normalizeUnique

`normalizeUnique(bagkv) : Res[Map]`

- If any key appears more than once -> `Fail(t_sda_duplicate_key, ...)`
- Otherwise -> `Ok(map)`

### 7.2 normalizeFirst / normalizeLast

`normalizeFirst(bagkv) : Map` (first wins)

`normalizeLast(bagkv) : Map` (last wins)

These are policy functions; they never fail.

### 7.3 Rationale

Normalization is not an optimization. It is a change of algebra:

- BagKV permits multiplicity.
- Map requires uniqueness.

SDA forbids silent conversion.

---

## 8. Set / Bag / Seq operators (closed algebra)

### 8.1 Set operators

For sets `A`, `B`:

- union: `A ∪ B`
- intersection: `A ∩ B`
- difference: `A \ B`

ASCII:
- `A union B`
- `A inter B`
- `A diff B`

### 8.2 Bag operators

Bags support:

- bag union (add multiplicities): `A ⊎ B`  (ASCII: `A bunion B`)
- bag difference (subtract multiplicities, floor at 0): `A ⊖ B` (ASCII: `A bdiff B`)

If the host does not support `⊎` / `⊖` glyphs, it must support the ASCII names.

### 8.3 Seq operators

Sequences support:

- concatenation: `A ++ B`

### 8.4 Membership

- `x ∈ Set{...}` is defined.
- `x ∈ Seq[...]` is defined as positional-carrier membership with a boolean result; order is ignored for the membership test itself.
- `x ∈ Bag{...}` is defined (ignores multiplicity; presence test).
- `x ∈ Map{...}` is defined as key membership and therefore requires `x` to denote a key value in the carrier's key domain.
- `x ∈ Prod{...}` is defined as field-label membership and therefore requires `x` to denote a field label.
- For multiplicity, hosts may provide `count(x, bag)`.

Standalone note:

- In standalone SDA, `Map` keys are strings and `Prod` field labels are strings, so map or prod membership is a string-label membership test.
- In standalone SDA, if the left-hand side of map or prod membership is not a string label, the result is `Fail(t_sda_wrong_shape, "wrong shape")` rather than `false`.
- Hosts that extend the `Map` key domain must preserve the same carrier-relative meaning.

### 8.4.1 Division by zero

Numeric division is defined only for numeric operands with a non-zero divisor.

- If either operand is non-numeric, the result is `Fail(t_sda_wrong_shape, "wrong shape")`.
- If the divisor is numeric zero, the result is `Fail(t_sda_div_by_zero, "division by zero")`.
- Otherwise numeric division returns the exact rational quotient.

### 8.5 Algebraic laws (Informative)

These laws are provided to guide implementations and conformance tests.

Set laws (for `∪` and `∩`):
- Commutative: `A ∪ B = B ∪ A`, `A ∩ B = B ∩ A`
- Associative: `(A ∪ B) ∪ C = A ∪ (B ∪ C)` and similarly for `∩`
- Idempotent: `A ∪ A = A`, `A ∩ A = A`

Bag laws (for `⊎`):
- Commutative: `A ⊎ B = B ⊎ A`
- Associative: `(A ⊎ B) ⊎ C = A ⊎ (B ⊎ C)`

Seq laws (for `++`):
- Associative: `(A ++ B) ++ C = A ++ (B ++ C)`
- Not commutative in general.

---

## 9. Comprehensions (the query core)

SDA provides a comprehension form to express filtering and projection.

### 9.1 Set comprehension

Unicode:

`{ a ∈ A ∣ P(a) }`

ASCII:

`{ a in A | P(a) }`

Semantics:

- v1 comprehensions support exactly one generator (`a ∈ A`).
- The bound variable is scoped only within the predicate and yield expression.
- Nested comprehensions are allowed.
- If `A` is a `Set`, result is a `Set`.
- If `A` is a `Seq`, result is a `Seq` (preserves order).
- If `A` is a `Bag`, result is a `Bag` (preserves multiplicity).

This is the "same comprehension, carrier-preserving" rule.

### 9.2 Projection in comprehensions

Allow a `yield` clause:

Unicode:

`{ yield E(a) ∣ a ∈ A ∣ P(a) }`

ASCII:

`{ yield E(a) | a in A | P(a) }`

If `yield` is absent, yield the bound variable.

Grammar note (Normative):

- In the standalone grammar, the generator and the optional predicate are separated explicitly.
- The generator is `IDENT ∈ Expr` or `IDENT in Expr`.
- If a predicate is present, it follows a second separator `∣` or `|`.
- The standalone core does **not** treat `a ∈ A ∧ P(a)` as shorthand for a generator plus predicate.

### 9.3 Predicate language

Predicates use:

- booleans, comparisons (`=`, `≠`, `<`, `<=`, `>`, `>=`)
- logical operators `∧`, `∨`, `¬`
- membership `∈`

Example:

`{ a ∈ A ∣ a⟨name⟩ = "steve" ∧ a⟨city⟩ ∈ Set{"la","ny"} }`

### 9.4 BagKV comprehensions

When the source carrier `A` is a `BagKV`, the bound variable ranges over
bindings of the form `k -> v`.

Binding form:

- `b ∈ A` where `A : BagKV` binds `b` as a pair `(k, v)`.

Accessors:

- `b⟨key⟩` yields `k`
- `b⟨val⟩` yields `v`

Example:

```
{ b ∈ headers ∣ b⟨key⟩ = "content-type" }
```

Projection example:

```
{ yield b⟨val⟩ ∣ b ∈ headers ∣ b⟨key⟩ = "content-type" }
```

Carrier preservation:

- If `A` is `BagKV`, the result of the comprehension is a `Bag` of yielded values.

Important:

- If the comprehension yields bindings, the result is still a `Bag` (of `Bind` values), not a `BagKV`.
- In the standalone surface, such a yield should be written using the explicit constructor:
  - `yield Bind(k, v)`
- Hosts MAY offer `yield k -> v` as profile sugar, but that is not required for standalone conformance.
- To treat that result as a keyed bag again, the program must convert explicitly:
  - `asBagKV(bagOfBindings) -> Res[BagKV]`

Rationale:

A comprehension is a selection/projection. Producing a keyed carrier (`BagKV`) would otherwise require implicit synthesis and validation of keys.
SDA forbids that implicit algebra change.

---

## 10. Pipe operator (composition)

SDA provides a pipe operator for left-to-right composition.

### 10.1 Syntax

- Unicode / ASCII: `E |> R`

### 10.2 Semantics

`E |> R` evaluates `E`, then evaluates `R` with the result of `E` bound to a
special placeholder.

- Unicode placeholder: `•` (U+2022 BULLET)
- ASCII placeholder: `_` (reserved token; not an identifier)

These placeholders are equivalent; hosts may accept either surface form.

Formally:

- Evaluate `E` to value `v`.
- Evaluate `R` in an environment where `• = v` (Unicode surface) or `_ = v` (ASCII surface).
- The value of the expression is the result of evaluating `R`.

No implicit application rule:

- `|>` does **not** mean implicit function argument insertion.
- In particular, `x |> f()` is **not** shorthand for `f(x)` in the standalone profile.
- Any use of the piped value on the right-hand side must be expressed through the placeholder or through host-defined sugar that preserves the same meaning.

The placeholder is read-only, cannot be declared or assigned, and is scoped only
to the right-hand side of the nearest enclosing `|>`.

### 10.3 Associativity and precedence

- `|>` is **left-associative**.
- `a |> b |> c` parses as `(a |> b) |> c`.
- `|>` has lower precedence than arithmetic, comparison, and logical operators,
  but higher precedence than statement termination (`;`).

### 10.3.1 Desugaring law (Informative)

`E |> R` is equivalent to evaluating `E` once and substituting its value into the placeholder in `R`:

- Desugaring: `E |> R  ≡  (let tmp = E; R[tmp/•])`

This is a law of meaning, not a required surface syntax feature.

### 10.4 Intended use

The pipe operator is the primary mechanism for multi-step data transformation in
raw SDA.

Examples:

```
input
|> normalizeUnique(_)
|> •<"content-type">!
```

```
A
|> { a ∈ • ∣ a<city> = "la" }
```

### 10.5 No implicit effect handling

The pipe operator is purely syntactic composition.

- It does **not** implicitly unwrap `Opt` or `Res` (including values referenced via the placeholder).
- Failure propagation must be explicit via library functions or eliminators.

This restriction is intentional and preserves algebraic clarity.

### 10.6 Placeholder binding (• / _)

The pipe placeholder has two equivalent spellings:

- Unicode: `•` (U+2022 BULLET)
- ASCII: `_`

Rules:

- The placeholder refers to the value produced by the **left-hand side** of the nearest enclosing `|>`.
- The placeholder is **read-only** and **not a normal identifier**:
  - it cannot be declared (`let _ = ...` is illegal)
  - it cannot be assigned
  - it cannot be shadowed by user variables
- The placeholder is in scope **only** on the right-hand side of its `|>`.
- Using `•` / `_` when no enclosing `|>` provides a binding MUST fail with:
  - `Fail(t_sda_unbound_placeholder, "unbound placeholder")`  ;; stable code + stable msg
- `•` / `_` are reserved tokens.
- They may not be used as identifiers, lambda parameters, or Map keys.

Examples:

```
A |> { a ∈ • ∣ a<city> = "la" }
```

```
_  ;; error: unbound placeholder
```

---

## 11. Functions (mixed: core combinators + standalone helpers)

SDA keeps the irreducible algebra small. This section therefore distinguishes:

1. **Core combinators**: pure functions that are part of the mathematical SDA surface.
2. **Standalone profile helpers**: pure inspection or convenience helpers provided by the
  standalone SDA profile.

Hosts may add more helper functions, but they may not change the meaning of the core combinators.

### 11.1 Core combinators

- `normalizeUnique(bagkv) -> Res[Map]`
- `normalizeFirst(bagkv) -> Map`
- `normalizeLast(bagkv) -> Map`
- `Bind(k, v) -> Bind`                ;; constructs a binding value
- `asBagKV(bag) -> Res[BagKV]`        ;; explicit Bag-of-Bind -> BagKV conversion
- `mapOpt(opt, f) -> Opt`             ;; Option functor map
- `bindOpt(opt, f) -> Opt`            ;; Option monadic bind (Some -> f(v), None -> None)
- `orElseOpt(opt, other) -> Opt`      ;; Option fallback
- `mapRes(res, f) -> Res`             ;; Result functor map (Ok -> Ok(f(v)))
- `bindRes(res, f) -> Res`            ;; Result monadic bind / andThen (Ok -> f(v), Fail -> Fail)
- `orElseRes(res, other) -> Res`      ;; Result fallback

### 11.2 Standalone profile helpers

The standalone profile also provides a small set of pure inspection helpers:

- `typeOf(x) -> Str`
- `keys(map) -> Set`
- `values(map) -> Seq`
- `count(x, bag) -> Num`

These helpers are part of standalone SDA, but they are not the irreducible algebraic core.
Other hosts may omit them, rename them, or provide additional helpers, provided the core algebra
remains unchanged.

Standalone helper contracts:

- `typeOf(x) -> Str`
  - returns the standalone SDA type tag for `x`
  - required tags are the names of the SDA value kinds: `null`, `bool`, `num`, `str`, `bytes`,
    `seq`, `set`, `bag`, `map`, `prod`, `bagkv`, `bind`, `some`, `none`, `ok`, `fail`, `fn`
- `keys(map) -> Set`
  - defined for `Map`
  - returns the set of keys present in the map
  - in standalone SDA with string-only map keys, the result is `Set[Str]`
- `values(map) -> Seq`
  - defined for `Map`
  - returns a sequence of values ordered by ascending key order in the standalone profile
  - this canonical order is part of standalone determinism and must not depend on parse order,
    host object insertion order, or runtime hash iteration order
- `count(x, bag) -> Num`
  - defined for `Bag`
  - returns the multiplicity of `x` in the bag

Profile discipline:

- These helper contracts define the standalone SDA profile only.
- A host MAY provide additional helper overloads for other carriers, but such overloads are host
  extensions, not part of standalone SDA.
- In particular, hosts must not silently broaden standalone `keys(map)`, `values(map)`, or
  `count(x, bag)` and still present that broader surface as the standalone core contract.

### 11.3 Semantics of core combinators

These combinators are pure (no IO) and deterministic.

- `mapOpt(None, f) = None`
- `mapOpt(Some(v), f) = Some(f(v))`

- `bindOpt(None, f) = None`
- `bindOpt(Some(v), f) = f(v)`

- `orElseOpt(None, other) = other`
- `orElseOpt(Some(v), other) = Some(v)`

- `mapRes(Fail(code,msg), f) = Fail(code,msg)`
- `mapRes(Ok(v), f) = Ok(f(v))`

- `bindRes(Fail(code,msg), f) = Fail(code,msg)`
- `bindRes(Ok(v), f) = f(v)`

- `orElseRes(Fail(_, _), other) = other`
- `orElseRes(Ok(v), other) = Ok(v)`

`asBagKV(bag)` semantics:

- If `bag` is not a `Bag`, return `Fail(t_sda_wrong_shape, "wrong shape")`.
- If any element is not a `Bind(k,v)`, return `Fail(t_sda_wrong_shape, "wrong shape")`.
- In standalone SDA, if any binding key `k` is not a `Str`, return `Fail(t_sda_wrong_shape, "wrong shape")`.
- Otherwise return `Ok(BagKV{ ... })` containing the same bindings in unspecified order (bag semantics).

Standalone helper notes:

- `typeOf` is observational only; it must not be used to justify hidden coercions.
- `keys` and `values` are profile helpers over `Map` in standalone SDA.
- `count(x, bag)` is a pure cardinality helper and does not change carrier semantics.

Hosts may add more helpers or profile conveniences, but not by redefining the above core meanings.

---

## 12. Error codes (stable)

These codes are stable and MUST be used by conforming implementations.

- `t_sda_wrong_shape`
- `t_sda_div_by_zero`
- `t_sda_unbound_name`
- `t_sda_not_callable`
- `t_sda_arity_mismatch`
- `t_sda_missing_key`
- `t_sda_duplicate_key`
- `t_sda_selector_not_static`
- `t_sda_duplicate_label_in_selector`
- `t_sda_reserved_placeholder`
- `t_sda_invalid_map_key`
- `t_sda_invalid_bagkv_key`
- `t_sda_unknown_field` (compile-time where applicable)
- `t_sda_unbound_placeholder`

`asBagKV(...)` uses `t_sda_wrong_shape` for any non-`Bag` input or any element that is not a `Bind(k,v)`.

`Fail(code, msg)` must include:
- `code`: one of the stable tags above
- `msg`: a stable short message (English) suitable for tests

Boundary note:

- These stable codes constrain the language boundary where SDA itself defines failure.
- Static validity and dynamic failure are two views of the same core semantics; hosts may choose
  earlier rejection, but may not assign different meanings to the underlying condition.
- This specification assigns stable tags only to SDA-defined semantic conditions.
- Other standalone or host diagnostics may exist, but they are not part of the core SDA error algebra unless this specification assigns them a stable tag.
- Parser wording, host exception types, source spans, IDE diagnostics, and other rendering details are
  part of the standalone profile or host tooling, not the mathematical core.
- Where this specification names a stable tag for a parse-time or compile-time condition, conformance
  applies to the tag and semantic condition, not to any particular host-specific diagnostic format.

Standalone profile note:

- Reserved placeholder misuse in declaration-like positions uses `t_sda_reserved_placeholder`.
- Invalid standalone `Map` literal keys use `t_sda_invalid_map_key`.
- Invalid standalone `BagKV` literal keys use `t_sda_invalid_bagkv_key`.

---

## 13. Worked examples

### 13.1 JSON-ish filter (carrier-preserving)

Input `A` is a sequence of records (Prod values).

```
{ a ∈ A ∣ a⟨name⟩ = "steve" ∧ a⟨city⟩ ∈ Set{"la","ny"} }
```

### 13.2 Map access (optional vs required)

```
let emailOpt = m<email>?;   ;; Unicode: m⟨email⟩?
let emailReq = m<email>!;   ;; returns Res
```

### 13.3 BagKV normalization

```
let mRes = normalizeUnique(headers);
let m = bindRes(mRes, m => m);   ;; explicit unwrapping
let ctRes = m<"content-type">!;
let ct = bindRes(ctRes, ct => ct);
```

### 13.4 BagKV duplicate behavior

For `b = BagKV{ "k" -> 1, "k" -> 2 }`:

- `b<k>?` -> `None`
- `b<k>!` -> `Fail(t_sda_duplicate_key, "duplicate key")`

### 13.5 Building BagKV via comprehension + explicit conversion

```
let pairs = { yield Bind("x", 1) ∣ a ∈ Seq[1,2,3] };
let headersRes = asBagKV(pairs);
let headers = bindRes(headersRes, h => h);   ;; explicit Bag -> BagKV with explicit unwrapping
```

### 13.6 Result/Option combinators in a pipeline

```
input
|> normalizeUnique(_)               ;; Res[Map]
|> bindRes(_, m => m<"content-type">!)
```

```
let city = mapOpt(user<city>?, c => c);
```

---

## 14. Conformance requirements

A conforming SDA implementation MUST:

1. Implement the three eliminators with the exact semantics in §6.
2. Implement normalization semantics in §7.
3. Preserve carrier in comprehensions (§9.1).
4. Emit stable error codes (§12).
5. Provide both Unicode and ASCII spellings for operators listed in §0.
6. Implement pipe semantics (§10) with correct scoping of `•` / `_` and left-associativity.
7. Implement BagKV comprehension binding and projection semantics (§9.4).
8. Preserve the semantic distinction between `Prod` total projection and `Map` or `BagKV` access.
9. If claiming standalone profile conformance, implement the helper contracts in §11.2 exactly.
10. Implement equality for every comparable core value kind exactly as defined in §1.2.
11. Preserve the same semantic condition whether a core error is detected statically or during evaluation.
12. Treat `|>` as placeholder-based composition unless documenting a host extension that preserves the same core meaning.
13. Treat `Bind(k, v)` as the standalone binding constructor, and not require general `k -> v` expression sugar for standalone conformance.

### 14.1 Minimal conformance suite outline (Informative)

A minimal test suite SHOULD include canonical cases for:

- Placeholder scoping:
  - `_` / `•` outside of any `|>` must raise `t_sda_unbound_placeholder`.
  - Nested pipes bind `_` / `•` to the nearest enclosing `|>`.
- BagKV duplicate behavior:
  - `BagKV{ "k" -> 1, "k" -> 2 }<"k">? = None`
  - `BagKV{ "k" -> 1, "k" -> 2 }<"k">! = Fail(t_sda_duplicate_key, "duplicate key")`
- Normalization:
  - `normalizeUnique` fails on duplicates and succeeds otherwise.
  - `normalizeFirst` / `normalizeLast` are total and deterministic.
- Equality:
  - `Prod{a: 1, b: 2} = Prod{b: 2, a: 1}`
  - `BagKV{"k" -> 1, "k" -> 2} = BagKV{"k" -> 2, "k" -> 1}` extensionally with multiplicity
  - `Some(Null) != None`
  - `Ok(1) != Fail("x", "y")`
- Standalone helper profile:
  - `keys(Map{"b" -> 2, "a" -> 1}) = Set{"a", "b"}` extensionally
  - `values(Map{"b" -> 2, "a" -> 1}) = Seq[1, 2]` in ascending key order
  - `count(1, Bag{1, 2, 1}) = 2`
- Carrier preservation in comprehensions:
  - Source `Seq` yields `Seq`, source `Set` yields `Set`, source `Bag` yields `Bag`.
- Null vs absence:
  - `Map{ "x" -> Null }<"x">? = Some(Null)` and `Map{ }<"x">? = None`.

---


## 15. Non-goals (v1)

- Full static typing (hosts may add it)
- IO, networking, database bindings
- Implicit schema inference
- Dot-access syntax
- Pipeline stages as a core SDA construct (embedding delimiters are host-defined).

---

# Appendix A — Minimal grammar (Informative)

This appendix is a compact, reader-friendly grammar sketch so the surface forms can be parsed
without hunting through the spec. It is informative; the normative semantics are in the main
sections.

Notes:
- Terminals in quotes are ASCII spellings; Unicode spellings are listed alongside where relevant.
- Whitespace is generally insignificant except to separate tokens.

## A.1 Lexical tokens (sketch)

- `IDENT`  ::= `[A-Za-z_][A-Za-z0-9_]*` except reserved keywords/tokens
- `STRING` ::= `" ... "` with escapes `\"`, `\\`, `\n`, `\t`
- `NUM`    ::= host-defined numeric literal surface (implementations SHOULD support decimal)

Reserved tokens (not identifiers):
- Operators/keywords: `in and or not != -> => union inter diff bunion bdiff` and `yield`
- Pipe placeholder: `_` and (Unicode) `•`

## A.2 Program and statements

```
Program   ::= { Stmt }
Stmt      ::= LetStmt | ExprStmt
LetStmt   ::= "let" IDENT "=" Expr ";"
ExprStmt  ::= Expr ";"
```

## A.3 Expressions (core)

```
Expr      ::= Pipe
Pipe      ::= Or { "|>" Or }
Or        ::= And { ("or" | "∨") And }
And       ::= Not { ("and" | "∧") Not }
Not       ::= { ("not" | "¬") } Cmp
Cmp       ::= Add { ("=" | "!=" | "≠" | "<" | "<=" | "≤" | ">" | ">=" | "≥") Add }
Add       ::= Mul { ("+" | "-") Mul }
Mul       ::= Unary { ("*" | "/") Unary }
Unary     ::= { ("-" ) } Postfix

Postfix   ::= Primary { SelectorAccess | CallSuffix }
SelectorAccess ::= "<" Selector ">" [ "?" | "!" ]
              |  "⟨" Selector "⟩" [ "?" | "!" ]
CallSuffix ::= "(" [ Args ] ")"

Primary   ::= Literal
          |  Compr
          |  IDENT
          |  Placeholder
          |  Lambda
          |  "(" Expr ")"

Placeholder ::= "_" | "•"

Args      ::= Expr { "," Expr }

Lambda    ::= IDENT ("=>" | "↦") Expr
```

## A.4 Selectors

```
Selector  ::= IDENT | STRING
```

Selector shorthand:
- A bare `IDENT` selector is treated as the string label of the same text (e.g. `email` ≡ `"email"`) **only** in selector positions.
- This shorthand does **not** apply to Map literal keys (see §3.4).

## A.5 Literals and carriers

```
Literal   ::= Null | Bool | NUM | STRING
          |  SeqLit | SetLit | BagLit | MapLit | ProdLit | BagKVLit
          |  SomeLit | NoneLit | OkLit | FailLit | BytesLit

Null      ::= "null"    ;; standalone reserved word; case-insensitive in the repository profile
Bool      ::= "true" | "false"
BytesLit  ::= "Bytes" "(" STRING ")"   ;; constructor spelling shown canonically; standalone keywords are case-insensitive
SomeLit   ::= "Some" "(" Expr ")"
NoneLit   ::= "None"
OkLit     ::= "Ok" "(" Expr ")"
FailLit   ::= "Fail" "(" Expr "," Expr ")"

SeqLit    ::= "Seq" "[" [ ExprList ] "]"
SetLit    ::= "Set" "{" [ ExprList ] "}"
BagLit    ::= "Bag" "{" [ ExprList ] "}"

MapLit    ::= "Map" "{" [ MapEntryList ] "}"
MapEntryList ::= MapEntry { "," MapEntry }
MapEntry  ::= STRING ("->" | "→") Expr

ProdLit   ::= "Prod" "{" [ ProdFieldList ] "}"
ProdFieldList ::= ProdField { "," ProdField }
ProdField ::= IDENT ":" Expr

BagKVLit  ::= "BagKV" "{" [ BindList ] "}"
BindList  ::= BindEntry { "," BindEntry }
BindEntry ::= (Selector | STRING) ("->" | "→") Expr

ExprList  ::= Expr { "," Expr }
```

## A.6 Comprehensions

Set/bag/seq comprehensions share one surface form; the output carrier preserves the input carrier (§9.1).

```
Compr     ::= "{" ComprBody "}"
ComprBody ::= [ "yield" Expr "|" ] IDENT ("in" | "∈") Expr [ ("|" | "∣") Pred ]
Pred      ::= Expr
```

Examples:
- `{ a in A | a<name> = "steve" }`
- `{ yield a<city> | a ∈ A | a<name> = "steve" }`

```

---

# Appendix B — Embedding examples (Informative)

This appendix is **informative**. It shows how a host language may embed SDA.
Nothing in this appendix changes SDA semantics.

## B.1 Oberon-7z style embedding examples

Assumptions:

- The host language has a string type `STRING` and a tree value type `Tree`.
- The host exposes an SDA evaluator:

  - `SDA.Eval(program: STRING; input: Tree): SDA.Result`
  - where `SDA.Result` can represent either a value or `Fail(code,msg)`.

- The host chooses its own delimiter syntax for embedded SDA. Examples below use `{{ ... }}` as an embedding delimiter.

### B.1.1 Filter a sequence of records

Input: `A` is a `Seq` of `Prod` values.

SDA:

```
{ a ∈ A ∣ a⟨name⟩ = "steve" ∧ a⟨city⟩ ∈ Set{"la","ny"} }
```

Host (Oberon-7z sketch):

```
MODULE Example;

IMPORT SDA;

VAR
  input: Tree;
  out: SDA.Result;
  program: STRING;

BEGIN
  program := "{ a ∈ A ∣ a<name> = \"steve\" and a<city> in Set{\"la\",\"ny\"} }";
  (* The host decides how to bind top-level names like A.
     One common approach: provide `input` as the implicit root, and set A := input. *)
  out := SDA.Eval(program, input);
  (* Host inspects `out` and converts to its preferred representation. *)
END Example.
```

### B.1.2 Normalize untrusted key/value bags

Input: `headers` is a `BagKV` (duplicate keys allowed).

SDA:

```
headers
|> normalizeUnique(_)
|> bindRes(_, m => m<"content-type">!)
```

Host (Oberon-7z sketch):

```
MODULE Headers;

IMPORT SDA;

VAR
  headers: Tree;
  out: SDA.Result;
  program: STRING;

BEGIN
  program := "headers |> normalizeUnique(_) |> bindRes(_, m => m<\"content-type\">!)";
  out := SDA.Eval(program, headers);
END Headers.
```

### B.1.3 Optional vs required extraction

SDA:

```
let emailOpt = user<email>?;
let emailReq = user<email>!;
```

Host (Oberon-7z sketch):

```
MODULE OptReq;

IMPORT SDA;

VAR
  user: Tree;
  out: SDA.Result;
  program: STRING;

BEGIN
  program := "let emailOpt = user<email>?; let emailReq = user<email>!; emailReq;";
  out := SDA.Eval(program, user);
END OptReq.
```

Notes:

- The embedding delimiter (e.g. `{{ ... }}`) is intentionally **not** part of SDA.
- Name binding (e.g. what `A`, `user`, `headers` refer to) is host-defined.
- Error handling is host-defined; SDA guarantees stable error codes/messages at the language boundary.
```
