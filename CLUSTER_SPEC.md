# DingoDB Cluster Architecture

Status: Draft v0.1  
Scope: Partitioning, replication, consistency, placement, failover, repair, and
distributed SDA examination

## 1. Purpose

A DingoDB cluster scales ingestion, hot access, retention, and examination
across multiple machines and failure domains without converting the cluster
control plane into a new single point of data loss.

The cluster model is:

> A federation of independently recoverable partitions and segments.

Consensus decides who may coordinate writes. It does not give bytes their
meaning. If every cluster catalog and consensus member is lost, surviving
segments remain self-identifying, independently verifiable, and salvageable.

## 2. Requirement language

The requirement words defined by the
[architecture specification](OVERVIEW.md) apply.

This specification distinguishes:

- **physical survival** — independently verified frames still exist;
- **replica durability** — the configured number of independent copies
  acknowledged a write;
- **logical commitment** — sufficient consensus evidence establishes that a
  write belongs to committed state;
- **query completeness** — every partition and tier required by the declared
  query scope was covered.

An implementation MUST NOT collapse these claims.

## 3. Goals

A conforming cluster is designed to provide:

- horizontal ingestion and read scaling;
- strong ordering within a partition;
- no mandatory global ordering;
- direct hot-path routing without cluster-wide locks;
- explicit behavior during network partitions;
- immutable-segment replication and repair;
- online rebalancing;
- tier-aware distributed examination;
- catalog-independent cluster salvage;
- preservation of conflicting verified data;
- deterministic coverage and uncertainty reporting.

## 4. Non-goals

The initial cluster profile does not promise:

- atomic transactions spanning arbitrary partitions;
- a global serial order over every event;
- synchronous global secondary indexes;
- write availability in both sides of a partition while also claiming one
  linearizable history;
- that one surviving replica proves a formerly replicated write was committed;
- that consensus metadata is authoritative payload storage.

## 5. Terms

**Cluster**  
A set of nodes participating under one cluster identifier and control policy.

**Node**  
One independently failing process and its attached storage.

**Failure domain**  
A group expected to fail together, such as a process, host, rack, zone,
provider, or archive medium.

**Partition**  
The unit of write ordering, ownership, replication, failover, and balancing.

**Partition key**  
The stable logical value used to select a partition.

**Replica set**  
The nodes assigned to retain a partition's authoritative frames.

**Leader**  
The node currently authorized to order writes for a strong partition.

**Term**  
A monotonically increasing partition leadership generation established by
consensus.

**Placement epoch**  
A monotonically increasing generation of the partition-to-replica assignment.

**Partition directory**  
The derived routing map from partitions to replica sets, leaders, terms, and
placement epochs.

**Control plane**  
The replicated service that coordinates membership, placement, policy, and
leadership.

**Data plane**  
The direct path for item reads, writes, replication, segment movement, and
queries.

**Coverage**  
Evidence describing which partitions, replicas, indexes, and tiers
participated in an operation.

## 6. Cluster invariants

### 6.1 Independent Node Salvage

Every storage node MUST remain salvageable as a collection of ordinary
DingoDB segments if removed from the cluster.

Node-local authoritative data MUST NOT require a live control plane, remote
metadata service, or other node merely to identify and verify its frames.

### 6.2 Control Plane Is Not Payload Authority

The control plane MAY authorize writes and accelerate location discovery. It
MUST NOT be the only holder of:

- item or event identity;
- payload interpretation metadata;
- segment identity;
- frame integrity evidence;
- the only mapping from a segment to its store and partition.

Loss of the entire control plane may stop strongly consistent new writes. It
MUST NOT prevent local reads, scans, or salvage of available segments.

### 6.3 Partition-Local Coordination

Ordinary reads and writes to one partition MUST NOT require agreement from
unrelated partitions.

The cluster MUST NOT place a cluster-wide consensus operation, global
sequence, or global lock on the ordinary item hot path.

### 6.4 Explicit Split-Brain Behavior

During a network partition, the system MUST choose behavior according to the
declared consistency mode. It MUST NOT acknowledge conflicting writes on both
sides and later describe the result as one linearizable history.

### 6.5 Conflict Preservation

If two verified frames claim the same logical identity but different content,
both MUST survive examination until an explicit, evidence-recorded resolution
policy is applied.

Physical encounter order, node identity, or catalog preference MUST NOT
silently destroy a conflicting frame.

