# Core Application API and RQL implementation plan

Status: **developer-ready v1.0**

Program: `APP`

Priority: `P1-PATH`

Baseline expansion authority:
[MUST_ADD.md](../MUST_ADD.md). APP packages are retained and mapped into APB;
their original exclusions do not define the final
`dingo-application-baseline-v1`.

Audience: SDK, server, store, query, protocol, test, and documentation
implementers

Normative companions:

- [HEAP_SPEC.md](../HEAP_SPEC.md)
- [RQL_SPEC.md](../RQL_SPEC.md)
- [RESIDUUM_PREDICATE_SPEC.md](../RESIDUUM_PREDICATE_SPEC.md)
- [DX_SPEC.md](../DX_SPEC.md)
- [COLLECTION_CONTRACT_SPEC.md](../COLLECTION_CONTRACT_SPEC.md)
- [TESTING_STRATEGY.md](../TESTING_STRATEGY.md)
- [HEAP_APPLICATION_READY_PLAN.md](HEAP_APPLICATION_READY_PLAN.md)

This plan governs the first ordinary application-facing vertical slice after
Heap isolation. It deliberately excludes the Node.js client. A second language
binding must not freeze mistakes in the Rust and wire contracts.

## 1. Outcome

At acceptance, an application developer can:

1. open one authenticated Heap locally or remotely;
2. create, list, and open collections without knowing physical layout or raw
   collection identifiers;
3. put, get, delete, inspect history, and manage field indexes through one
   coherent Rust API;
4. query documents using either a typed builder or the RQL Application Core;
5. page deterministically with opaque authenticated cursors;
6. receive typed receipts, coverage, budgets, and stable errors; and
7. obtain the same observable semantics from embedded and qualified remote
   execution.

The release is successful when an ordinary Rust application can complete this
journey without importing legacy unqualified storage types, constructing wire
JSON, handling collection identifiers as strings, or understanding ResiduumDB's
directory structure.

## 2. Product decision

The public abstraction is:

```text
authenticated Heap session
        |
        +-- create/list/open collection
        |
        +-- Heap-bound collection handle
                |
                +-- typed data operations
                +-- history
                +-- indexes
                +-- query builder / RQL
```

Embedded and remote are transport choices, not different database products.
They share public value types, errors, defaults, and behavioral tests.

The SDK MUST NOT expose a public method that accepts both a Heap handle and a
caller-supplied collection id. A collection handle captures both identities.
Cross-Heap composition remains impossible by construction as specified by
`HEAP_SPEC.md`.

## 3. Scope

### 3.1 Included

- qualified collection create, list, and open;
- one backend-neutral synchronous Rust façade;
- JSON and opaque-byte put/get/delete;
- safe durability selection and typed receipts;
- per-key history with explicit hole evidence;
- field-index list/create/drop/rebuild;
- shared canonical predicate AST;
- typed query builder;
- RQL Application Core parsing and compilation;
- deterministic ordering, limits, bounded pages, and continuation;
- complete-by-default coverage;
- scan/materialization budgets and cancellation;
- structured explain;
- protocol schemas, fixtures, compatibility tests, and user documentation.

### 3.2 Excluded

These items were excluded from the original APP slice. Items admitted by
[MUST_ADD.md](../MUST_ADD.md) are now required through their APB packages and
MUST NOT be treated as product-level deferrals.

- Node.js, TypeScript, Python, Java, and other language clients;
- writes or DDL inside RQL;
- RRE, collection contracts, referential integrity, and Atomics;
- SQL-ish+ and SQL-to-RQL compilation;
- enrichment, nested `within`, and cross-collection RQL execution in this
  delivery package;
- exact ranked access and Residuum Order Wavelets;
- offset pagination;
- aggregation, watches, change streams, and bulk mutation;
- text, vector, and geospatial retrieval;
- an async Rust API;
- a new cluster protocol or new storage format.

The existing `dql-source-v0.1` enrichment compiler remains supported as a
separate compatibility surface. This package does not claim that it has become
the complete `dql-plan-v1` runtime.

## 4. Conformance profiles

This package freezes the following identifiers:

