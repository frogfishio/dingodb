# DingoDB transaction extension proposal

Status: proposal  
Target: post-embedded-foundation transaction profile  
Normative impact: `OVERVIEW.md`, `FORMAT_SPEC.md`, `DX_SPEC.md`,
`CLUSTER_SPEC.md`, and SDK compatibility policy

## 1. Summary

DingoDB should add transactions deliberately, without presenting an
unbounded “ACID everywhere” API that the storage and cluster models cannot
honestly support.

The proposed model is:

1. **single-key atomicity** on every backend;
2. **serializable local transactions** within one embedded or single-server
   store;
3. **serializable partition transactions** within one cluster partition;
4. **explicit workflows and sagas** across partitions;
5. no arbitrary distributed transaction claim in the initial profile.

The defining DingoDB behavior is that transaction evidence remains examinable
after damage. Physically surviving prepared members are preserved even when
commit cannot be proven, but they do not silently enter committed logical
state.

This gives ordinary applications useful transactional guarantees while
preserving DingoDB’s larger promise:

> Most databases manage the current state of an application. DingoDB manages
> the lifetime of the data itself.

## 2. Motivation

DingoDB already provides immutable events, single-key writes, durability
receipts, history, partition-local ordering, and wire kinds for batch prepare
and batch commit. The specifications also describe version-conditional writes
and partition batches.

What is missing is one coherent transaction contract covering:

- scope;
- isolation;
- durability;
- idempotent retries;
- conflict handling;
- crash recovery;
- damage and partial survival;
- indexes and history;
- cluster commitment;
- operator examination.

Without that contract, adding a generic `transaction()` method would create
false expectations. Callers may assume arbitrary cross-collection,
cross-partition serializability, while the implementation may only provide a
best-effort batch.

The extension should therefore make the coordination boundary visible and
enforceable.

## 3. Goals

The initial transaction profile MUST provide:

- atomic create, put, replace, and delete for one key;
- optimistic concurrency through stable versions;
- atomic multi-key transactions in an embedded/single-server store;
- atomic multi-key transactions when every clustered key maps to one
  partition;
- serializable execution within the declared scope;
- stable transaction identity and idempotent commit;
- bounded transaction size, duration, and resource use;
- explicit requested and achieved durability;
- deterministic recovery after process interruption;
- evidence-preserving salvage after physical damage;
- history and examination that retain transaction boundaries;
- equivalent logical semantics across supported backends.

## 4. Non-goals

The initial profile does not provide:

- atomic writes across arbitrary cluster partitions;
- one global serial order over a cluster;
- a globally consistent snapshot across unrelated partitions;
- SQL transactions or relational constraint semantics;
- long-running interactive transactions with unbounded locks;
- external side-effect atomicity;
- exactly-once application effects;
- automatic conversion of a cross-partition transaction into a hidden saga;
- exposure of physically prepared members as committed values.

A future distributed transaction profile may be proposed separately. It must
not weaken independent recovery or hide uncertain outcomes.

## 5. Governing principles

### 5.1 Scope is part of the guarantee

Every transaction declares its coordination scope before mutation:

- one key;
- one local store;
- one cluster partition.

If an operation falls outside the scope, DingoDB fails before recording any
member. It never silently weakens atomicity or splits the transaction.

### 5.2 Explicit transactions are serializable within scope

The first profile exposes one isolation level for read-write transactions:
`serializable`.

Avoiding a menu of weaker isolation levels keeps the ordinary contract clear.
Optimizations may use optimistic validation internally, but observable results
must be equivalent to a serial order within the scope.

Read-only snapshot transactions may be added as a separate bounded API. A
multi-partition read never claims one global snapshot unless a future protocol
proves it.

### 5.3 Physical survival is not logical commitment

A prepare frame or transaction member may survive without a valid commit.
That frame remains recoverable evidence, but it is not visible through the
ordinary committed-state API.

### 5.4 No silent uncertainty

After a timeout, disconnect, damaged commit frame, or missing consensus
evidence, the result is not guessed.

The API returns one of:

- committed;
- definitely not committed;
- conflict before commit;
- outcome unknown, with a stable transaction ID and resolution handle.

### 5.5 Retry identity is durable

Every transaction has a caller-stable `transaction_id`. Retrying the same ID
and content returns the original outcome. Reusing the ID with different
content is a consistency violation.

