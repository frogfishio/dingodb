# DingoDB Architecture Specification

Status: Draft v0.1
Scope: Storage, recovery, retention, examination, and performance architecture

## 1. Purpose

DingoDB is a damage-tolerant, high-performance, long-retention database for
arbitrary digital material.

It is designed around four promises:

1. **Independent survival** — damage causes localized loss. Every intact data
   island before or after a hole remains recoverable.
2. **Extreme speed** — indexed hot reads approach memory-store performance and
   ingestion is append-oriented, parallel, and minimally coordinated.
3. **Massive retention** — one logical database may span storage tiers,
   machines, media generations, and decades.
4. **SDA examination** — every recoverable item and every reported hole has a
   deterministic representation that SDA can examine.

DingoDB stores structured records, events, logs, documents, binary objects,
application state, unknown formats, malformed input, and uninterpreted bytes.
It does not require a payload to be understood before preserving it.

The governing rule is:

> What is gone is gone. What remains still lives.

## 2. Requirement language

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, MAY, and OPTIONAL are normative.

An implementation conforms to this specification only when it satisfies every
MUST and MUST NOT requirement applicable to the features it implements.

## 3. Terms

**Store**
A logical DingoDB namespace containing items, segments, chunks, derived
structures, and recovery evidence.

**Item**
The logical unit submitted by a caller. An item consists of an envelope and a
payload.

**Envelope**
Self-describing metadata sufficient to identify, frame, verify, and interpret
an item without consulting a global catalog.

**Payload**
Structured data or opaque bytes preserved by DingoDB.

**Frame**
The independently delimited and verified physical representation of one
storage event or part of an item.

**Segment**
An immutable, independently discoverable collection of frames. A segment is
the unit of movement, replication, tiering, sealing, and coarse verification.

**Chunk**
An independently addressed piece of a large payload.

**Data island**
A contiguous region containing one or more independently verifiable frames,
surrounded by damaged, missing, unknown, or unscannable regions.

**Hole**
A known discontinuity where bytes, frames, segments, or dependencies are
missing, corrupt, overwritten, or undecodable.

**Verified**
The stored bytes satisfy all applicable framing and integrity checks.

**Recovered**
An item or frame has been rediscovered without relying on the original catalog
or index.

**Complete**
All dependencies required for the claimed logical interpretation are present
and verified.

**Uncertain**
Physical data survives, but DingoDB cannot prove that the derived logical
interpretation is complete.

**Authoritative data**
Frames, envelopes, payload chunks, and explicit storage events accepted under
a declared durability mode.

**Derived data**
Indexes, catalogs, caches, search structures, projections, and snapshots that
can be regenerated from authoritative data.

## 4. Fundamental invariants

### 4.1 Independent Survival Invariant

DingoDB MUST recover every independently verifiable frame or segment that
remains physically available.

A missing, corrupt, overwritten, or undecodable region MUST NOT prevent
recovery of valid data before or after that region.

Recovery MUST NOT stop permanently at the first damaged region.

### 4.2 Localized Loss Invariant

Physical damage to one frame MUST NOT invalidate another frame solely because
of adjacency, ordering, or a broken global chain.

Loss of a segment MUST NOT make unrelated segments unreadable.

Loss of every catalog, manifest, index, and snapshot MUST NOT prevent a full
salvage scan of surviving authoritative data.

### 4.3 Explicit Hole Invariant

DingoDB MUST report discovered discontinuities as holes. It MUST NOT silently
join data on opposite sides of a hole and claim continuity.

The recovery interface MUST expose, when known:

- the physical or logical range affected;
- the reason for the hole;
- the evidence used to locate its boundaries;
- the items or derivations whose completeness may depend on it.

### 4.4 Physical Survival vs Semantic Completeness

Physical survival and semantic completeness are separate claims.

An intact update event after a hole is physically valid even when an earlier
event required to reconstruct current state may be missing. DingoDB MUST return
the surviving event. A state projection that depends on the missing event MUST
be marked incomplete or uncertain.