| Profile | Meaning |
|---|---|
| `dingo-rust-app-v1` | public Rust application API |
| `dql-app-core-v1` | accepted RQL Application Core source surface |
| `dql-plan-v1` | canonical logical plan shape defined by `RQL_SPEC.md` |
| `dingo-predicate-v1` | shared total predicate semantics |
| `dingo-cursor-v1` | authenticated query continuation |
| `rpc-v1` | qualified Heap wire envelope |

`dql-app-core-v1` is a conformance level of RQL v1, not a competing language.
Its accepted grammar is the subset in section 9. A runtime MUST report the
source and plan profiles separately.

Unsupported RQL v1 syntax MUST fail with `QueryInvalid` and diagnostic code
`dql_feature_unavailable`. It MUST NOT be ignored, weakened, or executed using
an accidental fallback.

## 5. Public Rust API

Names and signatures below are normative. Definitions may live in existing
modules, but an implementation change to their observable shape requires an
amendment to this plan before code review.

```rust
pub struct HeapClient { /* sealed backend + Heap binding */ }
pub struct CollectionClient { /* Heap binding + immutable CollectionId */ }

pub struct CollectionInfo {
    pub heap_id: HeapId,
    pub collection_id: CollectionId,
    pub name: String,
    pub descriptor_hash: [u8; 32],
}

pub struct CreateCollectionOptions {
    pub operation_id: Option<OperationId>,
}

pub struct CreateCollectionResult {
    pub collection: CollectionClient,
    pub receipt: CollectionCreateReceipt,
}

pub struct CollectionCreateReceipt {
    pub receipt_id: ReceiptId,
    pub operation: AdminOperation, // CreateCollection
    pub heap_id: HeapId,
    pub collection_id: CollectionId,
    pub descriptor_hash: [u8; 32],
    pub created_at: SystemTime,
}

impl HeapClient {
    pub fn id(&self) -> HeapId;
    pub fn create_collection(
        &mut self,
        name: &str,
    ) -> Result<CreateCollectionResult, Error>;
    pub fn create_collection_with(
        &mut self,
        name: &str,
        options: CreateCollectionOptions,
    ) -> Result<CreateCollectionResult, Error>;
    pub fn open_collection(
        &mut self,
        name: &str,
    ) -> Result<CollectionClient, Error>;
    pub fn list_collections(&mut self) -> Result<Vec<CollectionInfo>, Error>;
}

impl CollectionClient {
    pub fn heap_id(&self) -> HeapId;
    pub fn id(&self) -> CollectionId;
    pub fn name(&self) -> &str;

    pub fn put<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<WriteReceipt, Error>;
    pub fn put_with<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: &T,
        options: PutOptions,
    ) -> Result<WriteReceipt, Error>;
    pub fn put_bytes(
        &mut self,
        key: &str,
        value: &[u8],
    ) -> Result<WriteReceipt, Error>;
    pub fn get(&mut self, key: &str)
        -> Result<Option<serde_json::Value>, Error>;
    pub fn get_bytes(&mut self, key: &str)
        -> Result<Option<Vec<u8>>, Error>;
    pub fn delete(&mut self, key: &str)
        -> Result<DeleteReceipt, Error>;

    pub fn history(&mut self, key: &str)
        -> Result<KeyHistory, Error>;
    pub fn indexes(&mut self) -> IndexManager<'_>;

    pub fn query(&mut self) -> QueryBuilder<'_>;
    pub fn dql(
        &mut self,
        source: &str,
        parameters: &Parameters,
        options: QueryRunOptions,
    ) -> Result<QueryPage, Error>;
    pub fn explain_dql(
        &mut self,
        source: &str,
        parameters: &Parameters,
        options: QueryRunOptions,
    ) -> Result<QueryExplanation, Error>;
}
```

Construction uses the current qualified entry points:

```rust
let deployment = DingoDeployment::open("./data")?;
let heap: HeapClient = deployment.open_heap(heap_cap).into();

let heap: HeapClient =
    RemoteHeap::connect(endpoint, RemoteHeapOptions::new(credential))?.into();
```

The existing `Heap`, `HeapCollection`, and `RemoteHeap` may remain public during
a deprecation window, but all ordinary documentation uses `HeapClient` and
`CollectionClient`. The façade MUST wrap or reuse existing implementations; it
must not create a second storage engine.

