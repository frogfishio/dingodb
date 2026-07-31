# Residiuum Order Wavelet implementation plan

Status: developer-ready v1.0

Program release: P5

Normative source: [ORDER_WAVELET_SPEC.md](./ORDER_WAVELET_SPEC.md)

Admission owner: [DIRECT_ACCESS_SPEC.md](../direct-access/DIRECT_ACCESS_SPEC.md)

## 1. Delivery decision

DOW is the exact physical path for:

```text
where P
order by K
at rank k
```

It does not sort matches at query time. It navigates exact conditional counts
through a versioned order structure.

The first release supports immutable single-node scalar order:

- boolean, signed integer, unsigned integer;
- exact decimal under a frozen scale profile;
- byte string and UTF-8 bytewise order;
- missing/null/value tags;
- ascending and descending;
- deterministic subject tie-break;
- complete and survivors coverage domains.

Locale collation, floating-point order, arrays, objects, distributed
selection, and adaptive codecs are not part of P5.

## 2. Ownership

Implement inside the existing vertical slice:

```text
crates/residiuum-store/src/order_wavelet/
  tree.rs
  matrix.rs
  dictionary.rs
  block.rs
  codec.rs
  forest.rs
  build.rs
  verify.rs

crates/residiuum-sdk/src/direct.rs
crates/residiuum-server/src/direct.rs
crates/residiuum-examine/src/direct.rs
```

`tree.rs` is the simple executable oracle. `matrix.rs` is allowed to be fast
only after it is proven differential-equivalent for every supported profile.

Every dictionary, block, level bitmap, prefix directory, delta, cache entry,
and forest root is bound to:

```text
HeapId
collection identity
order-domain identity
read view/source frontier
coverage domain and known-hole commitment
profile version
content hash
```

## 3. Frozen semantic boundary

Let \(M\) be the exact predicate membership bitmap and \(K_i\) the canonical
order tuple of document \(i\). DOW returns:

\[
\operatorname{SelectOrdered}(M,K,k)
  =\operatorname{arg\,kth}_{i:M_i=1}(K_i,\operatorname{subject}_i)
\]

The subject is the final strict tie-break. Thus every admitted document has
one total position.

At each wavelet level \(\ell\), the chosen branch is determined by the exact
number of live matches routed left:

\[
c_\ell=\operatorname{rank}_1(M_\ell \land \neg B_\ell,n_\ell)
\]

If \(k\leq c_\ell\), navigate left; otherwise navigate right with
\(k\leftarrow k-c_\ell\). No branch may use an estimate.

Unknown source coverage, missing level material, or unverifiable counts makes
complete-domain selection unavailable. It never becomes a best-effort exact
answer.

## 4. Work packages

### DOW-0 — Mathematical reference and corpus

Entry:

- DDA-0 canonical scalar and order-domain profiles accepted.

Deliver:

- stable-sort oracle using canonical `(order tuple, subject)` keys;
- simple pointer/tree wavelet reference;
- exact dictionary encoding and inverse;
- levelwise rank/select reference;
- supported-type corpus with missing, null, duplicate, boundary, descending,
  and multi-field tie cases.

Tests:

- exhaustive sequences over small alphabets;
- every predicate bitmap and every valid rank for small inputs;
- reference result equals stable-sort oracle;
- dictionary order preservation:

\[
x <_{\mathrm{RQL}} y
\iff
\operatorname{code}(x)<\operatorname{code}(y)
\]

- two-Heap and two-order-domain substitution rejection.

Exit:

- `Unit`, `Property`, `Differential`, `Isolation`;
- semantic and encoding decisions are closed before persistent bytes.

### DOW-1 — Immutable wavelet blocks

Entry: DOW-0 accepted; DDA-1 immutable rank blocks accepted.

Deliver:

- canonical dictionary and strict tuple codes;
- immutable wavelet matrix block;
- one exact rank/select bitmap per level;
- block cardinalities and value-range summaries;
- source/rank-block commitments;
- deterministic build, verify, reopen, and SDA projection.

Tests:

- matrix equals reference tree and stable-sort oracle;
- all supported types, directions, tuple lengths, and tie cases;
- restart and deterministic rebuild;
- bit flip, truncated level, swapped dictionary, wrong source frontier,
  wrong Heap, and wrong order domain.

Exit:

- one immutable block can answer exact ordered kth selection;
- `Property`, `Differential`, `Crash`, `Damage`, `Isolation`.