### 6.6 Placement Is Derived

Placement determines where data should be, not what data is.

Loss or corruption of a placement record MUST NOT invalidate a verified frame.
A placement directory MUST be rebuildable from control-plane history, node
inventories, and segment-local descriptors.

### 6.7 Coverage Is Part of Every Distributed Result

A distributed query, scan, or recovery result MUST state its coverage.

An unavailable partition, replica, index, or tier MUST NOT be represented as
an empty successful partition.

## 7. Node roles

Nodes MAY perform one or more roles:

- **storage** — retain authoritative segments and chunks;
- **ingest** — accept and coordinate writes;
- **query** — plan and merge distributed SDA work;
- **index** — build and serve derived indexes;
- **control** — participate in cluster consensus;
- **gateway** — authenticate, route, and enforce admission policy;
- **archive** — manage high-latency or offline tiers.

Roles are deployment choices, not distinct storage formats.

A small cluster MAY run every role on every node. A large cluster SHOULD
separate resource-intensive indexing and archival work from latency-sensitive
ingest paths.

## 8. Partitioning

### 8.1 Partition key

Every partitioned event MUST have a stable partition key.

The default partition key is:

1. `subject_id`, when the event has one;
2. otherwise `item_id`;
3. otherwise `event_id`.

Applications MAY declare another key before data is written.

Events that must share strong ordering MUST use the same partition key.

Changing a partition key is a migration, not an in-place metadata edit.

### 8.2 Virtual partitions

The cluster SHOULD map partition keys into a large fixed space of virtual
partitions using a published deterministic hash profile.

Virtual partitions are assigned to replica sets through the partition
directory.

The virtual partition count and hash profile belong to a store generation.
They MUST NOT change silently.

### 8.3 Partition identity in storage

Every clustered authoritative frame MUST carry or locally derive:

- cluster identifier;
- store identifier;
- partition identifier;
- placement epoch observed by the writer;
- partition term;
- partition log position or event ordering key;
- event identifier.

These fields allow segments from destroyed nodes to be regrouped without the
original directory.

### 8.4 Large payload chunks

Chunk placement MAY use chunk identity rather than the parent item's partition
key.

The parent manifest MUST identify every chunk and the placement profile used.
Losing chunk-placement metadata MUST not erase the chunk identity or prevent a
content-addressed salvage search.

## 9. Consistency modes

A store or namespace declares one of two core write modes.

### 9.1 `partition-linearizable`

This is the default mode for mutable logical subjects.

Properties:

- one consensus-authorized leader orders writes in a partition term;
- a quorum establishes logical commitment;
- acknowledged committed writes form one linearizable order per partition;
- minority replicas cannot acknowledge committed writes;
- reads that claim linearizability consult the leader, a valid leader lease,
  or a read quorum under the consensus protocol;
- no ordering is implied between different partitions.

During loss of quorum, verified existing data remains readable according to
the selected read mode. New committed writes pause.

### 9.2 `convergent-append`

This mode favors ingestion availability for immutable, naturally mergeable
events.

Properties:

- any authorized reachable ingest node MAY accept a uniquely identified
  append;
- events do not claim one real-time total order;
- replicas reconcile by event identity and content hash;
- conflicts and duplicates remain explicit;
- projections MUST declare a deterministic merge rule;
- a network split may accept events on multiple sides.

This mode is suitable for independent logs, telemetry, captured objects, and
other data where retaining both sides is more important than a single current
value.

It MUST NOT be advertised as linearizable.

### 9.3 Read modes

Read requests declare one of:

**linearizable**  
Return data proven current under the partition consensus protocol, or fail
explicitly when quorum evidence is unavailable.

**bounded-stale**  
Return replica state no older than a declared term, position, or age bound.

**available**  
Return the best verified local or reachable state with exact position,
coverage, and uncertainty.

**salvage**  
Return all relevant verified, partial, conflicting, and uncommitted physical
evidence without projecting one authoritative current state.

The default for `partition-linearizable` stores is `linearizable`. Recovery
tools default to `salvage`.

## 10. Per-partition consensus

### 10.1 Protocol requirement

The strong cluster profile uses a proven leader-and-quorum replicated-log
protocol equivalent in safety properties to Raft.

The project MAY adopt an existing implementation. It MUST publish:

- election and term rules;
- log matching and commitment rules;
- membership-change protocol;
- persistence boundaries;
- leader lease assumptions, if leases are used;
- read-index or quorum-read behavior;
- snapshot and recovery behavior.

Inventing an informal consensus protocol does not conform.

### 10.2 Replica count and quorum

For `N` voting replicas, the normal write quorum is:

```text
floor(N / 2) + 1
```

The default active profile SHOULD use three voting replicas across three
independent host failure domains.

Deployments requiring zone-loss tolerance SHOULD place voters across at least
three zones and evaluate correlated-failure assumptions explicitly.

Non-voting learners MAY receive copies without affecting quorum.

### 10.3 Leadership fencing

Every leader write carries the current term and placement epoch.

Replicas MUST reject a write from:

- an older term;
- an obsolete placement epoch after the new epoch is activated;
- a node not authorized by the committed membership;
- an event position conflicting with an already accepted different frame.

Wall-clock time alone MUST NOT fence a former leader.

### 10.4 Commitment and physical survival

A frame may physically survive without sufficient evidence that it committed.

Cluster recovery distinguishes:

- `committed` — quorum/consensus evidence proves commitment;
- `prepared` — frame was accepted by at least one replica but commitment is
  unproven;
- `conflicting` — verified evidence cannot belong to one valid committed log;
- `unknown-commit` — required consensus evidence is missing.

Prepared and unknown-commit frames remain available in salvage mode. They MUST
NOT silently enter committed current state.

### 10.5 Commit evidence

Replica logs and periodic partition checkpoints MUST retain enough term,
position, membership, and commit-index evidence to reconstruct commitment when
the required quorum evidence survives.

The cluster MAY emit signed or quorum-attested commit certificates to make
commit proof more portable. A certificate is additional authoritative
evidence, not a prerequisite for physical frame recovery.

## 11. Replication

### 11.1 Frame replication

Replicas SHOULD transfer exact encoded authoritative frames. A receiver MUST
verify the frame before acknowledging it.

Re-encoding during ordinary replication is forbidden. Format migration is a
separate evidence-recorded operation.

### 11.2 Acknowledgement

A replicated acknowledgement identifies:

- consistency mode;
- durability mode;
- partition identifier;
- term and log position when applicable;
- placement epoch;
- number and class of acknowledgements;
- whether logical commitment is proven.

“Replicated” without those details is not a complete durability claim.

### 11.3 Anti-entropy

Replica sets SHOULD exchange hierarchical inventories based on:

- partition ranges;
- segment identities;
- frame or region integrity roots;
- event-position ranges;
- content-addressed chunk sets.

Anti-entropy MUST compare verified identities and hashes. It MUST NOT infer
equality merely from file size, modification time, or catalog generation.

### 11.4 Repair

Repair copies verified frames or reconstructs erasure-coded material into new
storage.

Repair MUST:

- preserve item and event identities;
- verify the source and destination;
- record source replicas or fragments;
- record the repair tool and algorithm;
- avoid overwriting the only surviving conflicting evidence;
- publish new placement only after verification.

## 12. Control plane

### 12.1 Responsibilities

The control plane coordinates:

- cluster membership;
- node identity and authorization;
- partition placement;
- leadership and terms;
- store consistency and durability policy;
- storage-tier policy;
- schema/profile registrations;
- placement and membership epochs.

It does not store the only copy of user payloads.

### 12.2 Availability

The control plane itself MUST use replicated consensus across independent
failure domains.

Loss of control-plane quorum:

- stops membership and placement changes;
- may stop new strong writes whose leadership cannot be proven;
- does not invalidate current leader leases within their documented safety
  assumptions;
- does not stop node-local salvage or verified local reads;
- does not make segments undecodable.

### 12.3 Disaster reconstruction

If every control-plane copy is lost, an administrator can construct a
replacement control plane by:

1. inventorying surviving nodes and media;
2. grouping frames by cluster, store, and partition identifiers;
3. verifying segment and consensus evidence;
4. retaining every conflict and unknown-commit frame;
5. selecting a new cluster generation through an explicit recovery ceremony;
6. writing new placement records without rewriting surviving payload frames.

The reconstructed control plane MUST use a new recovery generation and MUST
not claim continuity that the surviving evidence cannot prove.

## 13. Routing

Clients MAY route through gateways or directly to partition leaders.

The hot path SHOULD support:

1. cache partition directory entries;
2. route directly to the leader or selected replica;
3. receive an explicit stale-epoch response on misrouting;
4. refresh only the affected directory entry;
5. retry using the same event identifier.

Retries MUST be idempotent by event identifier and content identity.

Directory caches MUST have bounded staleness or epoch validation. A stale
route may cost a redirect; it MUST NOT authorize an obsolete writer.

## 14. Rebalancing

Rebalancing one partition follows:

1. commit a new placement plan containing old and proposed replica sets;
2. add destination nodes as non-voting learners;
3. copy and verify immutable segments;
4. stream and verify the active log tail;
5. wait until destinations reach the declared safe position;
6. perform a consensus-safe membership change;
7. activate a new placement epoch;
8. retain old replicas for a configurable safety window;
9. reclaim old copies only after independent placement and durability checks.

Failure at any step leaves either the old placement authoritative or an
explicit joint configuration. It MUST NOT leave an unrecorded ownership gap.

Immutable historical segments SHOULD move without rewriting.

## 15. Node and network failures

### 15.1 Node loss

Node loss affects only partitions and unique archival material present on that
node.

Partitions retaining quorum continue strong operation.

Partitions retaining verified copies but not quorum remain readable under
available or salvage modes. Strong writes pause.

### 15.2 Network partition

In `partition-linearizable` mode, only a side retaining quorum may commit.
Minority-side reads expose their mode, position, and uncertainty.

In `convergent-append` mode, multiple sides may accept unique events.
Reconciliation preserves both sides and applies the declared deterministic
projection later.

### 15.3 Corrupt replica

A corrupt replica MUST NOT repair healthy replicas merely because it has a
newer timestamp or catalog generation.

Repair source selection uses verified hashes, consensus positions, replica
agreement, and recorded provenance.

A corrupt frame is quarantined as evidence when policy permits. Healthy frames
in the same segment remain eligible for replication and salvage.

### 15.4 Catastrophic partial destruction

If half the cluster media is deleted or overwritten:

- every intact frame on remaining media stays independently readable;
- recoverable replicas are regrouped by embedded identities;
- lost partitions and ranges become explicit holes;
- a surviving minority does not fabricate quorum commitment;
- convergent events remain available;
- strong-state projections become uncertain where commitment or history
  evidence is insufficient;
- catalogs and placement are rebuilt from surviving evidence.

## 16. Distributed indexing

Indexes are partition-local or explicitly partitioned derived structures.

A global secondary index is a distributed collection of index partitions. It
MUST expose:

- source partition coverage;
- authoritative positions covered;
- build version;
- known stale or missing index partitions;
- tiers excluded from indexing.

An index miss proves absence only when the index proves complete coverage for
the query scope and authoritative frontier.

Index rebuilding MUST NOT block authoritative segment examination.

## 17. Distributed SDA examination

### 17.1 Execution

A distributed SDA query proceeds:

1. resolve the declared store and partition scope;
2. record the directory and index generations used;
3. prune candidate partitions and tiers;
4. send the pure SDA program or a verified equivalent plan to workers;
5. verify authoritative candidate frames;
6. evaluate bounded partition-local pages;
7. merge results using a declared deterministic order;
8. return coverage, holes, conflicts, and continuation state.

Pushdown MUST preserve SDA semantics. An optimization that changes carrier,
ordering, absence, duplicate, `Null`, or failure behavior is invalid.

### 17.2 Coverage record

Distributed query coverage includes:

- requested partitions;
- completed partitions;
- unavailable partitions;
- per-partition term and position;
- indexes and coverage frontiers used;
- tiers searched;
- tiers excluded, offline, or timed out;
- resource limits reached;
- consistency/read mode.

A partial distributed result is valid data with incomplete coverage. It MUST
not be represented as a complete result.

### 17.3 Deterministic merge

Unless a query declares another total ordering, cluster merge order follows
the DingoDB SDA profile using stable partition and unit identity, never worker
completion order.

Set and Bag results retain their SDA extensional semantics. Sequence results
require an explicit deterministic partition merge order.

### 17.4 Query coordinator failure

Query pages are bounded and carry query identity, coverage, and continuation
state.

A replacement coordinator MAY resume from authenticated continuation state.
It MUST not claim partitions already returned unless duplicate delivery is
explicitly allowed by the query profile.

## 18. Tiered clustered storage

Active replication and archival redundancy are separate policies.

A partition MAY use:

