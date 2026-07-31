# Residiuum Direct Access (DDA) specification

Status: **Normative design v1.0-draft; not yet implemented**

Profiles:

```text
dingo-direct-access-v1
dingo-rank-map-v1
dingo-selection-artifact-v1
dingo-direct-cursor-v1
```

Audience: storage, index, RQL, SDK, server, cluster, examination, security,
and conformance implementers

Normative companions:
[RQL_SPEC.md](../../wip/query/RQL_SPEC.md),
[ORDER_WAVELET_SPEC.md](../order-wavelets/ORDER_WAVELET_SPEC.md),
[RESIDIUUM_PREDICATE_SPEC.md](../../reference/query/RESIDIUUM_PREDICATE_SPEC.md),
[COLLECTION_CONTRACT_SPEC.md](../rre/COLLECTION_CONTRACT_SPEC.md),
[HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md),
[CLUSTER_SPEC.md](../cluster/CLUSTER_SPEC.md),
[SDA_PROFILE.md](../../reference/query/SDA_PROFILE.md),
[INDEXING_STRATEGY_PROPOSAL.md](../../done/implementation/INDEXING_STRATEGY_PROPOSAL.md), and
[DX_SPEC.md](../../reference/product/DX_SPEC.md)

Implementation plan:
[doc/todo/direct-access/DIRECT_ACCESS_IMPLEMENTATION_PLAN.md](./DIRECT_ACCESS_IMPLEMENTATION_PLAN.md)

## 1. Decision

Residiuum SHALL support **direct access to ranked query answers**.

Given:

- one authenticated Heap;
- a frozen read view;
- a total RQL predicate;
- a strict deterministic result order;
- a one-based result rank `k`; and
- a requested page size `l`;

Residiuum may return results `k` through `k + l - 1` without enumerating the
preceding `k - 1` matching documents when the admitted physical plan possesses
enough exact counting information.

The governing product rule is:

> A numeric position is a rank query, not permission to walk past everything
> before it.

An implementation MUST NOT implement a direct-access request by silently
scanning, decoding, filtering, sorting, or discarding work proportional to the
requested rank.

When exact direct access is unavailable, Residiuum MUST do one of:

1. build an exact derived selection artifact under an explicit build policy
   and resource budget;
2. execute an explicitly requested sequential compatibility plan;
3. refuse the request with a stable reason.

It MUST NOT disguise option 2 as direct access.

## 2. Requirement language

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, MAY, and OPTIONAL are normative.

An implementation conforms to this specification only for the access classes
and index families it declares and passes in the conformance suite.

## 3. Scope

Version 1 specifies:

- exact ranked access over one root RQL result sequence;
- filtered and deterministically ordered document results;
- natural immutable-document-key order;
- declared scalar secondary orders;
- conditioned order-wavelet navigation for indexed scalar orders;
- immutable rank blocks and segment-local rank/select structures;
- exact bitmap predicate algebra;
- query-specific selection artifacts;
- frozen read views across pages;
- authenticated direct cursors;
- Heap isolation;
- distributed rank directories;
- damage and incomplete-coverage semantics;
- resource admission, explanation, errors, and conformance.

Version 1 does not promise direct access for:

- arbitrary Turing-complete predicates;
- opaque callbacks or user code;
- approximate vector ranking;
- ranking functions without a frozen deterministic specification;
- arbitrary computed sort expressions;
- unbounded recursive traversal;
- every possible RQL enrichment shape;
- every possible Boolean predicate without an applicable index or build;
- exact global rank through an unknown data hole.

Text, vector, and spatial retrieval may later use DDA when their dedicated
specifications define exact membership and a strict deterministic order.
Approximate-nearest-neighbour ranking is not exact DDA.

## 4. Terms

**Direct access**
: Retrieval by result rank using exact counting and selection structures whose
  positioning work is not proportional to the requested rank.

**Rank**
: A one-based product-facing position in a result sequence. The mathematical
  kernel uses the corresponding zero-based ordinal.

**Rank/select**
: Operations over a bit sequence that count set bits before a position and
  locate the position of a specified set bit.

**Order domain**
: The immutable sequence of document identities induced by one canonical
  strict total order in one frozen read view.

**Ordinal**
: A zero-based position inside an order domain or rank block.

**Rank block**
: An immutable contiguous interval of an order domain with independently
  verifiable membership and rank/select metadata.

**Predicate bitmap**
: A bit sequence aligned to an order domain in which bit `i` is one exactly
  when the document at ordinal `i` satisfies the canonical predicate.

**Rank map**
: The exact predicate bitmap, its block cardinalities, prefix-count directory,
  order-domain identity, and verification evidence.

**Selection artifact**
: A derived, immutable, Heap-local rank map built for a canonical query,
  parameter binding, read view, and order.

**Direct-access certificate**
: Machine-readable evidence that a physical plan's membership, ordering,
  count, coverage, and read-view obligations are sufficient for exact ranked
  access.

**Frozen read view**
: An immutable description of the authoritative and derived frontiers,
  manifests, generations, and coverage used by every access to one logical
  query result. It need not claim global linearizability.

**Candidate index**
: An index that yields a superset or approximation of possible matches and
  therefore requires authoritative predicate verification.

**Exact index**
: An index whose declared semantics prove exact membership for its supported
  predicate under its covered read view. “Exact” describes its logical
  membership semantics; it does not turn derived index bytes into
  authoritative document bytes.

**Rank domain**
: The universe relative to which ranks are stated: `complete` or `survivors`.

## 5. Logical model

### 5.1 Query universe

Let:

\[
H
\]

be one immutable Heap identity,

\[
V
\]

be one frozen read view, and

\[
U_{H,V}
\]

be the finite set of live root documents in the query's bound collections
that are logically visible in `V`.

Heap isolation requires:

\[
H_1 \ne H_2
\implies
U_{H_1,V_1} \cap U_{H_2,V_2} = \varnothing
\]

at the logical identity layer. Physical sharing or deduplication, if ever
implemented, MUST NOT weaken this disjoint logical universe.

### 5.2 Predicate

Let:

\[
P: U_{H,V} \rightarrow \{0,1\}
\]

be the normalized root predicate under `dingo-predicate-v1`.

Residiuum predicates are total and two-valued. Null, absence, numeric comparison,
type mismatch, and path traversal therefore have the meanings fixed by
[RESIDIUUM_PREDICATE_SPEC.md](../../reference/query/RESIDIUUM_PREDICATE_SPEC.md); an index MUST NOT import
SQL three-valued truth accidentally.

### 5.3 Strict order

Let:

\[
\kappa(d) =
(k_1(d), k_2(d), \ldots, k_m(d), id(d))
\]

be the canonical sort tuple. `id(d)` is the immutable document identity added
as the final tie-breaker by RQL.

The comparison profile for every `k_i`, including Null and Absent placement,
is part of the order-domain identity. Since `id` is unique:

\[
d_1 \ne d_2 \implies \kappa(d_1) \ne \kappa(d_2)
\]

and lexicographic comparison of `κ` defines a strict total order.

Let:

\[
\pi_{H,V,\kappa}: [0,n) \rightarrow U_{H,V}
\]

be the unique bijection that enumerates documents in ascending `κ` order,
where:

\[
n = |U_{H,V}|
\]

and:

\[
i < j
\iff
\kappa(\pi(i)) <_{\mathrm{lex}} \kappa(\pi(j))
\]

