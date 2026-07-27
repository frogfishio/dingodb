Yes.

Start from the kernel and do not lose it.

Enrichment Language Sketch

Thesis

Enrichment extends SDA with algebraic operations for relating one dataset to another dataset.

Its primitive is the match bag.

For a left value l and right dataset R:

{ r ∈ R | kR(r) = kL(l) }

Everything else in the language is an interpretation of that expression.

Enrichment stays pure.

It does not perform acquisition.
It does not perform orchestration.
It does not hide match policy in host code.

Rule: enrichment is expressed as match-bag formation plus explicit cardinality and output operators.

⸻

01. Core semantic object

For:
	•	left carrier L
	•	right carrier R
	•	left key function kL
	•	right key function kR

define:

Match(l, R, kL, kR) = { r ∈ R | kR(r) = kL(l) }

Semantically, this is a Bag[R].

Duplicates are preserved unless an explicit operator applies a policy.

Rule: the primitive enrichment result is always a bag of matches.

⸻

02. Core surface form

02.1 Match bag comprehension

Surface:

{ r ∈ R | kR(r) = kL(l) }

This is the canonical enrichment form.

Requirements:
	•	l is the current left value
	•	R is a right dataset value
	•	kL and kR are SDA expressions
	•	equality uses SDA equality
	•	result carrier is Bag

Example:

{ c ∈ customers | c.id = l.customer_id }

Rule: enrichment syntax should preserve the explicit match relation in surface form.

⸻

03. Cardinality operators

These operators interpret a match bag.

03.1 one?

one?(B)

Meaning:
	•	0 matches -> None
	•	1 match -> Some(r)
	•	>1 matches -> Fail(t_enr_duplicate,"duplicate match")

Type:

Bag[T] -> Opt[T] | Fail

03.2 one!

one!(B)

Meaning:
	•	0 matches -> Fail(t_enr_missing,"missing match")
	•	1 match -> r
	•	>1 matches -> Fail(t_enr_duplicate,"duplicate match")

Type:

Bag[T] -> T | Fail

03.3 first

first(B)

Meaning:
	•	0 matches -> None
	•	>=1 matches -> first by source order

Type:

Bag[T] -> Opt[T]

This is only valid when source order is defined.

If source order is not defined, evaluation MUST fail with:

Fail(t_enr_unordered_policy,"unordered policy")

03.4 last

Same as first, but chooses last.

03.5 only

only(B)

Meaning:
	•	1 match -> r
	•	otherwise -> fail

This is a stricter spelling of exact uniqueness.

Rule: cardinality is never implicit. A program must say how a match bag is interpreted.

⸻

04. Row result operators

These operators say how interpreted matches enter the result.

04.1 Attach

l + { name: E }

Attach a field to the current left row.

Example:

l + { customer: one!({ c ∈ customers | c.id = l.customer_id }) }

Requirements:
	•	l must be a Prod
	•	field collision policy is explicit or host-defined
	•	default collision policy in v0.1: fail on duplicate field name

Failure:

Fail(t_enr_field_collision,"field collision")

04.2 Merge

merge(l, r)

Merge two products.

Requirements:
	•	both operands must be Prod
	•	field collision policy must be explicit

Possible forms later:
	•	mergeLeft
	•	mergeRight
	•	mergeFail

Default v0.1:

mergeFail(l, r)

04.3 Expand

Expansion is expressed by yielding pairs over the match bag.

Canonical form:

{ yield merge(l, r) | l ∈ L, r ∈ { s ∈ R | kR(s) = kL(l) } }

This is the one-to-many form.

Rule: expansion is not a separate hidden join mode; it is comprehension over the match bag.

⸻

05. Dataset-level forms

Enrichment over a left carrier is expressed as SDA-style comprehensions.

05.1 Attach one

{ yield l + { customer: one!({ c ∈ customers | c.id = l.customer_id }) } | l ∈ orders }

05.2 Attach optional

{ yield l + { customer: one?({ c ∈ customers | c.id = l.customer_id }) } | l ∈ orders }

05.3 Attach group

{ yield l + { items: { i ∈ items | i.order_id = l.id } } | l ∈ orders }

05.4 Expand many

{ yield merge(l, i) | l ∈ orders, i ∈ { i ∈ items | i.order_id = l.id } }

If multi-generator comprehensions are not yet part of SDA, Enrichment may introduce them.

Rule: enrichment over datasets is expressed by mapping or expanding over left rows using match-bag expressions.

⸻

06. Source declarations

The language needs semantic source declarations, not transport declarations.

06.1 Source kinds
	•	Index[K,V]
	•	MultiIndex[K,V]
	•	Dataset[V]

Index means unique key semantics.
MultiIndex means duplicate keys allowed.
Dataset means no uniqueness claim.

06.2 Declared form

Conceptually:

source customers : Index[Str, Customer]
source items     : MultiIndex[Str, Item]

These declarations specify semantic expectations only.