### 5.1 Ownership and concurrency

The v1 API is synchronous. A remote `HeapClient` and its collection handles
share a serialized session internally. The SDK makes no `Send` or `Sync` claim
until dedicated compile-time and stress tests prove it. Applications needing
parallel remote work open a bounded pool of Heap sessions.

No public lifetime may tie a collection handle to a temporary collection-name
string. Handles bind immutable identifiers and retain the name only for
display.

### 5.2 Typed values, not tuples

Remote operations MUST return the same public receipt and metadata types as
embedded operations. Raw `(String, String)`, `(Vec<Value>, bool)`, and
`Vec<(String, String)>` results are adapter internals, not the v1 API.

Required types:

```rust
pub struct KeyHistory {
    pub versions: Vec<HistoryEntry>,
    pub has_known_holes: bool,
}

pub struct HistoryEntry {
    pub kind: HistoryKind,
    pub event_id: EventId,
    pub version: VersionId,
    pub segment_id: SegmentId,
    pub known_gap_before: bool,
    pub value: Option<StoredValue>,
}

pub enum StoredValue {
    Json(serde_json::Value),
    Bytes(Vec<u8>),
}
```

History is oldest first. A damaged or incomplete payload never becomes `None`
or JSON `null`; it produces explicit hole evidence or a typed damage error.

## 6. Collection creation contract

Wire operation `106`, `collection_create`, is promoted from `reserved` to
`active` only when all package tests pass.

### 6.1 Request

The operation id belongs to the qualified RPC envelope, not the operation
arguments:

```json
{
  "v": 1,
  "id": 42,
  "operation_id": "00112233445566778899aabbccddeeff",
  "op_id": 106,
  "args": {
    "canonical_name": "orders"
  }
}
```

- `canonical_name` is validated and normalized once using the existing Heap
  object-name profile before any mutation.
- `operation_id` is exactly 16 random bytes encoded as 32 lowercase hex
  characters.
- the SDK generates an operation id once per logical call unless the caller
  supplies one for controlled retry;
- transport retries preserve the same id;
- malformed or missing mutation ids fail before effect on qualified remote
  execution.

### 6.2 Response

```json
{
  "collection_id": "immutable-id-encoding",
  "canonical_name": "orders",
  "descriptor_hash": "32-byte-hash-encoding",
  "receipt": {
    "receipt_id": "32-lowercase-hex-characters",
    "operation": "create_collection",
    "heap_id": "immutable-id-encoding",
    "object_id": "immutable-id-encoding",
    "descriptor_hash": "32-byte-hash-encoding",
    "created_at": 1785360000
  }
}
```

The SDK decodes identifiers and receipt fields into typed values before
returning. The receipt returned for an idempotent replay is the original
receipt, including its original `receipt_id` and `created_at`.

### 6.3 Semantics

For Heap `H`, canonical name `N`, operation id `O`, and generated collection id
`C`:

```text
create(H, N, O) = Created(C) | Replayed(C) | finite typed error
```

The following are mandatory:

- identical `(H, principal, O, N)` retry returns the identical logical result;
- reuse of `(H, principal, O)` with another request fingerprint fails with
  `ConsistencyViolation`;
- an existing active `N` created under another operation fails with
  `AlreadyExists`;
- the same name in two Heaps produces distinct collection identities;
- authorization is checked before existence, catalog, or timing details are
  disclosed;
- `HeapAdmin` is required;
- the catalog entry becomes visible atomically or not at all;
- crash at every persistence boundary recovers to absent or one valid
  collection, never two identities for one live canonical name;
- catalog rebuild from authoritative records preserves the result.

Collection creation does not create a directory contract. Physical placement
remains an implementation detail.

## 7. Data operation semantics

### 7.1 Keys and payloads

- keys use the current canonical key validation profile;
- JSON is encoded using the current typed JSON envelope;
- bytes remain distinguishable from JSON for their entire lifetime;
- default maximum payload is the current host limit: 16 MiB;
- a type mismatch is `TypeMismatch`, never an implicit conversion.

### 7.2 Writes

