# Residiuum future retrieval roadmap

Status: **deferred product direction — not an active delivery priority**

Scope: deterministic text search, vector search, and geospatial search

Audience: product designers, spec authors, and future implementers
Companions: [OVERVIEW.md](../../reference/product/OVERVIEW.md),
[INDEXING_STRATEGY_PROPOSAL.md](../../done/implementation/INDEXING_STRATEGY_PROPOSAL.md),
[ATOMICS_SPEC.md](../atomics/ATOMICS_SPEC.md), [RQL_SPEC.md](../../wip/query/RQL_SPEC.md),
[DIRECT_ACCESS_SPEC.md](../direct-access/DIRECT_ACCESS_SPEC.md),
[SDA_PROFILE.md](../../reference/query/SDA_PROFILE.md), and [HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md)

## 1. Purpose

This document preserves the intended direction for three important retrieval
capabilities without promoting them into the current implementation program:

1. deterministic text search;
2. vector search;
3. geospatial search.

These capabilities matter to Residiuum's eventual position as a serious
general-purpose document database. They are deliberately deferred while the
core product, Heap isolation, operational qualification, Atomics, and the
generic index lifecycle mature.

This is a roadmap, not a normative wire or API specification. No syntax,
algorithm, index representation, or milestone in this document is a compatibility
promise until promoted into a dedicated specification.

## 2. Governing decision

Residiuum should drive all three retrieval families from:

> **One authoritative document store, one common derived-index substrate, and
> several mathematically appropriate index engines.**

A document is authoritative once. Search indexes are derived, disposable
projections:

```text
authoritative document storage
            |
            +-- text projection ------ inverted index
            +-- vector projection ---- exact / ANN index
            +-- spatial projection --- spatial index
```

The three projections MUST NOT be forced into one universal physical index
format:

- text search needs terms, postings, positions, and corpus statistics;
- vector search needs vector blocks or approximate-nearest-neighbour
  structures;
- geospatial search needs cells, bounds, or spatial trees.

Their mathematics differ. Their lifecycle, authority, isolation, evidence,
and failure semantics should not.

## 3. Prerequisite: the common derived-index substrate

No retrieval family should be implemented as an isolated subsystem. Before
product work begins, Residiuum needs a generic index substrate that all three can
reuse.

The substrate owns:

- Heap ownership and isolation;
- immutable, versioned index definitions;
- collection and field scope;
- source document identifiers and revisions;
- atomic index obligations;
- build, catch-up, and query frontiers;
- immutable checksummed segments;
- deterministic publication and replacement;
- damage detection and explicit coverage;
- rebuild, compaction, and garbage collection;
- snapshots and recovery;
- query consistency modes;
- authoritative revision validation;
- SDA examination and provenance;
- common lifecycle, metrics, and operator tooling.
- exact bitmap/rank-select integration where the retrieval family defines an
  exact membership set and deterministic order.

The shared lifecycle may reuse Residiuum Direct Access rank blocks, frozen read
views, and selection artifacts. Approximate vector results MUST NOT inherit an
exact DDA certificate.

A conceptual origin record is:

```text
IndexEntryOrigin {
    heap_id,
    collection_id,
    document_id,
    document_revision,
    index_definition_id,
    index_definition_version
}
```

Every derived entry MUST be attributable to exactly one authoritative source
revision, exactly one index definition version, and exactly one Heap.

The substrate may reuse suitable Hydra/Chimera machinery, but this document
does not require those structures to become a universal search format. Existing
authority doctrine remains unchanged: deleting derived indexes MUST NOT destroy
authoritative documents or prevent salvage of surviving documents.

## 4. Shared correctness doctrine

### 4.1 Authority

For every retrieval family:

- the stored document and its committed revision are authoritative;
- projections and indexes are derived accelerators;
- a missing or corrupt index is rebuildable from surviving authoritative data;
- an index MUST NOT be the only surviving description of a document;
- rebuilding an index MUST NOT rewrite historical authority.

For externally generated vectors, the vector stored by the application is
authoritative data. An ANN structure built from it is derived. Residiuum should
not initially call an embedding service during indexing.

### 4.2 Heap isolation

Every index definition, segment, posting, vector node, spatial entry,
statistic, cache key, frontier, and query plan MUST be Heap-scoped.

Two Heaps MUST NOT share:

- term dictionaries;
- document-frequency or ranking statistics;
- ANN graphs or centroids;
- spatial index nodes;
- result caches;
- index-build state;
- query-time candidate pools.

This is both an isolation requirement and a confidentiality requirement.
Cross-Heap corpus statistics can leak facts even when no foreign document is
returned.

