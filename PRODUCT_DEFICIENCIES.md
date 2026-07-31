# Residiuum product deficiency and missing-API register

Status: **normative product-gap inventory v1.0-draft**

Date: 2026-07-31

Audience: product, API, storage, query, Heap, verification, and release
engineering

Companions:

- [DX_SPEC.md](DX_SPEC.md)
- [MASTER_DELIVERY_PLAN.md](MASTER_DELIVERY_PLAN.md)
- [DEFECTS.md](DEFECTS.md)
- [doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md)
- [RQL_SPEC.md](RQL_SPEC.md)
- [RRE_SPEC.md](RRE_SPEC.md)
- [ATOMICS_SPEC.md](ATOMICS_SPEC.md)
- [FUTURE_ROADMAP.md](FUTURE_ROADMAP.md)

## 1. Decision

A defect means an implemented or promised contract is wrong.

A product deficiency means a reasonable application needs an operation,
contract, or supported journey that Residiuum does not presently provide as a
coherent public product—even when lower-level pieces exist.

This register answers:

> If storage qualification passed tomorrow, what would still prevent an
> ordinary developer from choosing Residiuum and building a complete application
> without inventing a database wrapper?

The answer is not “implement everything MongoDB, PostgreSQL, or Couchbase
implements.” Each missing capability is classified as:

```text
CORE       ordinary dependable document-database contract
DEFINING   required for Residiuum's mathematical product proposition
OPERABLE   required to run and evolve the product safely
ECOSYSTEM  required for adoption beyond a Rust-first niche
EXPANSION  valuable, but must not displace the above
REJECTED   deliberately outside the product model
```

## 2. Snapshot and evidence rule

This is an implementation snapshot, not a reading of aspirations.

Status values:

```text
absent
scaffold
partial
implemented-unqualified
specified
qualified
```

A specification does not make an API present. A low-level Store method does
not make a Heap-confined embedded/remote application operation present. An
embedded-only method does not establish backend parity.

The audit uses:

- actual public Rust methods in `residiuum-store` and `residiuum-sdk`;
- reserved/scaffolded methods in `app_v1`;
- active server/Heap RPC dispatch;
- capability and delivery status documents;
- the promised ordinary experience in `DX_SPEC.md`; and
- mature database expectations only where they solve recurring application
  work rather than serve parity theatre.

External calibration:

- MongoDB exposes per-operation bulk results, resumable collection/deployment
  change streams, update operators/upsert, and schema validation:
  [bulk write](https://www.mongodb.com/docs/manual/reference/command/bulkwrite/),
  [change streams](https://www.mongodb.com/docs/manual/changeStreams/),
  [update methods](https://www.mongodb.com/docs/manual/reference/update-methods/index.html),
  and
  [schema validation](https://www.mongodb.com/docs/manual/core/schema-validation/).
- Couchbase exposes opaque compare-and-swap versions and atomic sub-document
  mutations:
  [concurrent mutations](https://docs.couchbase.com/rust-sdk/current/howtos/concurrent-document-mutations.html)
  and
  [sub-document operations](https://docs.couchbase.com/rust-sdk/current/howtos/subdocument-operations.html).
- PostgreSQL exposes commit-coupled asynchronous notification, while explicitly
  documenting setup races and delivery timing:
  [LISTEN](https://www.postgresql.org/docs/current/sql-listen.html) and
  [NOTIFY](https://www.postgresql.org/docs/current/sql-notify.html).

These examples establish recurring application needs. They do not dictate
Residiuum's syntax, implementation, or guarantees.

## 3. Product baseline

Before RRE, Atomics, Direct Access, search, or cluster breadth becomes the
principal product program, a developer must be able to:

```text
open a Heap
→ provision and retire collections
→ create/add/get/inspect/replace/patch/delete values safely
→ use optimistic concurrency
→ stream bounded bulk operations
→ query/page/count under an explicit read view
→ read exact history and recover prior evidence
→ subscribe from a resumable change checkpoint
→ import/export without whole-dataset materialization
→ discover effective limits and capabilities
→ operate the same semantics locally and remotely
```

This baseline is called `dingo-application-baseline-v1`.

## 4. Definitive deficiency register

### 4.1 Core application surface

| ID | Missing product contract | Current truth | Class | Priority |
|---|---|---|---|---|
| `PD-001` | One backend-neutral, Heap-bound Rust client | `HeapClient`/`CollectionClient` are contract scaffolds; legacy embedded and remote surfaces differ | CORE | `C0` |
| `PD-002` | Conditional create, replace, and delete | Put/delete exist; ordinary version/CAS mutation API is absent | CORE | `C0` |
| `PD-003` | Document-path lookup and atomic mutation | Applications must transfer/fetch/modify/replace whole JSON values | CORE | `C0` |
| `PD-004` | Generated-key add and explicit upsert | Described by DX; not one qualified public operation | CORE | `C0` |
| `PD-005` | Bounded bulk mutation with per-item truth | Store `put_many` exists; no complete Heap/remote ordered/unordered result contract | CORE | `C0` |
| `PD-006` | Exact historical and last-complete reads | History exists; historical chunk bodies and recovery selection are incomplete | CORE | `C0`, DEF-099 |
| `PD-007` | Coverage-aware key/document enumeration | Lower partial-aware pieces exist; ordinary Collection/backend parity is absent | CORE | `C0`, DEF-100 |
| `PD-008` | Stable snapshot/read-view API | Cursors are generation-fenced; no durable read view for long scan/export/query composition | CORE | `C0` |
| `PD-009` | Complete RQL Application Core execution | Specification exists; public compiler/executor/remote parity remain APP scaffolds | CORE | `C0` |
| `PD-010` | Exists/count/distinct and bounded aggregation baseline | Applications must materialize or hand-roll scans | CORE | `C1` |
| `PD-011` | Collection lifecycle | Create/list/open exist or are planned; describe/rename/retire/drop/purge contracts are incomplete | CORE/OPERABLE | `C0` |
| `PD-012` | Resumable change feed/watch | History is pull-by-key; no collection/Heap committed-event subscription | CORE | `C1` |
| `PD-013` | Streaming resumable import/export | Low-level export/salvage exists; application formats/checkpoints/parity are absent | CORE/OPERABLE | `C1` |

### 4.2 Mathematical and integrity surface

| ID | Missing product contract | Current truth | Class | Priority |
|---|---|---|---|---|
| `PD-014` | Scoped Atomics | Fully specified; not implemented as application operations | DEFINING | `C1` |
| `PD-015` | RRE document rules | Fully specified; compiler/runtime/activation absent | DEFINING | `C1` |
| `PD-016` | Referential integrity and cross-document RRE | Specified after document rules/Atomics; absent | DEFINING | `C1` |
| `PD-017` | Unique, compound, partial, and sparse index contracts | Basic field indexes exist; constraint-grade index semantics are incomplete | CORE/DEFINING | `C1` |
| `PD-018` | Rule/index/Atomic examination through SDA | Designs exist; one stable application and Studio inspection API is absent | DEFINING | `C1` |
| `PD-019` | Collection-level jurisdiction/default scoping | Proposal exists; no qualified public definition/enforcement surface | DEFINING | `C2` |

### 4.3 Lifecycle and operational surface

| ID | Missing product contract | Current truth | Class | Priority |
|---|---|---|---|---|
| `PD-020` | Per-value expiry and retention policy | No first-class TTL/expiry/legal-hold semantics | OPERABLE | `C1` |
| `PD-021` | Unified background-job API | Compaction/scrub/migration expose different low-level controls | OPERABLE | `C1` |
| `PD-022` | Heap-level backup/restore/retire/export | Store operations exist; authenticated Heap-scoped product journey remains incomplete | OPERABLE | `C0` |
| `PD-023` | Capability/limit/policy discovery | Limits live across Store, SDK, RPC, and config; applications cannot negotiate one effective contract | CORE/OPERABLE | `C0` |
| `PD-024` | Schema/rule/index evolution and migration planner | Individual migrations exist; application data-model evolution is not one resumable plan | OPERABLE | `C1` |
| `PD-025` | Online maintenance policy API | Low-level operations exist; schedule/budget/plan/apply/status policy is incomplete | OPERABLE | `C2` |
| `PD-026` | Encryption-at-rest and field/key policy surface | Doctrine exists; ordinary provision/rotate/rekey/status APIs are absent | OPERABLE | `C2` |
| `PD-039` | Heap-scoped quotas and resource-policy administration | Host budgets exist; application/operator quota/status/change contracts are incomplete | OPERABLE | `C2` |

### 4.4 SDK and ecosystem surface

| ID | Missing product contract | Current truth | Class | Priority |
|---|---|---|---|---|
| `PD-027` | Async Rust client and bounded connection pool | Current intended v1 façade is synchronous | ECOSYSTEM | `C1` |
| `PD-028` | Node.js/TypeScript client | Intentionally deferred until Rust/wire contracts stabilize | ECOSYSTEM | `C2` |
| `PD-029` | Stable language-neutral HTTP/JSON or equivalent gateway | Residiuum RPC is internal/product-specific; no broad client bridge | ECOSYSTEM | `C2` |
| `PD-030` | Application test utilities | No complete temporary Heap, deterministic clock/fault, fixture, and assertion kit | ECOSYSTEM | `C1` |
| `PD-031` | Migration adapters from JSON files, Mongo-style data, and SQL rows | Cross-compilers cover languages/rules, not operational data migration | ECOSYSTEM | `C2` |
| `PD-032` | Stable SQL→RQL and JSON Schema→RRE product commands | Specifications exist; shipped compiler/CLI/library contracts are absent | ECOSYSTEM | `C2` |
| `PD-040` | Prepared/registered query plans | Plans are designed; no stable prepare/bind/execute/invalidate lifecycle | CORE/ECOSYSTEM | `C2` |
| `PD-041` | Bounded query-driven update/delete | Query and point mutation are separate; no safe planned multi-match mutation | CORE | `C2`, after Atomics |

### 4.5 Deliberate expansion

| ID | Capability | Disposition |
|---|---|---|
| `PD-033` | Full aggregation/data-processing framework | Deliver a bounded baseline first; do not clone Mongo pipelines blindly |
| `PD-034` | Text search | First search expansion after core/defining contracts |
| `PD-035` | Vector search | After text and shared derived-index substrate |
| `PD-036` | Geospatial search | After vector unless a concrete customer workload changes priority |
| `PD-037` | Native object-store archive, lifecycle, erasure | Separate archive product milestone |
| `PD-038` | Production multi-node cluster | Separate qualification milestone; not an embedded baseline dependency |

## 5. Exact missing core contracts

### 5.1 PD-001 — One real application client

Required:

```rust
pub struct HeapClient { /* Heap-bound backend */ }
pub struct CollectionClient { /* HeapId + CollectionId bound */ }

impl HeapClient {
    pub fn create_collection(...);
    pub fn open_collection(...);
    pub fn list_collections(...);
    pub fn capabilities(...);
}
```

Every ordinary method uses the same types and semantics for embedded and
qualified remote execution. Backend-specific handles remain advanced adapters,
not the documentation default.

Acceptance:

- one consumer crate changes only its constructor between local and remote;
- no ordinary method accepts a caller-supplied Heap or collection identifier;
- no raw tuple/wire JSON escapes the adapter; and
- all lower outcomes have total public projections.

### 5.2 PD-002 — Conditional single-key mutations

Required:

```rust
create(key, value, CreateOptions)
replace(key, value, ReplaceOptions { if_version })
delete_with(key, DeleteOptions { if_version, if_present })
upsert(key, value, UpsertOptions)
```

Semantics:

- `create` succeeds only under proven current absence;
- `replace` succeeds only for the exact observed version;
- conditional delete cannot delete a newer replacement;
- `upsert` explicitly reports inserted versus replaced;
- all operations accept stable operation IDs and return achieved durability;
- version checks and mutation are one Key Atomic; and
- damage/incomplete coverage prevents an absence-dependent create/upsert.

This is the minimum optimistic-concurrency contract. Mature document stores
expose an opaque version/CAS for this reason; applications should not build
lost-update prevention themselves.

### 5.3 PD-003 — Document-path lookup and atomic mutation

Required first profile:

```rust
lookup(key, [Get(path), Exists(path)], LookupOptions)

mutate(
    key,
    [
        Set(path, value),
        Remove(path),
        Increment(path, checked_delta),
        ArrayAppend(path, values),
        Test(path, predicate),
    ],
    MutateOptions { if_version, create_parents, durability },
)
```

Rules:

- all lookup paths observe one document version and avoid returning unrelated
  document bytes where the storage/layout profile can do so;
- all operations apply to one decoded document version or none apply;
- paths and values use canonical RQL/RRE scalar semantics;
- arithmetic is checked;
- binary values reject document mutation;
- RRE validates the proposed final document;
- receipt includes prior/new version and per-operation result;
- failure does not publish a partial document; and
- implementation MAY rewrite the physical document initially, but the public
  contract must not require a client read/modify/write race.

This directly reduces repeated large-document rewrites and is conceptually
similar to mature document-store sub-document operations, without importing
their syntax.

### 5.4 PD-004 — Generated-key add and explicit upsert

Required:

```rust
add(value, AddOptions) -> AddReceipt { key, write }
upsert(key, value, UpsertOptions) -> UpsertReceipt { action, write }
```

Generated keys are collision-safe, returned explicitly, stable as opaque
values, and optionally sortable only under a named key profile. An upsert
never guesses absence from incomplete coverage.

### 5.5 PD-005 — Bounded bulk mutation

Required:

```rust
bulk_write(
    impl Iterator<Item = BulkOperation>,
    BulkOptions {
        ordered,
        max_in_flight,
        result_page_size,
        durability,
        atomic_scope,
    },
) -> BulkResultStream
```

Each input has an operation ID and exactly one result:

```text
committed(receipt)
rejected(error)
uncertain(recovery handle)
not_attempted(reason)
```

Bulk is bounded and streaming. `ordered=false` may continue after item failure.
Bulk does not imply cross-key atomicity. If `atomic_scope` is requested, the
operations compile to PD-014 and reject before effect when the scope is
invalid.

### 5.6 PD-006/007 — Recovery and coverage-aware enumeration

The exact contracts are governed by DEF-099 and DEF-100:

```rust
get_version(key, event_id)
find_last_complete(key, options)
scan_keys_page(options)
scan_json_partial_page(options)
```

These are baseline product APIs, not operator-only escape hatches.

### 5.7 PD-008 — Stable read views

Required:

```rust
let view = heap.read_view(ReadViewOptions {
    consistency,
    max_age,
    retention_budget,
})?;

view.collection("orders").query(...);
view.export(...);
view.close();
```

A read view binds:

```text
HeapId
authoritative frontier(s)
coverage
rule/index/query semantic versions
expiry
resource budget
```

All cursors derived from the view observe the same declared state. The first
profile may be local and bounded-duration. It must pin only the minimum
required authority/reclamation frontier and fail explicitly when the retention
budget cannot be honored.

Generation-fenced restart-on-mutation cursors remain useful; they are not a
replacement for a consistent export or multi-query read.

### 5.8 PD-009 — RQL Application Core

The existing APP-4–APP-7 plan remains correct but incomplete in implementation.
The baseline needs:

```rust
collection.query()
collection.dql(source, parameters, options)
collection.explain_dql(...)
```

with identical builder/RQL plans, deterministic order, bounded pages,
authenticated continuations, complete-by-default coverage, budgets,
cancellation, and embedded/remote parity.

### 5.9 PD-010 — Bounded aggregate baseline

Do not begin with an unrestricted pipeline language. Provide:

```text
exists(filter)
count(filter, CountOptions)
distinct(path, filter, DistinctOptions)
group_count(paths, filter, GroupOptions)
numeric min/max/sum/average(path, filter, AggregateOptions)
```

Every result carries:

```text
coverage
consistency/read view
documents/bytes examined
overflow/precision policy
continuation or explicit non-pageability
```

Incomplete coverage cannot produce an exact count or aggregate. Approximate
results require a separately named type and error bound.

### 5.10 PD-011 — Collection lifecycle

Required:

```rust
describe_collection
rename_collection
retire_collection
restore_retired_collection
plan_purge_collection
purge_collection
```

Rules:

- rename changes a name binding, not immutable `CollectionId`;
- retirement immediately prevents new ordinary writes but preserves
  examination/history;
- name reuse is explicit and cannot alias stale handles/cursors;
- purge is privileged, planned, evidenced, retention-aware, and irreversible;
- all lifecycle changes are idempotent operations with receipts; and
- collection drop is never a casual boolean convenience method.

### 5.11 PD-012 — Resumable watch/change feed

Required first profile:

```rust
collection.watch(WatchOptions {
    from,
    include_values,
    filter,
    heartbeat,
    batch_size,
}) -> ChangeStream
```

Each event includes:

```text
HeapId / CollectionId
event_id / item_id
kind
commit/durability position
current rule profile
optional before/after version references
coverage/gap/replay evidence
resume checkpoint
```

The first contract is at-least-once, ordered within its declared authority
scope, resumable within retention, and explicit about gaps. Notifications are
published only after the mutation reaches the watch's declared durability.
Exactly-once application effects are not claimed.

### 5.12 PD-013 — Streaming import/export

Required:

```rust
import_jsonl / import_json / import_bytes_tree
export_jsonl / export_bytes_tree / export_sda / export_evidence
```

Operations:

- stream input/output;
- use bounded memory and backpressure;
- checkpoint/resume;
- report per-item errors and uncertainty;
- bind Heap/collection/read view/rule profile;
- preserve unsupported or invalid material as explicit opaque evidence when
  requested; and
- never make “skip bad row” an implicit policy.

## 6. Exact defining and operational contracts

### 6.1 PD-014 — Atomics

Use [ATOMICS_SPEC.md](ATOMICS_SPEC.md), not a generic transaction façade.
Required progression:

```text
Key Atomic
→ LocalHeap/partition-scoped Atomic batch
→ explicit multi-partition workflow records
```

A `transaction` compatibility method may compile to the same plan, but cannot
widen its scope or guarantee.

### 6.2 PD-015/016 — RRE and relationships

Use [RRE_SPEC.md](RRE_SPEC.md). Document-local deterministic rules precede
referential integrity. Activation over existing data is a resumable validation
job whose successful frontier is committed atomically.

### 6.3 PD-017 — Index completeness

Required sequence:

1. compound indexes with explicit ordering/collation;
2. partial/sparse indexes whose predicate is canonical and versioned;
3. unique indexes only inside a declared Atomic enforcement scope;
4. index build status, coverage, failure, pause/resume, and rebuild; and
5. query hints only as verified planning constraints, never correctness
   bypasses.

### 6.4 PD-020 — Expiry, retention, and legal hold

Expiry is not a background delete timer alone.

Required model:

```text
visible_until
retention_until
purge_eligible_after
legal_hold
policy_id/version
```

Logical expiry appends or derives an evidenced transition according to policy.
Physical reclamation remains separate, asynchronous, observable, and blocked
by snapshots, backups, holds, and retention. Watches and history disclose
expiry. Wall-clock uncertainty cannot silently purge data.

### 6.5 PD-021 — Unified jobs

All long-running operations implement:

```rust
plan()
start(operation_id)
status(job_id)
pause(job_id)
resume(job_id)
cancel(job_id)
events(job_id, checkpoint)
```

Job state is durable, Heap-confined, restartable, and exposes bytes/items
planned, processed, remaining, failures, coverage, and authority effect.

### 6.6 PD-023 — Capability discovery

Required:

```rust
heap.capabilities() -> CapabilityDocument
```

It includes stable profile identifiers and effective:

```text
operations and dialects
durability/consistency modes
logical payload/chunk/frame limits
query/page/result budgets
Atomic scopes
RRE profiles
index kinds
watch retention
backup/import/export formats
server/cluster maturity
```

Clients negotiate before work. Unsupported options reject; they are never
ignored or silently weakened.

## 7. Ecosystem minimum

### 7.1 Async Rust

Async is required before a network-server product claim. It must provide
deadlines, cancellation, streaming pages/results, bounded pooling, reconnect,
retry classification, and backpressure without changing semantics from the
synchronous client.

### 7.2 Node.js/TypeScript

Node follows the stable Rust and wire contracts. It must not expose raw RPC
JSON as the public API. Promise, async-iterator, cancellation, bytes, bigint,
error, and packaging semantics receive their own conformance plan.

### 7.3 Test utilities

Official utilities provide:

```text
temporary isolated Heap
deterministic key/clock/operation-id sources
fault and crash child harness
fixture import
assertion helpers for receipts/coverage/damage
embedded/remote shared behavior runner
```

An application should be able to test retry, damage, partial coverage, and
version conflicts without copying Residiuum's internal test harness.

## 8. Explicitly deferred or rejected

The following are not baseline deficiencies:

- offset pagination;
- arbitrary stored procedures, triggers, or Turing-complete callbacks;
- implicit cross-partition transactions;
- silent validation bypass;
- automatic repair that replaces conflicting evidence;
- exactly-once change-stream side effects;
- unbounded aggregation pipelines;
- ODM/ORM magic that weakens typed errors or coverage;
- global uniqueness without a declared coordination scope; and
- text/vector/geospatial search before the common derived-index substrate and
  application baseline qualify.

These are deliberate product choices, not forgotten APIs.

## 9. Required delivery order

The recommended order after DEF-098–DEF-104 and Core Storage Qualification is:

```text
Application Foundation
  PD-001 coherent client
  → PD-002 conditional writes
  → PD-004 add/upsert
  → PD-006/007 recovery and coverage
  → PD-011 collection lifecycle
  → PD-023 capability discovery

Application Data Plane
  PD-003 document mutation
  → PD-005 bulk
  → PD-008 read views
  → PD-009 RQL
  → PD-010 aggregate baseline
  → PD-012 watches
  → PD-013 import/export

Mathematical Product
  PD-015 document RRE
  → PD-014 LocalHeap Atomics
  → PD-016 relationships
  → PD-017 constraint-grade indexes

Operability and Adoption
  PD-020 retention
  → PD-021 unified jobs
  → PD-024 evolution planner
  → PD-027 async Rust
  → PD-028 Node.js
```

Some independent work may run in parallel, but no later group changes the
semantic decisions of an earlier group.

## 10. Work-package recommendation

Do not create 41 simultaneous projects. Admit four bounded programs:

```text
APB — Application Baseline
  PD-001…PD-013 and PD-023

MAT — Mathematical Product
  PD-014…PD-019 and PD-041

OPS — Lifecycle and Operability
  PD-020…PD-026 and PD-039

ECO — Ecosystem
  PD-027…PD-032 and PD-040
```

`PD-033…PD-038` remain expansion backlog.

The existing APP plan supplies much of APB but must be amended: its current
scope deliberately excludes conditional create/replace/delete, document-path
mutation, generated-key add, bulk mutation, stable read views, aggregates,
watches, collection retirement, and import/export. Those omissions are no
longer acceptable for `dingo-application-baseline-v1`.

## 11. Baseline acceptance

`dingo-application-baseline-v1` accepts only when:

- one Heap-bound client works locally and remotely;
- create/add/get/inspect/replace/mutate/delete expose exact version semantics;
- bulk work is bounded and every input has a truthful outcome;
- key/document scans distinguish empty from incomplete;
- long scans and exports can bind one declared read view;
- RQL, builder, index, and slow scan agree;
- counts/aggregates carry coverage and bounds;
- history and explicit prior-version recovery work;
- collections can be retired and purged safely;
- watches resume or expose gaps;
- import/export resume with per-item evidence;
- clients can inspect effective capabilities and limits;
- errors never collapse to defaults/absence;
- the same behavior suite passes embedded and qualified remote surfaces; and
- the complete packaged application journey passes the verification strategy.

Only then is it reasonable to say:

> Residiuum provides an ordinary document-database application experience.

Atomics, RRE, Direct Access, search, archive, and cluster then extend a complete
product rather than compensating for a missing baseline.
