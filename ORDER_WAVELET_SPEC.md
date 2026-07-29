# Dingo Order Wavelet (DOW) specification

Status: **Normative design v1.0-draft; not yet implemented**

Profiles:

```text
dingo-order-wavelet-v1
dingo-order-dictionary-v1
dingo-order-wavelet-block-v1
dingo-order-wavelet-cursor-v1
```

Audience: query planner, index, storage, SDK, cluster, examination, and
conformance implementers

Normative companions:
[DIRECT_ACCESS_SPEC.md](DIRECT_ACCESS_SPEC.md),
[DQL_SPEC.md](DQL_SPEC.md),
[DINGO_PREDICATE_SPEC.md](DINGO_PREDICATE_SPEC.md),
[HEAP_SPEC.md](HEAP_SPEC.md),
[CLUSTER_SPEC.md](CLUSTER_SPEC.md),
[SDA_PROFILE.md](SDA_PROFILE.md), and
[INDEXING_STRATEGY_PROPOSAL.md](INDEXING_STRATEGY_PROPOSAL.md)

## 1. Decision

DingoDB SHALL support exact ranked navigation through a filtered result in a
declared DQL order without sorting the matching documents at query time and
without enumerating the preceding matches.

The semantic structure is the **Dingo Order Wavelet**:

```text
exact DQL match bitmap
        +
versioned wavelet order index
        +
rank/select branch counts
        =
exact kth filtered document in DQL order
```

The governing rule is:

> Sorting is compiled into a derived decision structure; a query navigates
> exact conditional counts.

For:

```text
where P
order by K
at rank k
```

DOW computes:

\[
\operatorname{SelectOrdered}(P,K,k)
\]

with positioning work independent of the numeric magnitude of `k`, subject to
the admitted bitmap and order-index costs defined here.

DOW is a physical implementation of the logical ordering and direct-selection
requirements in `dingo-direct-access-v1`. It does not change DQL semantics.

## 2. Requirement language

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, MAY, and OPTIONAL are normative.

An implementation conforms only for the scalar codecs, tuple shapes, block
profiles, and execution modes it declares and passes in §25.

## 3. Problem statement

Let:

- `H` be one Heap;
- `V` be one frozen read view;
- `D = (d_0,\ldots,d_{n-1})` be the live documents in ascending immutable
  document-ID order;
- `P(d)` be one total DQL predicate;
- `K(d)` be the explicit DQL sort tuple excluding the implicit document-ID
  tie-breaker.

The desired order is:

\[
\kappa(d)=(K(d),id(d))
\]

using DQL's exact scalar, Null, Absent, direction, and collation rules.

Define the matching set:

\[
Q=\{d_i\mid P(d_i)=1\}
\]

The ordinary strategy computes `Q`, comparison-sorts it by `κ`, then selects
position `k`. DOW instead preprocesses `K(d_i)` into a stable rank/select
decision structure and transports the exact predicate bitmap through it.

The output MUST equal:

\[
\operatorname{sort}_{\kappa}(Q)[k-1]
\]

for every valid one-based rank:

\[
1 \le k \le |Q|
\]

### 3.1 Why an ordinary ordered index is insufficient

An ordinary order tree may know the total population of subtree `R`:

\[
N_R=|R|
\]

The filtered selection algorithm needs:

\[
N_R(P)
=
\left|
\{d\in R\mid P(d)=1\}
\right|
\]

`N_R` does not determine `N_R(P)`. Two admissible datasets can have identical
tree shape and subtree population but put every predicate match in opposite
branches.

Scanning ordered leaves and testing `P` computes the answer but can require:

\[
\Theta(k)
\]

matching-prefix enumeration.

A compound index can precompute `N_R(P)` for one fixed predicate shape, but
doing so for every combination of filter fields and sort fields creates an
unbounded index-product problem.

DOW instead computes conditioned counts:

\[
N_R(P)=|M_P\cap R|_1
\]

from one exact predicate bitmap and one reusable order decision structure.
This separates filter indexing from order indexing while retaining exact
joint counts at query time.

## 4. Terms

**Source order**
: Ascending immutable document-ID order. It is the stable input sequence to
  the order wavelet.

**User sort tuple**
: The explicit DQL `order by` tuple before the implicit document-ID
  tie-breaker.

**Order symbol**
: A dense ordinal representing one distinct user sort tuple under one order
  dictionary.

**Order dictionary**
: An immutable bijection between distinct user sort tuples and dense symbols
  preserving DQL order.

**Wavelet node**
: One prefix decision over order-symbol bits, containing the stable sequence
  of documents reaching that prefix and a branch bitmap.

**Branch bitmap**
: At a node, bit `j` says whether the document at node-local position `j`
  takes the zero or one child.

**Candidate bitmap**
: An exact DQL match bitmap transported into a wavelet node's local sequence.
  “Candidate” here means candidate for ordered selection, not an approximate
  index result.

**Conditioned count**
: The number of predicate matches taking one wavelet branch.

**Descent**
: Stable projection of a node-local candidate bitmap into one child.

**Order-wavelet block**
: An independently verifiable immutable portion of the source sequence with
  its dictionary references, wavelet bitmaps, rank/select data, and origin.

**Wavelet forest**
: Multiple immutable order-wavelet generations or blocks participating in one
  frozen view.

**Global value directory**
: An exact ordered union of sort tuples represented by a wavelet forest, used
  when members do not share one dense symbol dictionary.

## 5. DQL order domain

### 5.1 User tuple

For DQL order terms:

```text
order by
  p1 dir1 nulls n1 missing m1,
  ...
  pt dirt nulls nt missing mt
```

define:

\[
K(d)=
(
\operatorname{ord}_1(value(d,p_1)),
\ldots,
\operatorname{ord}_t(value(d,p_t))
)
\]

where each `ord_i` incorporates:

- field path;
- scalar family;
- exact numeric semantics;
- direction;
- Null placement;
- Absent placement;
- string collation/version;
- byte ordering;
- semantic profile version.

Products, sequences, bags, sets, and maps remain invalid DQL v1 sort keys.

### 5.2 Strict total order

The user tuple may contain ties. DQL appends immutable document identity:

\[
\kappa(d)=(K(d),id(d))
\]

Therefore:

\[
d_a \ne d_b
\implies
\kappa(d_a)\ne\kappa(d_b)
\]

and:

\[
<_{\kappa}
\]

is a strict total order.

### 5.3 Direction

Direction is part of each order term, not a runtime reversal applied after
selection.

For a term domain `(X,<)`, descending order uses:

\[
x <_{\mathrm{desc}} y
\iff
y < x
\]

The order dictionary is constructed using the resulting DQL comparator.

### 5.4 Null and Absent

Null and Absent are distinct values. Their placement follows DQL exactly.

If both occupy the same end, DQL's specified relative order remains:

\[
\mathrm{Null}<\mathrm{Absent}
\]

within that placement category unless the DQL profile is later versioned.

An order index that collapses Null and Absent cannot certify DOW.

## 6. Order dictionary

### 6.1 Finite view dictionary

Let:

\[
\mathcal{A}_V
=
\{K(d_i)\mid 0\le i<n\}
\]

be the finite set of distinct user sort tuples in the frozen view.

Sort its members using the exact DQL comparator:

\[
a_0 <_K a_1 <_K \cdots <_K a_{\sigma-1}
\]

where:

\[
\sigma=|\mathcal{A}_V|
\]

Define the order dictionary:

\[
\rho_V(a_j)=j
\]

Then:

\[
x <_K y
\iff
\rho_V(x)<\rho_V(y)
\]

and:

\[
\rho_V:
\mathcal{A}_V\rightarrow[0,\sigma)
\]

is a bijective order embedding.

This construction avoids relying on machine byte order or an unproved
order-preserving serialization for integers, exact decimals, strings, Null,
or Absent.

### 6.2 Symbol sequence

Define:

\[
S_0[i]=\rho_V(K(d_i))
\]

for:

\[
0\le i<n
\]

`S₀` is in source document-ID order, not sort-value order.

### 6.3 Width

For:

\[
\sigma>1
\]

the fixed symbol width is:

\[
h=\lceil\log_2\sigma\rceil
\]

Symbols use unsigned big-endian bit significance:

\[
\operatorname{bit}(x,\ell)
=
\left\lfloor
\frac{x}{2^{h-1-\ell}}
\right\rfloor
\bmod 2
\]

for:

\[
0\le\ell<h
\]

If:

\[
\sigma\le1
\]

the tree has one leaf and height zero.

### 6.4 Dictionary identity

An order dictionary identity binds:

```text
heap_id
collection_id
read_view/generation
ordered paths
directions
Null/Absent placement
scalar and numeric profiles
collation/version
ordered tuple entries or dictionary root hash
symbol width
```

Two symbol values are comparable only inside the same dictionary identity.

### 6.5 Dictionary representation

Implementations MAY use:

- sorted front-coded tuples;
- compressed radix or Patricia tries;
- wavelet tries for variable-length tuple encodings;
- monotone perfect hashes plus verified tuple storage;
- another deterministic exact dictionary.

Every lookup MUST verify the exact tuple. Hashes or fingerprints alone cannot
establish dictionary equality.

## 7. Semantic wavelet construction

### 7.1 Tree definition

The normative semantic model is a binary wavelet tree. A physical wavelet
matrix MAY flatten levels when it preserves this model exactly.

At depth `ℓ`, a node `v` represents one prefix:

\[
p_v\in\{0,1\}^{\ell}
\]

and contains the stable subsequence:

\[
S_v
=
\left[
S_0[i]
\mid
\operatorname{prefix}_{\ell}(S_0[i])=p_v
\right]
\]

in ascending original source position `i`.

### 7.2 Branch bitmap

For internal node `v` at depth `ℓ`, define:

\[
B_v[j]
=
\operatorname{bit}(S_v[j],\ell)
\]

The zero-child sequence is:

\[
S_{v0}
=
\left[
S_v[j]\mid B_v[j]=0
\right]
\]

The one-child sequence is:

\[
S_{v1}
=
\left[
S_v[j]\mid B_v[j]=1
\right]
\]

Both filters are stable: relative order is preserved.

### 7.3 Rank mapping

Use exclusive branch rank:

\[
\operatorname{rank}_b(B,i)
=
\left|
\{j\mid 0\le j<i\land B[j]=b\}
\right|
\]

If parent position `i` takes branch `b`, its child position is:

\[
\tau_{v,b}(i)
=
\operatorname{rank}_b(B_v,i)
\]

For all parent positions taking `b`, `τ` is a strictly increasing bijection
onto:

\[
[0,|S_{vb}|)
\]

### 7.4 Select mapping

The inverse mapping is:

\[
\tau^{-1}_{v,b}(q)
=
\operatorname{select}_b(B_v,q)
\]

where `select_b(B,q)` returns the zero-based position of the `q`th `b` bit.

Thus a leaf-local position can be mapped back to its original source ordinal
by applying inverse select mappings from leaf to root.

### 7.5 Stable tie order

If:

\[
S_0[i]=S_0[j]
\land
i<j
\]

then both symbols take the same branch at every level. Since each partition is
stable, their relative order remains:

\[
i_{\mathrm{leaf}}<j_{\mathrm{leaf}}
\]

Since source order is immutable document-ID order, every leaf lists documents
with equal user sort tuple in ascending document-ID order.

Therefore the wavelet leaf order is exactly:

\[
(K(d),id(d))
\]

without including document ID in the symbol alphabet.

## 8. Predicate bitmap

### 8.1 Root bitmap

Let:

\[
M_0[i]
=
\begin{cases}
1 & P(d_i)=1 \\
0 & P(d_i)=0
\end{cases}
\]

`M₀` MUST be exact under `dingo-predicate-v1` and aligned to:

- the same Heap;
- the same frozen read view;
- the same collection/source universe;
- the same source document-ID order;
- the same liveness/tombstone frontier.

### 8.2 Approximate candidates

If the planner has only:

\[
\operatorname{Matches}(P)\subseteq C
\]

then `C` cannot be used as `M₀` for exact ordered selection.

The planner must verify candidates and construct an exact `M₀`, build a
selection artifact, or refuse direct access.

### 8.3 Cardinality

The number of filtered answers is:

\[
m=|M_0|_1
\]

If:

\[
k>m
\]

the exact result is absent and the page is complete under the declared rank
domain.

## 9. Filter transport through the wavelet

### 9.1 Child projection

Let `C_v` be an exact candidate bitmap aligned to node sequence `S_v`.

Define stable descent into branch `b`:

\[
\operatorname{Descend}_b(C_v,B_v)[q]
=
C_v[
\operatorname{select}_b(B_v,q)
]
\]

for:

\[
0\le q<|S_{vb}|
\]

Equivalently, `Descend` deletes positions taking the other branch while
preserving the remaining bits' relative order.

### 9.2 Conditioned branch count

The number of matching documents in branch `b` is:

\[
c_b(v)
=
\sum_{i=0}^{|S_v|-1}
C_v[i]\,[B_v[i]=b]
\]

where `[condition]` is one when true and zero otherwise.

In bitmap notation:

\[
c_0(v)=|C_v\cap\neg B_v|_1
\]

\[
c_1(v)=|C_v\cap B_v|_1
\]

and:

\[
c_0(v)+c_1(v)=|C_v|_1
\]

### 9.3 Projection cardinality

For:

\[
C_{vb}=\operatorname{Descend}_b(C_v,B_v)
\]

we have:

\[
|C_{vb}|_1=c_b(v)
\]

because `select_b` is a bijection between child positions and parent
positions taking branch `b`.

### 9.4 Physical operations

An implementation MAY compute descent using:

- compressed bitmap intersection plus stable compaction;
- rank/select projection;
- SIMD bit extraction;
- Roaring container projection;
- materialized node-local predicate bitmaps;
- another exact equivalent.

The physical operation MUST equal §9.1. Merely intersecting two same-length
bitmaps without translating child coordinates is insufficient for descent.

## 10. Exact conditioned selection

### 10.1 Algorithm

Input:

- wavelet root;
- exact root match bitmap `M₀`;
- zero-based requested result ordinal `r`, where `0 ≤ r < |M₀|₁`.

Algorithm:

```text
v := root
C := M0

while v is not a leaf:
    z := popcount(C AND NOT Bv)

    if r < z:
        C := Descend0(C, Bv)
        v := zero_child(v)
    else:
        r := r - z
        C := Descend1(C, Bv)
        v := one_child(v)

q := select1(C, r)
i := inverse_wavelet_position(v, q)
return document_at_source_ordinal(i)
```

### 10.2 Branch decision

At node `v`, all zero-child symbols precede all one-child symbols because
their next most-significant differing bit is `0` versus `1`.

If:

\[
r<c_0(v)
\]

the requested result is the `r`th match in the zero child.

Otherwise it is the:

\[
r-c_0(v)
\]

th match in the one child.

### 10.3 Leaf decision

At the selected leaf, every document has the same user sort tuple. By §7.5,
leaf order is ascending immutable document ID.

Therefore:

\[
q=\operatorname{select}_1(C,r)
\]

selects the correct tie-broken document.

### 10.4 Product rank

For one-based DQL rank `k`:

\[
r=k-1
\]

The algorithm returns:

\[
\operatorname{sort}_{(K,id)}
\left(
\{d\mid P(d)=1\}
\right)[k-1]
\]

### 10.5 Worked example

Eight documents in source-ID order have dense price symbols:

```text
source ordinal i   0 1 2 3 4 5 6 7
price symbol       2 0 3 1 2 1 0 3
matches P          1 0 1 1 0 1 1 0
```

The filtered order oracle is:

```text
(symbol 0, id 6)
(symbol 1, id 3)
(symbol 1, id 5)
(symbol 2, id 0)
(symbol 3, id 2)
```

Request one-based rank:

\[
k=4
\]

so:

\[
r=3
\]

With two-bit symbols, the root most-significant-bit bitmap is:

```text
Broot = 1 0 1 0 1 0 0 1
Mroot = 1 0 1 1 0 1 1 0
```

Matching zero-branch count:

\[
|M_{\mathrm{root}}\cap\neg B_{\mathrm{root}}|_1=3
\]

Since:

\[
r=3\not<3
\]

choose the one branch and update:

\[
r:=3-3=0
\]

Stable descent into root-one positions `(0,2,4,7)` gives:

```text
symbols = 2 3 2 3
C       = 1 1 0 0
Bnext   = 0 1 0 1
```

The conditioned zero count is one. Since `r=0`, choose zero, reaching symbol
`2`. The selected set bit is its first matching occurrence. Inverse select
mapping returns source ordinal `0`.

Thus DOW returns:

```text
(symbol 2, id 0)
```

which is exactly the fourth filtered document in `(price,id)` order. It never
enumerated the first three results.

## 11. Correctness proof

### 11.1 Node invariant

At every visited node `v`, maintain:

1. `S_v` is the stable source-order subsequence whose symbols share prefix
   `p_v`;
2. `C_v[j]=1` exactly when the document represented by `S_v[j]` satisfies
   `P`;
3. `r` is the zero-based rank sought among set bits of `C_v` in symbol order
   below `v`, with source order breaking complete-symbol ties.

The invariant holds at the root by definition of `S₀` and `M₀`.