`put` is upsert. It returns a `WriteReceipt` containing the key, event id,
version, requested and achieved acknowledgement, commit state, store id, and
segment id.

Default durability is `Durable`.

The implementation MUST either:

1. honor a requested durability mode and report the achieved mode; or
2. reject the request with `DurabilityUnavailable`.

It MUST NOT silently accept an option it does not implement. A stronger
acknowledgement may satisfy a weaker request only if the ordering of durability
modes is specified in the receipt module and the receipt reports both values.

Every remote mutation uses a stable client-generated operation id across
transport retry.

### 7.3 Reads

`get` returns `Ok(None)` only when authoritative state says the key is absent.
Unavailable partitions, unreadable payloads, detected damage, unsupported
format, and incomplete coverage are errors or explicit evidence; none is
absence.

### 7.4 Deletes

Deleting an absent key is successful and returns `removed: false`. A tombstone
receipt is still returned when a mutation was durably recorded. Delete
durability and idempotency follow the write rules.

## 8. Index contract

The v1 manager surface is:

```rust
pub struct IndexSpec {
    pub name: String,
    pub fields: Vec<FieldPath>,
}

pub struct IndexInfo {
    pub spec: IndexSpec,
    pub state: IndexState,
    pub frontier: Frontier,
}

impl IndexManager<'_> {
    pub fn list(&mut self) -> Result<Vec<IndexInfo>, Error>;
    pub fn create(&mut self, spec: IndexSpec) -> Result<IndexInfo, Error>;
    pub fn drop(&mut self, name: &str) -> Result<DropIndexReceipt, Error>;
    pub fn rebuild(&mut self, name: &str) -> Result<IndexInfo, Error>;
}
```

An index is an acceleration structure, not authority. Query results with a
ready/current index and results from a complete scan MUST be logically equal.
A missing, stale, or damaged index causes a safe scan, an explicit incomplete
result when allowed, or a typed failure. It never causes a false empty result.

Index names and paths are validated before effect. Index administration rights
are distinct from ordinary CRUD rights.

## 9. RQL Application Core

The accepted source grammar is:

```ebnf
query              = [ "explain" ], from-clause,
                     { where-clause },
                     [ project-clause ],
                     [ order-clause ],
                     [ limit-clause ],
                     [ page-clause ],
                     [ after-clause ],
                     [ consistency-clause ],
                     [ coverage-clause ],
                     [ budget-clause ] ;
```

All productions have the meanings and exact grammar defined in
`RQL_SPEC.md`. The Application Core adds no alternative syntax.

For `CollectionClient::dql`, the `from` source MUST resolve to that handle's
immutable collection id. A different source name or identity fails before
execution. This redundancy keeps copied RQL readable while preventing a caller
from smuggling another collection into the handle.

Included features:

- root predicates from `dingo-predicate-v1`;
- named parameters;
- projection;
- scalar ordering with explicit null/missing placement;
- an implicit immutable-key tie-break;
- total limit and page size;
- opaque continuation;
- `available` and `current` consistency;
- complete or explicitly allowed-incomplete coverage;
- document, byte, and result-memory budgets;
- explain.

Excluded clauses fail with `dql_feature_unavailable`, including `enrich`,
`within`, `at rank`, and direct/build/sequential access policy.

### 9.1 One semantic plan

RQL text and `QueryBuilder` compile into the same validated `RqlPlanV1`.
Neither executes its own predicate semantics.

```rust
collection
    .query()
    .where_(field("status").eq(param("status")))
    .project(["id", "status"])
    .order_by(field("created_at"), Desc)
    .limit(1_000)
    .page_size(100)
    .coverage(CoveragePolicy::Complete)
    .budget(QueryBudget::documents(50_000))
    .run(&params)?;
```

Plan binding replaces source names with immutable Heap-confined collection
identities. The compiler inserts every default and the immutable-key
tie-breaker before hashing.

Persistent, exchanged, or cursor-bound plans require the canonical byte
encoding profile demanded by `RQL_SPEC.md` section 15. An ephemeral Rust AST is
not sufficient for protocol or cursor identity.

### 9.2 Defaults and ceilings

