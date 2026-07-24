# SDA GRAMMAR

Below is a pseudogrammar (EBNF-ish) that’s meant to be spec-facing, but also deliberately close to what you’d implement in flex/bison (or Menhir, tree-sitter, etc.). I’m assuming the lexer normalizes Unicode/ASCII synonyms into canonical tokens (so the parser stays small).

--- 

1) Lexical layer assumptions (normative for the grammar)

1.1 Canonical tokens (after lexing)

The lexer MUST produce these canonical tokens (examples of spellings are informative):
	•	Keywords:
LET, YIELD, NULL, TRUE, FALSE,
SEQ, SET, BAG, MAP, PROD, BAGKV,
SOME, NONE, OK, FAIL,
UNION, INTER, DIFF, BUNION, BDIFF,
IN, AND, OR, NOT
	•	Operators / punctuation:
ARROW (→ or ->)
LAMBDA (↦ or =>)
PIPE (|>)
EQ (=)
NEQ (≠ or !=)
LT <  LE <=/≤  GT >  GE >=/≥
PLUS +  MINUS -  STAR *  SLASH /
LPAREN ( RPAREN )
LBRACK [ RBRACK ]
LBRACE { RBRACE }
LANGLE < / ⟨ (see note)
RANGLE > / ⟩ (see note)
QMARK ?  BANG !
COLON :  COMMA ,  SEMI ;
BAR | / ∣
	•	Atoms:
IDENT (identifier)
STRING (double-quoted string)
NUM (numeric literal)
PLACEHOLDER (either _ or •, normalized to one token)

Note on < vs selector brackets
In the surface language, selector access uses either:
	•	ASCII < selector > or Unicode ⟨ selector ⟩.

For a bison-friendly approach, the lexer should return distinct tokens for selector brackets, e.g.:
	•	SEL_L and SEL_R for (< and >) when used in selector access,
	•	leaving </> as normal comparison operators otherwise.

If you don’t want lexer context-sensitivity, you can still parse it by grammar/precedence, but it’s messier.

---

2) Program structure

Program        ::= { Stmt } EOF ;

Stmt           ::= LetStmt
                 | ExprStmt ;

LetStmt        ::= LET IDENT EQ Expr SEMI ;

ExprStmt       ::= Expr SEMI ;


---

3) Expressions (with precedence)

This is written to map cleanly to bison precedence declarations.

3.1 Top level and pipe

Expr           ::= Pipe ;

Pipe           ::= Or
                 | Pipe PIPE Or ;

	•	PIPE is left-associative.

3.2 Boolean logic

Or             ::= And
                 | Or OR And ;

And            ::= Not
                 | And AND Not ;

Not            ::= Cmp
                 | NOT Not ;

3.3 Comparisons

Cmp            ::= Setish
                 | Cmp EQ  Setish
                 | Cmp NEQ Setish
                 | Cmp LT  Setish
                 | Cmp LE  Setish
                 | Cmp GT  Setish
                 | Cmp GE  Setish ;

3.4 Set/bag/seq operators and membership

Setish         ::= Add
                 | Setish UNION  Add
                 | Setish INTER  Add
                 | Setish DIFF   Add
                 | Setish BUNION Add
                 | Setish BDIFF  Add
                 | Add IN Add ;

Notes:
	•	IN is membership (tokenized from ∈ or in).
	•	UNION/INTER/DIFF/BUNION/BDIFF are the keyword forms (from either glyph or ASCII name, lex-normalized).

3.5 Arithmetic (optional but consistent)

Add            ::= Mul
                 | Add PLUS  Mul
                 | Add MINUS Mul ;

Mul            ::= Unary
                 | Mul STAR  Unary
                 | Mul SLASH Unary ;

Unary          ::= Postfix
                 | MINUS Unary ;

3.6 Postfix (selector access, calls)

Postfix        ::= Primary { PostfixOp } ;

PostfixOp      ::= SelectorAccess
                 | CallSuffix ;

CallSuffix     ::= LPAREN [ ArgList ] RPAREN ;

ArgList        ::= Expr { COMMA Expr } ;

Selector access

SelectorAccess ::= SEL_L Selector SEL_R [ QMARK | BANG ] ;

	•	No suffix = total projection (only valid on Prod / schema-known).
	•	? = optional extraction
	•	! = required extraction

Selector       ::= IDENT
                 | STRING ;

(Additional rule from your spec: bare IDENT selector means the label text; that’s semantic, but the grammar accepts it.)

---

