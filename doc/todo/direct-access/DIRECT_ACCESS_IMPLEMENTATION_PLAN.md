# Direct Access implementation plan

Status: developer-ready v1.0

Program release: P4

Normative source: [DIRECT_ACCESS_SPEC.md](./DIRECT_ACCESS_SPEC.md)

Order companion: [ORDER_WAVELET_SPEC.md](../order-wavelets/ORDER_WAVELET_SPEC.md)

## 1. Delivery decision

Direct Access is not offset pagination.

It is an exact, admitted operation over a named rank domain:

```text
query + parameters + order + read view + coverage
                         ↓
                 exact rank map
                         ↓
                    select(k)
```

The first release supports:

- one-based `from_rank`;
- natural subject order;
- exact document-local predicate bitmaps;
- immutable read views;
- complete and survivors rank domains;
- authenticated Heap-bound continuation cursors;
- explicit `direct`, `built`, `sequential`, and `refused` classifications.

It MUST NOT scan or enumerate ranks `1..k-1` while reporting `direct`.
Secondary scalar order becomes direct only through qualified DOW packages.

## 2. Ownership

The first implementation is a vertical slice:

```text
crates/residiuum-store/src/rank/
  bitvec.rs
  block.rs
  directory.rs
  codec.rs
  build.rs
  verify.rs

crates/residiuum-sdk/src/
  direct.rs

crates/residiuum-server/src/
  direct.rs
  token_keys.rs

crates/residiuum-examine/src/
  direct.rs
```

Do not create a storage-independent façade that duplicates the authoritative
read-view, coverage, or Heap identity types. Pure rank/select algorithms MAY
live in a small internal module with no filesystem, clock, network, or global
state.

## 3. Frozen boundaries

### 3.1 Rank

`Rank` is a non-zero `u64`. Public numeric rank is one-based.

For an ordered exact result sequence \(R\):

\[
\operatorname{at}(R,k)=R[k-1],\qquad 1\leq k\leq |R|
\]

`from_rank(k).limit(n)` returns the half-open logical interval:

\[
R[k-1\ ..\ \min(k-1+n,|R|)]
\]

Rank zero, overflow, or a page limit outside policy is rejected before
planning.

### 3.2 Public request

The SDK surface is:

```rust
let page = heap
    .collection("products")?
    .query("status = $status")
    .bind("status", "active")
    .order_by_subject()
    .from_rank(101)?
    .limit(100)?
    .access(AccessPolicy::DirectOrRefuse)
    .run()
    .await?;
```

Equivalent RQL:

```text
from products
where status = $status
order by @subject asc
at rank 101
limit 100
access direct
```

The response includes:

```text
items
first_rank
next_rank?
next_cursor?
access_class
rank_domain
read_view_id
coverage
```

HTTP `?start=101&limit=100` is only an adapter spelling. The adapter MUST
compile it to `at rank 101`; it MUST NOT implement it by discarding 100
preceding rows.

### 3.3 Access policy

```rust
pub enum AccessPolicy {
    DirectOrRefuse,
    Build { budget: BuildBudget },
    SequentialCompatibility,
}
```

- `DirectOrRefuse` either proves direct eligibility or returns a stable reason.
- `Build` may materialize an exact selection artifact, within declared bounds.
- `SequentialCompatibility` may scan and discard, but the response MUST say
  `access_class = sequential` and expose examined work.

There is no silent fallback between these policies.

### 3.4 Cursor profile

Before DDA-4, the normative spec MUST receive a wire-profile amendment for
`residiuum-direct-cursor-v1`.

The profile MUST use:

- deterministic CBOR;
- XChaCha20-Poly1305 with a 256-bit key;
- a fresh random 192-bit nonce;
- token key identifier and generation;
- all §19.1 fields as authenticated data or ciphertext;
- a distinct key derived for each Heap and authority generation;
- constant-time authentication failure;
- a hard encoded-token size limit;
- current and immediately preceding token-key generation only during an
  explicit grace period.

The token root key is an online service secret, never the Heap master key.
The per-Heap token key is HKDF-SHA-256:

```text
PRK = HKDF-Extract(salt = HeapId, IKM = deployment_token_root)
key = HKDF-Expand(
    PRK,
    info = "residiuum-direct-cursor-v1" || 0x00 || authority_generation_be,
    length = 32
)
```

The visible envelope is canonical:

```text
profile = "residiuum-direct-cursor-v1"
key_id
nonce[24]
ciphertext_and_tag
```

The AEAD associated data is the canonical encoding of `profile`, `key_id`,
and `nonce`. Every cursor field in §19.1 is inside the ciphertext. Add pinned
`chacha20poly1305` and `hkdf` workspace dependencies; no alternative cipher or
KDF is accepted under this profile.

Existing cursors whose keys are derived from public identifiers alone are not
security precedents and MUST NOT be reused for this profile.

## 4. Work packages

### DDA-0 — Semantic oracle and persistent profiles

Entry:

- RRE predicate semantics are frozen for the supported subset;
- HeapId and immutable read-view identities exist.

Deliver:

- slow exact `filter → stable sort → index` oracle;
- plain bit-vector `rank1` and `select1`;
- canonical scalar/tuple order fixtures;
- direct/build/sequential/refused planner result;
- frozen block, directory, hash, Merkle, cursor, and compatibility profiles;
- generated corpus including empty, singleton, duplicate, null, missing,
  minimum/maximum scalar, and damaged-source cases.

Tests:

- exhaustive bit vectors through length 16;
- property laws for rank/select duality;
- differential result equality against the slow oracle;
- canonical bytes and rejection fixtures;
- two-Heap identity substitution failures.

Exit:

- `Unit`, `Property`, `Differential`, `Isolation`;
- no persistent DDA bytes exist before the profile freeze.

Non-goals: compression, DOW, distributed rank.

### DDA-1 — Natural-order immutable rank blocks

Entry: DDA-0 accepted.

Deliver:

- `RankBlockV1` over immutable source blocks;
- live-membership bitmap;
- exact prefix-count directory;
- subject locator mapping;
- complete/survivors domain identity;
- checksums, child hashes, source frontier, known-hole commitment;
- deterministic rebuild and SDA projection.

Required laws for bitmap \(B\):

\[
\operatorname{rank}_1(B,\operatorname{select}_1(B,k))=k
\]

\[
\operatorname{select}_1(B,k)
  =\min\{i\mid\operatorname{rank}_1(B,i)=k\}
\]

Tests:

- differential lookup for every valid rank;
- append, tombstone, compaction, restart;
- missing/corrupt rank block and source block;
- complete-domain refusal on unknown coverage;
- survivor-domain ranks never masquerade as complete-domain ranks.

Exit:

- natural-order unfiltered `at rank k` is independent of numeric `k`;
- `Unit`, `Property`, `Differential`, `Crash`, `Damage`, `Isolation`.

### DDA-2 — Exact predicate bitmap algebra

Entry: DDA-1 accepted; RRE-0 predicate truth table accepted.

Deliver:

- exact index-capability declarations;
- equality, finite membership, presence, null, exact type, and supported
  Boolean composition;
- `AND`, `OR`, and complement only within the same exact universe;
- exact cardinality and select over the resulting bitmap;
- rejection of candidate-only, lossy, stale, or differently bound bitmaps.

For exact membership bitmap \(M_P\):

\[
\operatorname{result}(P,k)
  =\operatorname{subject}\big(\operatorname{select}_1(M_P,k)\big)
\]

Tests:

- three-valued missing/null/type fixtures;
- Boolean algebra properties where their preconditions hold;
- differential comparison to RQL evaluation;
- mixed Heap/view/ruleset/coverage bitmap rejection;
- damaged member index cannot yield `complete`.

Exit:

- supported natural-order filtered queries are exact;
- `Property`, `Differential`, `Isolation`, `Damage`, `Performance`.

### DDA-3 — Scalar order admission and DOW seam

Entry: DDA-2 accepted.

Deliver:

- canonical `order_domain_id`;
- exact ascending/descending and tuple/tie-break identities;
- planner seam for `SelectOrdered(P,K,k)`;
- explicit refusal until a qualified DOW index covers the requested view;
- no runtime full-sort path classified as direct.

Tests:

- order identity changes for direction, null policy, collation, codec, and
  tie-break changes;
- mock order-provider fixtures proving the admission seam preserves rank,
  view, coverage, and order identity;
- stale/incomplete order index refusal.

Exit:

- the planner can prove or reject ordered direct access without ambiguity;
- `Unit`, `Differential`, `Isolation`.

### DDA-4 — Selection artifacts, cursor, and public journey

Entry: DDA-0 through DDA-3 accepted for the advertised query class.

Deliver:

- bounded exact selection-artifact build and lease;
- authenticated/encrypted cursor issuance and verification;
- SDK, RQL, CLI, and protocol surfaces;
- explain output with access class, proof obligations, effective budgets,
  coverage, examined work, and refusal reason;
- expiry, key rotation, authority-generation invalidation, and artifact
  collection behavior;
- SDA projection excluding secrets.

Tests:

- page concatenation equals the oracle exactly;
- continuation remains on the same view and plan;
- cursor edit, cross-Heap use, cross-authority use, expiry, replay, rotation,
  missing artifact, and changed coverage;
- `start=100001` direct performance does not grow with the numeric rank;
- full clean-state Rust and CLI journey.

Exit:

- `Direct Rank` may be advertised only for the conformance rows that pass;
- `Unit`, `Property`, `Differential`, `Isolation`, `Damage`, `Journey`,
  `Performance`.

### DDA-5 — Distributed global rank

State: deferred until cluster semantics and LocalHeap behavior are qualified.

### DDA-6 — Adaptive optimization

State: deferred. Adaptive structures may change cost, never semantics,
identity, coverage, or evidence.

## 5. Stable refusal surface

At minimum:

```text
dda_rank_zero
dda_rank_overflow
dda_limit_invalid
dda_not_exact
dda_order_unsupported
dda_view_unavailable
dda_coverage_incomplete
dda_budget_exceeded
dda_artifact_expired
dda_cursor_invalid
dda_cursor_expired
dda_cursor_authority_stale
dda_profile_unsupported
```

Errors carry structured details but never secret key material or cross-Heap
identifiers the caller is not authorized to observe.

## 6. Release evidence

P4 requires:

- checked-in oracle corpus and canonical bytes;
- reproducible cold/warm benchmarks at ranks 1, 1,000, 100,000, and
  100,000,000 over the same immutable view;
- measurements of positioning work separately from result fetch;
- chaos runs with absent/corrupt source and rank material;
- a two-Heap noninterference suite;
- SDA examples for healthy, incomplete, stale, and damaged rank maps;
- capability and documentation status updated in the same change.

The performance claim is not “constant-time query.” It is:

> For a qualified direct plan, positioning work is independent of the numeric
> magnitude of the requested rank, subject to the declared directory, bitmap,
> cache, and result-fetch costs.
