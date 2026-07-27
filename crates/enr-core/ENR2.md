Yes.

If those are the core values, then ENR needs a small set of primitive operators over them.

Not fifty helpers. A real basis.

ENR Primitive Operators

Thesis

ENR is the algebra of:
	1.	forming candidates
	2.	refining candidates
	3.	resolving candidates
	4.	combining resolved results
	5.	explaining outcomes

So the primitive operators should follow that order.

⸻

01. Candidate formation

01.1 candidates

Forms a CandidateBag from a left value and a right dataset.

Conceptually:

candidates(l, R, pred) -> CandidateBag[L,R]

or keyed form:

candidates(l, R, kL, kR) -> CandidateBag[L,R]

Mathematically:

candidates(l, R, kL, kR) = { Candidate(l,r,...) | r ∈ R and kR(r)=kL(l) }

Requirements:
	•	MUST preserve duplicates from R
	•	MUST produce one Candidate per matching right value
	•	MUST attach SourceRef
	•	MUST attach initial Provenance
	•	MAY attach initial Evidence

Rule: all enrichment begins with candidates.

⸻

01.2 unionCandidates

Combines candidate bags from multiple sources.

unionCandidates(C1, C2, ... , Cn) -> CandidateBag

Requirements:
	•	MUST preserve duplicates unless a later operator resolves them
	•	MUST preserve per-candidate provenance
	•	MUST NOT merge candidates implicitly

Use:
	•	multi-source enrichment
	•	fallback source stacks
	•	parallel candidate generation

⸻

02. Candidate refinement

These operators keep the same basic type:

CandidateBag -> CandidateBag

They do not resolve; they only shape the candidate space.

02.1 whereCandidate

Filter candidates by predicate.

whereCandidate(C, p) -> CandidateBag

Example uses:
	•	keep only active right rows
	•	keep only candidates above score threshold
	•	keep only exact-class candidates

Rule: filtering removes candidates but does not choose among remaining ones.

⸻

02.2 annotateCandidate

Adds evidence, class, score, rank, or other metadata.

annotateCandidate(C, f) -> CandidateBag

where f maps candidate to extra annotation.

Typical uses:
	•	attach score
	•	attach match class: exact / fallback / fuzzy
	•	attach policy notes

Rule: annotation MUST NOT change left/right identity of a candidate.

⸻

02.3 rankCandidates

Computes an ordering signal.

rankCandidates(C, keyFn) -> CandidateBag

This sets or rewrites rank.

Rule: ranking prepares for resolution but does not itself resolve.

⸻

02.4 preferCandidates

Keeps only maximally preferred candidates under an ordering/policy.

preferCandidates(C, ord) -> CandidateBag

Examples:
	•	prefer exact over fallback
	•	prefer latest timestamp
	•	prefer source priority
	•	prefer highest score

Unlike rankCandidates, this can shrink the bag.

Rule: preference is refinement, not final resolution. If multiple equally preferred candidates remain, ambiguity remains.

⸻

02.5 dedupeCandidates

Collapses equivalent candidates under an explicit equivalence.

dedupeCandidates(C, eq) -> CandidateBag

This is not general duplicate dropping.
It is explicit candidate equivalence resolution.

Use:
	•	same right entity from multiple sources
	•	duplicate raw records representing same candidate

Rule: deduplication MUST state the equivalence basis.

⸻

03. Resolution

Resolution interprets a CandidateBag and produces a Resolved.

This is where ambiguity becomes outcome.

03.1 resolveOptional

resolveOptional(C) -> Resolved[R]

Meaning:
	•	no candidates -> NoMatch
	•	exactly one candidate -> Unique
	•	more than one -> Ambiguous or fail, depending on policy mode

Good default:
	•	0 -> NoMatch
	•	1 -> Unique
	•	>1 -> Ambiguous

This is richer than old one?.

⸻

03.2 resolveRequired

resolveRequired(C) -> Resolved[R]

Meaning:
	•	no candidates -> Rejected(t_enr_missing, ...)
	•	exactly one -> Unique
	•	more than one -> Rejected(t_enr_duplicate, ...) unless pre-refined to one

This is richer than old one!.

⸻

03.3 resolveFirst

resolveFirst(C) -> Resolved[R]

Meaning:
	•	choose first candidate under defined order
	•	record rejected alternatives in Decision

If order is undefined: fail with t_enr_unordered_policy.

⸻

03.4 resolveBest