They do not specify HTTP, files, auth, retries, or caching.

Those belong to Axiom.

Rule: source declarations state match semantics, not acquisition mechanics.

⸻

07. Optional keyed sugar

If you want shorter surface syntax later, add sugar on top of the kernel, not instead of it.

Example sugar:

R[kR = kL(l)]

desugars to:

{ r ∈ R | kR(r) = kL(l) }

Example:

one!(customers[c.id = l.customer_id])

This is acceptable only if it preserves the visible equality relation.

⸻

08. Carriers

08.1 Match carrier

The primitive match result is always Bag.

Why:
	•	duplicates must be preserved
	•	uniqueness must not be assumed
	•	policies must be explicit

08.2 Left carrier preservation

For attachment-style enrichment:
	•	Seq -> Seq
	•	Bag -> Bag
	•	Set -> Set

For expansion-style enrichment:
	•	Seq -> Seq
	•	Bag -> Bag
	•	Set -> Bag

Same rule as your earlier draft.

⸻

09. Failures

Required stable failures:
	•	t_enr_missing — required match missing
	•	t_enr_duplicate — uniqueness violated
	•	t_enr_unordered_policy — first/last used without defined order
	•	t_enr_field_collision — illegal merge/attach collision
	•	t_enr_wrong_shape — invalid row shape for attach/merge
	•	t_enr_invalid_key — key expression produced invalid key type

Rule: every cardinality and merge policy must have stable failure semantics.

⸻

10. Provenance

Provenance is orthogonal to matching.

A host or later language revision may annotate matched values with:
	•	source name
	•	source version
	•	matched key
	•	match count
	•	selected policy branch

This should not change the meaning of the match operators.
It only enriches explanation.

Rule: provenance decorates enrichment; it does not replace its semantics.

⸻

11. Minimal law set

11.1 Primitive law

All enrichment forms reduce to:

{ r ∈ R | kR(r) = kL(l) }

plus explicit interpretation.

11.2 No implicit uniqueness

A source match bag is never implicitly collapsed.

11.3 No implicit dropping

Rows are not dropped unless the enclosing comprehension or policy says so.

11.4 Null is not no match

If a matched row contains Null, that is still a match.
No match means the match bag is empty.

11.5 Duplicate preservation

Duplicates in R remain duplicates in the match bag until a cardinality operator resolves them.

⸻

12. Minimal useful subset

If you want the smallest possible first version, it is just this:
	•	match bag comprehension
{ r ∈ R | kR(r) = kL(l) }
	•	one?
	•	one!
	•	first
	•	last
	•	attach via l + { field: E }
	•	merge(l, r)
	•	dataset comprehensions using yield

That is enough to express:
	•	optional lookup
	•	required lookup
	•	grouped one-to-many
	•	expanded one-to-many

without inventing a second commandy language.

04. Enrichment (Normative)

Enrichment extends SDA with pure operations for relating a left dataset to a right dataset.

It is the algebra of matching and combination.

It does not perform acquisition, orchestration, retries, caching, auth, file IO, or HTTP.
Those belong to Axiom.

Rule: all enrichment is expressed as formation of a match bag plus explicit interpretation of that bag.

04.1 Primitive semantic object

For:
	•	left value l
	•	right dataset R
	•	left key function kL
	•	right key function kR

define the primitive match result:

Match(l, R, kL, kR) = { r ∈ R | kR(r) = kL(l) }

Requirements:
	•	Match(...) MUST preserve duplicates from R
	•	Match(...) MUST return a Bag
	•	equality in the predicate MUST use SDA equality
	•	Null is a value, not absence
	•	an empty match bag means no match

Rule: the primitive enrichment result is always a bag of matches.

04.2 Core surface form

The canonical surface form is:

{ r ∈ R | kR(r) = kL(l) }

Example:

{ c ∈ customers | c.id = l.customer_id }

Requirements:
	•	l is the current left value
	•	R is a dataset value
	•	kL and kR are SDA expressions
	•	the result is a Bag

This form is the normative basis of all other enrichment operators.

04.3 Cardinality operators

Cardinality operators interpret a match bag.

04.3.1 one?
one?(B)

Meaning:
	•	count(B) = 0 -> None
	•	count(B) = 1 -> Some(v)
	•	count(B) > 1 -> Fail(t_enr_duplicate,"duplicate match")

Type:

Bag[T] -> Opt[T] | Fail

04.3.2 one!
one!(B)

Meaning:
	•	count(B) = 0 -> Fail(t_enr_missing,"missing match")
	•	count(B) = 1 -> v
	•	count(B) > 1 -> Fail(t_enr_duplicate,"duplicate match")

Type:

Bag[T] -> T | Fail

04.3.3 first
first(B)

Meaning:
	•	count(B) = 0 -> None
	•	count(B) >= 1 -> first element under source-defined order

Type:

Bag[T] -> Opt[T]

If source order is not defined, evaluation MUST fail with:

Fail(t_enr_unordered_policy,"unordered policy")