### 11.2 Descent preservation

Assume the invariant at `v`.

`Descend_b` uses the inverse rank/select bijection to retain exactly the
parent positions taking branch `b`, with their candidate bits unchanged and
in stable order. Therefore invariant items 1 and 2 hold for child `vb`.

All values in `v0` precede all values in `v1`. The branch rule either:

- retains `r` in `v0`; or
- subtracts exactly the number of earlier matching values before entering
  `v1`.

Thus invariant item 3 also holds.

By induction, the invariant holds at the leaf.

### 11.3 Leaf correctness

At a leaf, symbol order is exhausted. All surviving documents share user tuple
`K`. Stable construction preserves document-ID order. Selecting set bit `r`
therefore returns the unique document with exactly the required number of
matching predecessors under `(K,id)`.

### 11.4 Theorem

For exact `M₀`, compatible order wavelet `W`, and:

\[
0\le r<|M_0|_1
\]

the algorithm returns:

\[
\operatorname{sort}_{(K,id)}
\left(
\{d_i\mid M_0[i]=1\}
\right)[r]
\]

No result before ordinal `r` is enumerated.

## 12. Page enumeration

### 12.1 Prohibited implementation

A page of size `l` MUST NOT invoke a fresh root-to-leaf selection independently
for every rank when an iterator can reuse traversal state.

### 12.2 Ordered match iterator

After locating the first answer, maintain:

```text
OrderWaveletIterator {
  read_view_id
  order_wavelet_id
  predicate_rank_map_id
  traversal_stack
  current_leaf
  current_leaf_match_position
  next one-based rank
}
```

Within one leaf, enumerate subsequent set bits of the leaf candidate bitmap.
When exhausted, traverse to the next leaf whose conditioned match count is
nonzero, reusing ancestor projections or certified cached node counts.

### 12.3 Page semantics

For starting rank `k` and page size `l`, output remains:

\[
[
A[k-1],
\ldots,
A[\min(k-1+l,m)-1]
]
\]

where `A` is the complete DQL-ordered match sequence.

### 12.4 Continuation

The external continuation remains `dingo-direct-cursor-v1`. It binds:

- order-wavelet generation;
- order dictionary;
- match/rank map;
- frozen read view;
- next rank;
- traversal-state hash or a reproducible rank location;
- rank domain and coverage.

A server MAY reconstruct traversal from next rank rather than serializing an
implementation-specific stack.

## 13. Composite order

### 13.1 Tuple symbols

Multiple DQL order terms form one user tuple:

\[
K(d)=(K_1(d),\ldots,K_t(d))
\]

The dictionary compares tuples lexicographically using each term's declared
direction and Null/Absent rules.

The tuple, not each field independently, receives dense symbol:

\[
\rho_V(K(d))
\]

This prevents incorrect composition of independent per-column ranks.

### 13.2 Why independent ranks are insufficient

In general:

\[
(\operatorname{rank}_A(a),\operatorname{rank}_B(b))
\]

does not by itself provide a compact one-dimensional symbol unless the pair is
ordered lexicographically and encoded under one proved order embedding.

DOW therefore uses:

- one composite tuple dictionary;
- a nested wavelet/trie with equivalent lexicographic semantics; or
- a separately certified composite order index.

### 13.3 Prefix-compatible indexes

An order wavelet built for:

```text
(country asc, city asc, price desc)
```

may directly support an order using the same leading terms only if its
dictionary and leaf stability prove that the omitted suffix does not disturb
the required document-ID tie order.

This is generally false when omitted terms precede document ID in the built
order. The planner MUST prove compatibility rather than assume SQL-style
index-prefix folklore.

## 14. Strings and variable-length values

### 14.1 Semantic options

Strings and bytes may be handled by:

1. the finite-view order dictionary in §6;
2. a wavelet trie over a proved lexicographic symbol encoding;
3. another exact order-preserving dictionary.

### 14.2 Wavelet trie

A wavelet trie combines:

- a Patricia/radix trie over distinct strings or tuple encodings;
- node bitmaps recording which branch each source-order occurrence follows;
- rank/select structures over those bitmaps.

Conditioned branch selection uses the same equations as §9–§11.

### 14.3 Terminators

Any direct string encoding MUST distinguish a string from its proper
extension:

\[
"a" < "aa"
\]

under code-point lexicographic order.

A terminator must compare below every continuation symbol, be unambiguous, and
be escaped or structurally separate. C-style zero termination without an
escape proof is not sufficient for arbitrary bytes.

### 14.4 Collation

The DQL v1 string order is Unicode scalar/code-point lexicographic order.
Locale collation, normalization, case folding, and ICU version behavior are
not silently imported.

A future collation profile creates a distinct order dictionary and wavelet
generation.

## 15. Immutable blocks

### 15.1 Block shape

An order-wavelet block contains:

```text
OrderWaveletBlockV1 {
  profile: "dingo-order-wavelet-block-v1"
  heap_id
  collection_id
  block_id
  source_id_lower
  source_id_upper
  source_count
  source_revision_manifest
  read_frontier
  dictionary_id
  dictionary_generation
  tuple_paths_and_order_profile
  wavelet_shape
  branch_bitmaps
  rank_select_directories
  leaf/source mappings
  coverage
  checksum_tree
}
```

### 15.2 Source interval

Blocks SHOULD cover disjoint contiguous source document-ID intervals:

\[
I_0<I_1<\cdots<I_{b-1}
\]

This makes concatenating equal-value leaf occurrences across block order
equivalent to global document-ID tie order.

If physical source partitions do not follow document-ID ranges, the derived
order blocks may reference documents from multiple physical partitions.

### 15.3 Independent survival

Every block MUST:

- be self-delimiting;
- authenticate or checksum its complete derived content;
- identify its source revisions;
- be examinable without adjacent blocks;
- fail locally when damaged;
- be rebuildable from surviving authority.