DingoDB MUST NOT discard verified data merely because its full semantic
context is unavailable.

### 4.5 No Essential Derived State

Derived data MUST be deletable and rebuildable.

No read, recovery, migration, or repair procedure may require a unique,
irreplaceable index or manifest to interpret an authoritative frame.

### 4.6 Payload Neutrality

DingoDB MUST be able to preserve opaque bytes without understanding their
media type, schema, encoding, or application semantics.

Failure to decode a payload MUST NOT invalidate a verified envelope or the
payload bytes.

### 4.7 Stable Evidence

Recovery and verification results MUST distinguish facts from inference.
Implementations MUST NOT label unverified salvage as verified data.

## 5. Logical data model

### 5.1 Item envelope

Every item MUST carry, directly or through a locally self-contained frame:

- format family and version;
- item identifier;
- event identifier;
- event kind;
- payload representation;
- payload length or chunk manifest;
- integrity algorithm identifier;
- payload integrity value;
- envelope integrity value;
- creation or ingestion time, when supplied;
- enough framing information to skip or recover the item.

An item MAY additionally carry:

- logical subject identifier;
- schema or media-type identifier;
- source and provenance;
- labels and user metadata;
- causal parents;
- application sequence;
- compression and encryption descriptors;
- signatures;
- retention policy;
- previous-version references;
- derived extraction metadata.

Envelope fields required for recovery MUST NOT exist only in a global catalog.

### 5.2 Identifiers

Item and event identifiers MUST remain stable across tier migration,
replication, reindexing, and physical relocation.

Physical addresses MUST NOT be used as logical identifiers.

An implementation MAY use random, time-sortable, caller-supplied, or
content-derived identifiers. The identifier scheme MUST be recorded in the
envelope or store profile.

Identifier collision handling MUST be deterministic and MUST NOT silently
replace existing authoritative data.

### 5.3 Payload representations

The core storage model recognizes:

- inline opaque bytes;
- inline structured values;
- chunked opaque bytes;
- chunked structured values;
- external references, when explicitly requested by the caller.

An external reference is not preserved content. APIs and examination results
MUST distinguish stored payloads from references to content outside DingoDB.

### 5.4 Events and state

DingoDB records immutable storage events. Core event kinds are:

- `put` — associate a payload and envelope with a logical subject;
- `delete` — record a logical deletion;
- `link` — record a relationship;
- `checkpoint` — record a self-contained state projection;
- `metadata` — add or supersede descriptive metadata;
- `repair` — record an explicit repair or reconstruction;
- `purge` — attest that selected authoritative content was deliberately
  removed.

An update is a new event. Existing sealed authoritative frames MUST NOT be
modified in place.

Applications MAY define additional event kinds.

### 5.5 Current-state projections

Current state is a derived projection over surviving events.

A projection MUST declare:

- the ordering rule it uses;
- its conflict policy;
- its behavior when dependencies are missing;
- whether it requires a gap-free event history;
- how uncertainty is represented.

No projection may claim complete current state when a known hole could change
the result.

## 6. Physical storage architecture

### 6.1 Store layout

A filesystem-backed store SHOULD use a layout equivalent to:

```text
store/
  store-info/
  active/
  segments/
  chunks/
  catalogs/
  indexes/
  snapshots/
  recovery/
```

Directory names are not normative. Their roles are.

Authoritative segments and chunks MUST remain interpretable if `catalogs/`,
`indexes/`, `snapshots/`, and ordinary store manifests are deleted.

### 6.2 Active and sealed segments

New frames are appended to active segments. A segment becomes immutable when
sealed.

At most the incomplete tail of an active segment may be lost due to an
interrupted append under a durability mode that acknowledged earlier frames.

A sealed segment MUST NOT be modified in place. Repair, compaction, metadata
augmentation, and re-encoding create new segments and preserve evidence
linking them to their sources.