### DOW-2 — Bounded compressed representation

Entry: DOW-1 accepted.

Deliver:

- frozen plain and compressed bitmap codecs;
- bounded decoder allocation and nesting;
- per-level checksums/content hashes;
- explicit codec capability negotiation;
- reference fallback for build/verification, never for silent query semantics.

Tests:

- each codec differential-equivalent to plain bits;
- hostile lengths, counts, padding, non-canonical values, and decompression
  limits;
- corruption localizes the unavailable coverage;
- reproducible size/time corpus.

Exit:

- compressed structures retain exact answers and bounded decoding;
- `Unit`, `Property`, `Differential`, `Damage`, `Performance`.

### DOW-3 — Pages, caches, and Direct Access integration

Entry: DOW-2 and DDA-3 accepted.

Deliver:

- multi-block exact conditional-count navigation;
- bounded page/cache policy keyed by all universal binding fields;
- DDA planner admission and `order_domain_id`;
- `dingo-direct-cursor-v1` continuation for ordered pages;
- explain/SDA evidence for visited levels, counts, coverage, and cache state;
- explicit build/refuse behavior for absent artifacts.

Tests:

- concatenated pages equal the stable-sort oracle;
- positioning work does not grow with numeric `k`;
- cold/warm/evicted cache equivalence;
- cache entries cannot cross Heap, view, coverage, predicate, or order domains;
- missing/corrupt page refuses complete-domain direct access;
- concurrent readers remain on their bound immutable view.

Exit:

- qualified immutable filtered scalar-order queries may report
  `access_class = direct`;
- `Differential`, `Isolation`, `Damage`, `Journey`, `Performance`.

### DOW-4 — Base/delta mutation and compaction

Entry: DOW-3 accepted; authoritative commit frontiers are available.

Deliver:

- immutable base plus bounded ordered deltas;
- exact global value selection across base and deltas;
- tombstone and replacement handling;
- deterministic compaction to a new forest root;
- atomic root publication;
- old-view retention while referenced by a live cursor;
- crash recovery and orphan collection.

For sources \(S_1,\dots,S_m\), global selection uses exact count reduction:

\[
C(v)=\sum_{j=1}^{m}
  |\{x\in S_j\mid M(x)\land K(x)\leq v\}|
\]

The smallest dictionary value with \(C(v)\geq k\), followed by exact
tie-break selection, determines the result.

Tests:

- arbitrary insert/replace/delete histories against the slow oracle;
- compaction failpoint at every publication boundary;
- old and new cursor views never mix roots;
- delta ceiling causes build/backpressure/refusal, not unbounded query work;
- damaged delta/base yields explicit coverage loss.

Exit:

- mutable collections preserve exact ordered direct access within published
  bounds;
- `Property`, `Differential`, `Crash`, `Damage`, `Isolation`, `Performance`.

### DOW-5 — Distributed selection

State: deferred until the cluster read-view and global-rank profiles are
normative and qualified.

## 5. Admission contract

DOW is eligible only when all are true:

```text
exact predicate membership
supported canonical order
strict subject tie-break
exact counts
matching Heap/view/frontier
declared rank domain
complete required artifacts
budget admitted
```

Otherwise the planner returns `build`, `sequential`, or `refused` according to
the caller's explicit access policy. It MUST NOT label a runtime sort as DOW.

## 6. Stable refusal surface

At minimum:

```text
dow_order_unsupported
dow_dictionary_mismatch
dow_view_mismatch
dow_coverage_incomplete
dow_level_missing
dow_count_unverifiable
dow_delta_limit
dow_budget_exceeded
dow_profile_unsupported
```

## 7. Release evidence

P5 requires:

- canonical dictionary, block, matrix, and forest fixtures;
- exhaustive small-model corpus;
- randomized histories differential against stable sort;
- reproducible cold/warm benchmarks with positioning and fetch separated;
- size disclosure for plain and compressed codecs;
- two-Heap noninterference and artifact-substitution tests;
- damage/chaos runs at dictionary, block, level, delta, and root boundaries;
- SDA examples for complete, survivors, stale, and damaged states;
- capability and documentation truth updated in the same change.

The allowed claim is:

> For a qualified DOW plan, Residiuum selects the kth exact filtered result in a
> declared scalar order by navigating exact conditional counts, without
> enumerating or sorting the preceding matches at query time.