Loss of one order block MUST NOT make another intact block undecodable.

### 15.4 Representation

The semantic tree MAY be encoded as:

- pointer-based or succinct wavelet tree;
- levelwise wavelet matrix;
- multiary wavelet matrix;
- wavelet trie;
- hybrid selected by immutable block shape.

The representation is conforming only if it produces identical conditioned
counts, selections, and inverse mappings.

## 16. Multiple blocks with one dictionary

Assume blocks:

\[
W_0,\ldots,W_{b-1}
\]

share one compatible dictionary and bit width.

Each block has exact predicate bitmap:

\[
M_s
\]

At a common wavelet prefix `v`, define global zero count:

\[
Z(v)
=
\sum_{s=0}^{b-1}
\left|
C_{s,v}\cap\neg B_{s,v}
\right|_1
\]

If:

\[
r<Z(v)
\]

descend every block into zero. Otherwise:

\[
r:=r-Z(v)
\]

and descend every block into one.

At the selected value leaf, blocks are visited in ascending source-ID interval
order. Their selected leaf counts form a prefix directory. Local select then
returns the globally correct document-ID tie.

This algorithm performs one common symbol decision per depth, independent of
the requested rank.

## 17. Wavelet forests with different dictionaries

### 17.1 Need

Immutable base blocks and append deltas may have distinct local dictionaries.
Their numeric symbols MUST NOT be compared directly.

### 17.2 Global value directory

Let:

\[
G=(g_0,\ldots,g_{\gamma-1})
\]

be the exact sorted union of distinct user tuples represented by all covered
forest members.

`G` binds:

- every participating dictionary/generation;
- the exact DQL comparator;
- source and index frontiers;
- coverage and holes.

### 17.3 Conditioned cumulative count

For forest member `s`, define:

\[
\operatorname{CountLE}_s(x)
=
\left|
\{d\in W_s\mid P(d)=1\land K(d)\le_K x\}
\right|
\]

The global cumulative count is:

\[
C(x)
=
\sum_s \operatorname{CountLE}_s(x)
\]

`C` is monotone:

\[
x\le_K y
\implies
C(x)\le C(y)
\]

### 17.4 Global value selection

The user tuple of zero-based rank `r` is:

\[
x^*
=
\min
\{x\in G\mid C(x)>r\}
\]

Binary search over `G`, or an equivalent count-guided tree, finds `x*` without
enumerating the first `r` matches.

The rank within the equal-value class is:

\[
r_{\mathrm{tie}}
=
r-C_{<}(x^*)
\]

where:

\[
C_{<}(x^*)
=
\sum_s
\left|
\{d\in W_s\mid P(d)=1\land K(d)<_K x^*\}
\right|
\]

Select `r_tie` across equal-value leaf postings in global document-ID order.

### 17.5 Member count algorithm

`CountLE_s(x)` is evaluated through the member's wavelet:

- map `x` to the member dictionary's lower/upper-bound symbol;
- navigate symbol-prefix branches;
- add conditioned counts for branches wholly below the boundary;
- descend only the boundary branch.

No matching document bodies are decoded.

### 17.6 Preferred compaction

The global-directory forest algorithm is exact but may require:

\[
O(f\log\gamma)
\]

conditioned member-count operations for `f` forest members.

Background compaction SHOULD periodically rebuild hot order indexes into a
shared dictionary/generation so §16's single descent applies. A small append
delta may remain a separate forest member.

## 18. Distributed execution

### 18.1 Logical arrangement

Physical data partitions and derived order-wavelet blocks are independent
partitionings.

A distributed order index is organized by:

- Heap;
- collection;
- order definition;
- source-ID intervals;
- dictionary/generation;
- frozen read view.

### 18.2 Coordinator algorithm

The coordinator:

1. binds the frozen distributed read view;
2. obtains an exact predicate bitmap/circuit for every covered order block;
3. verifies compatible order profiles and dictionaries;
4. uses shared-dictionary descent or global-directory selection;
5. locates the result leaf and source-ID interval;
6. fetches only requested authoritative rows;
7. returns coverage and a DDA cursor.

Worker completion order never affects branch counts or output order.

### 18.3 Count reduction

Branch counts use exact integer addition:

\[
Z=\sum_s Z_s
\]

Overflow is forbidden. Counts use a profile capable of representing every
admitted document cardinality; v1 APIs expose unsigned 64-bit counts and
refuse a universe exceeding:

\[
2^{64}-1
\]

documents.

### 18.4 Coordinator replacement

A replacement coordinator reconstructs selection from:

- canonical plan and parameters;
- frozen read view;
- order-wavelet and dictionary generations;
- predicate rank-map identities;
- next result rank;
- rank domain and coverage.

It does not rely on the prior coordinator's heap memory.

## 19. Mutation and lifecycle

### 19.1 Derived authority

Order dictionaries, wavelets, rank/select directories, global value
directories, and traversal caches are derived.

Their deletion cannot delete or reinterpret authoritative documents.

### 19.2 Write obligations

A committed mutation records applicable order-index obligations:

```text
document revision R
must be observed by
order index definition O generation-compatible successor
```

`consistency current` waits for the required order and predicate structures to
cover the bound authoritative frontier.

### 19.3 Base plus delta

The recommended mutable shape is:

```text
immutable compacted order-wavelet base
+ small immutable append wavelets
+ exact liveness/tombstone bitmaps
+ background merge
```

Deletes do not remove bits in-place from old immutable structures. The frozen
view's exact live bitmap intersects predicate membership before selection.

### 19.4 Dictionary change

Adding a previously unseen sort tuple may:

- enter a delta dictionary;
- enter a persistent wavelet trie;
- trigger a new shared generation.

It MUST NOT renumber a dictionary still referenced by a live read view.

### 19.5 Compaction

Compaction atomically publishes:

- new source coverage;
- new dictionary/generation;
- rebuilt order blocks;
- translation/replacement evidence;
- retirement eligibility for old blocks.

Old blocks remain readable until dependent views expire or migrate through a
proved equivalent replacement.

## 20. Damage and coverage

### 20.1 Exact count dependency

If an unavailable order-wavelet block may contain a matching document, its
conditioned branch contribution is unknown.

For a branch count:

\[
Z=Z_{\mathrm{known}}+Z_{\mathrm{missing}}
\]

if `Z_missing` is unknown, the branch decision may change. Exact
complete-domain selection is therefore impossible.

### 20.2 Strict behavior

Under complete coverage, a missing or invalid:

- source interval;
- exact predicate bitmap;
- order dictionary portion;
- wavelet branch bitmap;
- rank/select directory;
- leaf/source mapping;
- global value-directory range;

fails direct ordered access whenever it can affect membership or order.

### 20.3 Survivors domain

Survivors execution constructs a new explicit universe from covered blocks
and recomputes conditioned counts there.

It MUST state:

```text
rank_domain: survivors
missing source/order intervals
dictionary coverage
predicate coverage
```

It never reuses a complete-domain cursor as though ranks were unchanged.

### 20.4 Local recovery

An intact block remains examinable and rebuildable even when:

- adjacent order blocks are lost;
- the global value directory is lost;
- another dictionary generation is corrupt;
- the selection cache is absent.

Loss may increase cost or prevent a complete rank claim; it does not erase
surviving authority.

## 21. Planner contract

### 21.1 DOW eligibility

A DQL plan may use DOW only when:

\[
\mathrm{ExactPredicateBitmap}
\land
\mathrm{CompatibleOrderWavelet}
\land
\mathrm{ExactDictionary}
\land
\mathrm{StableTieOrder}
\land
\mathrm{FrozenReadView}
\land
\mathrm{RequiredCoverage}
\]

### 21.2 Direct classification

A DOW plan is `DIRECT` under DDA when:

- predicate bitmap construction uses exact admitted indexes;
- order-wavelet structures are already published;
- no authoritative prefix scan is required;
- no query-time comparison sort is required;
- all proof obligations are discharged.

It may be:

```text
DIRECT / READY
```

when node-local conditioned maps/counts are cached or materialized, or:

```text
DIRECT / COMPOSED
```

when exact bitmap projection/intersection is performed at query setup.

### 21.3 Build classification

If DingoDB must scan authoritative documents or comparison-sort a new tuple
dictionary/result mapping, the plan is `BUILDABLE`, not `DIRECT`, until the
artifact is verified and atomically published.

### 21.4 Refusal

The planner refuses direct DOW when:

- the order expression is unsupported;
- a predicate index is only approximate and verification exceeds policy;
- the order index has incompatible semantics;
- dictionary or coverage is incomplete;
- frozen-view retention cannot be guaranteed;
- the direct work exceeds budget.

It may offer `access build` or explicit `access sequential`.

DOW is not mandatory for every sortable field. For high-cardinality,
rarely-sorted, or write-dominant fields, its `n·h` pre-compression bit budget
may cost more than its benefit. The planner/index advisor compares it with a
composite index, ordinary ordered scan, or query-specific build and discloses
the choice.

### 21.5 Security and isolation

Authorization is checked before dictionary lookup, conditioned counting, rank
disclosure, or artifact reuse.

Every durable or cached key includes at least:

```text
heap_id
collection_id
order_definition_id/version
dictionary/wavelet generation
read_view
predicate/rank-map identity where query-specific
```

Two Heaps MUST NOT share logical:

- order dictionaries;
- branch bitmaps;
- value-frequency or conditioned-count statistics;
- global value directories;
- traversal or selected-result caches.

Cross-Heap statistics can disclose value presence and cardinality even when no
foreign document is returned. Explain and exact total counts therefore require
the corresponding query/explain capability.

Physical block deduplication, if supported, occurs below Heap-scoped identity,
encryption, authorization, and cache namespaces. It cannot permit one Heap to
address or infer another Heap's order structure.

Encryption at rest and key lifecycle follow Heap/database doctrine. Derived
order indexes receive the same or stronger confidentiality classification as
the indexed fields.

### 21.6 Resource budgets

DOW admission accounts separately for:

```text
dictionary bytes read
branch bitmap bytes read
rank/select directory bytes read
conditioned projection CPU
temporary candidate bitmap bytes
forest members and global-directory probes
archive blocks staged
authoritative result rows fetched
```

A `DIRECT` plan may fail its resource budget even though its work is
independent of rank. It MUST report `dow_budget_exhausted`; it MUST NOT
silently switch to comparison sort or sequential prefix enumeration.

Long bitmap, forest, build, and archive operations MUST check cooperative
cancellation at bounded intervals. A cancelled partial count or projection is
never cached or certified as complete.

## 22. Explain and API

### 22.1 Structured explain

```text
order_wavelet {
  profile: "dingo-order-wavelet-v1"
  order_definition
  order_dictionary_ids
  order_wavelet_blocks
  source_id_intervals
  predicate_bitmap_plan
  execution: SharedDictionary | ForestDirectory
  direct_setup: Ready | Composed
  symbol_count
  height_or_trie_depth
  estimated_branch_bitmap_bytes
  estimated_conditioned_operations
  global_value_directory?
  tie_order: ImmutableDocumentId
  read_view_id
  coverage
  known_holes
  proof_obligations
}
```

### 22.2 Human explain

Human explain MUST answer:

1. Which structure replaces query-time sorting?
2. Is the predicate bitmap exact?
3. How many order-wavelet blocks and generations participate?
4. Does selection use shared descent or forest value search?
5. Is positioning cost independent of requested rank?
6. What build, bitmap, archive, and row-fetch work remains?

### 22.3 Rust surface

