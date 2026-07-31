# Residiuum Competitive Goals

Status: Product strategy v0.1; target checklists retained, execution order
superseded by [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md)
Scope: The three competitive stages from embedded database to survivable data
platform  
Audience: Product, architecture, implementation, benchmarking, documentation,
and release engineering

The stage descriptions below remain useful competitor-specific acceptance
checklists. Their historical numbering is **not** the current implementation
queue. Residiuum now delivers the trustworthy SQLite-replacement core first,
then the Mongo-facing mathematical single-node proposition, and only later the
Couchbase-facing distributed product. `MASTER_DELIVERY_PLAN.md` governs.

## 1. Purpose

Residiuum does not win by becoming a feature-complete copy of every database.

It wins a sequence of increasingly demanding decisions:

1. choose Residiuum instead of SQLite plus loose files;
2. choose Residiuum instead of Couchbase for embedded and edge document data;
3. choose Residiuum instead of MongoDB for operational document data that must
   remain recoverable and useful for a very long time.

Each stage establishes the credibility needed for the next. Skipping a stage
would turn the product thesis into an unsupported claim.

The sequence is:

```text
Stage 1                   Stage 2                    Stage 3

SQLite + files     ->     Couchbase / edge    ->    MongoDB
embedded                  embedded + sync            operational platform

easy local storage        dependable fleet data     dependable data at scale
```

Redis remains a latency reference. PostgreSQL remains a correctness and
operational-maturity reference. Object storage remains a physical tier and an
architectural alternative. They are not the three opening product contests.

## 2. What “beat” means

“Beat” never means “is universally better.”

Residiuum beats an incumbent at a stage when a defined customer can make a
rational choice based on evidence:

> For this workload, Residiuum is at least as easy to adopt and operate, satisfies
> every mandatory requirement, and provides materially better survival and
> long-term data ownership.

A stage is won only when all five dimensions pass:

### 2.1 Fitness

The target workload works without requiring users to build missing database
machinery around Residiuum.

### 2.2 Dependability

Crash, corruption, upgrade, backup, restore, capacity, and security behavior
are tested and documented.

### 2.3 Developer experience

A new user can reach a correct production-shaped result quickly using the
primary SDK, CLI, examples, and diagnostics.

### 2.4 Operational evidence

Published benchmarks, failure tests, compatibility results, and restore drills
support the claim.

### 2.5 Distinctive victory

Residiuum demonstrates a recovery or longevity outcome the incumbent does not
normally provide: independently verified surviving data, explicit holes, and
continued examination after partial destruction.

Feature presence alone does not win a stage.

## 3. Rules that apply to every stage

### 3.1 The wedge stays intact

Every capability MUST preserve the founding rule:

> What is gone is gone. What remains still lives.

Transactions, encryption, compression, indexing, replication, clustering, and
tiering MUST NOT introduce avoidable all-or-nothing recovery domains.

### 3.2 Ordinary use must be ordinary

Damage tolerance cannot require every user to become a storage engineer.

The common path remains:

```text
open -> put -> get -> find -> stream
```

Advanced durability, policy, clustering, and salvage are progressive
disclosure.

### 3.3 Claims name their profile

Performance, durability, consistency, security, recovery, and scale claims
MUST state:

- deployment profile;
- acknowledgement mode;
- dataset and working-set size;
- concurrency;
- hardware and storage;
- failure assumptions;
- configuration;
- measurement method.

“Faster,” “durable,” “encrypted,” and “production-ready” are incomplete claims
without those boundaries.

### 3.4 Compatibility is a product capability

Users MUST have:

- versioned formats and protocols;
- supported upgrade and rollback paths;
- portable export;
- retained readers for old data;
- explicit behavior for unsupported future or damaged material.

### 3.5 Doctrine is enforced

The contracts in [DATABASE_DOCTRINE.md](./DATABASE_DOCTRINE.md) are acceptance
criteria, not marketing decoration. Current gaps remain visible in
[doc/wip/doctrine/DOCTRINE_GAPS.md](../../wip/doctrine/DOCTRINE_GAPS.md).

---

## 4. Stage 1 — Beat SQLite plus loose files