| Setting | Default | Maximum / rule |
|---|---:|---:|
| page size | 64 rows | 4,096 rows |
| coverage | complete | incomplete only by explicit opt-in |
| consistency | available | caller may request current |
| order | immutable key ascending | field order adds key tie-break |
| limit | absent | host result ceiling still applies |
| payload | host profile | currently 16 MiB |
| result bytes | host profile | currently 64 MiB |

Page size is not SQL `LIMIT`. `limit` caps the total logical result across all
pages. `page size` caps one response.

Unbounded scans may be rejected by server policy with
`QueryBudgetRequired`. No default or budget permits silent truncation.

The Application Core parser/decoder hard ceilings are:

| Input | Ceiling |
|---|---:|
| UTF-8 RQL source | 1 MiB |
| JSON/parameter nesting | 64 |
| bound parameters | 1,024 |
| encoded parameters | 16 MiB total |
| normalized predicate nodes | 4,096 |
| path segments in one path | 64 |
| projected output items | 1,024 |
| order terms before key tie-break | 16 |
| continuation token | 64 KiB |
| complete RPC frame | negotiated, at most current 16 MiB default |

A server may configure tighter values and reports them in explain. Raising a
ceiling beyond this table requires a new resource profile; clients cannot raise
it through query budgets.

## 10. Query result contract

```rust
pub struct QueryPage {
    pub query_id: QueryId,
    pub plan_hash: [u8; 32],
    pub heap_id: HeapId,
    pub collection_id: CollectionId,
    pub rows: Vec<QueryRow>,
    pub next: Option<Continuation>,
    pub exhausted: bool,
    pub coverage: CoverageEvidence,
    pub frontiers: Vec<SourceFrontier>,
    pub known_holes: Vec<HoleEvidence>,
    pub consistency: ConsistencyEvidence,
    pub ordering: Vec<NormalizedOrderTerm>,
    pub remaining_limit: Option<u64>,
    pub stats: QueryStats,
}

pub struct QueryRow {
    pub key: String,
    pub value: serde_json::Value,
}
```

`coverage.complete` means the requested logical result has complete source
coverage up to the returned frontier. `exhausted` means no logical row remains
after this page. `next` is present if and only if `exhausted` is false.

An empty page with incomplete coverage is not equivalent to “no matches.”
Coverage and frontier evidence are returned even for empty pages.

Rows are deterministic under an unchanged authoritative state:

```text
sort_tuple(row) =
  (declared normalized sort terms..., immutable_key)
```

All supported scalar families, missing, and null have total ordering rules from
`RQL_SPEC.md`. Unsupported or mixed values produce specified ordering or a
typed error; they are never ordered by incidental JSON serialization.

## 11. Continuation security and consistency

A `dingo-cursor-v1` token contains or commits to:

- cursor profile and key id;
- Heap id and immutable collection id;
- authority epoch or equivalent credential fence;
- canonical plan hash;
- canonical parameter hash;
- normalized order and last sort tuple;
- source and index frontiers;
- remaining total limit;
- effective page size;
- coverage and consistency modes;
- issued-at and expiry times.

The token is authenticated with a non-public rotating cursor key. The key MUST
not be derived only from public identifiers, a capability id, or token fields.
Servers retain the current key and a bounded verification window for previous
keys. Embedded mode stores the secret in Heap-confined protected metadata.

Tokens expire 15 minutes after issue. Every successful next page receives a
new token and therefore a new 15-minute window. Servers accept at most two
minutes of clock skew and never accept a token more than 17 minutes after its
recorded issue time.

The routine cursor key rotates every 24 hours. The immediately previous key is
retained for 17 minutes, then erased. Authority cycling or explicit emergency
cursor-key rotation invalidates all prior keys immediately; this is an
intentional security fence, not a pagination error. Cursor key material is
never logged, exported in telemetry, or returned by an API.

Verification is constant-time after bounded parsing. Before execution it
checks profile, MAC, Heap, collection, authority fence, plan, parameters,
expiry, and policy. Any mismatch fails with `ConsistencyViolation` or the
more specific stable error and returns no rows.

For the first profile, pagination is generation-fenced:

- mutation of the relevant authoritative generation invalidates continuation;
- the client restarts the query;
- duplicates or omissions are not silently accepted.