### 6.3 Segment self-description

Each segment MUST contain enough local information to:

- identify it as a DingoDB segment;
- determine its format version;
- determine the integrity algorithms in use;
- locate candidate frame boundaries;
- distinguish an active segment from a sealed segment when possible;
- verify surviving frames independently;
- enumerate or reconstruct its recoverable contents.

A sealed segment SHOULD contain redundant summary information near physically
separated regions, normally a header and trailer. Loss of either summary MUST
NOT make individually recoverable frames inaccessible to salvage scanning.

### 6.4 Frame requirements

Every authoritative frame MUST provide:

- a synchronization marker with sufficiently low accidental-match
  probability;
- a format/version discriminator;
- a frame type;
- a bounded encoded length;
- a header integrity check;
- a body integrity check;
- an item or event identifier;
- an unambiguous end boundary or a validated route to the next candidate
  boundary.

Critical framing fields MUST be protected by an integrity check independent of
the payload body check.

A corrupt length field MUST NOT force the scanner to skip the remainder of a
segment. The scanner MUST be able to resume searching for a later
synchronization marker.

Frame decoding MUST apply implementation-defined safety limits before
allocating memory based on untrusted lengths.

### 6.5 Synchronization and false positives

The physical encoding MUST support resynchronization after arbitrary bytes.

Candidate synchronization markers MUST be validated using additional
independent evidence such as version constraints, bounded lengths, header
checksums, body checksums, and identity fields.

The format specification MUST publish the probability or security assumptions
behind accidental marker acceptance.

Recovery tools MUST permit stricter verification modes that trade speed for
additional evidence.

### 6.6 JSON and human examination

JSONL MAY be used as an import, export, diagnostic, or profile format. Plain
newline framing alone does not satisfy the independent-survival requirements
for the primary high-performance format.

The canonical physical encoding MAY be binary. It MUST be openly specified and
MUST have a deterministic, lossless diagnostic projection suitable for SDA and
standard text tooling.

Human inspectability means that no proprietary service or irreplaceable
metadata is required to examine surviving data. It does not require the hot
path to parse textual JSON.

### 6.7 Chunked payloads

Payloads larger than an implementation-defined inline threshold SHOULD be
stored as independently verified chunks.

Each chunk MUST have:

- a stable chunk identifier;
- encoded and logical lengths;
- integrity algorithm and value;
- compression/encryption descriptors when applicable;
- sufficient framing for independent recovery.

A chunk manifest MUST identify every required chunk and its logical order.

If some chunks are missing, DingoDB MUST return the surviving chunks and an
explicit completeness map. It MUST NOT present a partial payload as complete.

Content-addressed chunks MAY be deduplicated. Reference counts are derived
state and MUST NOT be the sole evidence that a chunk is live.

### 6.8 Compression

Compression MUST be independently bounded. Damage to one compression unit MUST
NOT require discarding unrelated frames or chunks.

Segment-wide compression that makes all later frames dependent on an earlier
compression stream state does not conform to independent survival.

### 6.9 Encryption

Encryption domains SHOULD be frame- or chunk-local. Loss or corruption of one
authentication tag MUST NOT prevent authentication of unrelated data.

Key loss is a semantic hole even if ciphertext bytes survive. Examination MUST
report `encrypted-unavailable`, not `corrupt`, unless corruption is separately
proven.

## 7. Write path and durability

### 7.1 Append path

The ordinary write path SHOULD consist of:

1. validate bounded envelope fields;
2. assign identifiers;
3. encode and verify a complete frame in memory;
4. reserve append space;
5. append the frame;
6. publish visibility according to the selected durability mode;
7. update derived structures asynchronously where allowed.

Readers MUST NOT observe a frame as verified until the complete frame and its
integrity evidence are readable.

### 7.2 Durability modes

Every acknowledged write MUST be associated with an explicit durability mode.
At minimum, implementations SHOULD expose:

**memory**
Acknowledgement after process-memory publication. Process or power failure may
lose the write.

**buffered**
Acknowledgement after transfer to the operating system or device queue. Power
failure may lose recent writes.

**durable**
Acknowledgement only after the authoritative bytes and required allocation
metadata have crossed the implementation's declared stable-storage boundary.

**replicated**
Acknowledgement after the durable condition holds on the configured number of
independent failure domains.

Implementations MUST document the exact failure boundary of each mode.
Performance claims MUST identify the durability mode measured.

### 7.3 Atomicity

A single frame is the minimum atomic storage unit.

An interrupted frame MUST be reported as incomplete or corrupt and MUST NOT
invalidate earlier or later complete frames.

Multi-item Atomics MAY be implemented using independently recoverable
prepare, member, and decision frames. Members that survive without a verified
committed decision remain physically recoverable but MUST NOT be exposed as
committed logical state. Every multi-item Atomic declares and remains inside
one qualified coordination scope.

### 7.4 Concurrency

Implementations SHOULD shard active append paths to avoid a single global
writer lock.

Ordering is guaranteed only within a declared ordering domain. If a caller
requires a total order, it MUST select or create one explicitly.

Wall-clock time MUST NOT be the sole conflict-resolution mechanism.

## 8. Recovery

### 8.1 Recovery objectives

Recovery maximizes verified salvage. It is not limited to restoring a
previously cataloged database image.

Recovery output MUST be able to contain:

- verified frames;
- verified envelopes with unavailable payloads;
- partial chunked payloads;
- candidate frames not yet fully verified;
- explicit holes;
- conflicts;
- rejected false-positive frame candidates;
- provenance describing where each result was found.

### 8.2 Recovery algorithm

A conforming salvage scan performs the logical equivalent of:

1. discover candidate media objects without trusting the primary catalog;
2. identify candidate segment regions;
3. search for synchronization markers;
4. validate bounded header fields;
5. validate header integrity;
6. locate and validate the body;
7. emit verified frames;
8. on failure, record evidence and resume scanning beyond the failed
   candidate;
9. reconcile duplicates and replicas without discarding conflicting verified
   material;
10. build a new catalog from recovered evidence.

The algorithm MUST make progress after encountering arbitrary garbage.

### 8.3 Recovery states

At minimum, the recovery API MUST distinguish:

- `verified-complete`;
- `verified-partial`;
- `verified-envelope`;
- `candidate-unverified`;
- `corrupt`;
- `missing`;
- `encrypted-unavailable`;
- `format-unsupported`;
- `conflicting`;
- `uncertain-derived-state`.

Implementations MAY add states but MUST NOT collapse unverified and verified
results into one state.

### 8.4 Hash chains and signatures

Hash chains MAY prove ordering or omission within a declared chain. They MUST
NOT be required to read or verify an otherwise independent later frame.

A broken chain creates evidence of a hole. It does not automatically invalidate
the surviving chain suffix.

Signatures MAY authenticate frames, segment summaries, and checkpoints.
Failure to authenticate MUST be reported separately from physical corruption.

### 8.5 Catalog loss

A recovered catalog is a new derived artifact. Recovery MUST preserve the
identity and evidence of the original segments; it MUST NOT rewrite recovered
authoritative bytes as a prerequisite to examination.

## 9. Massive-scale retention

### 9.1 Segment fabric

A DingoDB store is logically a segment fabric. Segments may reside:

- in memory;
- on local solid-state or rotating media;
- on network storage;
- in object storage;
- in archival storage;
- offline;
- on replicated or erasure-coded media.

Location is derived metadata. Relocating a segment MUST NOT change its logical
identity.

No operation over one segment may require rewriting all other segments.

### 9.2 Storage tiers

Implementations MAY define any number of tiers. A typical profile provides:

- **hot** — memory-resident indexes and active working data;
- **warm** — locally or remotely available sealed segments;
- **cold** — low-cost object storage;
- **archive** — high-latency, multi-decade retention.