resolveBest(C, ord) -> Resolved[R]

Meaning:
	•	choose best candidate under explicit ordering
	•	if tie remains, either:
	•	return Ambiguous
	•	or fail
	•	or keep all tied candidates, depending on policy

This is the grown-up version of first.

⸻

03.5 resolveGroup

resolveGroup(C) -> Resolved[Bag[R]]

Meaning:
	•	preserve all surviving candidates as a grouped value
	•	still emit decision/provenance

This is the first-class grouped-many result.

⸻

03.6 resolveReduce

resolveReduce(C, agg) -> Resolved[T]

Meaning:
	•	aggregate multiple candidates into one summary result

Use:
	•	sum balances
	•	choose canonical merged record
	•	summarize evidence across candidates

This is important if ENR is going to be big-boy software.

⸻

04. Combination

These operators combine a left row with a resolved enrichment result.

04.1 attachResolved

attachResolved(l, name, res) -> Prod

Attaches a resolved result under a field.

Examples:
	•	attach customer
	•	attach matches
	•	attach resolution

Policy choice:
	•	attach raw resolved object
	•	attach only resolved value
	•	attach both value and explanation

I’d make both available eventually.

⸻

04.2 mergeResolved

mergeResolved(l, res, policy) -> Prod | Fail

Merges the resolved value into the left row.

Requirements:
	•	resolved value must contain a product-compatible value
	•	field collision policy must be explicit

This is the structural version of enrichment.

⸻

04.3 expandResolved

expandResolved(l, C, f) -> Bag[T]

For each surviving candidate, emit one output row.

Equivalent to one-to-many expansion, but on candidate bags.

Use:
	•	explode all matches
	•	emit audit rows
	•	generate candidate report tables

⸻

04.4 attachCandidates

attachCandidates(l, name, C) -> Prod

Attaches the entire CandidateBag, not a resolved result.

Useful for:
	•	debugging
	•	audit mode
	•	later-stage resolution
	•	human review workflows

This is important. Sometimes the right answer is not to resolve immediately.

⸻

05. Explanation and quality

05.1 explainResolved

explainResolved(res) -> Prod

Returns structured explanation:
	•	chosen candidate
	•	rejected candidates
	•	policy used
	•	evidence summary
	•	provenance

This should be value-level, not just a debug printout.

⸻

05.2 qualityOf

qualityOf(Bag[Resolved[T]]) -> Quality

Produces summary stats:
	•	total
	•	unmatched
	•	unique
	•	chosen
	•	ambiguous
	•	failed

This is how ENR becomes operationally serious.

⸻

05.3 assertQuality

assertQuality(Q, predicate) -> Res[Quality]

Examples:
	•	unmatched must be zero
	•	ambiguity rate < 1%
	•	exact-match rate > 95%

This belongs in real enrichment.

⸻

06. Minimum serious basis

If you want the smallest primitive set that still feels real, I’d pick:

formation
	•	candidates
	•	unionCandidates

refinement
	•	whereCandidate
	•	annotateCandidate
	•	preferCandidates

resolution
	•	resolveOptional
	•	resolveRequired
	•	resolveBest
	•	resolveGroup

combination
	•	attachResolved
	•	mergeResolved
	•	expandResolved
	•	attachCandidates

explanation
	•	explainResolved
	•	qualityOf

That is a real algebra.

⸻

07. Laws

A few laws should hold.

07.1 formation before resolution

All resolution operators consume candidate bags, not raw datasets.

07.2 refinement preserves candidate identity

Filtering, ranking, and annotation do not create fake left/right identities.

07.3 resolution is explicit

No candidate bag is implicitly collapsed.

07.4 explanation survives resolution

A resolved result must retain enough information to explain the decision.

07.5 combination does not retroactively alter resolution

Attach/merge/expand consume a resolved or candidate form; they do not change the prior matching decision.

⸻

08. The center, now properly stated

So the real ENR flow is:

left row
→ candidates
→ refine
→ resolve
→ combine
→ explain

That is the serious version.


Yes.

Keep the surface expression-shaped, not command-shaped.
It should read like SDA extended with candidate and resolution operators.

ENR Surface Syntax (SDA-like)

Thesis

The surface should preserve this center:

{ r ∈ R | kR(r) = kL(l) }

But the primitive value is now a candidate bag, not just a raw bag of right rows.

So the surface should look like:
	•	comprehension
	•	function application
	•	operator wrapping
	•	explicit construction