### 5.6 Authority remains in transaction evidence

Indexes, catalogs, lock tables, and transaction-status caches are derived.
The authoritative outcome is reconstructed from verified prepare/member/
decision evidence and, for clusters, durable consensus evidence.

### 5.7 Transactions are bounded

The server enforces limits for:

- member count;
- encoded bytes;
- read-set size;
- duration;
- buffered memory;
- touched collections;
- generated history and index work.

Limits fail before commit with typed errors. A transaction is never allowed to
grow until it destabilizes the process.

## 6. Transaction scopes

### 6.1 Single-key atomic operation

This is the baseline available everywhere.

Supported preconditions:

- key must not exist;
- key must exist;
- visible version must equal `if_version`;
- value hash must equal an expected hash;
- no precondition.

The operation and precondition are evaluated atomically under the key’s
coordination scope.

Example:

```rust
let current = users.inspect("user-42")?;

users.replace(
    "user-42",
    &next_value,
    ReplaceOptions::new().if_version(current.version),
)?;
```

### 6.2 Local transaction

A local transaction may touch multiple keys and collections within one
embedded or single-server store.

Properties:

- serializable isolation;
- one store generation;
- one durable transaction ID;
- one atomic logical commit;
- no dependency on a cluster partition map;
- same damage-evidence model as partition transactions.

This is the ordinary transaction profile for embedded applications.

Example shape:

```rust
let mut tx = db.transaction(TransactionOptions::serializable())?;

let account = tx.collection("accounts")?.get("account-42")?;
tx.collection("accounts")?.replace(
    "account-42",
    &debit(&account, 100)?,
    ReplaceOptions::new().if_version(account.version),
)?;
tx.collection("ledger")?.create("entry-901", &entry)?;

let receipt = tx.commit()?;
```

### 6.3 Partition transaction

A partition transaction may touch multiple keys and collections only when
every operation maps to the same cluster partition.

The caller declares a partition key or receives a transaction handle already
bound to a partition:

```rust
let mut tx = db.partition_transaction(
    "account-42",
    TransactionOptions::serializable(),
)?;

tx.collection("accounts")?.put("account-42", &account)?;
tx.collection("ledger")?.create("account-42/entry-901", &entry)?;

let receipt = tx.commit()?;
```

The SDK computes the partition for every member before submission where
possible. The leader validates scope again. A stale client map cannot authorize
a cross-partition commit.

Properties:

- serializable ordering within one partition;
- one Raft log command or equivalent consensus decision;
- quorum commitment under the strong profile;
- no ordering promise relative to unrelated partitions.

### 6.4 Cross-partition workflow

Cross-partition work uses explicit workflow records, idempotent steps, and
compensation.

DingoDB may provide a saga helper, but it must expose:

- workflow identity;
- completed and pending steps;
- retries and deduplication;
- compensation attempts;
- uncertain outcomes;
- coverage and unavailable partitions.

The helper is not named `transaction` and does not claim atomic rollback.

Example shape:

```rust
let workflow = db.workflow("transfer-901")?;
workflow.step("debit", debit_command)?;
workflow.step("credit", credit_command)?;
workflow.compensation("refund", refund_command)?;
workflow.run()?;
```

## 7. Isolation and concurrency model

### 7.1 Serializable optimistic execution

The recommended initial implementation is optimistic:

1. Begin at a stable store or partition frontier.
2. Record every read version in the transaction read set.
3. Buffer writes without publishing them.
4. At commit, acquire the scope’s commit sequencer.
5. Validate that read and write preconditions still hold.
6. Assign one commit position.
7. append and persist transaction evidence;
8. publish all members together.

If validation fails, no member becomes committed and the caller receives
`TransactionConflict` or `VersionConflict`.

### 7.2 Why serializable rather than snapshot isolation

Snapshot isolation permits write skew and requires users to understand which
invariants are safe. DingoDB should not advertise an ordinary transaction API
while leaving common multi-key invariants vulnerable by default.

Because the initial coordination scopes are one local store or one partition,
a serializable commit sequencer is practical. Concurrency can be recovered
through sharded partitions and optimistic execution rather than weaker
semantics.

### 7.3 Read-only snapshots

A read-only snapshot binds to:

- a local durable frontier; or
- one partition term and committed position.

It is bounded by timeout and retention. If required history has been compacted
or damaged, the snapshot reports incomplete coverage rather than silently
reading a newer state.