04.3.4 last
last(B)

Meaning:
	•	count(B) = 0 -> None
	•	count(B) >= 1 -> last element under source-defined order

Type:

Bag[T] -> Opt[T]

If source order is not defined, evaluation MUST fail with:

Fail(t_enr_unordered_policy,"unordered policy")

04.3.5 only
only(B)

Meaning:
	•	count(B) = 1 -> v
	•	otherwise -> failure

Type:

Bag[T] -> T | Fail

This is equivalent to exact uniqueness.

Rule: enrichment MUST NOT collapse a match bag implicitly.

04.4 Row combination operators

These operators specify how interpreted matches enter the result.

04.4.1 Attach
l + { name: E }

Meaning:
	•	evaluate l
	•	evaluate E
	•	return a product equal to l with field name attached

Requirements:
	•	l MUST be Prod
	•	field name MUST be a valid field selector
	•	default field-collision policy in v0.1 is failure

On collision, evaluation MUST fail with:

Fail(t_enr_field_collision,"field collision")

Example:

l + { customer: one!({ c ∈ customers | c.id = l.customer_id }) }

04.4.2 Merge
merge(l, r)

Meaning:
	•	both operands are products
	•	return a combined product

Requirements:
	•	both operands MUST be Prod
	•	default collision policy in v0.1 is failure

On collision, evaluation MUST fail with:

Fail(t_enr_field_collision,"field collision")

If either operand is not a product, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

04.5 Dataset-level enrichment

Dataset-level enrichment is expressed by SDA-style comprehensions over left rows.

04.5.1 Attach required one
{ yield l + { customer: one!({ c ∈ customers | c.id = l.customer_id }) } | l ∈ orders }

04.5.2 Attach optional one
{ yield l + { customer: one?({ c ∈ customers | c.id = l.customer_id }) } | l ∈ orders }

04.5.3 Attach grouped many
{ yield l + { items: { i ∈ items | i.order_id = l.id } } | l ∈ orders }

04.5.4 Expand many
{ yield merge(l, i) | l ∈ orders, i ∈ { i ∈ items | i.order_id = l.id } }

If multi-generator comprehensions are supported, the above form is normative.
If multi-generator comprehensions are not supported in SDA, a conforming host MAY provide an equivalent enrichment expansion form, but MUST preserve the same semantics.

Rule: grouped enrichment preserves the left row count; expansion multiplies rows by match multiplicity.

04.6 Source declarations

Enrichment source declarations are semantic, not operational.

Minimum source kinds:
	•	Index[K,V]
	•	MultiIndex[K,V]
	•	Dataset[V]

Meaning:
	•	Index[K,V] declares unique-key semantics
	•	MultiIndex[K,V] declares duplicate keys may occur
	•	Dataset[V] declares no uniqueness semantics

Example:
	•	source customers : Index[Str, Customer]
	•	source items : MultiIndex[Str, Item]

Requirements:
	•	source declarations MUST NOT encode transport
	•	source declarations MUST NOT encode auth, retries, caching, timeouts, or HTTP behavior
	•	acquisition of source values belongs to Axiom

Rule: source declarations state match semantics only.

04.7 Carriers

04.7.1 Match carrier
The primitive match result MUST be Bag.

04.7.2 Left-carrier preservation
For attachment-style enrichment:
	•	Seq -> Seq
	•	Bag -> Bag
	•	Set -> Set

For expansion-style enrichment:
	•	Seq -> Seq
	•	Bag -> Bag
	•	Set -> Bag

This is normative.

04.8 Failures

Enrichment MUST provide stable failures.

Required failure codes:
	•	t_enr_missing — required match missing
	•	t_enr_duplicate — uniqueness violated
	•	t_enr_unordered_policy — first or last used without defined order
	•	t_enr_field_collision — illegal attach/merge collision
	•	t_enr_wrong_shape — invalid operand shape
	•	t_enr_invalid_key — invalid key type or key evaluation failure

Hosts MAY define additional enrichment failure codes, but MUST NOT change the meaning of the required codes.

04.9 Laws

04.9.1 Primitive reduction law
All enrichment forms reduce to:

{ r ∈ R | kR(r) = kL(l) }

plus explicit interpretation of the resulting bag.

04.9.2 No implicit uniqueness
A match bag MUST NOT be implicitly reduced to one value.

04.9.3 No implicit dropping
A left row MUST NOT disappear unless the enclosing expression explicitly omits it.

04.9.4 Null is not no match
If a matched row contains Null, that row still counts as a match.

04.9.5 Duplicate preservation
Duplicates in R MUST remain duplicates in the match bag until an explicit operator resolves or rejects them.

04.10 Provenance

Provenance is orthogonal to matching semantics.

A host or future revision MAY attach provenance metadata to matched values, including:
	•	source name
	•	source version
	•	matched key
	•	match count
	•	selected policy branch

Such metadata MUST NOT change the meaning of the enrichment operators defined above.