not like SQL statements or shell commands.

⸻

01. Primitive candidate form

01.1 Candidate comprehension

{ cand r ∈ R | P(l, r) }

Meaning:

form a CandidateBag from all r ∈ R satisfying predicate P.

Example:

{ cand c ∈ customers | c.id = l.customer_id }

This is the canonical ENR form.

It should desugar to something like:

candidates(l, customers, c => c.id = l.customer_id)

but the comprehension is the real surface.

01.2 Keyed sugar

Because equality-match is common, allow sugar:

{ cand c ∈ customers by c.id = l.customer_id }

or tighter:

{ cand c ∈ customers | c.id == l.customer_id }

I would keep the plain predicate form as the core and treat keyed forms as sugar.

⸻

02. Candidate refinement

These should look like ordinary SDA operators over expressions.

02.1 Filter candidates

where(C, p)

Example:

where(
  { cand c ∈ customers | c.id = l.customer_id },
  c => c.right.active = true
)

If you want lighter syntax, allow pipe:

{ cand c ∈ customers | c.id = l.customer_id }
|> where(_.right.active = true)

02.2 Annotate candidates

annotate(C, a => E)

Example:

annotate(
  { cand c ∈ customers | c.id = l.customer_id },
  c => Prod{ class: "exact" }
)

This adds annotation/evidence fields.

02.3 Prefer candidates

prefer(C, ord)

Example:

prefer(C, c => c.score)

or if you want a more SDA-ish operator family:

maxBy(C, c => c.score)

But prefer is probably better because it can preserve ties.

02.4 Rank candidates

rank(C, ord)

Example:

rank(C, c => c.right.updated_at)

This computes rank without resolving.

02.5 Dedupe candidates

dedupe(C, eq)

Example:

dedupe(C, c => c.right.entity_id)


⸻

03. Resolution operators

These should be unary operators over candidate-bag expressions.

03.1 Optional

one?(C)

Example:

one?({ cand c ∈ customers | c.id = l.customer_id })

03.2 Required

one!(C)

Example:

one!({ cand c ∈ customers | c.id = l.customer_id })

03.3 First / last

first(C)
last(C)

03.4 Best

best(C, ord)

Example:

best(C, c => c.score)

or piped:

C |> best(_.score)

03.5 Group

group(C)

This preserves all surviving candidates as a grouped result.

03.6 Reduce

reduce(C, agg)

Example:

reduce(C, cs => sum({ x ∈ cs | x.right.amount }))


⸻

04. Combination operators

These should stay close to SDA product construction.

04.1 Attach resolved value

l + { customer: one!({ cand c ∈ customers | c.id = l.customer_id }) }

That is already good surface syntax.

04.2 Attach candidate bag

l + { customer_candidates: { cand c ∈ customers | c.id = l.customer_id } }

04.3 Attach explanation

l + { customer_match: explain(one!({ cand c ∈ customers | c.id = l.customer_id })) }

04.4 Merge resolved result

merge(l, value(one!({ cand c ∈ customers | c.id = l.customer_id })))

If one! returns a resolved object instead of just the right value, then value(...) extracts the chosen value.

That suggests two helper eliminators:
	•	value(res)
	•	decision(res)

⸻

05. Dataset-level forms

These should remain SDA comprehensions.

05.1 Attach required one

{ yield
    l + {
      customer: value(one!({ cand c ∈ customers | c.id = l.customer_id }))
    }
| l ∈ orders }

05.2 Attach grouped candidates

{ yield
    l + {
      customer_candidates: { cand c ∈ customers | c.id = l.customer_id }
    }
| l ∈ orders }

05.3 Attach grouped resolved matches

{ yield
    l + {
      items: value(group({ cand i ∈ items | i.order_id = l.id }))
    }
| l ∈ orders }

05.4 Expand candidates

{ yield
    merge(l, c.right)
| l ∈ orders,
  c ∈ { cand i ∈ items | i.order_id = l.id } }

That is very close to SDA and keeps the match algebra visible.

⸻

06. Candidate field access

If candidate values are first-class, they need stable field access.

A candidate c should expose at least:
	•	c.left
	•	c.right
	•	c.source
	•	c.evidence
	•	c.score
	•	c.class
	•	c.rank
	•	c.provenance

So this is valid:

{ cand c ∈ customers | c.id = l.customer_id and c.status = "active" }

But inside annotation/refinement functions you want:

c.right.id
c.score
c.class
c.provenance.source.name

So the bound variable in { cand c ∈ R | ... } should denote a candidate, not a raw right row.

That is a big choice, but I think it is the right one.

Then the raw matched value is c.right.

Example:

{ cand c ∈ customers | c.right.id = l.customer_id }

This is semantically cleaner, though slightly heavier.

There are two options:

Option A: binder names raw right row

{ cand c ∈ customers | c.id = l.customer_id }

and candidate metadata gets added later.

Option B: binder names candidate

{ cand c ∈ customers | c.right.id = l.customer_id }

For readability, I would choose A as surface sugar, desugaring into candidate construction.

So surface users write:

{ cand c ∈ customers | c.id = l.customer_id }

but downstream operators see full candidates.

That keeps the syntax pleasant.

⸻

07. Suggested core forms

If I compress this to the real surface basis, I’d keep:

candidate formation

{ cand r ∈ R | P(l, r) }

refinement

where(C, p)
annotate(C, f)
prefer(C, ord)
rank(C, ord)
dedupe(C, eq)

resolution

one?(C)
one!(C)
first(C)
last(C)
best(C, ord)
group(C)
reduce(C, agg)

resolved eliminators

value(res)
decision(res)
provenance(res)

combination

l + { field: E }
merge(l, E)

That is a coherent SDA-like surface.

⸻

08. What it looks like together

Exact unique enrichment

{ yield
    l + {
      customer: value(
        one!({ cand c ∈ customers | c.id = l.customer_id })
      )
    }
| l ∈ orders }

Ambiguity-preserving enrichment

{ yield
    l + {
      customer_candidates:
        { cand c ∈ customers | c.email = l.email }
    }
| l ∈ users }

Ranked best-match enrichment

{ yield
    l + {
      customer: value(
        best(
          annotate(
            { cand c ∈ customers | normalize(c.email) = normalize(l.email) },
            c => Prod{ score: emailScore(l, c.right) }
          ),
          c => c.score
        )
      )
    }
| l ∈ users }

That already feels like a real language surface, not toy syntax and not SQL cosplay.

My recommendation

Use this as the initial surface rule:

ENR syntax is SDA expression syntax extended with candidate comprehensions and candidate-resolution operators.

That keeps the whole thing in one family.

Next step is to choose one of two directions:
tight grammar sketch, or operator-by-operator normative semantics.


Yes.

Appendix B — Minimal ENR grammar sketch (Informative)

This extends the SDA grammar with candidate comprehensions, candidate operators, and resolved-value eliminators.

Program   ::= { Stmt }

Stmt      ::= "let" IDENT "=" Expr ";"
          |  Expr ";"

Expr      ::= Pipe
Pipe      ::= Or { "|>" Or }

Or        ::= And { ("or" | "∨") And }
And       ::= Not { ("and" | "∧") Not }
Not       ::= { ("not" | "¬") } Cmp

Cmp       ::= Add { ("=" | "!=" | "≠" | "<" | "<=" | "≤" | ">" | ">=" | "≥") Add }

Add       ::= Mul { ("+" | "-") Mul }
Mul       ::= Unary { ("*" | "/") Unary }

Unary     ::= { "-" } Postfix

Postfix   ::= Primary { SelectorAccess }

SelectorAccess ::= "." IDENT
                |  "<" Selector ">" [ "?" | "!" ]

Primary   ::= Literal
          |  IDENT
          |  Placeholder
          |  Call
          |  Lambda
          |  "(" Expr ")"
          |  CandidateComp

Placeholder ::= "_" | "•"

Call      ::= IDENT "(" [ Args ] ")"
Args      ::= Expr { "," Expr }

Lambda    ::= IDENT ("=>" | "↦") Expr

Selector  ::= IDENT | STRING

ENR additions

CandidateComp ::= "{" "cand" IDENT InOp Expr "|" Expr "}"

InOp      ::= "in" | "∈"

This is the primitive ENR surface:

{ cand c ∈ customers | c.id = l.customer_id }

Candidate operators

These are parsed as ordinary function calls:

CandidateOpCall ::= IDENT "(" [ Args ] ")"

Reserved ENR operator names:

where
annotate
prefer
rank
dedupe

one?
one!
only
first
last
best
group
reduce

value
decision
provenance
explain
qualityOf
assertQuality

merge

So these are valid expressions:

one?({ cand c ∈ customers | c.id = l.customer_id })

