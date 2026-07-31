# Residiuum Database Doctrine

Status: Draft doctrine v0.1  
Scope: Product identity, trust, security, encryption, ownership, lifecycle,
retention, deletion, backup, recovery, tenancy, and operational responsibility  
Audience: Implementers, operators, SDK authors, reviewers, and users deciding
whether Residiuum is suitable for their data

## 1. What Residiuum is

Residiuum is an embedded-first, history-preserving relational document database.

It combines:

- ordinary collections of JSON and bytes;
- explicit relationship formation through ENR;
- deterministic transformation through SDA;
- a durable append-oriented record of authoritative events;
- rebuildable indexes and projections;
- evidence-preserving recovery.

Residiuum is not merely a bag of durable bytes. A database is a contract about:

- who may act;
- what an acknowledgement means;
- how long data exists;
- what deletion means;
- where copies may live;
- who holds the keys;
- what can be recovered;
- what the system can prove.

The doctrine is:

> Preserve evidence, apply policy explicitly, and never claim more protection,
> deletion, completeness, or authority than can be proven.

## 2. The three planes

Residiuum separates three planes.

### 2.1 Data plane

The data plane contains:

- authoritative event frames;
- payloads and chunks;
- transaction and commitment evidence;
- tombstones and purge attestations.

The data plane answers:

> What bytes and events survive?

### 2.2 Interpretation plane

The interpretation plane contains:

- current-state projections;
- ENR relationships;
- SDA transformations;
- indexes;
- schemas;
- extracted metadata;
- query plans.

The interpretation plane answers:

> What do the surviving events mean under this declared interpretation?

### 2.3 Policy plane

The policy plane contains:

- identity and authorization;
- data classification;
- encryption and key custody;
- retention and legal holds;
- replication and placement;
- backup policy;
- lifecycle rules;
- purge authority;
- audit and operational limits.

The policy plane answers:

> What is permitted to happen to this data?

No plane may silently impersonate another.

An index does not become authoritative data. A retention rule does not prove
physical erasure. A checksum does not prove authorship. Encryption does not
prove durability. A surviving frame does not prove logical commitment.

## 3. Governing axioms

### 3.1 Evidence before convenience

When convenience and truth conflict, Residiuum reports the inconvenient truth.

### 3.2 Safe silence is not acceptable

Unavailable data, denied data, expired data, missing data, and nonexistent data
are distinct outcomes.

### 3.3 Logical and physical events are distinct

Delete, expiration, compaction, purge, media erasure, and key destruction are
different operations with different evidence.

### 3.4 Security has a named adversary

Every security control MUST state what it protects against and what it does
not protect against.

### 3.5 Keys are part of durability

Encrypted bytes without recoverable keys are unavailable data, not durable
data.

### 3.6 Replication is not backup

Replication preserves service through expected failures. Backup preserves
recoverability across administrative mistakes, software defects, compromise,
and correlated destruction.

### 3.7 Retention outranks reclamation

A hold or minimum-retention obligation prevents purge regardless of TTL,
capacity pressure, or ordinary administrator preference.

### 3.8 Capacity pressure never invents consent

Residiuum MUST NOT delete authoritative data merely because storage is full.

### 3.9 Derived structures inherit sensitivity

Indexes, logs, temporary files, backups, metrics, and query traces may disclose
data even when the primary payload is encrypted.

### 3.10 Portability is part of ownership

Users own their data and must be able to export authoritative content and
recovery evidence without a proprietary service.

## 4. Deployment and trust profiles

Security is deployment-specific. Residiuum defines four profiles.

### 4.1 Embedded trusted-process

The application process is the trusted access subject.

Trust boundary:

- the process;
- the operating-system account;
- the store directory;
- configured key provider.

Properties:

- no network listener is required;
- filesystem permissions provide the primary access boundary;
- Residiuum does not isolate mutually hostile libraries inside one process;
- an attacker with application-process memory access can normally see
  plaintext and active data keys;
- full-disk encryption protects powered-off media but not a mounted,
  compromised host.

This is the default embedded profile.

### 4.2 Single-node service

Network clients are untrusted.

Properties:

- non-loopback listeners require TLS and authentication;
- authorization is default-deny;
- administrative, recovery, and purge privileges are separate;
- resource admission occurs before expensive parsing, allocation, retrieval,
  or SDA evaluation;
- payloads and credentials are redacted from logs by default.

### 4.3 Cluster

Nodes are authenticated but not assumed infallible.

Properties:

- peer traffic uses mutually authenticated transport;
- consensus controls write authority;
- replica identity and placement are authenticated;
- one compromised replica cannot redefine committed history alone;
- encryption keys need not be available to every role or tier;
- control-plane authority is separate from payload possession.

### 4.4 Client-confidential

The server operator is not trusted with selected plaintext.

Properties:

- sensitive fields or payloads are encrypted before Residiuum receives them;
- Residiuum preserves ciphertext and declared metadata;
- server-side filtering, indexing, ENR, and SDA are limited to information
  deliberately exposed by the client encryption profile;
- this profile provides confidentiality from the database server at the cost
  of database functionality.

Native at-rest encryption and client-confidential encryption solve different
problems and may be used together.

## 5. Ownership and responsibility

### 5.1 Application owner

The application owner defines:

- the meaning of collections and fields;
- data classification;
- retention requirements;
- allowed interpretations;
- business-level deletion semantics.

### 5.2 Database operator

The operator controls:

- deployment;
- heap authority ceremonies and issued system keys;
- durability and replication;
- key-provider configuration;
- backup and restore;
- lifecycle execution;
- upgrades;
- incident response.

### 5.3 Key custodian

The key custodian controls:

- root and wrapping keys;
- key policy;
- rotation;
- revocation;
- recovery escrow where allowed;
- destruction authorization.

The operator and key custodian MAY be different parties.

### 5.4 Residiuum

Residiuum is responsible for:

- enforcing declared policy within its supported boundary;
- exposing achieved guarantees;
- preserving unknown data where promised;
- producing evidence for lifecycle and recovery actions;
- refusing unsafe or contradictory configurations.

### 5.5 Shared responsibility

Residiuum cannot:

- recover keys destroyed outside it;
- delete unknown copies outside its managed domains;
- compensate for a backup that was never made;
- guarantee durability on dishonest or broken hardware;
- certify an operator's legal compliance;
- protect plaintext from a fully compromised process that legitimately
  decrypts it.

## 6. Data classification

Residiuum defines portable classification labels:

- `public`;
- `internal`;
- `confidential`;
- `restricted`.

Applications MAY add namespaced labels.

Classification influences defaults for:

- encryption;
- logging and diagnostics;
- backup destinations;
- replication geography;
- retention;
- export;
- support-bundle redaction;
- administrator visibility.

Classification is policy metadata, not proof that the data was classified
correctly.

Unknown classification in a service deployment defaults to the more
restrictive applicable policy.

## 7. Identity, authentication, and authorization

### 7.1 Database subjects are systems

Residiuum authorizes systems, not human accounts.

Every remote heap operation is associated with:

- exactly one immutable `HeapId`;
- a self-contained, master-signed HeapKey;
- proof that the caller holds the private key named by that certificate; and
- a transport channel bound to the validated heap capability.

Residiuum does not maintain human users, groups, roles, memberships, or
principal-to-permission grants for heap access. Applications MAY implement
RBAC, ABAC, ACLs, or another human authorization model above Residiuum and decide
which process receives which HeapKey.

Shared anonymous identity is permitted only in explicitly declared local
development profiles.

### 7.2 Per-heap authority

Each heap has an independent master signing authority cryptographically bound
to that heap. The master private key:

- has no data read or write rights;
- issues restricted HeapKeys for systems;
- is never accepted by the Residiuum network protocol; and
- is used only through a local, operating-system-protected authority tool.

HeapKey claims include the heap, authority generation, holder public key,
rights, validity interval, and optional constraints. They are canonical and
cryptographically authenticated.

The master authority and data-encryption key hierarchy are separate. Neither
key class can substitute for the other.

### 7.3 Network validation and hot-path authorization

Network admission is default-deny. A HeapKey is accepted only when its
signature, heap, authority generation, validity, holder proof, rights, and
constraints validate against the heap's immutable in-memory authority
snapshot.

Validation performs no user, role, group, grant, permission, or revocation
database lookup. An established channel carries an unforgeable heap capability.
Ordinary operations compare its validated authority revision and evaluate its
already-decoded rights; they perform no authorization-policy I/O.

Secrets are referenced through providers or protected files. They MUST NOT be
stored in command history, ordinary logs, store metadata, or connection URLs
by default.

### 7.4 Rights and scope

HeapKey rights cover ordinary data operations and separately named
high-impact operations, including:

- read, write, query, index, schema, history, and stream access;
- physical evidence inspection and export;
- backup and restore;
- lifecycle, tier, scrub, and repair management;
- cluster reconfiguration;
- hold, purge, and data-key operations.