Snapshot pagination is a future extension and requires its own retention
contract.

Raw `after_key` is not part of the public RQL API. It may remain an internal
primitive beneath authenticated continuation.

## 12. Error and retry contract

The existing `ErrorCode` vocabulary is authoritative. The façade maps embedded
and remote failures to the same code:

| Condition | Code |
|---|---|
| collection/key absent where absence is exceptional | `NotFound` |
| duplicate collection/index name | `AlreadyExists` |
| malformed RQL, predicate, path, key, or name | `QueryInvalid` or `ValidationFailed` |
| unsupported Application Core feature | `QueryInvalid` + `dql_feature_unavailable` |
| insufficient authority | `PermissionDenied` |
| invalid credential | `AuthenticationFailed` |
| stale/tampered/mismatched cursor | `ConsistencyViolation` |
| complete result cannot be proven | `CoverageIncomplete` |
| explicit budget required | `QueryBudgetRequired` |
| hard ceiling reached | `ResourceLimit` |
| requested durability unavailable | `DurabilityUnavailable` |
| damaged authoritative data | `DataDamaged` / `PayloadPartial` |
| protocol shape impossible under the profile | `ProtocolViolation` |
| cancellation/deadline | `DeadlineExceeded` |

Messages may improve without a breaking release. Codes and structured detail
fields are compatibility surfaces.

SDK automatic retry is allowed only when all are true:

- the error is classified retryable;
- the operation is read-only or carries a stable operation id;
- the deadline permits another attempt; and
- the same Heap, collection, request fingerprint, and authority remain bound.

Authorization, validation, damage, and cursor mismatch errors are not retried.

## 13. Wire work

Required protocol artifacts:

```text
spec/heap/rpc-v1/collection_create.request.json
spec/heap/rpc-v1/collection_create.response.json
spec/heap/rpc-v1/dql_query.request.json
spec/heap/rpc-v1/dql_query.response.json
spec/heap/fixtures/collection_create.accepted.json
spec/heap/fixtures/collection_create.rejected.json
spec/heap/fixtures/dql_query.accepted.json
spec/heap/fixtures/dql_query.rejected.json
```

Operation `118`, `dql_query`, carries both execution and explain; `explain:
true` returns the structured explanation and does not enumerate rows. No
second explain operation is introduced. Every active operation in
`operations-v1.json` MUST reference a checked-in request and response schema.

The remote request transports a canonical plan, parameters, options, and
continuation as separate fields. RQL source may be accepted for convenience,
but the server recompiles and validates it; it never trusts a client-declared
plan hash.

Protocol decoders:

- reject unknown required profile versions;
- bound strings, arrays, nesting, plan bytes, parameter bytes, and cursor bytes
  before allocation;
- reject duplicate semantic fields;
- reject unknown enum values;
- never preserve an untyped `serde_json::Value` beyond the adapter boundary
  where a public typed value exists.

## 14. Work packages

### APP-0 — Contract and fixture lock

Depends: `HAR-0` (principal may parallel; labor order is principal board)

Deliver:

- this plan and the RQL Application Core conformance section;
- public Rust compile fixtures;
- wire request/response golden fixtures;
- stable error mapping table;
- canonical plan and cursor test vectors.

**Artifact locations (2026-07-30 start):**

| Deliverable | Location |
|---|---|
| Contract index | `spec/app/v1/README.md` |
| Error mapping | `spec/app/v1/error_mapping_v1.json` |
| Plan vectors | `spec/app/v1/plan_vectors_v1.json` |
| Cursor vectors | `spec/app/v1/cursor_vectors_v1.json` |
| Rust compile surface | `crates/dingo-sdk/src/app_v1.rs` (`dingo-rust-app-v1`) |
| Contract tests | `crates/dingo-sdk/tests/app0_contract_lock.rs` |
| Verify script | `scripts/verify-app0-contract.sh` |
| Wire schemas (staged) | `spec/heap/rpc-v1/collection_create.*`, `dql_query.*` |
| Wire fixtures (staged) | `spec/heap/fixtures/collection_create.*`, `dql_query.*` |