best(
  annotate(
    { cand c ∈ customers | normalize(c.email) = normalize(l.email) },
    c => Prod{ score: emailScore(l, c.right) }
  ),
  c => c.score
)

Existing literals reused

Literal   ::= "Null" | "true" | "false" | NUM | STRING
          |  "Seq" "[" [ ExprList ] "]"
          |  "Set" "{" [ ExprList ] "}"
          |  "Bag" "{" [ ExprList ] "}"
          |  "BagKV" "{" [ BindList ] "}"
          |  "Map" "{" [ MapEntryList ] "}"
          |  "Prod" "{" [ ProdFieldList ] "}"

ExprList  ::= Expr { "," Expr }
BindList  ::= BindEntry { "," BindEntry }
BindEntry ::= Selector ("->" | "→") Expr
MapEntryList ::= MapEntry { "," MapEntry }
MapEntry  ::= STRING ("->" | "→") Expr
ProdFieldList ::= ProdField { "," ProdField }
ProdField ::= IDENT ":" Expr

Notes
	•	CandidateComp is an expression, not a statement form.
	•	The binder in { cand x ∈ R | P } is surface sugar for the matched right value in the predicate.
	•	Candidate metadata is accessed in downstream operators via candidate values, e.g. c.right, c.score, c.provenance.
	•	one? and one! are reserved spellings, not identifiers.

Minimal examples

{ cand c ∈ customers | c.id = l.customer_id }

one!({ cand c ∈ customers | c.id = l.customer_id })

l + {
  customer: value(one!({ cand c ∈ customers | c.id = l.customer_id }))
}

{ yield merge(l, c.right)
| l ∈ orders,
  c ∈ { cand i ∈ items | i.order_id = l.id } }

Next step should be the normative semantics for CandidateComp and one? / one!.


04.1 Candidate comprehension (Normative)

A candidate comprehension forms a CandidateBag from a left value and a right dataset.

04.1.1 Surface form

{ cand r ∈ R | P }

where:
	•	r is the right-side binder
	•	R is a right dataset expression
	•	P is a predicate expression evaluated with:
	•	r bound to each right-side element
	•	any outer bindings in scope, including the current left value such as l

Example:

{ cand c ∈ customers | c.id = l.customer_id }

04.1.2 Result type

The result of candidate comprehension is:

CandidateBag[L,R]

Conceptually:

Bag[Candidate[L,R]]

where L is the current left value type and R is the right element type.

04.1.3 Evaluation

Given current left value l, right dataset R = Bag{ r1, ..., rn }, and predicate P:

{ cand r ∈ R | P }

evaluates by iterating over each ri in R and evaluating P[r := ri].

For each ri such that P[r := ri] = true, the result MUST contain one candidate:

Candidate{ left: l, right: ri, source: sourceOf(R), evidence: initialEvidence(l, ri, P), score: None, class: None, rank: None, provenance: initialProvenance(l, ri, R) }

If P[r := ri] = false, no candidate is emitted for ri.

If P[r := ri] evaluates to a non-boolean value, evaluation MUST fail with:

Fail(t_enr_predicate_not_bool,"predicate not bool")

04.1.4 Duplicate preservation

If the right dataset contains duplicate elements that satisfy P, the resulting CandidateBag MUST contain distinct candidate entries preserving that multiplicity.

Rule: candidate comprehension preserves right-side multiplicity.

04.1.5 Carrier of the source dataset

A candidate comprehension accepts any right dataset carrier whose elements can be iterated.

Minimum required source carriers:
	•	Seq[T]
	•	Bag[T]
	•	Set[T]

The result is always a Bag[Candidate].

If R is not an iterable dataset carrier, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

04.1.6 Source identity

Each produced candidate MUST carry a SourceRef.

If the host supplied a declared source, source MUST reflect that declaration.

If the host did not supply source metadata, the host MUST still provide a stable anonymous source identity sufficient for provenance and explanation.

04.1.7 Initial provenance

Each produced candidate MUST carry initial provenance at least sufficient to record:
	•	source identity
	•	left key or relevant left-side binding values if available
	•	right key or relevant right-side binding values if available

Hosts MAY attach richer provenance.

04.1.8 Initial evidence

A candidate comprehension MAY attach initial evidence describing the predicate basis of the match.

Minimum conformance does not require a specific evidence schema, but any emitted evidence MUST be descriptive and MUST NOT alter candidate identity.

04.1.9 Binder scope