### 7.4 Locking

The initial profile should avoid long-lived user locks.

Short internal locks are permitted during validation and publication.
Transactions that exceed duration or resource limits expire before commit.
Deadlock detection is unnecessary if the implementation uses one ordered
commit sequencer per scope and does not hold user locks during transaction
construction.

## 8. API model

### 8.1 Core types

Conceptual API:

```rust
pub struct TransactionId([u8; 16]);

pub enum TransactionScope {
    LocalStore,
    Partition {
        partition_key: Vec<u8>,
    },
}

pub enum IsolationLevel {
    Serializable,
}

pub struct TransactionOptions {
    pub transaction_id: Option<TransactionId>,
    pub scope: TransactionScope,
    pub isolation: IsolationLevel,
    pub durability: DurabilityMode,
    pub timeout: Duration,
    pub max_operations: u32,
    pub max_bytes: u64,
}

pub struct TransactionReceipt {
    pub transaction_id: TransactionId,
    pub scope: TransactionScopeReceipt,
    pub isolation: IsolationLevel,
    pub operation_count: u32,
    pub commit_position: CommitPosition,
    pub requested_durability: DurabilityMode,
    pub achieved_durability: DurabilityMode,
    pub committed: bool,
    pub commit_evidence: Option<CommitEvidence>,
}
```

Exact public Rust types require an API review. The semantic fields are
normative.

### 8.2 Transaction operations

The first profile supports:

- `get` and `inspect`;
- `create`;
- `put`;
- `replace(if_version)`;
- `delete`;
- generated-key `add`;
- append-only event insertion.

Index creation, tier movement, schema changes, compaction, purge, and cluster
membership changes are administrative operations and cannot be transaction
members.

### 8.3 Commit outcomes

Conceptual result:

```rust
pub enum CommitOutcome {
    Committed(TransactionReceipt),
    NotCommitted {
        transaction_id: TransactionId,
        reason: TransactionAbortReason,
    },
    Unknown {
        transaction_id: TransactionId,
        recovery_handle: String,
        last_observed: Option<CommitPosition>,
    },
}
```

An SDK may expose definitely-not-committed conflicts as typed errors, but it
must preserve the distinction from unknown outcome.

### 8.4 Status resolution

Every backend supports:

```rust
db.transaction_status(transaction_id)?
```

Possible states:

- `not_found` — no evidence within complete declared coverage;
- `prepared`;
- `committed`;
- `aborted`;
- `conflicting`;
- `incomplete`;
- `unknown_commit`;
- `coverage_incomplete`.

`not_found` is legal only when the relevant scope and retention window have
complete coverage.

## 9. Wire representation

`FORMAT_SPEC.md` already reserves core frame kinds:

- `5` — batch prepare;
- `6` — batch commit.

The transaction extension freezes their envelope and body semantics.

### 9.1 Transaction identity

Every transaction frame and member carries or derives:

- transaction ID;
- store and segment identity;
- scope kind;
- partition ID when clustered;
- transaction ordinal;
- operation count;
- isolation profile;
- snapshot/read frontier;
- transaction content hash;
- created timestamp as diagnostic evidence only.

Wall-clock time never establishes ordering or commitment.

### 9.2 Prepare frame

The prepare frame contains a deterministic manifest:

- transaction ID;
- protocol/profile version;
- scope;
- expected member count;
- ordered operation descriptors;
- collection/key identity;
- operation kind;
- member event IDs;
- preconditions and observed versions;
- payload/content hashes;
- total logical and encoded bytes;
- snapshot frontier;
- isolation level.

The prepare frame does not make members visible.

### 9.3 Member frames

Transaction members use ordinary item-event and payload-chunk frames tagged
with:

- transaction ID;
- operation ordinal;
- member event ID;
- item identity;
- content hash.

They remain independently verifiable and salvageable.

An item event carrying a transaction ID is never applied to ordinary current
state unless the complete transaction decision validates.

### 9.4 Commit frame

The commit frame contains:

- transaction ID;
- hash of the prepare frame;
- hash/root covering the ordered member set;
- member count;
- local commit position;
- achieved durability;
- partition term/position and placement epoch when clustered;
- portable commit evidence when available.

A commit is valid only when:

1. prepare verifies;
2. every required member verifies;
3. member identities and hashes match the prepare manifest;
4. the commit references the exact prepare/member set;
5. scope and preconditions were valid at the assigned commit position;
6. clustered commitment is supported by durable consensus evidence.

### 9.5 Abort evidence

An abort does not require a durable frame when no prepared data was written.

If prepared members were persisted, an optional abort decision frame should be
defined in a later compatible core kind or transaction-decision extension.
Absence of commit alone means “not proven committed,” not necessarily a proven
explicit abort.

### 9.6 Recovery classification

Recovery groups frames by transaction ID and classifies:

- `verified-committed`;
- `verified-aborted`;
- `prepared-uncommitted`;
- `unknown-commit`;
- `incomplete-prepare`;
- `incomplete-members`;
- `conflicting`;
- `unsupported-profile`.

Only `verified-committed` enters ordinary logical state.

All other verified material remains available to examination and salvage.

## 10. Local commit protocol

Recommended append sequence:

```text
validate read/write set
    ↓
append BatchPrepare
    ↓
append transaction member frames
    ↓
append BatchCommit
    ↓
persist to requested durability
    ↓
publish all index changes atomically
    ↓
return receipt
```

Rules:

- Prepare, members, and commit should be contiguous in the initial
  implementation, but recovery must use identity and hashes rather than rely
  solely on adjacency.
- Visibility is published only after the commit frame reaches the requested
  durability.
- A memory-mode transaction may be visible in process but must not enter any
  persisted derived artifact before authoritative bytes are flushed.
- Index/catalog publication must install one transaction delta atomically.
- A crash before valid commit leaves prepared evidence but no committed
  logical mutation.
- A crash after durable commit but before response resolves to the same receipt
  by transaction ID.

## 11. Cluster commit protocol

### 11.1 Partition-linearizable mode

One partition transaction is one deterministic Raft state-machine command
containing or referencing the complete operation manifest.

Sequence:

1. Client sends stable transaction ID, manifest, and preconditions to the
   partition leader.
2. Leader verifies scope and limits.
3. Leader proposes the transaction command through Raft.
4. A quorum persists the log entry under the consensus durability contract.
5. After commitment, each replica applies the transaction by writing local
   transaction frames idempotently.
6. The leader returns a receipt with term, position, placement epoch, replica
   acknowledgements, and commit evidence.

The exact boundary between Raft-log persistence and Dingo segment persistence
must be specified before implementation. A “replicated durable”
acknowledgement cannot be returned unless the configured number of replicas
has durable evidence sufficient for recovery.

### 11.2 Leader failure

- Before proposal: definitely not committed.
- After local proposal but before quorum: prepared or outcome unknown.
- After quorum commit: committed even if the client never receives the
  response.
- Retry with the same transaction ID resolves or completes the original
  command; it never proposes altered content.

### 11.3 Convergent-append mode

Mutable multi-key transactions are not supported in convergent-append mode.

An append group may preserve a shared transaction/workflow identity, but it
does not claim atomic visibility across split sides. The API must name this a
group or workflow, not a transaction.

## 12. History, indexes, watches, and queries

### 12.1 History

Every committed member records:

- transaction ID;
- transaction ordinal;
- commit position;
- transaction member count.

History can be viewed as individual events or grouped transactions.
Prepared/uncommitted members appear only in examination/salvage views.

### 12.2 Primary index

The primary index applies all transaction mutations as one publication step.
Rebuild groups transaction evidence and ignores unproven members.

### 12.3 Secondary indexes

Secondary index updates are derived from the committed transaction frontier.
They may lag, but:

- all members share one source commit position;
- partial index application cannot prove absence;
- queries fall back to authoritative scan or return incomplete coverage.

### 12.4 Watches

A watch may expose:

- one transaction envelope containing ordered members; or
- member events carrying one transaction boundary.

It must not expose the first member as committed while later members remain
unpublished.

### 12.5 Queries

A transaction reads from its declared snapshot/frontier plus its own buffered
writes. It does not observe another transaction’s prepared members.

## 13. Chunks and large values

Chunked transaction members remain invisible until:

- every required chunk verifies;
- the member manifest verifies;
- the complete transaction commits.

If a commit survives but a chunk is later destroyed, the transaction remains
historically committed while the current payload becomes partial. DingoDB must
distinguish:

- commitment of the logical event;
- present completeness of its payload.