Tiering MUST preserve the item, event, segment, and chunk identities.

A query plan MUST be able to report that relevant data exists in an offline or
high-latency tier rather than silently treating it as absent.

### 9.3 Catalog hierarchy

Massive stores SHOULD use a hierarchy of replaceable catalogs:

- frame summaries inside segments;
- segment summaries;
- partition catalogs;
- global catalogs and search indexes.

Each higher layer accelerates discovery but is not authoritative.

Loss of a higher layer increases recovery or query cost; it MUST NOT erase the
lower-layer data it described.

### 9.4 Late understanding

Payload interpretation is late-bound.

An item stored as opaque bytes MAY later acquire:

- a decoder;
- a schema;
- extracted text;
- labels;
- relationships;
- full-text terms;
- embeddings;
- SDA-defined projections.

New interpretations MUST be stored as derived data or new immutable metadata
events. They MUST NOT rewrite the original payload silently.

### 9.5 Fifteen-year readability

Long-retention profiles MUST define:

- format-version support policy;
- integrity-scrubbing schedule;
- replica or erasure-code policy;
- media-refresh policy;
- encryption-key retention policy;
- migration evidence;
- obsolete-codec handling;
- catalog rebuild procedure.

Scrubbing MUST read and verify authoritative bytes, not merely trust catalog
checksums.

Migration MUST preserve original identities and record the transformation,
source integrity values, destination integrity values, and tool version.

### 9.6 Replication and erasure coding

Replication and erasure coding protect against actual media loss. They are
separate from independent frame recovery.

Redundancy groups SHOULD cross independent failure domains.

Repair MUST be evidence-preserving. A reconstructed copy MUST identify the
source fragments and reconstruction method used.

### 9.7 Garbage collection

Physical reclamation is explicit.

Garbage collection MUST NOT delete authoritative content based solely on a
single derived reference count, catalog, or index.

Before reclamation, an implementation MUST establish liveness using the
configured retention policy and sufficient independent evidence.

Purges SHOULD produce durable attestations identifying what was removed, why,
under whose authority, and from which redundancy domains.

## 10. Indexes and search

### 10.1 Index status

All indexes are derived. Examples include:

- identifier indexes;
- time indexes;
- event-kind and media-type indexes;
- label and metadata indexes;
- full-text indexes;
- relationship indexes;
- semantic/vector indexes;
- SDA-defined materialized projections.

An index entry is a candidate locator, not proof that an item exists or is
valid. Reads MUST verify authoritative frames according to the selected
verification policy.

### 10.2 Index freshness

Every index MUST expose or internally track:

- the authoritative range it covers;
- build version and parameters;
- known missing partitions;
- staleness or checkpoint position;
- whether its result can prove absence.

An incomplete index MUST NOT turn “not indexed” into “does not exist.”

### 10.3 Query pruning

At massive scale, query execution SHOULD proceed:

1. prune partitions using catalogs and summaries;
2. select candidate segments using indexes;
3. fetch or stage required tiers;
4. verify candidate frames;
5. decode only required payload regions;
6. evaluate SDA;
7. surface holes and uncertainty with results.

Full scans MUST stream and MUST NOT require the store to fit in memory.

## 11. SDA examination

### 11.1 Examination boundary

SDA is DingoDB's deterministic examination and transformation algebra.

DingoDB MUST expose an SDA value for:

- every verified envelope;
- every decodable structured payload;
- every opaque payload descriptor;
- every partial-payload map;
- every hole;
- every verification and uncertainty state.

Opaque bytes remain bytes. SDA is not required to infer a structure.

### 11.2 Examination record

The standalone DingoDB SDA profile exposes examination units with fixed fields
for identity, physical location, integrity, payload availability, holes,
provenance, and uncertainty. A minimal complete item is equivalent to:

```sda
Prod{
  unit_kind: "item",
  status: "verified-complete",
  store_id: Some("001122..."),
  segment_id: Some("aabbcc..."),
  item_id: Some("112233..."),
  event_id: Some("445566..."),
  event_kind: Some("put"),
  physical: Prod{
    source: "segments/0001.dingo",
    offset: Some(4096),
    encoded_length: Some(512),
    wire_major: Some(1),
    wire_minor: Some(0)
  },
  integrity: Prod{
    framing: "verified",
    structural: "verified",
    content: "verified",
    authentication: "not-present"
  },
  envelope: Map{},
  payload: Prod{
    availability: "complete",
    representation: "bytes",
    media_type: Some("application/octet-stream"),
    logical_length: Some(128),
    value: Some(Bytes("...")),
    extents: Seq[]
  },
  holes: Seq[],
  provenance: Seq[],
  uncertainty: Set{}
}
```

Hole units use the same outer product shape and carry their scope, reason,
certainty, and effects in the envelope. Exact field names and status tags are
defined by the
[DingoDB SDA examination profile](SDA_PROFILE.md).

### 11.3 Determinism

The same verified authoritative input, SDA program, and declared profile MUST
produce the same SDA value or stable failure.

Tier location, thread scheduling, hash-table iteration, and catalog ordering
MUST NOT change SDA semantics.

### 11.4 Resource limits

Hosts MAY impose explicit limits on bytes read, segments fetched, execution
time, memory, or result cardinality.

Exceeding a limit MUST produce an explicit result. It MUST NOT be reported as
absence or a complete empty result.

## 12. Performance model

### 12.1 Performance classes

DingoDB defines three distinct performance classes:

**Hot path**
Indexed reads over memory-resident working sets, targeting the performance
class of dedicated in-memory stores.

**Ingest path**
Sequential, sharded append with optional asynchronous indexing, targeting
sustained firehose ingestion.

**Archive path**
Catalog and index pruning followed by parallel streaming from large or
high-latency stores.

No single latency claim may be used for all three classes.

### 12.2 Benchmark disclosure

Published benchmarks MUST disclose:

- DingoDB version and format;
- durability mode;
- verification mode;
- hardware and storage;
- dataset size and working-set size;
- payload-size distribution;
- concurrency;
- compression and encryption;
- index freshness;
- replication;
- latency percentiles, not only averages;
- throughput;
- recovery or warm-up state.

Comparisons with Redis or other systems MUST use equivalent acknowledgement and
durability conditions.

### 12.3 Target metrics

Before a stable release, the project MUST publish reproducible targets for:

- point-read throughput and p50/p95/p99 latency;
- append throughput and latency for every durability mode;
- range and streaming throughput;
- index build and rebuild rate;
- salvage scan throughput;
- corruption resynchronization cost;
- catalog rebuild rate;
- cold-tier retrieval amplification;
- memory overhead per indexed item.

“Fast” is not a conformance property until attached to a benchmark profile.

## 13. Compaction and snapshots

### 13.1 Compaction

Compaction creates new immutable segments from verified source material.

Compaction MUST:

- record every source segment and relevant integrity value;
- preserve item and event identities;
- distinguish copied data from reconstructed data;
- commit the new segment before making old segments reclaimable;
- remain recoverable if interrupted at any step.

Compaction MUST NOT convert an uncertain history into an apparently complete
snapshot.

### 13.2 Snapshots

A snapshot is a derived, self-contained projection at a declared logical
frontier.

Snapshots MUST declare:

- source coverage;
- projection rules and version;
- known holes;
- completeness status;
- source evidence sufficient for verification.

Loss of all snapshots MUST NOT prevent event recovery.

## 14. Security and trust

Checksums detect accidental damage; they do not establish authorship.

Cryptographic hashes provide stronger integrity evidence; they do not by
themselves establish trusted origin.

Signatures establish claims relative to retained keys and trust policy; they do
not prove semantic truth.

Encryption protects confidentiality; it may reduce future recoverability if
keys are lost.