High-impact rights MUST NOT be implied by ordinary read/write access. No issued
HeapKey may issue another HeapKey.

Heap scope is enforced consistently across:

- reads;
- writes;
- indexes;
- history;
- watches;
- ENR;
- SDA;
- backup;
- export;
- salvage;
- diagnostics.

### 7.5 Cycling, grace, and revocation

A hard local authority cycle replaces the heap master generation and makes all
older HeapKeys cryptographically inert.

An optional graceful cycle MAY admit the immediately previous generation until
a bounded deadline. Its blacklist and complete authority state are always
resident in memory before readiness. Authority state is atomically replaced;
stale nodes fail closed; restore cannot roll the authority head backward.

### 7.6 Administrative bypass

Administrative bypass is explicit, separately authorized, and audited.

Backup and physical recovery operate on declared physical scope, not on
ordinary query visibility. Otherwise authorization filters could silently omit
data from a backup.

## 8. Tenancy doctrine

### 8.1 Strong isolation

Qualified heaps are the logical security boundary described by
[HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md). Until the relevant heap qualification gate is
met, separate stores, operating-system identities, processes, and key domains
remain the recommended boundary for mutually hostile tenants.

### 8.2 Shared-store tenancy

Shared-store tenants require:

- a cryptographically authenticated HeapKey for exactly one heap;
- heap-bound capabilities and complete-path enforcement;
- per-tenant quotas;
- per-tenant encryption context or key domain where required;
- audit attribution;
- index and cache isolation;
- query resource isolation;
- lifecycle and export scoping.

### 8.3 No accidental tenancy

Using collection names or key prefixes alone does not create a security
boundary.

### 8.4 Side channels

Residiuum does not claim complete tenant isolation while shared structures leak:

- key existence;
- value size;
- access timing;
- index cardinality;
- cache state;
- compaction behavior;
- resource consumption.

Profiles that share these structures MUST document the leakage.

## 9. Encryption doctrine

### 9.1 Encryption goals

Residiuum distinguishes:

1. transport encryption;
2. native encryption at rest;
3. filesystem or volume encryption;
4. backup encryption;
5. client-confidential encryption;
6. integrity and signatures.

None substitutes automatically for another.

### 9.2 Transport

Transport encryption protects data and credentials against network
observation and modification.

Non-loopback service and cluster profiles require authenticated TLS.

TLS does not protect:

- plaintext in process memory;
- unencrypted storage;
- malicious authorized clients;
- compromised endpoints.

### 9.3 Filesystem and volume encryption

Filesystem or volume encryption protects lost or powered-off media.

It is a valid deployment control and may satisfy an operator's threat model.

It does not protect against:

- a compromised running host;
- a process with filesystem access;
- accidental exports;
- unencrypted backups;
- plaintext in logs or indexes outside the encrypted volume.

### 9.4 Native at-rest encryption

Native encryption protects authoritative Residiuum objects independently of
their containing filesystem.

The native format MUST preserve independent recovery:

- each frame or chunk is an independently authenticated encryption unit;
- corruption of one ciphertext or authentication tag does not prevent
  authentication of unrelated frames;
- encryption cannot require an unbounded segment-wide cipher stream;
- missing keys produce `encrypted-unavailable`, not `corrupt` or `missing`;
- unknown encrypted formats are preserved byte-for-byte.

### 9.5 Envelope encryption

Residiuum native encryption uses envelope encryption:

- data-encryption keys (DEKs) encrypt bounded data units;
- key-encryption keys (KEKs) wrap DEKs;
- wrapped DEKs may live beside ciphertext;
- root key material remains outside the store;
- one data unit may carry multiple wrapped copies of a DEK for migration or
  independent custodians.

This permits KEK rotation by rewrapping DEKs without rewriting large payloads.

### 9.6 Authenticated encryption

Encryption profiles use authenticated encryption with associated data (AEAD).

Associated data binds ciphertext to recovery-critical context, including the
applicable:

- format version;
- store and segment identity;
- frame kind;
- event or chunk identity;
- encoded length;
- algorithm profile;
- key identifier.

An implementation MUST either guarantee nonce uniqueness under a key or use a
profile resistant to accidental nonce reuse. Backup restore and cloned stores
are part of the nonce-safety analysis.

### 9.7 Cleartext recovery metadata

Some metadata must remain cleartext to permit framing, routing, and salvage.