4) Primary forms

Primary        ::= Literal
                 | IDENT
                 | PLACEHOLDER
                 | Lambda
                 | ParenExpr
                 | Comprehension ;

ParenExpr      ::= LPAREN Expr RPAREN ;

4.1 Lambda (single argument)

Lambda         ::= IDENT LAMBDA Expr ;

	•	LAMBDA is ↦ or => normalized.

---

5) Literals and carriers

Literal        ::= NULL | TRUE | FALSE | NUM | STRING
                 | SeqLit
                 | SetLit
                 | BagLit
                 | MapLit
                 | ProdLit
                 | BagKVLit
                 | OptLit
                 | ResLit ;

5.1 Seq / Set / Bag

SeqLit         ::= SEQ LBRACK [ ExprList ] RBRACK ;

SetLit         ::= SET LBRACE [ ExprList ] RBRACE ;

BagLit         ::= BAG LBRACE [ ExprList ] RBRACE ;

ExprList       ::= Expr { COMMA Expr } ;

5.2 Map (standalone restriction: keys are STRING only)

MapLit         ::= MAP LBRACE [ MapEntryList ] RBRACE ;

MapEntryList   ::= MapEntry { COMMA MapEntry } ;

MapEntry       ::= STRING ARROW Expr ;

	•	ARROW is → or ->, normalized.
	•	The “STRING-only keys in standalone” is enforced by grammar here.

5.3 Prod

ProdLit        ::= PROD LBRACE [ ProdFieldList ] RBRACE ;

ProdFieldList  ::= ProdField { COMMA ProdField } ;

ProdField      ::= IDENT COLON Expr ;

5.4 BagKV

BagKVLit       ::= BAGKV LBRACE [ BindEntryList ] RBRACE ;

BindEntryList  ::= BindEntry { COMMA BindEntry } ;

BindEntry      ::= Selector ARROW Expr ;

	•	This matches your current surface: BagKV{ k -> v, ... } where k can be IDENT or STRING (since selector positions allow both).

5.5 Option/Result wrappers

OptLit         ::= SOME LPAREN Expr RPAREN
                 | NONE ;

ResLit         ::= OK   LPAREN Expr RPAREN
                 | FAIL LPAREN Expr COMMA Expr RPAREN ;

(If you want Fail(code,msg) to require code be a STRING, do it in semantics or constrain here.)

---

6) Comprehensions

Comprehensions use { ... }, which overlaps with Set literal braces, so we disambiguate by the presence of IN/∈ and |/∣ in the body.

6.1 Comprehension surface

Comprehension  ::= LBRACE ComprBody RBRACE ;

ComprBody      ::= ComprNoYield
                 | ComprWithYield ;

ComprNoYield   ::= IDENT IN Expr [ ComprPred ] ;

ComprWithYield ::= YIELD Expr BAR IDENT IN Expr [ ComprPred ] ;

ComprPred      ::= BAR Expr ;

Notes:
	•	BAR is | or ∣, lex-normalized.
	•	This matches:
	•	{ a ∈ A ∣ P(a) }
	•	{ yield E(a) ∣ a ∈ A ∧ P(a) }

6.2 Important disambiguation rule (normative)

If the parser sees { … } and the body can be parsed as a comprehension (i.e., it contains a generator IDENT IN Expr), it MUST parse as Comprehension, otherwise it is one of:
	•	SetLit / BagLit / MapLit / ProdLit / BagKVLit depending on the leading keyword token (SET, BAG, etc.)
	•	Plain { ... } without a leading carrier keyword is reserved (or parse error), unless you explicitly add “bare set literal” in the future. (Right now your spec uses Set{...} not {...}.)

---

7) Bison-friendly precedence table (suggested)

In bison terms (highest to lowest; pick exact):
	•	Postfix selector/call binds tightest
	•	unary -
	•	* /
	•	+ -
	•	IN and set/bag ops (UNION/INTER/DIFF/BUNION/BDIFF)
	•	comparisons = != < <= > >=
	•	NOT
	•	AND
	•	OR
	•	PIPE (|>)

Also declare PIPE left-assoc.

---

8) Reserved identifier constraints (semantic rules)

These aren’t grammar necessities, but they’re spec-level constraints you probably want stated:
	•	PLACEHOLDER (_/•) is not an IDENT and cannot appear as:
	•	let _ = ...
	•	lambda parameter
	•	map key (already impossible with STRING-only map keys)
	•	keyword tokens (let, yield, in, etc.) are not identifiers.