### 4.3 Atomic obligations

A committed document mutation records the obligations created for every
applicable index definition:

```text
commit document revision R
+
commit obligations:
    article_text_v1 must observe R
    article_vector_v1 must observe R
    location_geo_v1 must observe R
```

Index construction may occur asynchronously. The obligation and its source
revision cannot be lost silently.

A query result MUST be validated against the authoritative live revision before
it is returned. This prevents stale postings or stale ANN/spatial entries from
resurrecting deleted or replaced documents.

The eventual normative specification should define at least these semantic
classes, regardless of their final names:

- **available** — query the currently published index and disclose its frontier,
  damage, and coverage;
- **current** — wait until the relevant index has observed the caller's required
  frontier;
- **exact** — provide exact results using authoritative validation and, where
  feasible, a bounded authoritative fallback.

An implementation MUST NOT label an approximate algorithm, incomplete index,
or bounded scan as exact.

### 4.4 Honest damage

Indexes should be independently checksummed and segmented so that local damage
does not make healthy regions unreachable.

If damage or lag could alter the result, the response MUST say so. It must not
silently present a shorter result set as complete.

Conceptually:

```text
coverage: incomplete
index_frontier: 9182771
damaged_segments: [41, 77]
```

When an exact fallback is requested and practical, Residiuum may scan surviving
authoritative data. If resource limits prevent completion, the result remains
explicitly incomplete.

### 4.5 Versioned interpretation

Index definitions are immutable versions. Any change that can change membership,
ordering, distance, score, tokenization, or geometry creates a new definition
version and a new derived build.

An implementation MUST NOT silently reinterpret an existing index after a
library, Unicode table, model, tokenizer, distance metric, spatial library, or
ranking upgrade.

### 4.6 SDA examinability

SDA examination should expose, as appropriate:

- the index definition and immutable version;
- source document revision;
- projection inputs and normalized representation;
- build and query frontiers;
- segment integrity and known holes;
- why a document became a candidate;
- why it matched or failed verification;
- its exact score, distance, or spatial relation;
- whether the result was exact, approximate, incomplete, or bounded;
- the tie-break rule and final ordering.

The goal is not merely searchable data. It is searchable data whose
interpretation and surviving evidence can be inspected.

## 5. Priority when this roadmap becomes active

The intended order is:

```text
generic index substrate
        |
        v
deterministic text search
        |
        v
exact vector search
        |
        v
segmented ANN + hybrid search
        |
        v
basic geospatial search
        |
        v
advanced spatial operations
```

This is the order **within the future retrieval program**. It does not override
the current product and production-readiness priorities.

Text comes first because it has the broadest ordinary utility and forces the
shared lifecycle to become real. Vector follows because exact vector retrieval
and then ANN enable semantic and hybrid search. Geospatial follows because it
is important but applies to a narrower set of common workloads.

## 6. Deterministic text search

### 6.1 Product intent

Text search should cover everyday document-database workloads such as:

- articles and knowledge bases;
- logs and operational records;
- product catalogs;
- messages and tickets;
- names, descriptions, and metadata.

The objective is not initially to reproduce every Elasticsearch/OpenSearch
feature. The objective is deterministic, durable, Heap-local lexical retrieval
with honest coverage and examinable ranking.

### 6.2 Semantic model

Text search is not merely an inverted index. These choices can change what a
query means and therefore belong to an immutable search profile:

- Unicode version;
- normalization form;
- tokenization;
- case folding and diacritic handling;
- language selection;
- stemming or lemmatization;
- stop-word set and version;
- field selection and weights;
- term-frequency and document-length formula;
- phrase and position semantics;
- query grammar;
- score constants;
- tie-break order.

Illustrative, non-normative definition:

```text
search_profile article_search_v1 {
    fields      = [title^3, body]
    unicode     = unicode_17
    normalize   = nfkc
    tokenizer   = words_v1
    language    = english_v1
    stemming    = porter2_v1
    stopwords   = english_stopwords_v1
    ranking     = bm25_v1(k1 = 1.2, b = 0.75)
}
```

`article_search_v1` never changes meaning. A semantic change creates
`article_search_v2` and a new build.

### 6.3 Heap-local ranking

Terms, postings, positions, document counts, document frequencies, length
statistics, and ranking parameters are local to the index scope within one
Heap.

Global inverse-document-frequency statistics across Heaps are forbidden.
They would violate the rule that two Heaps never meet and could leak the
existence or prevalence of terms in another Heap.

### 6.4 Damage and fallback