- replicated hot segments for latency;
- erasure-coded cold segments for capacity efficiency;
- additional offline copies for disaster survival.

Moving a segment to archive does not remove its partition identity.

The directory tracks probable location and availability. Segment-local
descriptors remain sufficient for disaster regrouping.

An archive tier returning “not presently mounted” creates coverage uncertainty,
not proof of absence.

## 19. Cross-partition operations

### 19.1 Reads

Multi-partition reads are not one linearizable snapshot unless a future
snapshot protocol explicitly establishes and reports one.

The default result reports a frontier per partition.

### 19.2 Writes

The v1 cluster profile does not define atomic cross-partition writes.

Applications use:

- independent idempotent events;
- sagas with compensating events;
- one partition key for data requiring atomic ordering;
- explicit batch workflows whose partial state remains examinable.

A qualified `Partition` Atomic profile may add indivisible commitment inside
one partition. A transaction-shaped API is only a compatibility projection of
that profile. Neither profile permits cross-partition atomicity, and both MUST
preserve physical salvage of prepared and partially replicated frames.

## 20. Performance strategy

The cluster hot path avoids global coordination:

```text
client → cached partition route → partition leader → replica quorum
```

Performance mechanisms include:

- many independent virtual partitions;
- sharded leaders and append paths;
- memory-resident partition indexes;
- pipelined and batched replication;
- direct reads from safe replicas;
- immutable-segment zero-copy transfer where supported;
- asynchronous global indexing;
- query pushdown and partition pruning.

Benchmark reports MUST disclose:

- partition count and distribution;
- replica count and failure-domain placement;
- consistency and read mode;
- quorum and durability mode;
- cross-zone or cross-region latency;
- rebalancing or repair activity;
- hot-set locality;
- skew and largest-partition load.

Redis-class latency is a hot, routed, memory-resident target. It is not a claim
that quorum replication across distant regions is free.

## 21. Security

Nodes have stable cryptographic identities.

Control-plane membership changes, placement epochs, and leadership terms MUST
be authenticated.

Data-plane peers MUST authenticate before accepting replication or repair
traffic.

Authorization to write does not imply authority to purge.

Disaster reconstruction and force-reconfiguration operations MUST create
durable audit evidence and a new recovery generation.

## 22. Conformance tests

A conforming cluster implementation MUST test:

1. leader loss before and after local append;
2. leader loss before and after quorum replication;
3. acknowledgement loss and idempotent retry;
4. old-leader writes after a new term;
5. stale placement routes;
6. minority and majority network partitions;
7. simultaneous convergent appends on both sides;
8. conflicting event identifiers;
9. control-plane quorum loss;
10. complete control-plane destruction followed by reconstruction;
11. replica corruption with a misleading newer timestamp;
12. online partition movement interrupted at every persistent step;
13. loss of source nodes during rebalancing;
14. missing global and partition indexes;
15. partial distributed queries;
16. offline archive tiers;
17. query-coordinator replacement;
18. deletion of half the cluster's segment files;
19. salvage of a node with no cluster software running;
20. deterministic SDA results under randomized worker completion order.

Every test MUST verify physical survival, logical commitment, and query
coverage as separate claims.

## 23. Default deployment profiles

### 23.1 Development

- one node;
- one replica;
- partition-linearizable semantics without fault tolerance;
- explicit warning that replicated durability is unavailable.

### 23.2 Dependable local cluster

- three voting storage nodes;
- three control-plane voters;
- virtual partitions distributed across all nodes;
- quorum durable acknowledgement;
- hot and warm local storage;
- independent backup or archive copy.

### 23.3 Zone-tolerant cluster

- at least three voting replicas across three zones;
- control-plane voters across the same or stronger failure domains;
- quorum writes;
- partition leaders placed near clients where possible;
- asynchronous cold-tier erasure coding;
- periodic destructive recovery exercises.

### 23.4 Massive archive cluster

- convergent append where application semantics permit;
- independently replicated ingest buffers;
- immutable sealed segments;
- content-addressed chunks;
- partitioned catalogs and indexes;
- erasure-coded cold storage;
- offline or provider-independent archive copy;
- scheduled scrubbing and format migration.

## 24. Governing principle

Clustering increases the number of copies and coordinators. It must not
increase the amount of data that one broken coordinator can make meaningless.

> Consensus controls the right to write. The data remains able to speak for
> itself.