Large transaction limits should prevent one transaction from monopolizing an
active segment or Raft proposal. Oversized workflows should use staged objects
plus a small atomic reference update.

## 14. Compaction, tiering, and salvage

### 14.1 Compaction

Compaction must preserve:

- transaction ID and member ordering;
- commit decision and content root;
- event and item identities;
- enough evidence to prevent prepared members becoming committed;
- history required by active snapshots and deduplication retention.

It may emit a transaction checkpoint only when coverage and source frontiers
are explicit.

### 14.2 Tiering

Transaction evidence may span segments or media after migration. Tier
placement cannot become the only map from a commit to its members.

An offline tier may make transaction status or payload completeness uncertain.
It cannot be represented as abort or absence.

### 14.3 Salvage

Evidence-preserving salvage copies:

- prepare;
- surviving members and chunks;
- commit/abort evidence;
- holes and missing member identities;
- consensus evidence;
- recovery classification and provenance.

Live-state export includes only verified committed transactions whose required
current payloads are complete under the requested export policy.

## 15. Errors

Add or freeze stable codes:

- `version_conflict`;
- `transaction_conflict`;
- `transaction_scope_violation`;
- `transaction_too_large`;
- `transaction_expired`;
- `transaction_not_supported`;
- `transaction_id_reused`;
- `transaction_outcome_unknown`;
- `transaction_incomplete`;
- `durability_unavailable`;
- `partition_unavailable`;
- `coverage_incomplete`;
- `protocol_violation`.

Every error states:

- transaction ID when assigned;
- whether any authoritative evidence may exist;
- whether retry is safe;
- requested and achieved guarantees;
- a status/recovery handle when outcome is unknown.

## 16. Security and resource controls

Transactions add denial-of-service and contention risks.

Required controls:

- authenticate before allocating transaction buffers;
- authorize every collection and operation;
- cap concurrent open transactions per principal and store;
- cap members, bytes, duration, read set, and response size;
- avoid logging values or secrets;
- audit administrative overrides;
- bind transaction IDs to authenticated identity where policy requires;
- reject malformed manifests before expensive payload work.

## 17. Observability

Metrics:

- begun, committed, aborted, conflicted, expired, and unknown transactions;
- commit latency by durability and scope;
- validation failures;
- operations and bytes per transaction;
- open transaction count and age;
- prepared/uncommitted evidence discovered;
- deduplication hits and ID-reuse violations;
- partition transaction quorum and apply latency;
- index lag from transaction commit frontier.

Logs and traces include transaction ID, scope, partition, term/position,
requested/achieved durability, and stable error code. Payloads are excluded by
default.

## 18. Implementation phases

### Phase T0 — Freeze semantics and fixtures

- Amend normative specifications.
- Freeze transaction IDs, versions, errors, and recovery states.
- Define deterministic prepare/member/commit encodings.
- Add golden wire fixtures and malformed corpora.

Exit:

- Independent reviewers can determine commitment using only the specification
  and frames.

### Phase T1 — Single-key preconditions

- Implement create-if-absent and replace-if-version.
- Make remote retries idempotent by stable operation ID.
- Add committed/not-committed/unknown outcomes.

Exit:

- Single-key ambiguity and version races pass crash and retry tests.

### Phase T2 — Local write-only batches

- Implement bounded create/put/replace/delete batches.
- Write prepare, members, and commit.
- Rebuild indexes transaction-aware.
- Preserve evidence through salvage.

Exit:

- No partial logical visibility under every injected crash and damage point.

### Phase T3 — Serializable local transactions

- Add snapshot/read sets and optimistic validation.
- Add read-your-writes.
- Add deterministic conflict handling.
- Integrate secondary indexes, history, and watches.

Exit:

- A serializability checker validates randomized concurrent histories.

### Phase T4 — Remote transaction protocol

- Add versioned RPC operations and bounded transaction requests.
- Preserve transaction ID through timeout and reconnect.
- Add transaction-status resolution.

Exit:

- Embedded and remote backend conformance suites are equivalent.

### Phase T5 — Partition transactions

- Encode one batch as one Raft command.
- Persist consensus and state-machine evidence.
- Add leader failure, retry, fencing, and quorum tests.

Exit:

- Multi-process network cluster histories are serializable per partition.

### Phase T6 — Workflow helpers

- Add explicit saga/workflow records.
- Preserve retries, compensation, and uncertain steps.
- Keep naming and receipts distinct from transactions.