### 4.1 The customer

The Stage 1 customer is building:

- a local-first or embedded application;
- a Rust service with one authoritative local store;
- a desktop tool;
- an appliance, agent, or device;
- an application-specific server;
- a data collector that stores JSON, events, blobs, or mixed material.

They would otherwise select SQLite, a key-value engine, or a directory of
files with a small catalog.

They value zero administration and dependable local storage more than
distributed scale.

### 4.2 The decision to win

> Why should I embed Residiuum instead of SQLite and put the awkward binary data
> beside it?

The answer:

> Because Residiuum makes JSON, bytes, events, and retained history easy in one
> store—and damage to one region does not make every healthy region
> inaccessible.

This is not a contest for applications that principally need rich SQL,
relational constraints, a mature SQL tooling ecosystem, or drop-in SQLite
compatibility.

### 4.3 Mandatory product goals

#### A. Installation and first success

- one dependable Rust dependency and documented supported toolchain;
- zero-service embedded startup;
- safe defaults with no mandatory configuration;
- a five-minute tutorial covering open, put, get, find, and delete;
- actionable errors carrying stable codes and recovery guidance;
- no storage-format knowledge required for ordinary use.

#### B. Everyday data model

- first-class JSON and arbitrary bytes;
- collections and stable identifiers;
- atomic documented write semantics;
- predictable read-after-write behavior;
- bounded scans and streaming;
- useful secondary indexes;
- familiar filters compiled to RQL/SDA;
- explicit size, concurrency, and transaction limits.

#### C. Lifecycle of one local store

- open, close, reopen, move, copy, inspect, compact, back up, and restore;
- safe handling of process crashes and interrupted maintenance;
- explicit disk-full and read-only recovery behavior;
- stable migration across supported releases;
- full backup with verified restore to a new destination;
- export that does not require Residiuum to recover the user's content.

#### D. Local security

- filesystem-permission guidance;
- native encryption at rest with independent authenticated frames;
- a protected local key-provider profile;
- key backup, rotation, and loss behavior;
- redacted diagnostics and logs;
- no claim that encryption protects against a compromised application process.

#### E. Survival advantage

- forward and reverse resynchronization after corruption;
- catalog-independent discovery of valid units;
- verified data distinguished from holes and unsupported data;
- damage to one bounded region cannot prevent unrelated valid regions from
  being enumerated;
- SDA examination over the recovered result;
- machine-readable recovery evidence.

#### F. Product quality

- crash-injection tests at every durable write boundary;
- corruption campaigns covering truncation, overwrites, bit flips, holes,
  duplicated regions, and damaged indexes/catalogs;
- property tests and fuzzing for format and recovery readers;
- supported operating-system and filesystem matrix;
- deterministic golden archives retained across releases;
- no known silent-loss defect in the qualified profile.

### 4.4 Evidence required

Stage 1 requires a public, reproducible evidence pack:

1. embedded latency and throughput against SQLite for named JSON/bytes
   workloads;
2. resident and cold-read results with explicit acknowledgement modes;
3. crash matrix with acknowledged-write outcomes;
4. corruption matrix showing recovered units and reported holes;
5. backup/restore and upgrade/rollback drills;
6. storage amplification and compaction measurements;
7. a clean-machine tutorial run;
8. at least three non-trivial reference applications;
9. at least one store retained and read across multiple released format
   versions;
10. a limitations page that states where SQLite remains the better choice.

Performance does not need to win every query. It must be competitive for the
chosen document/bytes workloads and must not make the survivability advantage
too expensive to use.

### 4.5 Stage 1 exit gate

Stage 1 is complete when:

- an application team can replace SQLite plus its blob directory without
  creating new persistence infrastructure;
- embedded operation is supported rather than labelled experimental;
- encryption, backup, migration, compaction, and recovery are routine product
  paths;
- clean shutdown, crash, corruption, disk-full, and lost-index tests pass;
- the published evidence pack is reproducible outside the development tree;
- at least three independent applications have operated through a release
  upgrade and a restore drill;
- users select Residiuum specifically for mixed data and bounded-loss recovery,
  not merely because the project author requested it.