Every encryption profile publishes its metadata leakage, including whether it
reveals:

- store identity;
- segment identity;
- frame type;
- event identity;
- timestamps;
- collection;
- lengths;
- equality through content hashes;
- access patterns.

Profiles MAY encrypt private envelope fields while retaining a minimal
cleartext recovery header.

### 9.8 Derived data

Native at-rest protection covers:

- active and sealed authoritative frames;
- chunks;
- indexes;
- catalogs containing sensitive metadata;
- transaction state;
- spill files;
- temporary compaction outputs;
- scrub and recovery evidence;
- backups unless explicitly exported plaintext.

An encrypted payload with a plaintext full-text or semantic index is not
meaningfully encrypted against index readers.

Derived structures MAY be kept ephemeral and rebuilt after unlock, but this
tradeoff must be explicit.

### 9.9 Memory

Residiuum decrypts only for an authorized operation.

Implementations SHOULD:

- minimize plaintext lifetime;
- zero disposable key material where the platform permits;
- avoid plaintext swap or crash dumps in restricted profiles;
- bound decrypted caches;
- exclude keys and payloads from diagnostics;
- isolate KMS credentials from ordinary data-plane code.

Residiuum does not claim protection from an attacker with arbitrary read access
to process memory.

## 10. Key management

### 10.1 Key hierarchy

The default hierarchy is:

```text
external root / KMS key
        ↓ wraps
store or tenant KEK
        ↓ wraps
segment / frame / chunk DEK
        ↓ encrypts
bounded data unit
```

Profiles may omit an intermediate level but MUST document blast radius.

### 10.2 Providers

The key-provider interface is vendor-neutral.

Provider classes include:

- operating-system keystore;
- local protected key file;
- passphrase-derived unlock key;
- KMIP;
- cloud KMS;
- hardware security module;
- application-supplied provider.

### 10.3 Passphrases

Human passphrases are not used directly as encryption keys.

Local passphrase profiles use a versioned memory-hard KDF with stored salt and
parameters. Changing the passphrase rewraps keys; it need not rewrite data.

### 10.4 Key states

Key metadata records:

- identifier;
- provider;
- algorithm;
- creation time;
- activation time;
- state;
- wrapping relationships;
- affected scope;
- rotation history.

Core states are:

- `pre-active`;
- `active`;
- `decrypt-only`;
- `disabled`;
- `compromised`;
- `destroy-pending`;
- `destroyed`.

### 10.5 Availability

Before acknowledging an encrypted write, Residiuum proves that the required DEK
is durably recoverable under the declared key policy.

A healthy readiness check reports:

- whether required providers are reachable;
- which key scopes are unavailable;
- whether writes can create recoverable encrypted data;
- whether old data has decrypt-only key dependencies.

KMS unavailability may allow cached-key reads under policy. It MUST NOT cause
silent plaintext fallback.

### 10.6 Rotation

Rotation has two meanings:

**Wrapping-key rotation**  
New writes use a new KEK and existing DEKs are rewrapped. Old wrapping keys
remain until coverage proves rewrap completion.

**Data-key rotation**  
Data is decrypted and re-encrypted under new DEKs. This is required after a DEK
compromise or when policy changes the data-key cryptoperiod.

Rotating a KEK does not repair compromise of a DEK.

### 10.7 Destruction

Key destruction is a destructive data-lifecycle action.

Before destruction, Residiuum MUST produce a dependency inventory covering:

- live data;
- retained history;
- replicas;
- cold and offline tiers;
- backups;
- snapshots;
- migration copies;
- audit requirements.

Destroying a key while required ciphertext remains creates intentional
`encrypted-unavailable` data.

### 10.8 Key backup

Keys are not embedded in ordinary data backups in plaintext.

Recovery design MUST state:

- how keys are escrowed or independently backed up;
- who can restore them;
- how split custody works where required;
- how restore is tested;
- what happens when escrow is forbidden.

## 11. Data lifecycle

### 11.1 Lifecycle states

Authoritative data moves through explicit states:

```text
created
  ↓
live
  ↓
superseded or tombstoned
  ↓
retained
  ↓
eligible-for-purge
  ↓
purge-planned
  ↓
purged-with-scope-evidence
```

Legal hold, investigation hold, backup dependency, snapshot dependency,
transaction dependency, and repair dependency are overlays that may block
progress toward purge.

### 11.2 Creation

At creation, policy resolves:

- owner;
- classification;
- retention class;
- encryption scope;
- replication requirements;
- geographic or tier restrictions;
- schema and interpretation profile.