No separate user API is required. The ordinary DDA request:

```rust
let page = products
    .query()
    .where_eq("category", "book")
    .order_by("price", SortOrder::Asc)
    .page_size(100)
    .at_rank(100_001)?
    .access(AccessPolicy::Direct)
    .page()?;
```

may select DOW automatically.

Index administration SHOULD expose:

```rust
products.indexes().create_order_wavelet(
    "by-price",
    [OrderTerm::asc("price")],
)?;
```

The final API may use the generic index-definition builder; this example fixes
intent, not Rust naming.

## 23. Complexity

Let:

- `n` be source documents;
- `m` be predicate matches;
- `σ` be distinct user sort tuples;
- `h = ceil(log₂ σ)` for a balanced binary wavelet;
- `f` be forest members;
- `l` be page size;
- `C_j` be the cost of one compressed conditioned-count/projection operation
  at level `j`;
- `D(l)` be authoritative fetch/verification cost for returned rows.

### 23.1 Shared dictionary

One exact selection has:

\[
T_{\mathrm{select}}
=
O\left(
\sum_{j=0}^{h-1} C_j
+h
\right)
\]

The `h` term covers rank/select navigation and inverse mapping.

Critically:

\[
T_{\mathrm{select}}(k)
\notin\Theta(k)
\]

### 23.2 Ready node-conditioned state

If relevant conditioned branch counts and child projections are already
materialized:

\[
T_{\mathrm{select}}=O(h)
\]

plus bounded local bitmap select.

### 23.3 Page

With traversal reuse:

\[
T_{\mathrm{page}}
=
T_{\mathrm{first}}
+O(v+l)
+D(l)
\]

where `v` is visited nonempty traversal nodes after the first result.

It MUST NOT be implemented as:

\[
O(l\cdot T_{\mathrm{select}})
\]

when a reusable iterator is available.

### 23.4 Forest directory

Binary search over `γ` global distinct values has:

\[
T_{\mathrm{forest-select}}
=
O\left(
\log\gamma
\cdot
\sum_{s=0}^{f-1}
T_{\mathrm{CountLE},s}
\right)
\]

This is independent of `k` but may be more expensive than shared descent.

### 23.5 Construction

Given dictionary symbols, a balanced static wavelet has:

\[
O(nh)
\]

bit construction work in a simple implementation and stores:

\[
nh
\]

raw branch bits before compression and rank/select overhead.

Dictionary construction may require:

\[
O(n\log n)
\]

comparison work in the general case. Radix, trie, already ordered, or
incremental construction may improve this for supported encodings.

Construction cost is background/build cost, never hidden in a `DIRECT`
positioning metric.

## 24. Formal safety properties

### 24.1 Order-embedding correctness

\[
\forall x,y\in\mathcal A_V:
x<_K y
\iff
\rho_V(x)<\rho_V(y)
\]

### 24.2 Predicate transport correctness

For every node-local position:

\[
C_v[j]=1
\iff
P(document(v,j))=1
\]

### 24.3 Conditioned-count correctness

\[
c_b(v)
=
\left|
\{d\in v\mid P(d)=1\land d\text{ takes branch }b\}
\right|
\]

### 24.4 Ordered-selection correctness

\[
\operatorname{DOWSelect}(P,K,r)
=
\operatorname{sort}_{(K,id)}
(\{d\mid P(d)\})[r]
\]

### 24.5 Tie stability

\[
K(d_a)=K(d_b)
\land
id(d_a)<id(d_b)
\implies
d_a<_{\mathrm{DOW}}d_b
\]

### 24.6 Representation equivalence

Every physical tree, matrix, multiary matrix, or trie representation must
produce the same abstract:

- conditioned branch counts;
- selected tuple;
- tie rank;
- document identity;
- page sequence.

### 24.7 Heap noninterference

No dictionary entry, wavelet bit, branch count, predicate bitmap, cache,
global directory, or returned identity from Heap `H₂` may influence a query
bound to distinct Heap `H₁`.

## 25. Conformance

### 25.1 Oracle

The reference oracle:

1. freezes the same live document view;
2. evaluates `P` authoritatively;
3. comparison-sorts matches by exact DQL `(K,id)`;
4. returns the requested rank/page.

Every DOW representation and execution mode MUST equal it.

### 25.2 Dictionary

Test:

- empty and one-symbol dictionaries;
- every scalar family;
- exact integer/decimal cross-family order;
- Null versus Absent;
- ascending and descending;
- duplicate values;
- Unicode code-point strings;
- arbitrary bytes including zero;
- multi-field tuples;
- dictionary encode/decode and corruption.

### 25.3 Wavelet laws

For every node:

- stable partition;
- rank/child mapping;
- select inverse mapping;
- zero/one cardinality partition;
- equal-symbol source-order preservation;
- tree/matrix representation equivalence.

### 25.4 Conditioned selection

Generate arbitrary exact match bitmaps:

- empty;
- singleton;
- all;
- alternating;
- sparse;
- dense;
- clustered;
- independent and highly correlated with sort values.

For every valid and past-end rank, compare DOW with the oracle.

### 25.5 Pages

- every page size boundary;
- pages crossing leaves and blocks;
- large equal-value runs;
- no duplicates or omissions;
- continuation and coordinator restart;
- traversal reconstruction from next rank.

### 25.6 Forests and deltas

- one shared dictionary;
- different dictionaries;
- unseen delta values below, inside, and above base range;
- many small generations;
- global value-directory binary selection;
- compaction equivalence;
- pinned old read views.

### 25.7 Cluster

- physical hash partitions unrelated to source-ID blocks;
- worker reorderings;
- coordinator replacement;
- block movement;
- count overflow boundary;
- unavailable partition or order block;
- complete versus survivors rank.

### 25.8 Damage

Corrupt or remove independently:

- dictionary entries;
- branch bitmap containers;
- rank/select directories;
- leaf mappings;
- global value directory;
- predicate bitmap;
- authoritative result row.

No affected complete rank may be issued. Intact blocks remain examinable.

### 25.9 Performance assertions

Instrumentation MUST prove:

- authoritative rows examined before the target page do not grow with `k` in
  a direct plan;
- requested rank is not used as a loop bound;
- query-time comparison sort is absent;
- bitmap and wavelet work is reported separately;
- cold build is not reported as warm direct selection.

## 26. Stable errors

```text
dow_order_unsupported
dow_order_profile_mismatch
dow_dictionary_missing
dow_dictionary_corrupt
dow_dictionary_incompatible
dow_symbol_out_of_range
dow_wavelet_missing
dow_wavelet_corrupt
dow_rank_select_invalid
dow_predicate_bitmap_inexact
dow_predicate_alignment_mismatch
dow_source_interval_overlap
dow_source_interval_gap
dow_global_directory_missing
dow_global_directory_incomplete
dow_count_overflow
dow_read_view_mismatch
dow_coverage_incomplete
dow_rank_domain_mismatch
dow_budget_exhausted
dow_profile_unsupported
```

Errors include safe identifiers, the failed proof obligation, affected source
or order ranges, and remediation. They exclude secret values and cross-Heap
statistics.

## 27. SDA examination

SDA MUST project every surviving:

- order-index definition;
- exact DQL order profile;
- dictionary header and bounded entries;
- source-ID interval;
- wavelet shape;
- branch-bitmap descriptor;
- rank/select-directory descriptor;
- leaf/source mapping;
- global value-directory node;
- origin/source revision;
- frontier, coverage, checksum, and hole.

An examiner can verify locally:

\[
|S_{v0}|+|S_{v1}|=|S_v|
\]

\[
\operatorname{rank}_0(B_v,|B_v|)
+\operatorname{rank}_1(B_v,|B_v|)
=|B_v|
\]

and sampled/full rank-select inverse laws without unrelated blocks.

Recovered order blocks remain derived evidence. They never replace missing
authoritative documents.

## 28. Implementation sequence

### DOW-0 — mathematical reference

- plain bit-vector rank/select;
- explicit semantic wavelet tree;
- exact arbitrary-bitmap descent;
- comparison-sort oracle;
- exhaustive small-model tests.

Exit: every finite model up to the configured exhaustive bound agrees with the
oracle for every match bitmap and rank.

### DOW-1 — immutable natural implementation

- order dictionary;
- fixed-width scalar symbols;
- immutable blocks;
- rank/select sidecars;
- inverse document mapping;
- SDA examination.

Exit: one embedded block supports exact filtered `at rank`.

### DOW-2 — compressed physical profiles

- levelwise wavelet matrix;
- compressed bitmap containers;
- SIMD conditioned counts/projections;
- representation-equivalence corpus.

Exit: physical optimization changes cost only.

### DOW-3 — pages and selection caches

- reusable ordered iterator;
- next-leaf navigation;
- node-conditioned cache;
- DDA continuation integration;
- quotas and cancellation.

Exit: long pages avoid repeated root selection and remain bounded.

### DOW-4 — forest and mutation

- base plus delta dictionaries;
- global value directory;
- `CountLE`;
- forest selection;
- compaction and pinned views.

Exit: writes and unseen values do not require mutating live immutable
generations.

### DOW-5 — distributed execution

- global source-ID interval layout;
- worker count reductions;
- coordinator replacement;
- damage/coverage;
- multi-process fault tests.

Exit: clustered DOW equals the oracle under every admitted complete view and
remains honest under holes.

## 29. Release gates

DOW v1 is release-ready only when:

1. DQL scalar and tuple comparison profiles are frozen;
2. dictionary bytes and identities are canonical;
3. every physical representation passes the semantic-tree oracle;
4. arbitrary predicate-bitmap transport is proved and exhaustively tested on
   small models;
5. equal values always use immutable document-ID order;
6. shared and forest execution agree;
7. no direct path comparison-sorts matches;
8. no direct path loops to requested rank;
9. source revisions needed by a frozen view remain retrievable;
10. complete rank fails through any relevant hole;
11. survivors rank uses a newly identified universe;
12. Heap-scoped indexes, caches, and counts cannot collide structurally;
13. SDA independently examines blocks and damage;
14. cold-build, composed-select, ready-select, and fetch benchmarks are
    reported separately.

## 30. Research basis

This section is informative.

DOW builds on:

- wavelet trees for rank/select-backed range quantile and ordered reporting:
  [Range Quantile Queries: Another Virtue of Wavelet
  Trees](https://arxiv.org/abs/0903.4726);
- wavelet-tree range next-value, intersection, and information-retrieval
  algorithms:
  [New Algorithms on Wavelet Trees and Applications to Information
  Retrieval](https://arxiv.org/abs/1011.4532);
- the wavelet matrix as a levelwise representation suitable for large
  alphabets:
  [The Wavelet Matrix: An Efficient Wavelet Tree for Large
  Alphabets](https://repositorio.uchile.cl/handle/2250/133661);
- wavelet tries for compressed indexed sequences of strings and dynamic
  alphabets:
  [The Wavelet Trie: Maintaining an Indexed Sequence of Strings in Compressed
  Space](https://arxiv.org/abs/1204.3581);
- compact practical rank/select structures:
  [Engineering Compact Data Structures for Rank and Select Queries on Bit
  Vectors](https://arxiv.org/abs/2206.01149).

The specific Dingo composition is:

- exact DQL predicate bitmap as an arbitrary conditioned subset;
- stable wavelet projection of that subset;
- document-ID tie preservation;
- immutable damage-localized order blocks;
- shared-generation and forest execution;
- DDA certificates, frozen views, Heap isolation, and survivors-domain
  honesty.

That composition is the Dingo Order Wavelet proposition.
