# DingoDB Developer Experience Specification

Status: Draft v0.1  
Scope: Everyday API, query surface, CLI, errors, defaults, administration, and
progressive disclosure

## 1. Product requirement

DingoDB's unusual storage and recovery machinery MUST produce an ordinary,
pleasant database experience.

A developer who does not care about physical damage, archival formats,
consensus, or SDA should still choose DingoDB because it is an easy and fast
place to put JSON and bytes.

The everyday promise is:

> Open it. Put anything in. Get it back.

The advanced promise remains:

> When ordinary assumptions fail, DingoDB tells you exactly what survived.

## 2. DX success criteria

The initial stable release MUST make these journeys straightforward:

1. create a local database and store a JSON value in under one minute;
2. store and retrieve bytes without manual encoding;
3. query common JSON fields without learning SDA;
4. add an index without stopping or rewriting the database;
5. inspect history for one key;
6. stream a dataset larger than memory;
7. move from embedded use to a server without rewriting application logic;
8. connect to a cluster using the same data API;
9. understand whether an acknowledged write is memory, buffered, durable, or
   replicated;
10. diagnose and salvage damage using one discoverable tool.

No journey above may require understanding frames, segments, terms, placement
epochs, or recovery evidence unless the journey encounters a condition where
that information matters.

## 3. Principles

### 3.1 Zero ceremony

Opening a path creates or opens a database. A separate server, schema,
manifest, migration, or initialization command is not required for embedded
use.

### 3.2 Safe by default

Defaults favor durable, unsurprising behavior.

Faster, weaker acknowledgement modes are explicit. An API MUST NOT label a
memory or buffered acknowledgement “durable.”

### 3.3 Simple things are simple

Key lookup, JSON filtering, byte storage, pagination, and batch ingestion have
first-class APIs.

SDA is not required for common application operations.

### 3.4 Power through progressive disclosure

The product surface has layers:

1. values and collections;
2. indexes, history, batches, and watches;
3. SDA examination and coverage;
4. recovery evidence, holes, and physical provenance;
5. cluster placement and consensus diagnostics.

A user enters a deeper layer only when they request it or when correctness
requires surfacing it.

### 3.5 One mental model

Embedded, server, and clustered deployments expose the same logical collection
API.

Deployment changes connection and operational configuration, not application
data semantics.

### 3.6 No silent uncertainty

An ordinary result is ordinary only when DingoDB can support the claim it
makes.

A missing key, unavailable partition, stale incomplete index, damaged payload,
and resource-limited search are different outcomes.

### 3.7 Queries always have a correct path

A missing secondary index MUST NOT make a valid query illegal.

DingoDB may scan, ask for an explicit budget, or report that required tiers are
offline. It MUST NOT return a knowingly incomplete empty result.

## 4. Deployment model

### 4.1 Embedded

Canonical shape:

```ts
const db = await Dingo.open("./app.dingo");
```

If the path does not exist, it is created with safe defaults.

If it exists, DingoDB discovers the format and opens it without requiring a
separate manifest database.

Opening MUST NOT perform an unbounded full-store scan on the latency-sensitive
path when valid acceleration metadata exists.

### 4.2 Server

Canonical shape:

```ts
const db = await Dingo.connect("dingo://localhost:7434/app");
```

The logical API is the same as embedded use. Network-only concerns such as
authentication, deadlines, and retry policy are connection options.

### 4.3 Cluster

Canonical shape:

```ts
const db = await Dingo.connect("dingo://db.example.com/app");
```

Clients discover and cache partition routes automatically.

Ordinary SDK users do not manually select leaders, terms, replica sets, or
placement epochs.

Stale routing is retried safely using stable event identifiers.

## 5. Everyday logical model

### 5.1 Database

A database contains named collections and streams.

Physical segments, chunks, indexes, and tiers are not part of the everyday
logical namespace.

### 5.2 Collection

A collection maps stable keys to current values while retaining immutable
history underneath.

A collection accepts:

- structured values representable by the host SDK;
- raw bytes;
- explicit external references;
- optional metadata.

A collection does not require a schema. A schema MAY be attached later for
validation and tooling.

### 5.3 Key

A key is a non-empty byte string with an SDK-native UTF-8 convenience form.

SDKs MUST preserve arbitrary byte keys losslessly when their host language can
represent them.

Human-facing tools use a reversible escaped representation for non-text keys.

### 5.4 Value

The everyday SDK distinguishes:

- `JsonValue`;
- `Bytes`;
- `ExternalReference`.

SDKs MUST NOT silently convert invalid text bytes into replacement characters.

### 5.5 Item

Advanced item reads expose:

```ts
type Item<T> = {
  key: Key;
  value: T;
  version: Version;
  createdAt?: Timestamp;
  updatedAt?: Timestamp;
  metadata: Record<string, JsonValue>;
  health: "complete" | "partial" | "uncertain";
};
```

The simple API returns values. The inspection API returns items and evidence.

### 5.6 Stream

A stream is an append-oriented collection of events with generated or
caller-supplied identifiers.

Streams do not imply one global order. Ordering guarantees are declared by the
stream partition key and consistency mode.

## 6. Core API

Examples use TypeScript-like pseudocode. SDKs use idiomatic host-language
naming while preserving semantics.

### 6.1 Collection access

```ts
const users = db.collection<User>("users");
```

Collection access is lazy. Merely naming a collection does not require a
network or disk mutation.

The first write MAY create collection metadata atomically.

### 6.2 Put

```ts
const receipt = await users.put("user-42", {
  name: "Alice",
  status: "active"
});
```

`put` creates or replaces the current value for the key by appending an
immutable event.

It returns a receipt containing:

```ts
type WriteReceipt = {
  key: Key;
  eventId: EventId;
  version: Version;
  acknowledgement:
    | "memory"
    | "buffered"
    | "durable"
    | "replicated";
  committed: boolean;
  partition?: PartitionId;
};
```

The receipt MUST reflect the actual achieved guarantee, not merely the
requested option.

### 6.3 Create

```ts
await users.create("user-42", value);
```

`create` succeeds only when the key has no visible current value under the
requested consistency mode.

An existing value produces `AlreadyExists`.

### 6.4 Replace with version

```ts
await users.replace("user-42", nextValue, {
  ifVersion: current.version
});
```

Version-conditional replacement is the ordinary optimistic-concurrency
mechanism.

A mismatch produces `VersionConflict` containing expected and observed
versions when disclosure policy permits.

### 6.5 Get

```ts
const user = await users.get("user-42");
```

The result is:

- the complete current value;
- `null` only when absence is established for the declared read scope;
- a typed error when completeness cannot be established or the payload is
  damaged.

`get` MUST NOT return `null` for an unavailable partition, offline required
tier, incomplete index, or unreadable current frame.

### 6.6 Inspect

```ts
const item = await users.inspect("user-42");
```

`inspect` returns the item, integrity state, provenance, holes, coverage, and
uncertainty according to the DingoDB SDA profile.

It is the escape hatch when `get` cannot honestly return one ordinary value.

### 6.7 Delete

```ts
await users.delete("user-42");
```

Delete appends a tombstone. It does not physically purge historical bytes.

Deleting an absent key is idempotent by default and reports whether a visible
value changed.

Physical purge is a separate privileged retention operation.

### 6.8 Add with generated key

```ts
const receipt = await events.add(event);
```

`add` generates a stable key and returns it in the receipt.

Generated-key format is documented and sortable when the selected profile
claims sortability.

### 6.9 Bytes

```ts
const artifacts = db.collection<Bytes>("artifacts");
await artifacts.put("build-19", bytes);
const bytes = await artifacts.get("build-19");
```

Byte storage is a first-class operation. SDK callers do not base64-encode data
manually.

Large values are chunked transparently.

### 6.10 Bulk writes

```ts
const result = await users.putMany(entries, {
  durability: "durable",
  concurrency: 32
});
```

Bulk APIs stream input and bounded results. They do not require the complete
batch in memory.

The result identifies every accepted, rejected, committed, uncertain, and
failed entry.

Bulk operation does not imply cross-partition atomicity.

## 7. Query experience

### 7.1 Familiar filters

Common queries use SDK-native filters:

```ts
const result = users.find({
  status: "active",
  age: { $gte: 18 },
  country: { $in: ["TH", "SG"] }
});
```

The initial portable filter vocabulary includes:

- equality and inequality;
- `<`, `<=`, `>`, `>=`;
- membership;
- existence;
- prefix;
- containment for strings and sequences;
- boolean `and`, `or`, and `not`.

SDK filters compile to SDA with identical absence, `Null`, comparison, and
failure semantics.

### 7.2 Fluent builder

SDKs SHOULD also provide a typed builder:

```ts
const result = users
  .query()
  .where("status").eq("active")
  .and("age").gte(18)
  .orderBy("created_at", "desc")
  .limit(100);
```

Builders MUST generate a serializable query plan suitable for embedded,
server, and cluster execution.

### 7.3 Streaming

Query results are async streams or host-language equivalents:

```ts
for await (const row of result) {
  consume(row);
}
```

Streaming is the default for unbounded results.

Materializing helpers such as `toArray()` MUST require an explicit or
configured result limit.

### 7.4 Pagination

Pagination uses opaque authenticated continuation tokens.

Offset pagination MAY be offered for small stable result sets but is not the
default for massive data.

Continuation results retain query identity, order, scope, and coverage.

### 7.5 Sorting

A query that returns a `Seq` requires deterministic ordering.

When no order is requested, SDKs MUST document whether the result is an
unordered stream, stable identity order, or another profile-defined order.

Worker completion and filesystem enumeration order are never observable query
semantics.

### 7.6 Raw SDA

Advanced users can evaluate SDA (including the ENR1 enrichment kernel) as
**text**. Fluent filters and multi-collection equijoins do not replace this path.

```ts
const result = users.sda(`
  { yield u | u in input
      | getPath(u, Seq["status"]) = Some("active")
    }
`);
```

Multi-collection programs bind named collections under `input` as a map of
document arrays **and** as free names (host scan first; pure eval second).
Preferred enrichment surface uses ENR1 `Match` + `enrich` pipe sugar:

```ts
const attached = db.enrQuery()
  .bind("orders")
  .bind("customers")
  .run(`
    orders
    |> enrich {
        customer:
          one!(
            Match(
              l,
              customers,
              getPath(l, Seq["customer_id"]),
              getPath(r, Seq["id"])
            )
          )
      }
    |> refine {
        yield o + Map{
          "customer_name" -> getPath(o, Seq["customer", "name"])
        }
        | o in _
      }
  `);
```

`Match(l, R, kL, kR)` is the ENR1 primitive match bag (returns `Bag`).
`enrich { field: E }` attaches evaluated fields to each left row (`l` bound).
`refine { … }` is verb sugar for a bare SDA comprehension over the pipe `_`.

Verbose comprehensions (`bindOpt` / `{ yield o + Map{…} | o in orders }`) remain
valid. The host supplies bounded streams or pages. SDA remains pure.

Raw SDA/ENR text is an advanced capability, not the only query interface.

### 7.7 Query dialects

SDA (with ENR1) is the **mathematical** query language (see [SDA_SPEC.md](SDA_SPEC.md)).
Pure ENR + SDA is exact and often unpleasant to write; a small loud audience
will prefer it. Everyone else uses **dialects**: frontends that compile into
the same ENR+SDA IR and never redefine algebra semantics. This is **not** a
hybrid of co-equal languages.

**DQL (Dingo Query Language) is the official human dialect** — co-designed with
ENR for readable enrichment and nested projection. It is not SQL; foreign
SQL/Mongo/GraphQL surfaces remain optional comfort with known holes. Design:
[DQL_SPEC.md](DQL_SPEC.md).

**Null vs absence is the hard case.** SQL `NULL` and Mongo-style filters cannot
losslessly encode SDA’s distinction between a stored `null` and a missing key
(SDA_SPEC §4.0.1). If callers must separate those, they write pure SDA (or ENR
text), or DQL once it covers that construct faithfully. Foreign dialects may
approximate and MUST attach notes or refuse rather than quietly redefine the
algebra.

```ts
// Pure SDA (always available) — only path for exact null vs absence, etc.
await users.sda(`{ yield u | u in input | getPath(u, Seq["status"]) = Some("active") }`);

// Official human dialect (design; compile → same IR as pure ENR+SDA)
// await db.queryDialect("dql", `from orders\nenrich customer using customers …`);

// JSON / Mongo-style filter dialect (already the portable §7.1 object)
await users.find({ status: "active", age: { $gte: 18 } });

// Explicit dialect id (SDK: find_dialect / compile_dialect)
await users.findDialect("sql", "SELECT * WHERE status = 'active' AND age >= 18");
await users.findDialect("json", `{"status":"active"}`);
```

Builtin dialect ids:

| Id | Role |
|----|------|
| `sda` | Pure SDA/ENR1 text (parse-checked) |
| `dql` | **Official** Dingo Query Language → ENR1+SDA ([DQL_SPEC.md](DQL_SPEC.md); v0.1) |
| `json` / `mongo` | DX portable filter object → document predicate |
| `sql` | Partial `SELECT` / `WHERE` mimicry (foreign comfort; not DQL) |
| `graphql` | Reserved; not implemented |

None of the *foreign* dialects is a complete encoding of SDA, SQL, MongoDB, or
GraphQL. Mimicry is intentional: the product offers the pure language (hard),
DQL as the official human surface, plus comfortable foreign options. Hosts MAY
register additional dialects that compile to pure SDA / shared IR.

Normative detail: [doc/SDA/DIALECTS.md](doc/SDA/DIALECTS.md), [DQL_SPEC.md](DQL_SPEC.md).
Rust surface: `dingo-sdk::dialects` (`compile_dialect`, `DialectRegistry`,
`QueryDialect`).

### 7.8 Explain

```ts
const plan = await result.explain();
```

Explain output contains:

- indexes selected;
- partitions and tiers expected;
- scan estimates;
- ordering plan;
- SDA pushdown;
- dialect id and compiled pure SDA (when a dialect was used);
- consistency mode;
- coverage limitations;
- resource budget;
- whether absence can be proven.

Explain has a concise human form and stable structured form.

## 8. Index experience

### 8.1 Correct without an index

Every valid filter has a scan path over available authoritative data.

For an expensive scan, the SDK MAY require confirmation of a declared time,
byte, tier, or cost budget. It MUST explain why.

### 8.2 Create index

```ts
await users.indexes.create("by-email", {
  fields: ["email"]
});
```

Index creation is online and resumable.

Applications may continue to read and write while it builds.

The index exposes `building`, `ready`, `stale`, `partial`, `failed`, and
`rebuilding` states.

### 8.3 Automatic use, explicit creation

The query planner uses applicable indexes automatically.

DingoDB SHOULD recommend indexes using observed query patterns and estimated
benefit. It MUST NOT silently create unbounded durable indexes by default.

Development mode MAY offer explicitly enabled automatic indexes with a clear
storage budget.

### 8.4 Index deletion

Deleting an index never deletes authoritative data.

A query previously accelerated by that index remains correct through another
index or scan path.

### 8.5 Uniqueness

Unique constraints require a consistency scope.

The API MUST reject a unique index whose requested scope cannot be enforced by
the collection's partitioning and consistency mode.

It MUST NOT present best-effort duplicate detection as a unique constraint.

## 9. Batches and transactions

### 9.1 Single-key atomicity

Single-key writes and version checks are atomic within their partition.

This is the baseline transaction guarantee.

### 9.2 Partition batch

```ts
await db.batch({ partitionKey: "account-42" }, batch => {
  batch.put(accounts, "account-42", nextAccount);
  batch.add(entries, ledgerEntry);
});
```

A partition batch commits atomically when every operation resolves to the same
partition and the selected profile supports it.

The SDK validates the partition scope before submission when possible.

### 9.3 Cross-partition workflow

The initial profile does not expose a misleading general transaction API.

Cross-partition work uses idempotent events, sagas, or explicit workflow
records.

SDK helpers MAY support those patterns while preserving partial progress for
inspection.

## 10. History and time

### 10.1 History

```ts
for await (const version of users.history("user-42")) {
  inspect(version);
}
```

History returns immutable events, tombstones, conflicts, and known gaps.

### 10.2 Version read

```ts
const old = await users.getVersion("user-42", version);
```

A missing historical dependency or hole is explicit.

### 10.3 Time read

```ts
const old = await users.getAt("user-42", timestamp);
```

Time-based reads declare their clock and ordering assumptions.

If wall-clock evidence cannot establish one state, the result is conflicting
or uncertain rather than arbitrarily selected.

## 11. Watches and change streams

```ts
for await (const change of users.watch({ from: checkpoint })) {
  consume(change);
}
```

Watch checkpoints are opaque, durable, and resumable within their retention
contract.

A watch reports:

- event identity;
- partition and position;
- delivery semantics;
- gaps;
- replays;
- current coverage.

At-least-once delivery is the default. Event identifiers make consumer
deduplication straightforward.

Exactly-once application effects are not implied.

## 12. Durability experience

### 12.1 Defaults

Embedded and single-server stores default to `durable`.

Clusters default to `partition-linearizable` writes with replicated quorum
durability when the deployment provides a quorum.