Ops **106** / **118** stay `reserved` with null schema pointers in
`operations-v1.json` until APP-1 / APP-7; on-disk schemas are frozen for
implementers. Plan/cursor MAC bytes may remain labeled placeholders until
encoding profiles land in APP-4 / APP-6 — that is an explicit residual, not a
silent invent.

Exit:

- SDK, server, store, and query owners approve one contract;
- no unresolved placeholder affects an implementer's choice.

### APP-1 — Qualified collection provisioning

Depends: `APP-0`

Implements `HAR-1`:

- authoritative create transition;
- operation-id fingerprint and replay result;
- catalog publication/rebuild;
- operation 106 registry metadata (`active`, `returns_data: true`, schemas);
- embedded method;
- operation 106 server dispatch and schemas;
- qualified remote method;
- crash matrix and two-Heap isolation tests.

Exit:

- create/list/open parity passes embedded and remote;
- repeated and conflicting retries have the specified results;
- operation 106 may be marked active.

### APP-2 — Backend-neutral Rust façade

Depends: `APP-1`

Deliver:

- `HeapClient`, `CollectionClient`, typed metadata;
- embedded and remote adapters;
- no raw id/string tuples in ordinary API;
- deprecation notes for overlapping legacy paths;
- compile-fail tests for cross-Heap composition where Rust typing can express
  them.

Exit:

- the same application source runs against embedded and remote constructors;
- ordinary examples have no backend branch after construction.

### APP-3 — Data, history, and index parity

Depends: `APP-2`, `HAR-4`

Deliver:

- JSON/bytes CRUD parity;
- exact durability behavior;
- typed receipts and histories;
- typed index manager;
- common error translation;
- stable mutation operation ids.

Exit:

- shared behavior suite passes both backends;
- no absence/damage/coverage ambiguity remains;
- requested options are honored or rejected, never ignored.

### APP-4 — Canonical predicates and plan

Depends: `APP-0`

Deliver:

- one `dingo-predicate-v1` AST/parser/evaluator;
- `RqlPlanV1` logical structures;
- canonical encoding and domain-separated plan hash;
- name-to-immutable-id binding;
- builder-to-plan compilation;
- model and property tests for predicate totality.

Exit:

- builder and RQL predicate fixtures compile to identical canonical plans;
- scan and indexed evaluation agree with the model oracle.

### APP-5 — RQL Application Core compiler

Depends: `APP-4`

Deliver:

- complete parsing of section 9;
- parameters, projection, order, limit, page, coverage, consistency, budgets,
  and explain;
- ordered diagnostics;
- explicit rejection for non-Core RQL v1 constructs;
- `dql-app-core-v1` conformance corpus.

Exit:

- no supported syntax is implemented only in docs;
- no unsupported syntax is silently accepted;
- parser fuzzing is bounded and panic-free.

### APP-6 — Query execution and continuation

Depends: `APP-3`, `APP-5`, `HAR-4`

Deliver:

- bounded page executor;
- deterministic scalar order and key tie-break;
- total-limit accounting across pages;
- complete-by-default coverage/frontier evidence;
- budgets and cancellation;
- structured explain;
- rotating non-public cursor keys and generation fences.

Exit:

- pages concatenate to the complete scan oracle without duplicates/omissions;
- cursor tamper, replay context, Heap, collection, plan, parameter, expiry, and
  generation tests fail closed;
- memory remains bounded by effective limits.

### APP-7 — Qualified remote query parity

Depends: `APP-6`, `HAR-4`

Deliver:

- query/explain operations and schemas;
- server-side revalidation and planning;
- typed SDK decoding;
- retry/deadline/cancellation propagation;
- malformed/malicious response tests.

Exit:

- the shared embedded/remote query corpus has identical rows, order, coverage,
  continuation behavior, and error codes.

### APP-8 — Application journey and release evidence

Depends: `APP-1` through `APP-7`

Implements the API/query part of `HAR-6` and supports `HAR-7`.

Deliver:

- one copyable Rust quickstart;
- API reference and RQL Application Core guide;
- migration table from existing SDK methods;
- end-to-end local and remote example;
- benchmark disclosure for the added façade/query path;
- claim-to-evidence entries in `doc/VERIFICATION_STATUS.md`.