### 4.6 What not to build for Stage 1

Stage 1 does not require:

- SQL compatibility;
- a distributed cluster;
- mobile synchronization;
- every programming-language SDK;
- a graphical administration suite;
- Redis latency for durable writes;
- MongoDB query compatibility.

Those would distract from proving the embedded product.

---

## 5. Stage 2 — Beat Couchbase for survivable edge data

### 5.1 The customer

The Stage 2 customer operates data outside a continuously connected datacenter:

- industrial and field systems;
- intermittently connected devices;
- branch and edge servers;
- autonomous collectors;
- desktop fleets;
- applications that must continue locally during network loss.

They need an embedded document database plus secure synchronization to a
central service.

The initial Residiuum target is industrial, agent, desktop, and edge-node
deployment. A broad consumer-mobile contest additionally requires supported
Swift and Kotlin products and is not implied by Stage 2 unless those SDKs are
qualified.

### 5.2 The decision to win

> Why should I choose Residiuum instead of Couchbase Lite, Sync Gateway, and
> Couchbase Server?

The answer:

> Because Residiuum keeps the local-first document experience and dependable
> synchronization, while making local damage, missing history, and conflict
> resolution explicitly recoverable and examinable.

### 5.3 Mandatory product goals

#### A. Embedded-to-service continuity

- the same logical collection API in embedded and service deployments;
- local operation throughout extended disconnection;
- durable change feed with stable resumable positions;
- resumable push and pull replication;
- idempotent retries and duplicate suppression;
- explicit acknowledgement and consistency modes;
- no requirement to replace the embedded store when connecting it to a
  service.

#### B. Selective synchronization

- collection, identity, partition, and policy-based replication scopes;
- server-authoritative access checks;
- revocation behavior for material already present on a device;
- bounded bootstrap and resynchronization;
- bandwidth and storage budgets;
- attachment/blob chunking and resumption;
- visible coverage: what is local, remote, pending, unavailable, or excluded.

#### C. Conflict doctrine

- stable event and writer identities;
- deterministic conflict detection;
- declared conflict policies rather than implicit last-write-wins;
- application-defined merge through bounded deterministic SDA where suitable;
- preservation of unresolved alternatives;
- audit evidence explaining why a result won;
- clock-independent correctness, with wall time used only where explicitly
  safe.

#### D. Fleet security

- mutual authentication and TLS;
- self-contained, holder-bound HeapKeys for device and service identity;
- per-heap master authorities, credential expiry, and local-only cycling;
- named heaps with heap-bound handles and complete-path authorization as
  defined by [HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md);
- scoped rights, default denial, and no authorization-policy lookup on the
  established-channel hot path;
- optional bounded grace with an always-resident blacklist;
- native encrypted local stores;
- central and local key-custody profiles;
- device loss and credential-revocation procedures;
- tenant and fleet boundaries that apply to replication, query, backup,
  metrics, and support bundles.

#### E. Operability under bad networks

- retry behavior under loss, reordering, duplication, and long partitions;
- bounded queues and backpressure;
- resumable large transfers;
- protocol version negotiation;
- rolling client/server compatibility window;
- observability for lag, coverage, conflicts, rejected data, and blocked
  policy;
- remote diagnosis without exporting plaintext by default.

#### F. Survival across the fleet

- a damaged local store can salvage intact units and resume synchronization;
- a rebuilt catalog does not invent synchronization completeness;
- the service does not treat an offline device as proof that purge completed;
- repair distinguishes authoritative replacement from convergent merge;
- local, remote, and historical holes remain visible;
- central loss does not erase independently valid local evidence.

### 5.4 Evidence required

Stage 2 requires:

1. a repeatable fleet simulator with thousands of logical clients;
2. long-partition, reconnect-storm, duplicate, reorder, and packet-loss tests;
3. deterministic conflict corpus shared by client and server;
4. credential rotation, device loss, and revocation drills;
5. interrupted bootstrap and large-blob resumption tests;
6. mixed-version rolling upgrade tests;
7. per-tenant and per-device isolation tests across every access path;
8. local corruption followed by salvage, repair, and resumed replication;
9. declared convergence and coverage invariants checked by the test rig;
10. comparative deployment, bandwidth, latency, and recovery results for the
    named edge workload.