The inverted index should use independently recoverable immutable segments.
Loss of one segment must leave other postings usable.

If exact text search is requested, Residiuum may tokenize and scan surviving
source documents to fill index holes, subject to an explicit resource bound.
The response states whether the fallback completed.

### 6.5 Candidate RQL shape

The final syntax must remain visually compatible with RQL. An illustrative
surface is:

```text
from articles
search "damage tolerant database"
using article_search_v1
where status = "published"
take 20
expect complete
```

Illustrative result evidence:

```text
search:
  profile: article_search_v1
  scorer: bm25_v1
  index_frontier: 9182771
  coverage: complete
  damaged_segments: []
```

### 6.6 Delivery slices

**Text 0 — deterministic lexical core**

- fixed Unicode normalization;
- exact tokens;
- AND/OR terms;
- field restriction and ordinary RQL filtering;
- stable deterministic ordering;
- frontier and coverage disclosure.

**Text 1 — ranked and positional search**

- immutable language analyzers;
- positions and phrase search;
- field weights;
- a frozen, fully specified ranking function such as BM25;
- exact score explanation through SDA.

**Text 2 — product search conveniences**

- prefix and autocomplete;
- bounded fuzzy matching;
- highlighting;
- immutable synonym sets;
- facets where supported by the generic index substrate.

Initially excluded:

- arbitrary analyzer plugins;
- Turing-complete scripts;
- silent analyzer upgrades;
- opaque ML reranking;
- an unversioned natural-language query parser;
- a promise to support every language;
- arbitrary corpus-wide regular expressions.

## 7. Vector search

### 7.1 Product intent

Vector search enables:

- semantic document retrieval;
- similarity and recommendation;
- image, audio, and multimodal retrieval;
- retrieval-augmented generation;
- hybrid lexical and semantic search.

Residiuum should store vectors and retrieve them. It should not initially generate
embeddings. Model execution introduces external availability, nondeterminism,
cost, and hidden model-version semantics that do not belong in the first
database implementation.

### 7.2 Semantic model

A vector index definition freezes at least:

- source field and vector dimension;
- scalar representation;
- distance or similarity metric;
- normalization requirements;
- treatment of invalid values;
- exact or approximate algorithm class;
- algorithm and build parameters;
- deterministic tie-breaking;
- optional embedding provenance recorded with the document.

The application-supplied vector is authoritative. Quantized vectors, centroids,
graphs, neighbour lists, and other ANN structures are derived.

### 7.3 Exact versus approximate

The API and result evidence MUST distinguish:

- exact nearest-neighbour search;
- approximate nearest-neighbour search;
- incomplete search caused by damage or lag;
- bounded search terminated by an explicit resource limit.

These are independent dimensions. An undamaged ANN result is approximate but
may have complete index coverage. A damaged exact-vector index may use exact
distance calculations over incomplete coverage. Neither may be described simply
as “complete exact search.”

### 7.4 Damage-tolerant ANN

ANN must not depend on one monolithic graph whose damaged entry point makes
healthy vectors unreachable.

The intended direction is independently navigable, checksummed ANN segments
with:

- multiple validated entry points;
- explicit segment inventories;
- per-segment damage and coverage;
- authoritative revision validation;
- rebuild from stored vectors;
- optional exact scan of surviving vectors.

The exact segmentation and ANN algorithm require a dedicated future
specification and adversarial damage tests.

### 7.5 Delivery slices

**Vector 0 — exact search**

- typed fixed-dimension vectors;
- frozen distance metrics;
- exact top-k scan;
- deterministic ties;
- ordinary RQL filters;
- SDA distance explanation.

**Vector 1 — segmented ANN**

- independently navigable ANN segments;
- explicit approximation contract;
- configurable recall/latency trade-off;
- damage and coverage reporting;
- rebuild and compaction.

**Vector 2 — scale and hybrid prerequisites**

- frozen quantization profiles;
- resource budgets;
- filtered ANN;
- measured recall qualification;
- stable merge of per-segment candidates.

Initially excluded:

- embedding-model hosting;
- implicit network calls during indexing;
- silent dimension coercion;
- opaque or silently changing metrics;
- a claim of exactness for ANN;
- one cross-Heap candidate graph.

## 8. Hybrid text and vector search

Hybrid retrieval belongs after useful text search and vector search exist
independently.

Lexical score and vector similarity are not naturally interchangeable numbers.
Residiuum MUST NOT silently add them together. A hybrid profile freezes:

- the lexical and vector index versions;
- candidate counts and filtering order;
- normalization of each score family;
- the fusion formula and constants;
- tie-breaking;
- missing-candidate behavior;
- damage and lag behavior for either source.