Defaults are persisted with the event so later policy interpretation is
auditable.

### 11.3 Mutation

Mutation creates a new authoritative event.

The old version may cease to be current but remains subject to history and
retention policy.

### 11.4 Logical delete

Delete creates a tombstone.

It changes ordinary current-state visibility. It does not prove:

- physical removal;
- removal from history;
- removal from replicas;
- removal from backups;
- removal from indexes or caches;
- destruction of encryption keys.

### 11.5 Expiration

TTL or an expiry timestamp makes an item eligible for a lifecycle action.

Eligibility is not immediate erasure.

Expiration execution is:

- asynchronous;
- bounded;
- observable;
- resumable;
- subject to holds and retention minima;
- protected against destructive clock jumps.

Applications requiring an exact access cutoff invalidate access with a hard
authority cycle or a bounded graceful-cycle blacklist in addition to
asynchronous lifecycle deletion.

### 11.6 Compaction

Compaction rewrites verified live or retained evidence into new immutable
segments.

Compaction may reclaim tombstoned or superseded bytes only when policy proves
they are no longer required.

Compaction is not a secure-erasure guarantee.

### 11.7 Purge

Purge is the authorized attempt to remove authoritative evidence from declared
managed domains.

Purge is:

- high-friction;
- separately authorized;
- planned before execution;
- blocked by applicable holds;
- idempotent;
- audited;
- coverage-aware.

### 11.8 Purge evidence

A purge result states:

- requested logical scope;
- HeapKey certificate fingerprint, holder fingerprint, and authority
  generation/revision;
- policy and reason;
- active segments covered;
- replicas covered;
- indexes and caches invalidated;
- tiers covered;
- backups covered or explicitly excluded;
- unavailable domains;
- key actions;
- completion and uncertainty.

If an archive is offline or an external backup is outside Residiuum's control,
the result is `purge-incomplete`, not success.

### 11.9 Secure erasure

Residiuum does not promise that overwriting a logical file securely erases
physical flash, remapped sectors, snapshots, replicas, or external backups.

Supported erasure claims are:

- logical inaccessibility;
- managed-copy purge;
- verified media sanitize performed by an external certified mechanism;
- cryptographic erasure through destruction of a uniquely scoped recoverable
  DEK.

Cryptographic erasure is valid only when:

- the key scope is known;
- no other usable wrapped key copy exists;
- no plaintext copy exists in managed domains;
- key destruction is verified by the provider;
- affected collateral data is understood.

## 12. Retention and holds

### 12.1 Retention policy

Retention is a versioned policy containing:

- minimum retention;
- maximum retention or indefinite retention;
- history retention;
- tombstone retention;
- transaction and deduplication retention;
- backup retention;
- audit retention;
- allowed tiers and geographies;
- purge behavior after eligibility.

### 12.2 Governance and compliance modes

Residiuum distinguishes:

**Governance retention**  
Ordinary principals cannot shorten or bypass retention. A separately
authorized override may do so with explicit intent and audit.

**Compliance retention**  
The configured policy cannot be shortened or bypassed through ordinary
Residiuum administration before its deadline.

Compliance mode is a product mechanism, not a legal certification.

### 12.3 Legal and investigation holds

A hold:

- has stable identity;
- records scope, authority, reason, and creation time;
- may have no predefined end;
- overrides expiration and purge;
- remains effective across tier movement, replication, compaction, and backup
  policy;
- requires separate authority to remove.

### 12.4 Policy changes

Increasing minimum retention may apply immediately if authorized.

Shortening retention does not retroactively purge data without a new lifecycle
plan.

Changing policy records old and new policy, authority, affected scope, and
effective time.

## 13. Time doctrine

Wall clocks are evidence, not perfect order.

Residiuum records:

- supplied event time;
- ingestion time;
- logical or partition order;
- policy evaluation time;
- clock source and uncertainty where relevant.

Safety rules:

- expiry does not execute solely because a clock moved abruptly forward;
- lifecycle scheduling includes a configurable safety grace;
- cluster policy uses an authorized time source and bounded skew assumptions;
- holds do not expire from untrusted client clocks;
- monotonic process clocks measure durations but do not survive restart;
- time-travel queries state which ordering model they use.

## 14. Backup, restore, and point-in-time recovery

### 14.1 Backup purpose

A backup is an independently restorable recovery set, not another live
replica.

### 14.2 Backup scope

A backup manifest records:

- store and generation;
- consistency frontier;
- included segments, chunks, indexes, and metadata;
- excluded or offline tiers;
- encryption profile and required key identifiers;
- integrity values;
- software and format versions;
- retention and hold metadata;
- creating HeapKey and holder fingerprints, and time.

### 14.3 Backup classes

Residiuum distinguishes:

- full physical backup;
- incremental physical backup;
- logical export;
- continuous event archive;
- cluster-consistent backup;
- recovery evidence package.

They are not interchangeable.

### 14.4 Online backup

Online backup uses a declared consistent frontier. Copying arbitrary live files
is not a supported backup procedure unless the store is quiesced under the
documented boundary.

### 14.5 Point-in-time recovery

Point-in-time recovery requires:

- a verified base;
- a complete authoritative event sequence for the requested scope;
- transaction commitment evidence;
- retained encryption keys;
- an explicit target frontier.

A history hole makes PITR incomplete and must be reported.

### 14.6 Backup encryption

Backups of encrypted data remain encrypted by default.

Plaintext logical exports require explicit authorization and destination
policy.

Backup keys may use a different custodian and KEK from live storage.

### 14.7 Restore

Restore defaults to:

- a new destination;
- a new store generation;
- verification before activation;
- no overwrite of an existing store;
- explicit identity reassignment when cloning.

Restore tests include key availability and authorization, not only checksum
verification.

### 14.8 Restore drills

A backup is not accepted as healthy until a restore drill proves:

- bytes verify;
- keys resolve;
- expected collections and history are present;
- queries complete over declared scope;
- application-level checks pass;
- measured RPO and RTO meet policy.

## 15. Replication and availability

Replication policy records:

- replica count;
- failure domains;
- acknowledgement quorum;
- synchronization mode;
- repair behavior;
- geographic constraints.

Replication protects availability and durability against modeled failures.

It does not protect automatically against:

- authorized delete;
- corrupted application writes;
- compromised credentials;
- policy errors;
- key destruction;
- bugs replicated to every node;
- correlated operator action.

Those require history, backup, holds, and independent key policy.

## 16. Integrity, authenticity, and provenance

Residiuum reports separate states for:

- structurally framed;
- checksum verified;
- cryptographically integrity verified;
- authenticated encryption verified;
- signature authenticated;
- consensus committed;
- policy trusted;
- semantically decoded.

Checksums detect accidental damage. Hashes strengthen integrity evidence.
AEAD authenticates ciphertext under a key. Signatures authenticate a signer.
None proves that the business data is true.

Repair records source evidence and never silently replaces a conflicting
verified copy.

## 17. Derived data, caches, and indexes

Derived data:

- is rebuildable;
- carries a coverage frontier;
- inherits source classification;
- is included in the encryption and deletion threat model;
- cannot prove absence beyond its coverage;
- is invalidated or rebuilt after policy changes that affect visibility.

Sensitive query plans, spilled intermediate results, RQL/ENR materializations,
semantic indexes, and full-text terms are data for security purposes.

## 18. Logging, metrics, tracing, and audit

### 18.1 Operational telemetry

Telemetry excludes payloads, credentials, raw keys, authorization tokens, and
unbounded user-controlled labels by default.

Identifiers are minimized or pseudonymized according to profile.

Ratatouille is the operational telemetry channel. Telemetry is bounded,
asynchronous, best effort, and non-authoritative. It does not become audit
evidence because it was retained by an external collector, and telemetry
failure cannot change a database result.

The normative collection points, schemas, cardinality limits, Ratatouille
delivery profile, and failure semantics are defined by
[TELEMETRY_SPEC.md](../../todo/telemetry/TELEMETRY_SPEC.md).

### 18.2 Audit

Audit records:

- authentication events;
- authorization denials;
- HeapKey issuance receipts, authority cycles, blacklist changes, and policy
  changes;
- key operations;
- retention and hold changes;
- export, backup, restore, and salvage;
- purge and force-reconfiguration;
- repair and migration;
- configuration changes;
- security-relevant readiness failures.

Audit has independent retention and integrity policy.

Audit is not the same as application event history.

The normative durable audit design is the
[Residiuum Evidence Ledger](../../todo/evidence/EVIDENCE_LEDGER_SPEC.md). Required evidence is
atomically coupled to its protected operation and is never substituted by
telemetry, stdout/stderr, or file logging.

### 18.3 Support bundles

Support bundles are generated through an explicit redaction profile and list
their contents before export.

They MUST NOT include plaintext payloads or key material by default.