Exit:

- a clean consumer crate compiles using only documented public exports;
- the M1 critical journey can create, query, page, inspect history, and manage
  an index through this API;
- all required evidence is reproducible in CI;
- capability documentation says exactly which RQL conformance level shipped.

## 15. Dependency graph and permitted parallel work

```text
APP-0 ── APP-1 ── APP-2 ── APP-3 ──┐
   └──── APP-4 ── APP-5 ────────────┼── APP-6 ── APP-7 ── APP-8
HAR-4 ───────────────────────────────┘
```

`APP-1` and pure compiler work in `APP-4` may run in parallel after `APP-0`,
under the explicit M1 preparation allowance. `APP-3` and `APP-6` cannot be
accepted until the qualified HeapKey posture in `HAR-4` is accepted. `APP-6`
does not begin until the public data types and compiler are both stable.
Node.js work does not begin before `APP-8` acceptance and a separately admitted
binding plan.

## 16. Verification matrix

Every behavior test is parameterized over embedded and qualified remote unless
the test is explicitly transport- or crash-specific.

| Claim | Minimum evidence |
|---|---|
| Heap isolation | same names/keys in two Heaps, swapped credentials, handles, plans, and cursors |
| create atomicity | fault at every create persistence boundary plus reopen/catalog rebuild |
| retry safety | response loss, duplicate delivery, conflicting operation-id reuse |
| API parity | shared CRUD/history/index/query corpus against both backends |
| predicate truth | generated documents against a simple independent model |
| index equivalence | ready/stale/missing/damaged index versus complete scan |
| page correctness | concatenate all page sizes 1, 2, 63, 64, 65, 4096 |
| cursor security | bit flips and every binding-field mismatch |
| mutation fencing | mutation between pages invalidates continuation |
| ordering | all scalar families, mixed families, null, missing, ties |
| coverage honesty | unavailable source never becomes empty or complete |
| budget safety | exact boundary, one beyond, cancellation, server ceiling |
| parser safety | grammar corpus, mutation fuzz, depth/size ceilings |
| protocol safety | malformed frames, wrong types, unknown profiles, oversized values |
| crash safety | reopen after create/write/delete/index/catalog interruption |
| resource safety | bounded memory, handles, sockets, and cursor-key history |

The release suite contains:

- unit and property tests;
- model-based state-machine tests;
- golden canonical plan/cursor/protocol vectors;
- differential embedded/remote tests;
- fault-injection and crash/reopen tests;
- fuzz targets for RQL, predicate, plan, cursor, and wire decoders;
- one long-running concurrent application journey.

No test may call two implementations that share the same evaluator and label
the result independent differential evidence.

## 17. Acceptance checklist

The package is accepted only when every item is true:

- [ ] operation 106 is active and schema-backed;
- [ ] collection creation is crash-safe, idempotent, and Heap-confined;
- [ ] one documented Rust façade covers embedded and qualified remote;
- [ ] public results use typed ids, receipts, history, indexes, pages, and
      errors;
- [ ] unsupported durability is rejected or honestly upgraded and reported;
- [ ] RQL and builder compile to one canonical plan;
- [ ] the shipped language advertises `dql-app-core-v1`, not complete RQL v1;
- [ ] pagination is bounded, authenticated, Heap/plan/parameter-bound, and
      mutation-fenced;
- [ ] complete coverage is the default and false empty results are impossible;
- [ ] shared embedded/remote conformance is green;
- [ ] API, wire, plan, and cursor compatibility fixtures are checked in;
- [ ] docs contain one local and one remote journey;
- [ ] verification status links exact evidence and source revision;
- [ ] Node.js work remains out of scope.

## 18. Decisions intentionally deferred

The following require later, explicit specifications:

- async Rust shape and executor independence;
- Node.js packaging and Promise/stream conventions;
- multi-collection RQL host execution;
- persistent snapshots across mutations;
- bulk write and transactional/Atomic batches;
- exact ranked access;
- arbitrary scale ordering through Residuum Order Wavelets;
- query-plan compatibility across a breaking semantic profile.

Developers MUST NOT fill these gaps inside this package.