### 5.5 Stage 2 exit gate

Stage 2 is complete when:

- a qualified edge application needs no custom synchronization service;
- embedded and service APIs form one supported product;
- a device may work offline, reconnect, reconcile, and prove its resulting
  coverage;
- conflicts are deterministic, visible, and explainable;
- authentication, authorization, encryption, key rotation, and revocation
  operate across a real fleet lifecycle;
- client/server rolling upgrades are routine;
- local corruption and central unavailability have tested recovery stories;
- at least one independent deployment completes a disconnection, upgrade,
  credential rotation, local damage, and resynchronization exercise.

### 5.6 What not to build for Stage 2

Stage 2 does not require:

- parity with every Couchbase mobile platform on day one;
- MongoDB-compatible drivers or wire protocol;
- arbitrary cross-shard transactions;
- unbounded peer-to-peer topology;
- silent automatic conflict resolution;
- a claim that synchronization is backup.

---

## 6. Stage 3 — Beat MongoDB where data must outlive the database

### 6.1 The customer

The Stage 3 customer is choosing the primary operational document platform for:

- high-volume application data;
- JSON and binary material;
- event and history retention;
- multi-node availability;
- large and growing datasets;
- regulated or governed retention;
- data expected to remain valuable for years or decades.

They need the normal database to be excellent. They will not accept a weak
operational product merely to gain a superior disaster story.

### 6.2 The decision to win

> Why should I run Residiuum instead of MongoDB?

The answer:

> Because Residiuum provides the document experience, dependable operations, and
> horizontal scale required by the application, while preserving independently
> verifiable data, explicit history, governed lifecycle, and useful recovery
> after partial destruction.

The target is not every MongoDB workload. Initial qualification SHOULD focus
on append-heavy and read-heavy document/event workloads where long retention,
large mixed payloads, historical examination, and bounded-loss recovery matter.

### 6.3 Mandatory product goals

#### A. Credible document database

- polished CRUD, batch, streaming, and transaction APIs;
- JSON, bytes, chunks, and large objects;
- expressive filters, projections, sorting, pagination, and aggregation;
- exact ranked direct access that does not silently enumerate and discard a
  large result prefix, including filter-conditioned order navigation through
  [Residiuum Order Wavelets](../../todo/order-wavelets/ORDER_WAVELET_SPEC.md);
- online index creation, rebuild, validation, and removal;
- unique and compound index semantics where declared;
- schema-optional operation with enforceable validation when requested;
- change streams with durable resumable positions;
- documented transaction, isolation, and consistency semantics;
- stable explain plans and query/resource limits.

#### B. Production drivers and tools

- first-class Rust SDK;
- at least two additional SDKs selected from actual customer demand;
- versioned service protocol;
- connection pooling, deadlines, cancellation, retry, and idempotency guidance;
- import/export and a documented MongoDB migration path;
- administration CLI and automation-safe APIs;
- local development requiring minimal ceremony;
- actionable diagnostics rather than storage-engine folklore.

#### C. Production clustering

- multi-process, multi-host partitions;
- automatic placement, replication, leader election, and repair;
- horizontal scale with measured rebalancing behavior;
- no global hot-path dependency that defeats partition independence;
- explicit quorum-loss read and write behavior;
- rolling membership and topology changes;
- zone/rack awareness;
- online node replacement;
- split-brain prevention for strong writes;
- convergent append only under an explicitly selected policy;
- catalog/control-plane rebuild from verifiable inventories.

#### D. Security and tenancy

- TLS and mutual TLS;
- qualified logical heap isolation across data, query, history, indexes,
  backup, recovery, and administration;
- self-contained HeapKeys with least authority and proof of holder;
- local-only master-key issuance and cycling;
- hard-generation invalidation plus optional bounded grace and resident
  blacklist;
- application-owned human authorization above Residiuum rather than database
  RBAC;
- auditable high-impact operations;
- native envelope encryption and pluggable key providers;
- wrapping-key and data-key rotation with coverage evidence;
- backup-key separation;
- strong tenant isolation for every query, index, stream, replication,
  administrative, backup, and diagnostic path;