APIs and SDA examination MUST keep these claims distinct:

- physically framed;
- checksum-verified;
- cryptographically verified;
- signature-authenticated;
- policy-trusted;
- semantically decoded.

## 15. Format evolution

Unknown frame, envelope, event, codec, or payload versions MUST be skippable
when their bounded framing remains valid.

An implementation MUST preserve unknown verified payload bytes during copying,
replication, recovery, and migration unless an authorized purge explicitly
removes them.

New format versions MUST NOT reuse an old version identifier with changed
semantics.

Readers SHOULD support multiple historical versions. When support is absent,
they MUST report `format-unsupported` and preserve the bytes.

## 16. Conformance and destructive testing

A conforming implementation MUST be tested using reproducible fault injection.
The minimum suite includes:

1. truncate an active frame at every byte offset;
2. overwrite arbitrary byte ranges inside a segment;
3. delete a middle frame;
4. delete a middle segment;
5. destroy segment headers;
6. destroy segment trailers;
7. destroy every catalog, index, snapshot, and store manifest;
8. corrupt frame length fields;
9. insert arbitrary garbage between valid frames;
10. reorder or duplicate segments;
11. remove arbitrary payload chunks;
12. interrupt compaction at every persistent state transition;
13. present valid ciphertext without keys;
14. present unsupported but correctly framed versions;
15. introduce conflicting verified replicas.

For every case, tests MUST prove:

- valid islands remain discoverable;
- corrupt candidates are not reported as verified;
- known holes are exposed;
- later valid frames survive earlier corruption;
- derived completeness is not overstated;
- rebuilding derived state does not mutate authoritative evidence.

Long-retention profiles SHOULD additionally test bit rot, replica repair,
erasure reconstruction, media migration, and restoration without the original
software installation.

## 17. Non-goals

DingoDB does not promise:

- survival after every physical copy of data has been destroyed;
- semantic understanding of every payload;
- complete derived state when required events are missing;
- Redis-equivalent latency for cold or offline data;
- zero-cost durability;
- relational transactions or SQL compatibility by default;
- that one machine can hold an arbitrarily large store.

## 18. Design principle

DingoDB does not ask:

> Is the database intact?

It asks:

> Which pieces can still be proven intact?

The intended physical behavior is a deliberately damaged optical disc:
puncture it, scratch it, erase regions, and read every independently intact
island that remains.

> Put anything in. Keep it at scale. Damage it. Find what survived.

## 19. Clustering

A DingoDB cluster federates independently recoverable partitions and segments.

The cluster control plane coordinates membership, placement, policy, and
partition leadership through replicated consensus. It is not authoritative
payload storage and is never the sole holder of the information required to
identify or verify stored frames.

Ordinary strong writes coordinate within one partition. They do not require a
cluster-wide sequence or lock. Stores may instead select convergent append for
immutable events that should remain writable and mergeable across a network
split.

Every distributed result reports partition, index, and tier coverage.
Unavailable partitions are holes or uncertainty, not empty successful results.

The normative partitioning, replication, consistency, failover, rebalancing,
and distributed SDA rules are defined in
[CLUSTER_SPEC.md](CLUSTER_SPEC.md).

## 20. Developer experience

The storage, recovery, and cluster machinery is not the ordinary application
interface.

DingoDB exposes a collection-oriented API for JSON and bytes with familiar
put, get, delete, append, filter, index, history, batch, and watch operations.
Common filters compile to SDA; applications do not need to write SDA for
ordinary queries.

Embedded, server, and clustered deployments preserve the same logical API.
Safe durability is the default, and every write receipt reports the guarantee
actually achieved.

Healthy reads return ordinary values. Damage, incomplete coverage, and
uncertainty surface only when relevant and use typed, actionable errors with
an inspection path to the complete recovery evidence.

The normative everyday product contract is defined in
[DX_SPEC.md](DX_SPEC.md).