## 19. Resource and capacity doctrine

Every deployment defines budgets for:

- storage;
- memory;
- open files;
- connections;
- request size;
- result size;
- query execution;
- SDA evaluation;
- ENR fan-out;
- index build;
- compaction;
- backup;
- scrub;
- lifecycle work;
- KMS requests.

When authoritative storage approaches exhaustion, Residiuum:

1. reports pressure;
2. throttles background work where safe;
3. applies configured backpressure;
4. may reject new writes before safety margins are exhausted;
5. remains available for reads, export, repair, and authorized reclamation
   where possible.

It does not silently expire or evict authoritative data without an explicit
eviction policy whose semantics say that the data is disposable.

## 20. Configuration doctrine

Configuration is:

- versioned;
- validated before activation;
- redacted when displayed;
- divided into dynamic and restart-required settings;
- applied atomically;
- auditable.

Unsafe combinations fail closed. Examples include:

- non-loopback plaintext listener;
- encryption required but no usable key provider;
- purge policy without purge authority;
- retention shorter than an active hold;
- cluster acknowledgement without sufficient failure domains;
- plaintext backup destination for restricted data;
- lifecycle deletion when a required tier is offline.

## 21. Upgrade and compatibility doctrine

Residiuum distinguishes:

- storage wire version;
- network protocol version;
- SDA version and profile;
- RQL/ENR dialect version;
- SDK API version;
- policy document version;
- encryption profile version.

Upgrades:

- preflight compatibility and key availability;
- preserve unknown verified bytes;
- write migration evidence;
- support rollback until the declared irreversible boundary;
- do not destroy the last readable representation;
- require restore-tested backup for irreversible migration;
- maintain mixed-version cluster rules where claimed.

A security algorithm deprecation is a migration, not an in-place semantic
reinterpretation.

## 22. Incident doctrine

Incident response prioritizes:

1. stop unauthorized mutation;
2. preserve evidence;
3. establish coverage;
4. protect keys;
5. restore safe service;
6. repair from verified sources;
7. rotate compromised credentials or keys;
8. document uncertainty and affected scope.

Operators do not compact, purge, or overwrite suspected evidence before a
preservation decision.

A compromised DEK requires data-key rotation or accepted cryptographic erasure;
KEK rotation alone is insufficient.

## 23. Compliance doctrine

Residiuum provides mechanisms that may support compliance:

- encryption;
- access control;
- audit;
- retention;
- holds;
- purge evidence;
- backup and restore;
- data locality;
- key separation.

Residiuum does not declare a deployment compliant merely because a mechanism is
enabled.

Compliance depends on:

- configuration;
- operator process;
- key custody;
- infrastructure;
- jurisdiction;
- evidence;
- independent assessment.

Product documentation uses “supports” or “can help satisfy,” not
“is compliant,” unless a specific certified profile has been independently
assessed.

## 24. Default product profiles

### 24.1 `embedded-local`

- no listener;
- operating-system account is the local trust boundary;
- durable acknowledgement default;
- native encryption optional and visible;
- filesystem permissions required;
- backup reminder and health status;
- automatic safe maintenance;
- no hidden TTL or eviction.

### 24.2 `embedded-protected`

- native at-rest encryption required;
- local keystore, protected key file, or passphrase provider;
- key recovery status checked at open;
- encrypted backup default;
- crash dumps and support bundles treated as sensitive.

### 24.3 `server-secure`

- TLS and authentication required;
- default-deny HeapKey authorization;
- master authority inaccessible through the network protocol;
- complete in-memory authority snapshot before readiness;
- native encryption required unless an explicit volume-encryption exception is
  recorded;
- external secret and key providers;
- audit enabled;
- quotas and admission enabled;
- encrypted verified backups;
- readiness fails when required keys or safety margins are unavailable.

### 24.4 `cluster-secure`

- all `server-secure` requirements;
- mutual peer authentication;
- quorum durability;
- failure-domain placement;
- authenticated control-plane changes;
- per-node least privilege;
- independent backup domain;
- cluster-consistent key and restore drills.

### 24.5 `archive-retained`

- immutable sealed segments;
- governance or compliance retention;
- legal holds;
- encrypted independent archive;
- scheduled scrub and media refresh;
- key lifetime at least as long as ciphertext retention;
- format-reader preservation;
- periodic restore and migration rehearsal.

## 25. Operator questions Residiuum must always answer

For any store:

1. Who owns it?
2. Which deployment and threat profile applies?
3. Who can read, write, administer, export, hold, purge, and destroy keys?
4. Which data is encrypted, and which metadata remains visible?
5. Where are the keys, and how are they recovered?
6. What does the current durability acknowledgement mean?
7. How many copies exist, in which failure domains and geographies?
8. What is backup, and when was it last restored successfully?
9. What retention and holds apply?
10. What does delete do?
11. What remains after delete?
12. What evidence proves purge, and what domains were not covered?
13. What happens when storage is full?
14. What happens when the clock is wrong?
15. What happens when KMS is unavailable?
16. What is the current scrub and corruption state?
17. Which indexes or tiers are incomplete?
18. Which versions can still read the store?
19. What is logged and audited?
20. What claim is Residiuum unwilling to make?

If the product cannot answer one of these, that is a capability gap, not
operator trivia.

## 26. Required conformance themes

Implementation profiles derived from this doctrine test:

- unauthorized access across every path;
- key-provider outage and recovery;
- encrypted backup and restore;
- KEK rewrap coverage;
- DEK rotation;
- key destruction dependency checks;
- metadata leakage;
- nonce safety after backup restore and clone;
- TTL under clock jumps;
- hold versus expiration;
- lifecycle execution with offline tiers;
- deletion versus physical purge;
- purge with replicas and backups;
- capacity exhaustion;
- audit redaction and integrity;
- configuration rollback;
- upgrade with old keys and unknown formats;
- incident evidence preservation.

## 27. External lessons adopted deliberately

This doctrine follows mature, documented patterns:

- PostgreSQL separates column, storage, transport, and client-side encryption
  and states their distinct trust boundaries.
- PostgreSQL backup/PITR combines a base with retained write-ahead history
  rather than treating an arbitrary live file copy as sufficient.
- PostgreSQL row-security documentation warns that filtered visibility can
  silently damage backup completeness; Residiuum therefore separates physical
  backup scope from ordinary query authorization.
- MongoDB uses envelope encryption with externally managed master keys and
  internal database keys.
- MongoDB TTL makes data eligible for asynchronous deletion and documents that
  large expiry waves can affect performance.
- Redis documents a trusted-environment security model, ACLs, TLS, and the
  distinction between filesystem at-rest encryption and client-side
  encryption.
- SQLite documents explicit online-backup boundaries and the operational
  hazards of copying active database files casually.
- Amazon S3 separates lifecycle transition, expiration, version deletion,
  governance retention, compliance retention, and legal hold.
- AWS KMS documents envelope encryption and the crucial distinction between
  rotating a wrapping key and re-encrypting data keys.
- NIST SP 800-57 treats key lifecycle, inventory, cryptoperiod, compromise,
  archival, and destruction as a managed system rather than an algorithm
  choice.

Primary references:

- [PostgreSQL encryption options](https://www.postgresql.org/docs/current/encryption-options.html)
- [PostgreSQL row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [PostgreSQL continuous archiving and PITR](https://www.postgresql.org/docs/current/continuous-archiving.html)
- [PostgreSQL routine vacuuming](https://www.postgresql.org/docs/current/routine-vacuuming.html)
- [MongoDB encryption at rest](https://www.mongodb.com/docs/manual/core/security-encryption-at-rest/)
- [MongoDB TTL indexes](https://www.mongodb.com/docs/manual/core/index-ttl/)
- [Redis security model](https://redis.io/docs/latest/operate/oss_and_stack/management/security/)
- [SQLite online backup API](https://www.sqlite.org/backup.html)
- [SQLite corruption guidance](https://www.sqlite.org/howtocorrupt.html)
- [Amazon S3 Object Lock](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html)
- [Amazon S3 lifecycle management](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lifecycle-mgmt.html)
- [AWS KMS envelope encryption](https://docs.aws.amazon.com/kms/latest/developerguide/kms-cryptography.html)
- [AWS KMS key rotation](https://docs.aws.amazon.com/kms/latest/developerguide/rotate-keys.html)
- [NIST key-management guidance](https://csrc.nist.gov/projects/key-management/key-management-guidelines)

## 28. Final identity

Residiuum is not “raw storage with queries.”

It is a policy-bearing database:

- evidence survives independently;
- interpretation is explicit;
- security names its boundary;
- encryption includes key custody;
- deletion has stages;
- retention outranks reclamation;
- backup is restore-tested;
- operations expose achieved truth.

> Residiuum remembers the data, the meaning applied to it, and the rules governing
> what may happen to it.