- published threat model, vulnerability process, and external security review.

#### E. Lifecycle and long retention

- persisted classification and lifecycle policies;
- hot, warm, cold, and archive placement;
- native object-storage tiers;
- retention periods, governance retention, compliance profile, and legal holds;
- asynchronous TTL with observable lag;
- plan/apply purge across active data, indexes, replicas, tiers, and managed
  backups;
- explicit incomplete-purge state for unavailable domains;
- key-dependency inventory before crypto-erasure;
- stable segment identity across tier movement;
- retained readers and periodic clean-room recovery of old archives.

#### F. Backup and disaster recovery

- full and incremental backups;
- encrypted remote backup;
- point-in-time recovery;
- cluster-consistent recovery frontiers;
- independent retention and access policy;
- restore to a new destination;
- automated restore verification;
- measured recovery point and recovery time objectives;
- recovery from operator error, malicious deletion, corrupt software, regional
  loss, and key-provider outage.

#### G. Operational maturity

- liveness, readiness, and degraded-state reporting;
- metrics and alerts for latency, saturation, capacity, replica health,
  coverage, keys, backup, lifecycle, and purge;
- admission control and noisy-neighbour containment;
- capacity planning and safe disk-full behavior;
- rolling upgrades and downgrade boundaries;
- automated configuration validation and redaction;
- runbooks for ordinary maintenance and severe incidents;
- compatibility and support policy;
- reproducible release artifacts and software bill of materials.

#### H. The decisive survivability advantage

- independently authenticated and bounded data units;
- corruption cannot unnecessarily cross frame, chunk, or segment boundaries;
- forward/reverse discovery without trusting one global catalog;
- recovered data, missing data, denied data, unavailable keys, and unsupported
  formats remain distinct;
- a destroyed catalog or control plane can be reconstructed from surviving
  evidence;
- SDA can examine active, historical, archived, and partially recovered units;
- recovery produces a machine-readable coverage map;
- users can export recovered content and evidence without a proprietary cloud
  service.

### 6.4 Evidence required

Stage 3 requires a production qualification program, not a launch benchmark:

1. comparative MongoDB-shaped workloads with published configurations;
2. sustained load, burst, skew, large-object, and long-retention tests;
3. cluster tests across node, disk, process, network, zone, and control-plane
   failures;
4. Jepsen-style consistency testing for every declared consistency mode;
5. rolling upgrade, downgrade-boundary, rebalancing, and scale-out tests;
6. encrypted full/incremental/PITR backup and clean-room restore drills;
7. key-provider outage, rotation, compromise, and lost-key exercises;
8. lifecycle, hold, TTL, tier movement, and incomplete-purge tests;
9. destructive corruption campaigns against data, metadata, catalogs,
   indexes, replicas, and backups;
10. multi-year format compatibility represented by retained golden archives
    and scheduled old-version recovery;
11. an external security assessment and published remediation status;
12. independent production references with declared workload envelopes.

Residiuum MUST publish both wins and losses. A benchmark suite designed only to
produce a victory is not competitive evidence.

### 6.5 Stage 3 exit gate

Stage 3 is complete for a named workload profile when:

- a normal application can choose Residiuum without accepting inferior everyday
  database behavior;
- the supported cluster tolerates its declared failures without violating
  acknowledged-write or consistency contracts;
- security, backup, PITR, retention, holds, purge, upgrades, and capacity are
  productized and routinely exercised;
- operators can diagnose and recover the system without maintainers editing
  files by hand;
- scale and latency are competitive for the qualified workload;
- destructive testing demonstrates materially better partial recovery than the
  incumbent configuration;
- multiple independent production deployments have completed upgrades,
  failovers, restore drills, and key rotation;
- at least one production-shaped destructive recovery exercise reconstructs
  useful data and an honest coverage map after damage that prevents ordinary
  whole-database recovery.

Only then may Residiuum make the focused claim:

> For long-lived document and event workloads where partial data survival
> matters, Residiuum is a credible alternative to MongoDB.