The binder introduced by candidate comprehension is scoped only over the predicate expression.

It does not escape the candidate comprehension except through the produced candidate values.

Rule: { cand r ∈ R | P } binds r only inside P.

⸻

04.2 Resolution operators (Normative)

Resolution operators consume a CandidateBag and produce a resolved outcome.

They do not perform acquisition.
They do not generate candidates.
They interpret candidate multiplicity explicitly.

04.2.1 one?

Surface:

one?(C)

where C evaluates to a CandidateBag.

Meaning
Let C evaluate to a candidate bag with cardinality n.
	•	if n = 0, one?(C) returns:

Resolved{ value: NoMatch, decision: Decision{ outcome: "none", policy: "one?" }, provenance: provenanceOf(C) }
	•	if n = 1, one?(C) returns:

Resolved{ value: Unique(c.right), decision: Decision{ outcome: "unique", chosen_index: 0, policy: "one?" }, provenance: provenanceOf(C) }

where c is the sole candidate
	•	if n > 1, one?(C) returns:

Resolved{ value: Rejected(t_enr_duplicate,"duplicate match"), decision: Decision{ outcome: "failed", policy: "one?" }, provenance: provenanceOf(C) }

Type
CandidateBag[L,R] -> Resolved[R]

Failure conditions
If C does not evaluate to a CandidateBag, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

Rule: one? is optional on missing, strict on duplicates.

⸻

04.2.2 one!

Surface:

one!(C)

where C evaluates to a CandidateBag.

Meaning
Let C evaluate to a candidate bag with cardinality n.
	•	if n = 0, one!(C) returns:

Resolved{ value: Rejected(t_enr_missing,"missing match"), decision: Decision{ outcome: "failed", policy: "one!" }, provenance: provenanceOf(C) }
	•	if n = 1, one!(C) returns:

Resolved{ value: Unique(c.right), decision: Decision{ outcome: "unique", chosen_index: 0, policy: "one!" }, provenance: provenanceOf(C) }

where c is the sole candidate
	•	if n > 1, one!(C) returns:

Resolved{ value: Rejected(t_enr_duplicate,"duplicate match"), decision: Decision{ outcome: "failed", policy: "one!" }, provenance: provenanceOf(C) }

Type
CandidateBag[L,R] -> Resolved[R]

Failure conditions
If C does not evaluate to a CandidateBag, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

Rule: one! is strict on missing and strict on duplicates.

⸻

04.3 Resolved eliminators (Normative)

Because one? and one! return resolved values, eliminators are required.

04.3.1 value

Surface:

value(res)

Meaning
	•	if res.value = Unique(v), return v
	•	if res.value = Chosen(v), return v
	•	if res.value = NoMatch, return Fail(t_enr_missing,"missing match")
	•	if res.value = Ambiguous(...), return Fail(t_enr_duplicate,"duplicate match")
	•	if res.value = Rejected(code,msg), return Fail(code,msg)

Type
Resolved[T] -> T | Fail

Rule: value extracts the chosen value or fails.

04.3.2 decision

Surface:

decision(res)

Returns the decision record of res.

Type:

Resolved[T] -> Decision

04.3.3 provenance

Surface:

provenance(res)

Returns the provenance of res.

Type:

Resolved[T] -> Provenance

⸻

04.4 Laws for CandidateComp, one?, and one! (Normative)

04.4.1 Empty-source law

For any left value l and empty right dataset R:

{ cand r ∈ R | P } = Bag{}

Therefore:
	•	one?({ cand r ∈ R | P }) yields NoMatch
	•	one!({ cand r ∈ R | P }) yields Rejected(t_enr_missing,...)

04.4.2 Singleton-success law

If exactly one right element satisfies P, then both one? and one! succeed with that right value.

04.4.3 Duplicate-failure law

If more than one right element satisfies P, then both one? and one! reject with t_enr_duplicate.

04.4.4 Null-is-still-a-match law

If a candidate’s right value or a field within it is Null, that candidate still counts as a candidate.

Null does not mean no match.

04.4.5 No implicit collapse law

A candidate bag MUST NOT be implicitly reduced to one value.
Any collapse MUST occur through an explicit resolution operator.

⸻

04.5 Minimal examples (Normative)

Required unique match

value(one!({ cand c ∈ customers | c.id = l.customer_id }))

Optional unique match with explanation preserved

one?({ cand c ∈ customers | c.id = l.customer_id })