### 5.4 Match bit vector

Define:

\[
B_{P,\pi}[i] =
\begin{cases}
1 & \text{if } P(\pi(i)) = 1 \\
0 & \text{otherwise}
\end{cases}
\]

for:

\[
0 \le i < n
\]

The number of answers is:

\[
m = \sum_{i=0}^{n-1} B_{P,\pi}[i]
\]

### 5.5 Rank and select

DDA uses exclusive rank:

\[
\operatorname{rank}_1(B,i)
=
\sum_{j=0}^{i-1} B[j]
\]

for:

\[
0 \le i \le n
\]

and zero-based select:

\[
\operatorname{select}_1(B,r)
=
\min \left\{
i \mid
B[i]=1
\land
\operatorname{rank}_1(B,i+1)=r+1
\right\}
\]

for:

\[
0 \le r < m
\]

The zero-based `r`th answer is:

\[
A_{P,\pi}[r]
=
\pi(\operatorname{select}_1(B_{P,\pi},r))
\]

The product-facing one-based rank `k` maps to:

\[
r = k - 1
\]

`k = 0` is invalid. `k > m` yields an empty complete page, not an error.

### 5.6 Page

For one-based starting rank `k` and page size `l`:

\[
\operatorname{Page}(P,\pi,k,l)
=
\left[
A[r]
\mid
r \in
[k-1,\min(k-1+l,m))
\right]
\]

This definition is the semantic oracle for indexed, built, clustered, and
sequential implementations.

## 6. The information requirement

### 6.1 No universal shortcut

An exact implementation cannot identify the `k`th match unless it has enough
information to distinguish how many matching documents precede candidate
positions.

For an arbitrary previously unseen predicate over unindexed opaque documents,
that information must be obtained by:

- prior indexing;
- query-time examination;
- a previously built exact selection artifact; or
- a stronger declared constraint that logically determines membership.

DDA does not repeal this information requirement.

An adversary argument makes the boundary precise. Assume an algorithm neither
examines document `d` nor reads any exact derived fact determining `P(d)`, and
`d` precedes its proposed answer. Construct two admissible states identical in
everything the algorithm observes:

\[
S_0: P(d)=0
\]

\[
S_1: P(d)=1
\]

The proposed answer's rank differs by one between `S₀` and `S₁`, but the
algorithm has the same observations and therefore returns the same answer.
It must be wrong in at least one state. Consequently, exact direct access
requires prior information, query-time examination, or a constraint that
eliminates the unobserved degree of freedom.

### 6.2 Prohibited complexity

For a request admitted as `DIRECT`, positioning cost MUST NOT be:

\[
\Omega(k)
\]

merely because the requested rank is `k`.

The plan may have:

- fixed admission cost;
- predicate-index algebra cost;
- rank-directory lookup cost;
- result fetch cost;
- cost proportional to the number or compressed size of relevant index
  blocks;

provided those costs are disclosed and are independent of the numeric rank
except for logarithmic addressing.

### 6.3 Access classifications

Every planned ranked query MUST be classified as exactly one of:

**DIRECT**
: Exact membership counts and order addressing are already available or can
  be derived solely from admitted exact indexes. No authoritative document
  scan and no prefix enumeration is required.

**BUILDABLE**
: Residiuum can construct an exact selection artifact under the supplied build
  policy and budget. Construction may examine authoritative documents, but
  subsequent ranked accesses use the artifact.

**SEQUENTIAL**
: Exact execution requires scan/filter/discard or enumeration proportional to
  the requested rank or input size. This class requires explicit caller
  permission for a numeric-rank request.

**REFUSED**
: No exact admitted plan exists under the query semantics, coverage,
  consistency, policy, or resource bounds.

Approximation is not a fifth exact access class. An approximate feature MUST
use a separately named retrieval contract.

`DIRECT` explain MUST further disclose one setup state:

```text
READY       rank map and prefix directory already published
COMPOSED    exact bitmap/index algebra must construct a transient rank map
```

`READY` has only rank-directory positioning and page-fetch cost. `COMPOSED`
has additional `Z(P)` index-algebra setup cost, but that cost remains
independent of the requested numeric rank.

## 7. Direct-access certificate

Before a query is labelled `DIRECT`, the planner MUST produce a
`DirectAccessCertificateV1` containing at least:

```text
DirectAccessCertificateV1 {
  profile: "dingo-direct-access-v1"
  heap_id
  canonical_plan_hash
  parameter_hash
  predicate_profile
  predicate_semantics_hash
  order_domain_id
  order_profile
  read_view_id
  rank_domain: Complete | Survivors
  membership_proof
  order_proof
  count_proof
  coverage_proof
  index_definitions
  index_generations
  rank_map_id
  rank_map_hash
  source_frontiers
  index_frontiers
  known_holes
  complexity_bound
  expiry?
}
```

The certificate is derived evidence, not authority. It MAY be reconstructed.

Issuing a certificate asserts:

1. every set bit denotes one visible document satisfying `P`;
2. every covered visible document satisfying `P` contributes one set bit;
3. bitmap position follows the declared strict order;
4. block and global counts equal bitmap cardinality;
5. the rank domain and coverage claim are truthful;
6. all identities and profile versions are bound to one Heap and read view.

The membership and count proofs may rely on derived structures, but the
coverage proof must independently establish that their referenced
authoritative revisions were covered when the view was frozen. Returned
documents are still verified against authority.

Failure to discharge any assertion forbids `DIRECT`.

## 8. Immutable ordinal fabric

### 8.1 Stable source identity

Every indexed root document revision is represented by:

```text
DocumentOrigin {
  heap_id
  collection_id
  document_id
  document_revision
  authoritative_position
}
```

No ordinal is a document identity. An ordinal is meaningful only with its
order-domain and rank-block identities.

### 8.2 Rank blocks

An order domain is divided into immutable contiguous rank blocks:

\[
R_0, R_1, \ldots, R_{b-1}
\]

such that:

\[
\pi = R_0 \Vert R_1 \Vert \cdots \Vert R_{b-1}
\]

Each block stores or proves:

```text
RankBlockV1 {
  heap_id
  order_domain_id
  block_id
  order_lower_bound
  order_upper_bound
  length
  document_identity_mapping
  source_manifest
  checksum
  predecessor_hash?
  successor_hash?
}
```

Predecessor or successor hashes MAY assist examination but MUST NOT create a
global survival dependency. Loss of one block MUST NOT make an intact
nonadjacent block undecodable.

### 8.3 Physical alignment

Rank blocks MAY align with authoritative segments, derived index segments,
or independent micro-pages. The logical requirements are:

- independently verifiable boundaries;
- deterministic order;
- bounded local decoding;
- no cross-Heap membership;
- rebuildability from surviving authority;
- explicit source and index frontiers.

Hydra or Chimera MAY provide location and physical access. Neither name is a
semantic requirement of DDA.

## 9. Exact bitmap algebra

### 9.1 Atomic sets

For an indexed atomic predicate `a`, define:

\[
E(a) =
\{i \mid a(\pi(i)) = 1\}
\]

and its characteristic bitmap:

\[
\chi_{E(a)}[i] = 1 \iff i \in E(a)
\]

An exact atomic index MUST have the same semantics as authoritative
`dingo-predicate-v1` evaluation.

At minimum, index families MUST distinguish where relevant:

- path Absent;
- explicit Null;
- present non-null values;
- exact scalar type;
- exact numeric family semantics;
- sequence or product carrier;
- document revision and liveness.

### 9.2 Boolean compilation

Let `⟦P⟧` be the exact bitmap denotation of predicate `P`:

\[
\llbracket \mathrm{true} \rrbracket = U
\]

\[
\llbracket \mathrm{false} \rrbracket = \varnothing
\]

\[
\llbracket P \land Q \rrbracket
=
\llbracket P \rrbracket \cap \llbracket Q \rrbracket
\]

\[
\llbracket P \lor Q \rrbracket
=
\llbracket P \rrbracket \cup \llbracket Q \rrbracket
\]

\[
\llbracket \lnot P \rrbracket
=
U \setminus \llbracket P \rrbracket
\]

where `U` is the live-document bitmap for the exact read view and order
domain—not every ordinal ever allocated.

Intersection, union, difference, and complement operate only on bitmaps with
identical:

- Heap identity;
- order-domain identity;
- read-view identity or a proved compatible frontier;
- length and block boundaries;
- predicate and scalar semantic profiles.

An implementation MUST reject rather than coerce incompatible bitmaps.

### 9.3 Structural correctness

The compiler MUST establish:

\[
\forall i:
\llbracket P \rrbracket[i] = 1
\iff
P(\pi(i)) = 1
\]

by structural induction:

1. exactness is established for atomic predicates;
2. set intersection preserves conjunction;
3. set union preserves disjunction;
4. relative complement over the live universe preserves negation.

This proof obligation is part of index-family conformance.

### 9.4 Equality, membership, and presence

Equality and finite membership MAY use equality bitmaps:

\[
\llbracket x \in \{v_1,\ldots,v_t\} \rrbracket
=
\bigcup_{j=1}^{t}
\llbracket x=v_j \rrbracket
\]

Presence uses a dedicated exact presence bitmap or a proved union of exact
type/value partitions. Null MUST NOT be inferred from absence.

### 9.5 Ranges and high-cardinality values

Numeric or ordered-scalar ranges MAY use:

- bit-sliced indexes;
- range-encoded bitmaps;
- dictionary-coded value partitions;
- wavelet trees or wavelet matrices;
- exact ordered posting structures;
- another structure with an equivalent exact proof.

Approximate bins MAY prune candidates but MUST verify boundary bins before
creating an exact rank map.

String comparison MUST bind the exact RQL collation/version. A dictionary
whose order differs from RQL code-point order cannot certify RQL ordering.

For a non-negative fixed-width bit-sliced value with slice `X_i` denoting that
bit `i` is one, exact `x < c` may be constructed from most significant bit to
least significant bit:

```text
EQ := U
LT := ∅

for i from most-significant to least-significant:
  if c[i] = 1:
    LT := LT ∪ (EQ ∩ (U \ X[i]))
    EQ := EQ ∩ X[i]
  else:
    EQ := EQ ∩ (U \ X[i])
```

At each step, `LT` contains values already proved smaller at a more
significant bit, while `EQ` contains values equal to `c` on every processed
bit. Induction over bit positions proves that final `LT` is exactly `x < c`.
Signed integers, decimals, and heterogeneous numeric values require their
separately frozen monotone encodings; byte reinterpretation without such an
encoding is forbidden.

### 9.6 Candidate indexes

If:

\[
\mathrm{Matches}(P) \subseteq C
\]

then `C` is only a candidate superset.

Candidate intersection and union may remain useful for pruning, but candidate
membership is not an exact rank map. In particular:

\[
U \setminus C
\]

is generally a subset, not an exact implementation, of `¬P`.

Candidate results MUST be verified against authoritative values. The verified
outcome MAY be persisted as a selection artifact and then certified.

### 9.7 Nested and multivalued paths

An index over arrays, bags, maps, or nested products MUST preserve the carrier
and same-element semantics required by the RQL predicate.

Flattened postings that cannot distinguish:

```text
one child satisfying A and B
```

from:

```text
one child satisfying A and another child satisfying B
```

MUST NOT certify a `within` predicate. They may only produce candidates.

## 10. Rank/select representation

### 10.1 Required operations

Every certified local predicate bitmap supports:

```text
len()          -> UInt
cardinality()  -> UInt
rank1(i)       -> UInt
select1(r)     -> Optional<Ordinal>
next1(i)       -> Optional<Ordinal>
```

and obeys:

\[
\operatorname{rank}_1(B,0)=0
\]

\[
\operatorname{rank}_1(B,n)=\operatorname{cardinality}(B)
\]

\[
\operatorname{rank}_1(B,\operatorname{select}_1(B,r)+1)=r+1
\]

\[
r_1 < r_2
\implies
\operatorname{select}_1(B,r_1)
<
\operatorname{select}_1(B,r_2)
\]

### 10.2 Representation freedom

Implementations MAY use:

- plain succinct bit vectors with rank directories;
- Roaring-style array, bitmap, and run containers;
- Elias–Fano encoded sparse positions;
- run-length structures;
- wavelet structures;
- hardware-specific SIMD representations;
- a deterministic hybrid chosen per immutable block.

The representation choice MUST NOT alter semantics or persisted profile
meaning.

### 10.3 Derived and disposable

Rank/select sidecars are derived:

- their loss MUST NOT destroy authoritative data;
- corruption MUST be detected before use;
- they MAY be rebuilt;
- a false set or clear bit MUST NOT be accepted as valid after verification
  failure;
- an index hole MUST enter query coverage.

## 11. Segmented global selection

For rank block `j`, let:

\[
B_j
\]

be its exact local predicate bitmap and:

\[
c_j = |B_j|_1
\]

be its cardinality.

For a composed predicate, `c_j` is the cardinality of the **combined** local
bitmap:

\[
c_j =
\left|
\llbracket P \rrbracket_j
\right|_1
\]

It generally cannot be inferred from marginal atomic counts. For example,
knowing `|A|` and `|B|` does not determine:

\[
|A \cap B|
\]

without correlation information. A `COMPOSED` plan must therefore evaluate
the exact local bitmap circuit, use a persisted joint structure, or use
another proof that determines the joint count. Guessing from selectivity
estimates cannot produce a direct-access certificate.

Define block prefix counts:

\[
F(0)=0
\]

\[
F(t)=\sum_{j=0}^{t-1} c_j
\quad
1 \le t \le b
\]

For requested zero-based result ordinal `r`, find the unique block `q` such
that:

\[
F(q) \le r < F(q+1)
\]

Then:

\[
r_{\mathrm{local}} = r - F(q)
\]

\[
i_{\mathrm{local}}
=
\operatorname{select}_1(B_q,r_{\mathrm{local}})
\]

and the answer is the document mapped by local ordinal `i_local` in block
`q`.

`F` MUST be represented by an exact prefix-count directory supporting bounded
or logarithmic predecessor search. A flat linear walk over all preceding
matching documents is forbidden. A bounded walk over a small fixed number of
top-level directory entries is permitted.

When `c_j` values are query-specific, a composed setup may evaluate all
relevant compressed index blocks and construct `F` in parallel. That setup
cost is disclosed as `Z(P)` and does not depend on `k`. Repeated access SHOULD
reuse the transient or persisted rank map instead of recomputing the circuit.