SDA should expose both component scores and the fusion calculation.

Conceptually:

```text
hybrid_result {
    text_score,
    vector_distance,
    fusion_profile,
    final_score,
    text_coverage,
    vector_coverage
}
```

If either input is damaged, stale, bounded, or approximate, that fact survives
fusion and remains visible to the caller.

## 9. Geospatial search

### 9.1 Product intent

Geospatial search enables:

- nearby-place lookup;
- delivery and service-area queries;
- asset and device tracking;
- bounding-box filtering;
- mapping and location-aware applications.

The first goal is a small, predictable spatial core, not a complete GIS system.

### 9.2 Semantic model

A spatial index definition freezes at least:

- geometry type;
- coordinate reference system;
- axis order;
- coordinate units;
- longitude wrapping and pole behavior;
- planar or geodesic distance semantics;
- boundary inclusion rules;
- validity and normalization rules;
- precision or cell-resolution policy;
- deterministic tie-breaking.

“Distance” is not meaningful until these choices are fixed. Library defaults
must not silently define database behavior.

### 9.3 Delivery slices

**Geo 0 — points**

- one canonical point representation;
- one frozen Earth coordinate model;
- bounding-box queries;
- radius queries;
- exact distance verification;
- nearest-point search;
- ordinary RQL filters;
- SDA display of normalized coordinates and distance calculation.

**Geo 1 — bounded shapes**

- lines and polygons;
- intersects, contains, and within;
- explicit boundary semantics;
- shape validity checks;
- exact predicate verification after index candidate generation.

**Geo 2 — advanced spatial capability**

- additional coordinate systems where justified;
- complex geometry collections;
- topology-sensitive operations;
- spatial joins;
- specialized scale or precision profiles.

Initially excluded:

- “support every GIS operation”;
- implicit coordinate-system guessing;
- silently repaired invalid geometry;
- unbounded shape complexity;
- cross-Heap spatial trees;
- topology claims without exact verification.

## 10. Common query composition

Text, vector, and geospatial predicates should compose with ordinary RQL
filters. They should not create three parallel query languages.

The common planner eventually needs to express:

- candidate production by a specialized index;
- ordinary predicate filtering;
- authoritative revision validation;
- exact post-verification where required;
- declared ordering and limits;
- consistency and coverage requirements;
- resource bounds;
- provenance for the result.

The planner must preserve the distinction between:

- a predicate that determines membership;
- an accelerator that proposes candidates;
- a score or distance that orders candidates;
- a bound that may truncate work;
- evidence that limits what completeness can be claimed.

## 11. Shared operational requirements

Before any retrieval family is considered production-ready, it needs:

- online build from a declared source frontier;
- catch-up without losing concurrent mutations;
- atomic publication of a completed generation;
- resumable rebuild;
- crash-safe compaction;
- deterministic drop and replacement;
- capacity and write-amplification accounting;
- memory and disk budgets;
- backpressure;
- corruption injection and segment-loss tests;
- wipe-derived recovery tests;
- upgrade and downgrade policy;
- backup/restore treatment;
- Heap lifecycle integration;
- metrics for frontier, lag, coverage, damage, and rebuild progress.

Index backup is optional because indexes are derived. Index definitions and
enough immutable interpretation metadata to reproduce them are not optional.

## 12. Promotion gates

A retrieval family may move from this roadmap into active development only
when:

1. the common index substrate has a named owner and normative specification;
2. Heap isolation applies structurally to every new artifact and statistic;
3. Atomics defines index obligations and frontier semantics;
4. authority and rebuild behavior are explicit;
5. damage cannot silently become “zero matches”;
6. exact, approximate, incomplete, stale, and bounded outcomes are distinct;
7. the interpretation profile is immutable and versioned;
8. RQL composition is designed;
9. SDA examination is designed;
10. destructive, concurrency, and compatibility tests are specified before the
    implementation is accepted.

## 13. Product position

These capabilities are not the current priority, but they are strategically
important. Together with Heaps, capability-based access, Atomics, Data Rules,
referential integrity, damage tolerance, long retention, and SDA examination,
they move Residiuum beyond “raw storage” or “a fast document store.”

The intended long-term proposition is:

> A mathematically constrained, damage-tolerant document database in which
> authoritative data survives independently, derived interpretations are
> versioned and rebuildable, and every search result states what was examined,
> what may be missing, and why the result exists.

The story becomes credible only when each guarantee is implemented, attacked,
measured, and published. This roadmap records the direction without displacing
the work required to make the existing database dependable first.