### 6.6 What not to build for Stage 3

Residiuum does not need:

- MongoDB wire compatibility unless migration evidence shows it is decisive;
- every MongoDB query operator;
- every programming-language driver;
- a proprietary cloud service before the self-managed product is dependable;
- universal OLTP, analytics, cache, search, vector, and archive leadership;
- to outperform MongoDB on every workload;
- to hide holes or uncertainty to make recovery output look cleaner.

---

## 7. Stage dependencies

The stages accumulate; they do not replace one another.

| Capability | Stage 1 | Stage 2 | Stage 3 |
|---|---:|---:|---:|
| Zero-service embedded use | win condition | retained | retained |
| JSON and arbitrary bytes | win condition | retained | retained |
| Bounded corruption recovery | win condition | fleet-aware | cluster/tier-aware |
| Native encryption | local profile | fleet lifecycle | KMS and tenant profiles |
| Backup/restore | full single-node | service and client policy | incremental, remote, PITR |
| Synchronization | not required | win condition | change/replication substrate |
| Conflict evidence | local history | win condition | partition/cluster scope |
| Logical heaps | one implicit compatibility heap | named and authorized | externally qualified isolation |
| Production cluster | not required | central service | win condition |
| Governed lifecycle | basic local | fleet-aware | win condition |
| Massive tiering | not required | optional | win condition |
| SDK breadth | Rust | target edge platforms | demand-led production drivers |
| Operational burden | near zero | fleet-manageable | enterprise-grade |

Stage 2 cannot compensate for a weak embedded store. Stage 3 cannot compensate
for an unreliable synchronization and cluster substrate.

## 8. Scorecard and prioritization

Every proposed feature MUST answer:

1. Which stage does it advance?
2. Which mandatory goal does it satisfy?
3. What customer decision changes when it exists?
4. What evidence will prove it?
5. Does it preserve bounded recovery?
6. What ongoing compatibility and operational burden does it create?

Priority order within a stage:

1. remove a blocker to the target workload;
2. remove a cause of silent loss or false confidence;
3. complete an end-to-end operational path;
4. make the distinctive recovery advantage demonstrable;
5. improve ordinary DX;
6. improve measured performance inside the qualification envelope;
7. add optional breadth.

A half-built feature does not count merely because its type or command exists.

## 9. Competitive truth

The three stages tell one continuous story:

### Stage 1

> Residiuum is easier and safer than assembling a local database plus loose
> files when mixed data must survive damage.

### Stage 2

> Residiuum carries that safety from one intermittently connected device into a
> synchronized fleet without hiding conflicts or holes.

### Stage 3

> Residiuum carries that same evidence model into a production document cluster
> and across decades of retained data.

The strategy in one sentence:

> Beat SQLite on survivable embedded data, beat Couchbase on survivable edge
> data, then beat MongoDB on survivable operational data at scale.

## 10. Market references

These sources establish the incumbent capabilities Residiuum must respect:

- [SQLite appropriate uses](https://www.sqlite.org/whentouse.html) — embedded,
  local, simple, reliable, zero-administration storage;
- [Couchbase Mobile](https://docs.couchbase.com/home/mobile.html) — embedded
  JSON, edge operation, synchronization, and server continuity;
- [Couchbase Lite replication](https://docs.couchbase.com/couchbase-lite/current/java/replication.html)
  — secure bidirectional change synchronization through Sync Gateway;
- [MongoDB data modelling](https://www.mongodb.com/docs/manual/data-modeling/)
  — flexible documents and access-pattern-driven schema;
- [MongoDB transactions](https://www.mongodb.com/docs/manual/core/transactions/)
  — atomic operations across documents, collections, databases, and shards;
- [MongoDB change streams](https://www.mongodb.com/docs/manual/changestreams/)
  — notifications based on majority-committed changes;
- [MongoDB Atlas operational readiness](https://www.mongodb.com/docs/atlas/architecture/current/operational-readiness-checklist/)
  — backup, PITR, retention, maintenance, and disaster-recovery expectations.

These references define credible incumbent baselines. They do not establish
Residiuum capability; only Residiuum's own qualification evidence can do that.