For a page, selection continues with `next1` inside the block and crosses to
later nonempty blocks using the directory. It MUST NOT restart selection from
the beginning for every row.

## 12. Ordering

### 12.1 Natural order

When RQL has no explicit order, the order domain is ascending immutable
document key as defined by RQL.

Natural-order direct access requires an ordered identity mapping. A point-only
MPHF that cannot enumerate order is insufficient by itself.

### 12.2 Explicit scalar order

An explicit order domain binds:

- every ordered field path;
- direction;
- scalar comparison profile;
- Null placement;
- Absent placement;
- collation/version;
- immutable document-key tie-breaker;
- index definition and generation;
- frozen read view.

Changing any component creates a different order-domain identity.

### 12.3 Ordered index

An order index stores or derives:

\[
\pi_{H,V,\kappa}
\]

and supports bidirectional mapping where required:

```text
order ordinal -> document identity
document identity -> order ordinal
```

A directly selected DDA predicate bitmap MUST be aligned to the selected order
domain. A bitmap aligned to document-key order cannot be interpreted as price
order merely because it contains the same number of bits.

The normative Residiuum structure for combining an exact source-order predicate
bitmap with a different scalar result order is
[ORDER_WAVELET_SPEC.md](../order-wavelets/ORDER_WAVELET_SPEC.md). It transports the predicate
bitmap through stable wavelet partitions, uses exact conditioned branch
counts to select the result tuple, and preserves immutable document-ID order
for ties. That transport is a proved coordinate transformation, not
reinterpretation of source-order bit positions as sorted-order positions.

### 12.4 Composite direct indexes

A composite direct index MAY jointly encode:

- an order;
- commonly filtered paths;
- exact per-block predicate partitions;
- rank/select metadata;
- subtree or block cardinalities.

For predicates matching its declared access shape, it can answer rank directly
without constructing a full query-specific bitmap.

### 12.5 Arbitrary ordering

If no exact order-addressable index exists, Residiuum may:

1. build a query-specific selection artifact in requested order;
2. use a proved direct-selection algorithm for the admitted query/order class;
3. classify the plan as sequential or refused.

A full sort followed by dropping `k - 1` rows is not `DIRECT`.

### 12.6 Enrichment ordering

If root ordering depends on enrichment output, the plan is direct only when a
certified cross-source access structure establishes exact membership,
cardinality, and order. Otherwise it is `BUILDABLE`, `SEQUENTIAL`, or
`REFUSED`.

Root enrichment that does not affect root membership or order MAY occur after
direct root selection only when moving it cannot suppress, duplicate, reorder,
or introduce an observable query-level failure for a skipped root.

An `exactly_one` or other fallible cardinality obligation must therefore be:

- discharged for the frozen view by an active RRE/referential-integrity or
  exact-index proof;
- incorporated into the exact selection artifact; or
- evaluated before direct selection, making the plan buildable/sequential.

Projection and enrichment after selection must be total for the selected
roots. The direct plan, build plan, and full RQL semantic oracle MUST expose
the same result or stable failure.

### 12.7 Tiered authority

Rank maps and global count directories MAY remain on a hot tier while returned
authoritative documents reside on warm, cold, archive, or offline media.
DDA therefore separates:

```text
position latency
source-stage latency
document-fetch latency
```

Locating rank 100,001 MUST NOT require staging the first 100,000 documents.
Only blocks required for exact index/count evidence and the requested result
page are staged.

If the rank is known but a returned authoritative block is offline, the page
waits or stages under its tier budget, or returns an explicit availability
error. It does not substitute derived index content for authority or claim a
complete materialized page.

## 13. Selection artifacts

### 13.1 Purpose

A `BUILDABLE` plan may produce an immutable query-specific selection artifact:

```text
SelectionArtifactV1 {
  profile: "dingo-selection-artifact-v1"
  artifact_id
  heap_id
  canonical_plan_hash
  parameter_hash
  predicate_semantics_hash
  order_domain_id
  read_view_id
  rank_domain
  block_manifest
  block_bitmaps_or_positions
  block_cardinalities
  prefix_count_directory
  total_cardinality
  source_frontiers
  index_frontiers
  coverage
  known_holes
  created_at
  expires_at?
  checksum_tree
}
```

### 13.2 Construction

Artifact construction:

1. freezes the read view;
2. enumerates the order domain or uses exact indexes;
3. evaluates the canonical predicate exactly;
4. writes independently verifiable blocks;
5. computes exact block cardinalities;
6. builds the prefix-count directory;
7. verifies source, order, and total-cardinality invariants;
8. publishes the artifact atomically;
9. only then issues a direct-access certificate.

A partially built artifact MUST NOT be published as complete.

### 13.3 Cost disclosure

Building an artifact may be:

\[
O(n)
\]

or more expensive when sorting or enrichment is required.

That cost is allowed only under `access build` and an explicit effective
budget. Explain and result metadata MUST distinguish:

```text
build_cost
position_cost
page_cost
```

The build cost is independent of the requested rank but is not free.

### 13.4 Reuse

An artifact may be reused only when all of these match:

- Heap;
- canonical plan;
- parameters;
- predicate semantic profiles;
- read view;
- order domain;
- coverage/rank domain;
- index and source compatibility.

Cache keys MUST include every item above.

### 13.5 Lifecycle

Selection artifacts are derived, Heap-local, quota-controlled, expirable, and
rebuildable. They MUST NOT:

- become the only copy of authoritative documents;
- outlive required security policy;
- leak result cardinality across Heaps;
- silently retarget a newer read view;
- keep storage pinned without an observable lease or policy.

## 14. Frozen read views and mutation

### 14.1 Read-view identity

A frozen read view contains:

```text
FrozenReadViewV1 {
  heap_id
  collection_bindings
  partition_frontiers
  authoritative_segment_manifests
  liveness/tombstone_frontiers
  index_definitions
  index_generations
  index_frontiers
  tier_manifest
  coverage
  created_at
  expires_at?
}
```

The view freezes the logical input to ranking across pages. It does not
necessarily claim that all partitions were observed at one linearizable wall
clock instant.

### 14.2 Snapshot stability

For a fixed `H`, `V`, `P`, and `κ`:

\[
B_{P,\pi}
\]

MUST remain logically immutable for the lifetime of direct access.

New writes after `V` do not enter the result. Updates or deletes after `V` do
not move or remove results from `V`.

The frozen-view lease MUST retain or make reconstructible every authoritative
document revision needed to fetch a later page. Retaining only old index bits
while garbage-collecting the corresponding document revision is invalid.

### 14.3 Immutable base plus delta

The recommended implementation is:

```text
immutable rank/select base blocks
+ immutable append deltas
+ versioned liveness/tombstone bitmap
+ periodic derived merge
```

The published read view binds one compatible set of these components.

### 14.4 Compaction

Compaction may rewrite physical rank blocks, but it MUST:

- retain the old manifest while a live view depends on it; or
- publish a verified ordinal translation preserving the order domain; or
- expire affected views and return a stable stale-view error.

It MUST NOT resume against a different order domain silently.

### 14.5 Available and current

`consistency available` freezes the currently published compatible source and
index frontiers and reports them.

`consistency current` first obtains the required authoritative frontier and
waits for every required exact derived structure to cover it. If this cannot
be established under the deadline, direct access fails.

## 15. Damage and coverage

### 15.1 Complete rank

An exact rank in the complete domain requires:

\[
\Omega = U_{H,V}
\]

where `Ω` is the set of documents whose membership and order are covered.

If a missing segment, partition, tier, liveness map, order block, or index
block could contain a matching document before or within the requested page,
complete rank is not known.

### 15.2 Unknown contribution

Let:

\[
M
\]

be a missing order interval. If Residiuum cannot prove:

\[
\sum_{i \in M} B[i] = 0
\]

then every later global rank may be shifted by an unknown amount.

An implementation MUST NOT call a surviving document “the 100,001st complete
result” merely because it is the 100,001st match that remains readable.

### 15.3 Strict behavior

Under:

```text
coverage complete
```

any hole capable of changing membership, count, or order fails with:

```text
dda_coverage_incomplete
```

The error includes safe bounded evidence identifying affected blocks,
partitions, tiers, and frontiers.

### 15.4 Survivors rank domain

Under:

```text
coverage allow incomplete
rank domain survivors
```

define:

\[
U' = \Omega
\]

and compute rank over surviving covered data:

\[
B'[i] = 1
\iff
\pi'(i) \in \Omega
\land
P(\pi'(i))
\]

Every page MUST state:

```text
rank_domain: survivors
coverage_complete: false
known_holes
```

`survivors` is a different mathematical universe, not a weaker spelling of
complete rank.

### 15.5 Rank intervals

An implementation MAY report a rank interval when it possesses proved lower
and upper bounds for missing contributions:

\[
r_{\min} \le r_{\mathrm{complete}} \le r_{\max}
\]

It MUST NOT collapse an interval to an exact rank unless:

\[
r_{\min}=r_{\max}
\]

Rank intervals are diagnostic evidence and do not satisfy an exact
`at rank` request unless the interval is singular.

### 15.6 Damaged derived structures

A damaged rank map may:

- be rebuilt from authoritative data;
- be bypassed by another exact structure;
- downgrade the plan to `BUILDABLE`;
- cause incomplete survivors-domain execution when explicitly allowed;
- fail.

It MUST NOT be trusted because its cardinality appears plausible.

### 15.7 Damage after view creation

Coverage may worsen after a read view is frozen. A lost or newly unreadable
authoritative revision referenced by the rank map:

- does not authorize returning unverified index data as the document;
- invalidates complete page production where the loss matters;
- remains an explicit hole;
- cannot be removed from the rank sequence without creating a new survivors
  read view and rank map.

The server MUST NOT continue an old complete-domain cursor under silently
recomputed survivors ranks. It returns a stable coverage/view error and may
offer the caller an explicit restart in the survivors domain.

## 16. Distributed direct access

### 16.1 Distributed read view

A distributed view binds:

- partition-directory generation;
- requested partition set;
- per-partition committed frontier;
- per-partition authoritative manifest;
- index partition/generation/frontier;
- tier availability;
- coverage.

A distributed frozen view provides repeatability across pages. It does not
claim global linearizability unless a separate snapshot protocol proves it.

### 16.2 Global order blocks

Distributed direct access MUST operate over globally ordered rank blocks.
Source partitions may be hash-partitioned; the derived order index need not
use the same partitioning.

Every global block states:

```text
global order interval
contributing source partitions
exact local match count
coverage
block hash
```

### 16.3 Global rank directory

Let globally ordered blocks have counts:

\[
c_0,\ldots,c_{b-1}
\]

The coordinator locates the result block through the prefix function `F`
defined in §11, then sends bounded local select/fetch requests.

The directory may be:

- a persisted composite-index count tree;
- a query-specific selection artifact;
- a deterministic merge of independently certified subdirectories.

It MUST NOT obtain `F(k)` by fetching and discarding every earlier result.

### 16.4 Coordinator independence

A direct cursor contains or references sufficient authenticated state for a
replacement coordinator to:

- recover the frozen read view;
- authenticate the rank map and global directory;
- resume at the next result rank;
- preserve coverage and holes;
- avoid duplicates and omissions.

Coordinator-local memory alone is insufficient unless the API explicitly
declares a stateful, non-resumable session profile.

### 16.5 Partial partitions

An unavailable partition whose match contribution is unknown prevents
complete-domain global rank.

The coordinator MUST NOT advance that partition's coverage frontier. It may
serve the survivors domain only when requested and must preserve the missing
partition in every subsequent page's evidence.

### 16.6 Parallelism

Predicate bitmap evaluation, block cardinality construction, and selection
artifact building SHOULD run in parallel across independent immutable blocks.

Parallel completion order MUST NOT affect:

- global ordering;
- bit positions;
- prefix counts;
- hashes;
- page contents;
- error ordering where stable ordering is specified.

## 17. RQL surface

### 17.1 Syntax

RQL adds three terminal clauses:

```ebnf
rank-clause       = "at", "rank", unsigned ;
access-clause     = "access", ( "direct" | "build" | "sequential" ) ;
rank-domain-clause = "rank", "domain", ( "complete" | "survivors" ) ;
```

They occur after `page size` and before `after`:

```ebnf
[ limit-clause ],
[ page-clause ],
[ rank-clause ],
[ access-clause ],
[ rank-domain-clause ],
[ after-clause ],
```

`at rank` is one-based and MUST be at least `1`.

`at rank` and `after` are mutually exclusive.

The default rank domain is `complete`. `rank domain survivors` is legal only
with `coverage allow incomplete`; it is never inferred merely because damage
is encountered.

### 17.2 Defaults

For ordinary first-page or cursor continuation queries:

```text
access sequential
```

is not implied literally; the optimizer may use any equivalent faster plan.
The request does not require random rank access.

For a query containing `at rank` and no `access` clause, the default is:

```text
access direct
```

This prevents numeric random access from silently becoming scan-and-discard.

### 17.3 Policies

`access direct`
: Admit only `DIRECT`. Exact index algebra is allowed. Authoritative
  scan-and-build and rank-proportional enumeration are forbidden.

`access build`
: Admit `DIRECT` or `BUILDABLE`. Build work requires explicit effective
  budgets and produces an exact selection artifact before results are labelled
  direct.

`access sequential`
: Permit an exact sequential compatibility plan. Explain and result metadata
  MUST identify it as sequential and disclose its estimated and actual work.

### 17.4 Example

```text
from products
where category = "book"
  and price < 20
order by price asc
page size 100
at rank 100001
access direct
consistency available
coverage complete
budget {
  documents: 100,
  bytes: 16777216,
  result_bytes: 4194304
}
```

The document budget permits fetching at most the returned page. Exact derived
indexes may be read, but the first 100,000 matches may not be fetched,
verified, or discarded by a plan labelled `DIRECT`.

### 17.5 Canonical plan

`RqlPlanV1` gains:

```text
start_rank: Optional<PositiveUInt>  // one-based
access_policy: Direct | Build | Sequential | Ordinary
rank_domain: Complete | Survivors
```

All fields participate in canonical plan identity except a cursor's moving
next-rank state. A cursor separately authenticates that state.

## 18. SDK and HTTP surface

### 18.1 Rust

The intended Rust shape is:

```rust
let page = products
    .query()
    .where_eq("category", "book")
    .where_lt("price", 20)
    .order_by("price", SortOrder::Asc)
    .page_size(100)
    .at_rank(100_001)?
    .access(AccessPolicy::Direct)
    .page()?;
```

The type-safe API SHOULD use a nonzero rank type:

```rust
Rank::new(100_001) -> Option<Rank>
```

and MUST NOT accept zero through unchecked conversion.

### 18.2 Page result

```text
DirectQueryPage {
  rows
  first_rank?
  next_rank?
  next_cursor?
  total_matches?
  complete
  access_class
  rank_domain
  direct_certificate_id?
  read_view_id
  coverage
  frontiers
  known_holes
  work {
    index_blocks_read
    index_bytes_read
    authoritative_documents_examined
    authoritative_documents_returned
    build_time?
    position_time
    fetch_time
  }
}
```

`total_matches` is present only when exact cardinality is known under the
reported rank domain.

### 18.3 HTTP compatibility

An HTTP adapter may map:

```http
GET /product_list?start=100001&limit=100
```

to:

```text
at rank 100001
page size 100
access direct
```

provided the adapter documents `start` as one-based.

The adapter MUST NOT translate this request to sequential skip/discard unless
the caller explicitly selects a sequential compatibility policy.

Responses SHOULD include a cursor for the next page:

```json
{
  "first_rank": 100001,
  "next_rank": 100101,
  "next_cursor": "...",
  "access_class": "direct",
  "rank_domain": "complete"
}
```

Following `next_cursor` avoids re-planning while retaining direct rank
semantics.

## 19. Direct cursor

### 19.1 Contents

A `dingo-direct-cursor-v1` token binds at least:

```text
heap_id
authorization_generation
capability_scope_hash
canonical_plan_hash
parameter_hash
predicate_semantics_hash
order_domain_id
read_view_id
rank_map_id/hash
rank_domain
next one-based rank
remaining total limit?
page size
source/index frontiers
coverage hash
known-hole hash
token version
issued time
expiry
nonce
```

### 19.2 Authentication and confidentiality

The token MUST be authenticated with a domain-separated key that contains at
least 256 bits of secret entropy and is not derivable from a Heap ID, cluster
ID, public key, hostname, or other public identifier. Qualified coordinators
receive the current token key through the protected control plane.

The token-key generation and key identifier are authenticated fields. Rotation
invalidates the prior generation after the declared grace policy. Key material
MUST be zeroized on retirement and MUST NOT appear in diagnostics, SDA
projections, backups without secret wrapping, or crash dumps by design.

The token SHOULD be encrypted when its counts, order keys, block identities,
or frontiers could reveal sensitive metadata.

A token:

- is Heap-bound;
- is authority-generation-bound;
- is query- and parameter-bound;
- cannot weaken coverage or change rank domain;
- cannot change page size beyond policy;
- cannot extend expiry;
- cannot be edited or manufactured by clients.

Master-key recycling and authority invalidation follow
[HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md). A cursor issued under an invalid authority
generation MUST fail.

### 19.3 Stateful artifacts

A cursor referencing a selection artifact is cryptographically stateless as
authorization evidence but still requires the named derived artifact to
exist. If it has expired or been collected, Residiuum returns:

```text
dda_artifact_expired
```

It MUST NOT silently rebuild under a different read view.

### 19.4 Replay

Read cursor replay is allowed unless a stricter host policy declares
single-use tokens. Replay against the same immutable read view returns the
same logical page and evidence.

## 20. Admission and budgets

### 20.1 Separate budgets

DDA distinguishes:

```text
admission budget
index-algebra budget
artifact-build budget
authoritative verification budget
result-fetch budget
snapshot-retention budget
```

A general byte or document budget may be compiled into these classes, but
explain MUST show the effective allocation.

### 20.2 Direct admission

Before executing `access direct`, the planner proves:

\[
\mathrm{ExactMembership}
\land
\mathrm{StrictOrder}
\land
\mathrm{ExactCounts}
\land
\mathrm{CompatibleView}
\land
\mathrm{RequiredCoverage}
\]

If false, admission fails before returning rows.

### 20.3 Build admission

Before `access build`, Residiuum estimates:

- documents and bytes examined;
- tiers mounted;
- index and authoritative I/O;
- temporary and durable bytes;
- CPU and wall-clock ceiling;
- artifact lease;
- expected future reuse.

Budget exhaustion before atomic publication yields no valid artifact.

### 20.4 Sequential admission

A sequential numeric-rank plan MUST report:

```text
estimated_prefix_matches
estimated_documents_examined
worst_case_documents_examined
```

Host policy SHOULD impose a low default maximum and require explicit
confirmation beyond it.

## 21. Explain

Structured RQL explain gains:

```text
direct_access {
  requested_rank
  access_policy
  classification
  direct_setup: Ready | Composed | None
  rank_domain
  read_view
  order_domain
  exact_atomic_indexes
  candidate_indexes
  candidate_verification
  predicate_bitmap_plan
  rank_blocks
  count_directory
  selection_artifact {
    state
    estimated_build_cost
    lease
  }?
  rejected_plans
  complexity {
    setup
    position
    page
  }
  certificate_obligations
  certificate?
}
```

Human explain MUST clearly answer:

1. Will Residiuum examine preceding documents?
2. Is positioning cost dependent on the requested rank?
3. Is a selection artifact being built?
4. Which exact indexes establish membership and order?
5. What read view and rank domain are being used?
6. Can damage or missing coverage change the claimed rank?

## 22. Complexity contract

Let:

- `n` be visible documents in the order domain;
- `m` be matching documents;
- `b` be rank blocks;
- `l` be page size;
- `Z(P)` be the compressed-index work to evaluate predicate `P`;
- `D(l)` be document fetch/verification work for `l` returned rows.

For a ready materialized rank map:

\[
T_{\mathrm{position}} = O(\log b)
\]

and:

\[
T_{\mathrm{page}} = O(\log b + l + D(l))
\]

assuming bounded local `select` and `next` operations.

For exact bitmap algebra:

\[
T_{\mathrm{setup}} = O(Z(P))
\]

\[
T_{\mathrm{position}} = O(\log b)
\]

The critical property is:

\[
T_{\mathrm{position}}(k) \notin \Theta(k)
\]

For a ready rank map the stronger certified bound is:

\[
\forall k \in [1,m]:
T_{\mathrm{position}}(k)=O(\log b)
\]

Increasing the requested rank therefore does not cause proportional
enumeration.

For a buildable unindexed query:

\[
T_{\mathrm{build}} = O(n)
\]

or:

\[
O(n \log n)
\]

when a new comparison sort is necessary, followed by the ready-map bounds
above.

Implementations MUST publish benchmark results separately for:

- cold build;
- warm bitmap algebra;
- ready direct selection;
- page fetch;
- mutation/delta overhead;
- damaged or incomplete coverage.

## 23. Formal safety properties

### 23.1 Membership soundness

\[
\forall r < m:
P(A[r]) = 1
\]

No returned document fails the predicate.

### 23.2 Membership completeness

\[
\forall d \in U_{H,V}:
P(d)=1
\implies
\exists! r < m: A[r]=d
\]

Every matching root document occurs exactly once.

### 23.3 Order soundness

\[
\forall r_1 < r_2 < m:
\kappa(A[r_1]) <_{\mathrm{lex}} \kappa(A[r_2])
\]

### 23.4 Rank correctness

\[
A[r]
=
\pi(\operatorname{select}_1(B,r))
\]

is the unique result with exactly `r` matching predecessors.

### 23.5 Page slicing

For valid `k` and `l`, returned page `Q` satisfies:

\[
Q[t] = A[k-1+t]
\]

for every returned offset `t`.

### 23.6 Cursor continuation

If the first page starts at rank `k` and returns `q` rows, an unchanged valid
cursor starts at:

\[
k' = k + q
\]

Therefore consecutive pages over one frozen view contain no duplicate or
omitted answer.

### 23.7 Heap noninterference

For authorization and query state belonging to Heap `H`:

\[
\forall d \in \operatorname{Page}:
heap(d)=H
\]

and no bitmap, rank block, count, cache, artifact, or cursor from another Heap
may contribute to the computation.

### 23.8 Damage honesty

If an uncovered region has unknown match contribution, the system cannot
derive an exact complete-domain rank after that region. It must fail or change
the explicitly reported rank domain.

### 23.9 Representation equivalence

For every conforming representation `R`:

\[
\operatorname{decode}(R) = B_{P,\pi}
\]

and all rank/select answers equal the abstract definitions in §5.

## 24. Proof sketches

### 24.1 Predicate compilation

Atomic index conformance establishes exact characteristic functions.
Induction over the normalized predicate AST then establishes exactness for
`and`, `or`, and `not` by the corresponding set identities in §9.2. Because
the complement universe is the exact live bitmap for `V`, deleted or
not-yet-visible ordinals cannot be introduced by negation.

### 24.2 Segmented selection

The rank blocks form a disjoint ordered partition of `π`. Their exact
cardinalities partition all set bits of `B`. Prefix function `F` is monotone.
For each `0 ≤ r < m`, exactly one block satisfies:

\[
F(q) \le r < F(q+1)
\]

Subtracting `F(q)` yields the local set-bit ordinal. Local `select` therefore
returns the same position as global `select`.

### 24.3 Stable pagination

The read view freezes `π` and `B`. Page slicing is therefore slicing one
immutable mathematical sequence. Advancing by the number of returned rows
cannot duplicate or omit an element.

### 24.4 Survivors domain

Removing unknown regions changes `U` and hence changes `π`, `B`, and rank.
Calling the new sequence `survivors` is sound. Calling it the complete
sequence without proof that removed regions contribute zero is unsound.

### 24.5 Security binding

The cursor authentication tag covers Heap, authority generation, plan,
parameters, view, order, rank map, rank domain, next rank, limits, and expiry.
Under the assumed unforgeability of the selected MAC/AEAD, changing any bound
field without the key is rejected except with the algorithm's declared
negligible forgery probability.

## 25. Stable errors

Initial stable families:

```text
dda_rank_invalid
dda_direct_unavailable
dda_build_required
dda_sequential_permission_required
dda_order_not_addressable
dda_predicate_not_indexable
dda_candidate_verification_required
dda_index_incompatible
dda_index_stale
dda_rank_map_invalid
dda_rank_map_corrupt
dda_artifact_build_failed
dda_artifact_expired
dda_artifact_quota
dda_read_view_expired
dda_read_view_mismatch
dda_coverage_incomplete
dda_rank_domain_mismatch
dda_cursor_invalid
dda_cursor_expired
dda_authority_invalid
dda_heap_mismatch
dda_budget_required
dda_budget_exhausted
dda_profile_unsupported
```

Errors carry:

- safe query/plan identifier;
- requested rank and page size;
- access classification;
- failed proof obligation;
- safe bounded coverage evidence;
- remediation suggestions.

Secret parameter values, document values, cross-Heap statistics, raw tokens,
and cryptographic material are excluded.

## 26. SDA examination

SDA examination MUST be able to project every surviving:

- order-domain manifest;
- rank-block header;
- bitmap/container descriptor;
- document-ordinal mapping;
- block cardinality;
- prefix-directory node;
- selection-artifact manifest;
- direct-access certificate;
- frozen read-view manifest;
- cursor envelope metadata excluding secrets;
- checksum result;
- source/index frontier;
- coverage record;
- known hole.

The projection includes origin:

```text
heap_id
collection_id
document/source identity
source revision/frontier
index definition/version
rank-map profile
order-domain identity
read-view identity
```

SDA examination MUST distinguish:

```text
verified exact
verified candidate
stale
partial
damaged
unreadable
recovered without manifest
```

An examiner MAY recompute block cardinality and local rank/select laws without
opening unrelated blocks.

## 27. Conformance

### 27.1 Abstract model

Every implementation is tested against a simple oracle that:

1. materializes `U`;
2. evaluates `P` authoritatively;
3. sorts by canonical `κ`;
4. slices the result sequence.

The oracle is allowed to be slow. Production paths must equal it.

### 27.2 Rank/select properties

For empty, singleton, dense, sparse, alternating, run-heavy, and random
bitmaps:

- all boundary ranks;
- all valid selects;
- invalid select;
- `rank(select(r)+1)=r+1`;
- monotonic select;
- encode/decode;
- independent block recovery;
- corruption detection.

### 27.3 Predicate equivalence

For heterogeneous generated documents:

- authoritative evaluation equals exact bitmap evaluation;
- every Boolean combination;
- Null versus Absent;
- exact numeric comparisons;
- string order;
- membership;
- nested and multivalued paths;
- candidate indexes never certify before verification.

### 27.4 Order equivalence

- natural key order;
- every scalar family;
- ascending and descending;
- Null and Absent placement;
- duplicate sort values;
- immutable-key tie-break;
- order-domain mismatch rejection;
- collation/version mismatch rejection.

### 27.5 Direct access

Test:

- first, middle, last, and past-end ranks;
- page size zero policy and maximum;
- very large ranks;
- result sets with extremely low selectivity;
- rank independent positioning counters;
- no prefix document decoding in `DIRECT`;
- build then random access;
- artifact reuse and expiry;
- sequential fallback only when explicit.

### 27.6 Mutation and compaction

- writes after frozen view;
- updates changing predicate or sort value;
- deletes and tombstones;
- base-plus-delta publication;
- compaction with pinned view;
- ordinal translation;
- view expiry;
- restart and recovery.

### 27.7 Cluster

- different worker completion orders;
- coordinator replacement;
- partition movement;
- stale directory generation;
- unavailable partition;
- missing global rank block;
- inconsistent index frontier;
- survivors-domain continuation;
- no duplicate or omitted result after resume.

### 27.8 Security

- token bit flips;
- wrong Heap;
- wrong authority generation;
- changed query;
- changed parameters;
- changed order;
- changed rank domain;
- increased page size;
- extended expiry;
- cross-Heap artifact/cache collision attempts;
- redaction of diagnostics.

### 27.9 Damage

Inject holes into:

- authoritative segments;
- order-domain manifests;
- bitmap containers;
- cardinality directories;
- selection artifacts;
- cursor state;
- one or many partitions.

Complete-domain execution must fail whenever the hole could change rank.
Survivors-domain execution must retain explicit hole evidence across every
page.

## 28. Implementation sequence

### DDA-0 — semantic oracle and profiles

- freeze canonical read-view, order-domain, rank-map, artifact, certificate,
  and cursor encodings;
- implement slow oracle;
- implement rank/select property corpus;
- add structured explain classification.

Exit: every future implementation can be compared byte-for-byte or
value-for-value with the oracle.

### DDA-1 — natural-order immutable rank blocks

- stable document-key order blocks;
- live/tombstone bitmaps;
- static rank/select sidecars;
- prefix-count directory;
- local embedded direct access;
- SDA examination.

Exit: unfiltered and presence/equality-indexed natural-order queries support
exact `at rank`.

### DDA-2 — exact predicate bitmap algebra

- equality, membership, presence, Null, and type bitmaps;
- Boolean circuit compiler;
- exact/candidate distinction;
- post-verification artifacts;
- SIMD/compressed-container evaluation.

Exit: supported filtered queries produce certificates and equal the scan
oracle.

### DDA-3 — scalar range and secondary order domains

- bit-sliced/range or wavelet structures;
- `dingo-order-wavelet-v1` conditioned ordered selection;
- secondary order mappings;
- Null/Absent/collation profiles;
- composite direct indexes;
- ordered-page fetch.

Exit: declared scalar filter/order combinations support direct random rank.

### DDA-4 — selection artifacts and public surfaces

- `access build`;
- artifact quotas, leases, atomic publication, reuse, and GC;
- RQL grammar and plan encoding;
- Rust SDK;
- remote wire profile;
- HTTP adapter contract.

Exit: arbitrary buildable root queries can pay once and access any later rank
without prefix enumeration.

### DDA-5 — distributed global rank

- distributed frozen views;
- globally ordered block directory;
- coordinator-independent continuation;
- complete/survivors domains;
- failure injection and partition movement.

Exit: cluster direct pages remain deterministic and honest across coordinator
failure and partial damage.

### DDA-6 — adaptive optimization

- workload-driven index recommendations;
- optionally enabled adaptive/cracking structures;
- cached bitmap circuits;
- automatic representation selection;
- benchmark-driven tuning.

Exit: optimization changes cost, never semantics or evidence.

## 29. Release acceptance gates

Implementation begins with DDA-0 and proceeds behind capability labels. DDA
v1 is release-ready only when:

1. all persistent canonical encodings are frozen;
2. one-based external rank and zero-based internal ordinal conversions are
   tested at every boundary;
3. exact and candidate indexes are distinct types or equally strong runtime
   states;
4. the planner cannot label a scan/discard plan `DIRECT`;
5. read-view retention and expiry are implemented;
6. cross-Heap cache and artifact keys are structurally impossible;
7. complete and survivors rank domains cannot be confused;
8. certificate proof obligations are machine-checkable;
9. corruption of every derived layer is detected;
10. the slow oracle, direct plan, build plan, and sequential plan agree;
11. large-rank benchmarks disclose positioning work separately from build and
    fetch work;
12. cluster tests prove coordinator-independent resumption;
13. SDA can examine surviving blocks and holes independently;
14. operators can bound artifact storage and frozen-view retention;
15. the shipped capability matrix labels every implemented access class
    honestly.

## 30. Research basis

This section is informative, not normative.

The design is grounded in established and current research:

- rank and select over bit vectors can support constant-time static queries
  with sublinear auxiliary space:
  [Engineering Compact Data Structures for Rank and Select Queries on Bit
  Vectors](https://arxiv.org/abs/2206.01149);
- compressed bitmap indexes support fast set algebra and practical
  memory-mapped representations:
  [Consistently faster and smaller compressed bitmaps with
  Roaring](https://arxiv.org/abs/1603.06549);
- wavelet structures support ranked selection such as range quantiles:
  [Range Quantile Queries: Another Virtue of Wavelet
  Trees](https://arxiv.org/abs/0903.4726);
- dynamic succinct rank/select has strong modern update/query bounds:
  [Succinct Dynamic Rank/Select: Bypassing the Tree-Structure
  Bottleneck](https://arxiv.org/abs/2510.19175);
- direct-access database theory studies retrieval of the `k`th ranked query
  answer after preprocessing and characterizes tractable query/order classes:
  [Tractable Orders for Direct Access to Ranked Answers of Conjunctive
  Queries](https://cris.technion.ac.il/en/publications/tractable-orders-for-direct-access-to-ranked-answers-of-conjuncti/)
  and
  [Database Theory in Action: Direct Access to Query
  Answers](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICDT.2026.27);
- adaptive indexing can reorganize data incrementally in response to workload:
  [Stochastic Database
  Cracking](https://arxiv.org/abs/1203.0055).

Residiuum's contribution is not the invention of rank/select. It is the proposed
combination of:

- document-native RQL predicate semantics;
- immutable damage-localized storage;
- exact bitmap algebra;
- direct ranked access;
- cryptographically bound Heap-local continuation;
- frozen distributed read views;
- explicit complete versus survivors rank domains;
- independently examinable mathematical evidence.

That combination is the DDA product and correctness proposition.

## 31. Current implementation delta

This section is informative and records the repository state when this draft
was written.

The shipped paths are not DDA:

- `residiuum-store::cursor` pages raw live subjects in subject order with a
  generation fence;
- `Collection::scan_json_page` exposes that embedded raw scan;
- `Collection::find_with` supports filters, ordering, and a result limit but
  does not expose a unified filtered rank page;
- `residiuum-cluster::Cluster::scan_with` pages prefix-filtered subjects using one
  `after_subject` frontier and deterministic merge;
- no shipped path builds an exact predicate rank map, order domain, selection
  artifact, frozen DDA read view, or direct-access certificate;
- the existing store and cluster continuation tags use keys derived from
  public store/cluster identifiers and therefore do not satisfy §19.2; see
  `DEF-097`.

Until the relevant DDA stage passes:

- binaries MUST NOT advertise `at rank` or `access direct`;
- HTTP `start` remains an adapter concern and MUST NOT be advertised as
  efficient random access;
- existing continuation capability is labelled raw sequential paging;
- `dql-source-v0.1` remains unchanged.

## 32. Initial implementation ownership

The first implementation SHOULD remain a vertical slice across existing
crates rather than creating an empty subsystem tree:

```text
crates/residiuum-store/
  rank/
    bitvec.rs        abstract bitmap + plain rank/select reference
    block.rs         RankBlockV1 and document mapping
    directory.rs     exact prefix-count directory
    codec.rs         bounded canonical encodings
    verify.rs        checksums and local proof laws
    build.rs         immutable block construction
  order_wavelet/
    tree.rs          semantic reference tree
    matrix.rs        optimized levelwise representation
    dictionary.rs    exact RQL tuple order dictionary
    forest.rs        base/delta global value selection

crates/residiuum-sdk/
  direct.rs          Rank, AccessPolicy, DirectQueryPage, query builder
  filter.rs          exact/candidate index capability declarations

crates/residiuum-cluster/
  direct.rs          distributed read view and global rank directory

crates/residiuum-server/
  direct.rs          admission, artifact leases, cursor issuance
  token_keys.rs      secret generation, rotation, and protected distribution

crates/residiuum-examine/
  direct.rs          SDA projection of blocks, maps, views, and holes
```

The reference plain bit-vector implementation lands before compressed or
adaptive representations. It is the executable mathematical oracle for every
optimized representation.

Persistent DDA bytes MUST NOT be emitted until DDA-0 freezes:

- deterministic encoding rules and numeric labels;
- maximum lengths and nesting;
- domain-separated hash inputs;
- checksum/Merkle layout;
- cursor authentication/encryption algorithm and key profile;
- reader/writer compatibility matrix.

In-process types and the slow oracle may land before that freeze, but test
fixtures must identify themselves as ephemeral and must not become accidental
wire compatibility promises.