Exit:

- Cross-partition examples cannot be mistaken for atomic transactions.

## 19. Conformance tests

### 19.1 Atomicity

- crash before prepare;
- crash during prepare;
- crash after prepare;
- crash between every member;
- crash during chunk write;
- crash before commit;
- crash during commit;
- crash after durable commit before publication;
- crash after publication before response;
- damaged prepare, member, chunk, or commit;
- reordered and duplicated segment copies.

Expected: all members are visible or none are visible; noncommitted evidence
remains examinable.

### 19.2 Isolation

- lost-update race;
- write skew attempt;
- read/write conflict;
- phantom over indexed and scan paths;
- read-your-writes;
- concurrent create-if-absent;
- delete/recreate version race;
- transaction expiry during validation.

Expected: observed histories are serializable within scope.

### 19.3 Retry and identity

- response loss before and after commit;
- reconnect to another server/leader;
- duplicate request;
- same ID with different content;
- status lookup after restart and compaction;
- deduplication horizon expiry.

### 19.4 Scope

- two collections in one local store;
- all keys in one partition;
- accidental second partition;
- stale partition map;
- placement epoch change during commit;
- unsupported convergent mode.

### 19.5 Coverage and recovery

- offline tier containing a member;
- missing consensus evidence;
- control-plane loss;
- salvage without catalogs;
- partial payload after historical commit;
- unsupported future transaction profile.

### 19.6 Backend parity

Run the same logical corpus against:

- embedded local store;
- single-node remote server;
- in-process partition harness;
- multi-process network cluster.

Unsupported scopes must fail explicitly rather than degrade.

## 20. Performance requirements

Transactions must not reintroduce full-store work on commit.

Benchmark:

- one-key transaction overhead versus ordinary put;
- 2, 10, 100, and maximum-member batches;
- read-write contention;
- conflict-heavy workload;
- durable fsync modes;
- remote transaction latency;
- partition quorum latency;
- recovery scan with prepared transactions;
- index and watch publication;
- large chunked member behavior.

Reports disclose p50/p95/p99, throughput, durability, verification, member
count, bytes, contention, abort rate, replication, and hardware.

## 21. Compatibility and versioning

Transaction semantics require independent versions for:

- SDK transaction API;
- RPC transaction protocol;
- wire transaction profile;
- cluster transaction profile;
- examination projection.

Readers must preserve unknown future transaction frames losslessly. They must
not apply an unsupported transaction profile as committed state.

The wire profile must remain draft until crash, damage, retry, and
interoperability suites pass.

## 22. Required specification changes

If this proposal is accepted:

1. Amend `OVERVIEW.md` §7.3 with the transaction invariants and recovery
   classification.
2. Freeze `FORMAT_SPEC.md` batch prepare/commit envelopes and manifests.
3. Replace the short `DX_SPEC.md` §9 text with the scoped API and outcome
   model.
4. Amend `CLUSTER_SPEC.md` with partition transaction Raft and durability
   rules.
5. Add transaction cases to destructive and cluster conformance sections.
6. Add implementation tasks and release gates to `DEFECTS.md`.
7. Do not increment stable API/profile labels until compatibility review.

## 23. Open decisions

These require explicit resolution before Phase T0 closes:

1. Exact transaction ID width and generation profile.
2. Whether commit and abort share a general decision frame in a future wire
   minor.
3. Maximum initial member count and encoded bytes.
4. How long deduplication evidence must survive after compaction.
5. Exact local commit position representation.
6. Whether read-only snapshots are part of the first stable transaction API.
7. Whether secondary index maintenance is synchronous or frontier-based.
8. The durability boundary between committed Raft log entries and Dingo
   segment application.
9. How portable cluster commit evidence is encoded.
10. Whether transaction status has a separate retention policy.

## 24. Recommendation

Adopt this scoped model rather than a generic distributed transaction API.

The first customer-meaningful target should be:

> Serializable transactions across collections in one embedded store, and the
> same semantics for keys colocated in one cluster partition.

That is enough for account-and-ledger, metadata-and-object, state-and-outbox,
and other ordinary application invariants. It avoids claiming a global
transaction system before one exists.

Most importantly, it extends DingoDB’s core distinction:

> A transaction can lose evidence without DingoDB lying about its outcome.
> Whatever survives remains independently verifiable and examinable.