Attach required enrichment

l + { customer: value(one!({ cand c ∈ customers | c.id = l.customer_id })) }

Attach resolution metadata

l + { customer_match: one?({ cand c ∈ customers | c.id = l.customer_id }) }

Next, the natural section is first, last, best, group, and reduce.


04.6 Ordered and grouped resolution operators (Normative)

These operators consume a CandidateBag and produce a resolved outcome.

They refine or interpret multiplicity explicitly.
They do not generate candidates.
They do not perform acquisition.

⸻

04.6.1 first

Surface:

first(C)

where C evaluates to a CandidateBag.

Meaning
Let C evaluate to candidate bag Bag{ c0, c1, ..., cn-1 } under a defined source order.
	•	if n = 0, first(C) returns:

Resolved{ value: NoMatch, decision: Decision{ outcome: "none", policy: "first" }, provenance: provenanceOf(C) }
	•	if n >= 1, first(C) returns:

Resolved{ value: Chosen(c0.right), decision: Decision{ outcome: "chosen", chosen_index: 0, rejected_indices: Seq[1, ..., n-1], policy: "first" }, provenance: provenanceOf(C) }

Type
CandidateBag[L,R] -> Resolved[R]

Failure conditions
If C does not evaluate to a CandidateBag, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

If the candidate bag has no defined order, evaluation MUST fail with:

Fail(t_enr_unordered_policy,"unordered policy")

Rule: first is permissive on duplicates but requires defined order.

⸻

04.6.2 last

Surface:

last(C)

where C evaluates to a CandidateBag.

Meaning
Let C evaluate to candidate bag Bag{ c0, c1, ..., cn-1 } under a defined source order.
	•	if n = 0, last(C) returns:

Resolved{ value: NoMatch, decision: Decision{ outcome: "none", policy: "last" }, provenance: provenanceOf(C) }
	•	if n >= 1, last(C) returns:

Resolved{ value: Chosen(cn-1.right), decision: Decision{ outcome: "chosen", chosen_index: n-1, rejected_indices: Seq[0, ..., n-2], policy: "last" }, provenance: provenanceOf(C) }

Type
CandidateBag[L,R] -> Resolved[R]

Failure conditions
If C does not evaluate to a CandidateBag, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

If the candidate bag has no defined order, evaluation MUST fail with:

Fail(t_enr_unordered_policy,"unordered policy")

Rule: last is permissive on duplicates but requires defined order.

⸻

04.6.3 best

Surface:

best(C, ord)

where:
	•	C evaluates to a CandidateBag
	•	ord is an ordering expression or key function over candidates

Meaning
best(C, ord) selects the maximally preferred candidate under ord.

Let S be the subset of candidates in C whose ordering value is maximal under ord.
	•	if count(C) = 0, best(C, ord) returns:

Resolved{ value: NoMatch, decision: Decision{ outcome: "none", policy: "best" }, provenance: provenanceOf(C) }
	•	if count(S) = 1, best(C, ord) returns:

Resolved{ value: Chosen(s.right), decision: Decision{ outcome: "chosen", chosen_index: indexOf(s), rejected_indices: indicesOf(C \\ {s}), policy: "best" }, provenance: provenanceOf(C) }

where s is the sole best candidate
	•	if count(S) > 1, best(C, ord) returns:

Resolved{ value: Ambiguous(Bag{ s.right | s ∈ S }), decision: Decision{ outcome: "ambiguous", chosen_index: None, rejected_indices: indicesOf(C \\ S), policy: "best" }, provenance: provenanceOf(C) }

Type
CandidateBag[L,R] × (Candidate[L,R] -> K) -> Resolved[R]

where K is an orderable domain.

Failure conditions
If C does not evaluate to a CandidateBag, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

If ord yields values that are not orderable under SDA ordering rules, evaluation MUST fail with:

Fail(t_enr_unorderable_key,"unorderable key")

If ord fails for any candidate, the entire evaluation MUST fail with that failure.

Rule: best chooses only when a unique maximum exists. Ties remain explicit ambiguity.

⸻

04.6.4 group

Surface:

group(C)

where C evaluates to a CandidateBag.

Meaning
group(C) preserves all candidates as a grouped result.

Let C = Bag{ c0, ..., cn-1 }.

Then:
	•	if n = 0, group(C) returns:

Resolved{ value: Unique(Bag{}), decision: Decision{ outcome: "unique", chosen_index: None, policy: "group" }, provenance: provenanceOf(C) }
	•	if n >= 1, group(C) returns:

Resolved{ value: Unique(Bag{ c0.right, ..., cn-1.right }), decision: Decision{ outcome: "unique", chosen_index: None, policy: "group" }, provenance: provenanceOf(C) }

Type
CandidateBag[L,R] -> Resolved[Bag[R]]

Rule: group never treats multiplicity as an error. It preserves surviving multiplicity.

⸻

04.6.5 reduce

Surface:

reduce(C, agg)

where:
	•	C evaluates to a CandidateBag
	•	agg is an aggregation function over the bag of surviving right-side values, or over candidates if the host defines that variant

Meaning
Let V = Bag{ c.right | c ∈ C }.

Then reduce(C, agg) evaluates agg(V) and returns:

Resolved{ value: Unique(agg(V)), decision: Decision{ outcome: "unique", chosen_index: None, policy: "reduce" }, provenance: provenanceOf(C) }

Type
Minimum required form:

CandidateBag[L,R] × (Bag[R] -> T) -> Resolved[T]

Hosts MAY additionally support:

CandidateBag[L,R] × (CandidateBag[L,R] -> T) -> Resolved[T]

but MUST document which variant is implemented.

Failure conditions
If C does not evaluate to a CandidateBag, evaluation MUST fail with:

Fail(t_enr_wrong_shape,"wrong shape")

If agg fails, the whole expression MUST fail with that failure.

Rule: reduce summarizes multiplicity; it does not discard multiplicity silently.

⸻

04.7 Resolved outcome forms (Normative)

The following resolution-value variants are used by one?, one!, first, last, best, group, and reduce:
	•	NoMatch
	•	Unique(v)
	•	Chosen(v)
	•	Ambiguous(Bag[v])
	•	Rejected(code,msg)

04.7.1 Meaning of variants
	•	NoMatch means no candidates survived to resolution.
	•	Unique(v) means the outcome is determined without ambiguity and without arbitrary selection.
	•	Chosen(v) means one value was selected from multiple candidates by explicit policy.
	•	Ambiguous(Bag[v]) means multiple candidates remain equally valid under the given policy.
	•	Rejected(code,msg) means the resolution failed by policy.

Rule: ambiguity is a first-class resolution result and MUST NOT be silently collapsed.

⸻

04.8 Resolved eliminator law extensions (Normative)

04.8.1 value over additional outcomes

value(res) MUST behave as follows:
	•	Unique(v) -> v
	•	Chosen(v) -> v
	•	NoMatch -> Fail(t_enr_missing,"missing match")
	•	Ambiguous(vs) -> Fail(t_enr_duplicate,"duplicate match")
	•	Rejected(code,msg) -> Fail(code,msg)

Rule: value is strict. It extracts only determined outcomes.

⸻

04.9 Laws for ordered and grouped resolution (Normative)

04.9.1 first/last order law

first(C) and last(C) are defined only when the candidate bag carries a stable order.

If no stable order exists, evaluation MUST fail with t_enr_unordered_policy.

04.9.2 group multiplicity law

For any candidate bag C, group(C) preserves the multiplicity of c.right values induced by C.

04.9.3 best tie law

If two or more candidates are maximal under ord, best(C, ord) MUST return Ambiguous(...). It MUST NOT choose arbitrarily.

04.9.4 reduce totality law

reduce(C, agg) is total exactly when agg is total over the bag of right-side values.

04.9.5 empty-bag laws

For empty C:
	•	first(C) -> NoMatch
	•	last(C) -> NoMatch
	•	best(C, ord) -> NoMatch
	•	group(C) -> Unique(Bag{})
	•	reduce(C, agg) -> whatever agg(Bag{}) yields, or failure if agg fails on empty input

⸻

04.10 Minimal examples (Normative)

Choose first by source order

value(first({ cand c ∈ customers | c.email = l.email }))

Choose best by score

value(best(C, c => c.score))

Preserve all item matches

value(group({ cand i ∈ items | i.order_id = l.id }))

Summarize matched amounts

value(reduce({ cand p ∈ payments | p.order_id = l.id }, ps => sum(ps)))

Attach grouped enrichment

l + { items: value(group({ cand i ∈ items | i.order_id = l.id })) }

Attach best-match explanation

l + { customer_match: best(C, c => c.score) }

The next natural section is refinement operators: where, annotate, rank, prefer, and dedupe.