A development profile may choose weaker durability only after displaying and
persisting the choice.

### 12.2 Per-operation override

```ts
await events.add(event, { durability: "buffered" });
```

The returned receipt reports the achieved acknowledgement.

If the requested guarantee cannot be met, the operation fails with
`DurabilityUnavailable`; it does not silently downgrade.

### 12.3 Named profiles

Configuration MAY offer friendly profiles:

- `safe`;
- `balanced`;
- `fast`;
- `memory-only`.

Every profile expands to visible concrete durability, consistency,
replication, and verification settings.

Profiles are convenience, not hidden semantics.

## 13. Damage and recovery experience

### 13.1 Ordinary reads

Healthy data behaves normally.

Users do not handle a recovery wrapper on every successful `get`.

When damage affects the requested result, the SDK returns a typed error with:

- what operation failed;
- what remains verified;
- whether partial content exists;
- which guarantee cannot be made;
- a stable examination or recovery handle;
- an actionable next step.

### 13.2 Partial bytes

```ts
const partial = await artifacts.inspect("damaged-image");
```

Inspection exposes verified extents and holes.

The ordinary `get` does not concatenate verified extents and pretend the
result is complete.

### 13.3 Doctor

```text
dingo doctor ./app.dingo
```

`doctor` is read-only by default.

It reports store health, indexes, coverage, damaged ranges, unsupported
formats, and recommended actions.

It MUST NOT repair, delete, rewrite, or compact data without an explicit
command.

### 13.4 Salvage

```text
dingo salvage damaged.dingo --output recovered.dingo
```

Salvage writes to a different destination by default.

It preserves:

- verified frames;
- partial payloads;
- holes;
- conflicts;
- unsupported frames;
- provenance and scan parameters.

In-place destructive recovery requires an explicit high-friction option.

## 14. CLI

The CLI mirrors the logical API:

```text
dingo open ./app.dingo
dingo put ./app.dingo users/user-42 --json '{"name":"Alice"}'
dingo get ./app.dingo users/user-42
dingo find ./app.dingo users --where 'status = "active"'
dingo put-bytes ./app.dingo artifacts/build-19 ./build.bin
dingo history ./app.dingo users/user-42
dingo inspect ./app.dingo users/user-42
dingo index create ./app.dingo users by-email --field email
dingo doctor ./app.dingo
dingo serve ./app.dingo
```

Exact command grammar will be frozen with the first CLI implementation.

CLI requirements:

- concise human output by default;
- stable JSON output through `--json`;
- nonzero exit status for failed guarantees;
- no color or progress animation when output is not a terminal;
- streaming input and output;
- explicit confirmation for purge or destructive repair;
- commands display the database path or endpoint they will affect.

## 15. Error model

SDKs expose typed errors with stable machine codes.

Core everyday errors include:

- `AlreadyExists`;
- `NotFound` where an API chooses an error rather than nullable return;
- `VersionConflict`;
- `ValidationFailed`;
- `QueryInvalid`;
- `QueryBudgetRequired`;
- `ResourceLimit`;
- `CoverageIncomplete`;
- `PartitionUnavailable`;
- `DurabilityUnavailable`;
- `DataDamaged`;
- `PayloadPartial`;
- `FormatUnsupported`;
- `AuthenticationFailed`;
- `PermissionDenied`;
- `RateLimited`;
- `DeadlineExceeded`;
- `Internal`.

Every error contains:

- stable code;
- concise message;
- operation;
- retry classification;
- relevant identifiers;
- achieved versus requested guarantee;
- structured details;
- suggested action when one is known.

Errors MUST NOT require parsing English text.

Internal frame, checksum, consensus, or codec details appear in structured
causes and diagnostics, not as the only top-level explanation.

## 16. Schema experience

Collections are schemaless by default.

Optional schemas provide:

- write validation;
- generated SDK types;
- documentation;
- index suggestions;
- structured decoding;
- evolution rules.

Attaching a schema does not rewrite old payloads automatically.

A schema declares how it treats historical values that predate it:

- accept as legacy;
- validate on read;
- migrate lazily;
- reject from typed projections while preserving bytes.

Schema failure never destroys the stored original.

## 17. Import and export

First-class import formats SHOULD include:

- JSON;
- JSONL;
- raw files and directory trees;
- CSV through an explicit mapping;
- DingoDB diagnostic and survival formats.

Export SHOULD include:

- JSON or JSONL for compatible structured values;
- raw bytes;
- directory materialization;
- SDA-transformed streams;
- lossless diagnostic evidence.

Import and export stream by default and return resumable checkpoints.

Unsupported input is preservable as opaque bytes when requested.

## 18. Administration

Routine maintenance is automatic:

- active-segment sealing;
- index checkpointing;
- safe compaction;
- scrubbing;
- tier movement;
- replica repair;
- bounded cache management.

The operator configures policies and budgets, not individual segment actions.

Every automatic maintenance action is observable and interruptible.

Administrative commands separate:

- inspect;
- plan;
- apply.

A plan identifies expected bytes read, written, moved, or reclaimed before
large maintenance operations begin.

## 19. Observability

The product exposes:

- operation latency and throughput;
- durability-mode counts;
- active and sealed bytes;
- index status and lag;
- compaction debt;
- scrub status;
- detected holes and corrupt frames;
- partition and replica health;
- tier coverage;
- query scan and read amplification;
- cache and hot-index effectiveness.

Metrics use stable names and bounded label cardinality.

Logs are structured and include operation, store, partition where applicable,
event identity, and error code.

Tracing spans distinguish application latency, routing, quorum wait, durable
flush, index work, tier fetch, verification, decoding, and SDA evaluation.

## 20. SDK quality

An official SDK MUST provide:

- idiomatic async and streaming APIs;
- connection pooling where applicable;
- safe retry using stable event identifiers;
- deadlines and cancellation;
- typed values and errors;
- local and remote API parity;
- test utilities;
- deterministic serialization fixtures;
- compatibility policy;
- examples that run in continuous integration.

SDK methods MUST document:

- atomicity;
- consistency;
- durability;
- retry safety;
- ordering;
- memory behavior;
- possible partial or uncertain outcomes.

## 21. Local-to-cluster path

Moving from local to clustered use follows:

1. start the local database server;
2. connect using the same collection API;
3. create or join a cluster;
4. replicate and verify existing segments;
5. activate partition placement;
6. update only the connection string.

Application keys, collection names, values, and query semantics remain
unchanged.

Features that require a cluster, such as replicated acknowledgement, fail
clearly before the cluster exists.

## 22. Documentation structure

User documentation begins with tasks, not architecture:

1. install;
2. open;
3. put and get;
4. query;
5. index;
6. store bytes;
7. run as a server;
8. retain and back up;
9. inspect history;
10. diagnose damage;
11. cluster.

Frame and consensus documentation remains discoverable but is not required
reading for the quick start.

Every quick-start example is executable and tested.

## 23. Acceptance tests

The DX release gate includes:

1. a new user can install, open, put, and get without configuration;
2. opening the same database after process termination returns acknowledged
   durable data;
3. JSON and bytes round-trip exactly;
4. a common unindexed query returns correct results or requests an explicit
   budget;
5. adding an index requires no downtime;
6. query results stream without full materialization;
7. missing key and unavailable partition are distinguishable;
8. weak durability never masquerades as durable;
9. local and remote SDK conformance suites produce the same logical results;
10. retrying an acknowledged or ambiguously acknowledged write is idempotent;
11. a damaged payload produces an actionable typed error;
12. `inspect` exposes surviving extents and holes;
13. `doctor` performs no writes by default;
14. salvage defaults to a separate destination;
15. SDA and common-filter results agree on shared semantics;
16. query coverage survives pagination and distributed execution;
17. SDK examples compile and run;
18. destructive administrative commands require explicit targets and intent.

Usability testing SHOULD measure:

- time to first successful write;
- time to first query;
- percentage of errors resolved without reading source code;
- frequency with which users must understand advanced terminology;
- success moving an application from embedded to server mode.

## 24. MVP boundary

The first useful product does not require every long-term feature.

The MVP MUST include:

- embedded single-node operation;
- collections of JSON and bytes;
- put, create, replace, get, inspect, delete, and add;
- streaming scans and common filters;
- at least field and key indexes;
- safe durability defaults and receipts;
- automatic segment maintenance;
- history for a key;
- typed errors;
- doctor and non-destructive salvage;
- one excellent official SDK;
- reproducible correctness, corruption, and performance tests.

Clustering, archival tiering, erasure coding, semantic indexes, and additional
SDKs may follow without changing the everyday logical model.

## 25. Governing principle

The user should encounter DingoDB's complexity only when that complexity
protects them from a lie.

> Extraordinary internals. Ordinary database experience.
