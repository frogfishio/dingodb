# Residiuum jurisdiction, residency, and sovereign placement proposal

Status: implementation-ready proposal  
Target profile: jurisdiction-aware Residiuum
Normative impact: `OVERVIEW.md`, `FORMAT_SPEC.md`, `DX_SPEC.md`,
`CLUSTER_SPEC.md`, `TRANSACTIONS.md`, SDK compatibility policy, operator CLI,
and production release gates

## 1. Purpose

This proposal defines how Residiuum binds data to enforceable residency,
processing, movement, retention, and recovery policies.

The objective is not to attach geographic labels to partitions. The objective
is to ensure that every operation capable of storing, copying, processing,
deriving, backing up, repairing, exporting, or destroying governed data is
evaluated against a versioned policy and fails closed when compliance cannot
be established.

The model extends Residiuum’s long-horizon promise:

> Residiuum manages data across time, place, and failure.

- **Time:** data remains usable beyond the applications and systems that
  created it.
- **Place:** storage and processing remain within declared policy boundaries.
- **Failure:** surviving data and policy evidence remain independently
  recoverable and examinable.

This feature can support regulatory and contractual compliance programs. It
does not, by itself, certify compliance with any law. Legal interpretation,
organizational process, contracts, and external audit remain the operator’s
responsibility.

## 2. Requirement language

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, MAY, and OPTIONAL are normative.

An implementation conforms to this proposal only when it satisfies the
requirements and conformance tests for the claimed jurisdiction profile.

## 3. Executive model

Every governed item is bound to an immutable policy version:

```text
data
  ├── residency_domain_id
  ├── jurisdiction_policy_id
  ├── policy_epoch
  └── policy_hash
```

Every location and execution environment has authenticated attributes:

```text
node / media / worker
  ├── physical region and country
  ├── legal operator and contracting entity
  ├── cloud/provider and account
  ├── failure and trust domains
  ├── storage/processing capabilities
  └── signed or otherwise trusted attestation
```

Every operation is evaluated:

```text
evaluate(
  action,
  principal,
  data_policy,
  source_domain,
  destination_domain,
  execution_domain,
  policy_epoch
) → permit | deny | indeterminate
```

`indeterminate` is a denial for authoritative operations.

Transactions use a partition as their ordering scope and a residency domain as
their placement/processing scope. These concepts are related but distinct:

```text
partition_key: account-42
residency_domain: eu-regulated
```

The partition determines where atomic ordering occurs. The residency domain
determines where bytes, replicas, indexes, temporary material, backups, keys,
and processing are allowed to exist.

## 4. Goals

The initial jurisdiction profile MUST:

- bind every authoritative frame to a recoverable policy identity;
- enforce allowed storage and processing domains;
- constrain replicas, leaders, repair, rebalance, and tier movement;
- constrain backups, snapshots, indexes, caches, and temporary files;
- constrain encryption-key location and use where configured;
- make cross-domain export explicit, authorized, and auditable;
- preserve policy identity through salvage and disaster reconstruction;
- distinguish compliant, violating, unknown, and incomplete-evidence states;
- fail writes rather than silently weaken policy;
- integrate with partition transactions and transaction receipts;
- support policy tightening, loosening, and migration without relabeling bytes;
- support retention, legal hold, and authorized purge;
- expose deterministic policy decisions and stable machine error codes;
- remain usable when the live control plane is unavailable;
- keep policy enforcement testable without embedding jurisdiction-specific law
  into the storage engine.

## 5. Non-goals

The initial profile does not:

- encode or interpret all national or sector-specific laws;
- infer legal jurisdiction from an IP address;
- guarantee physical location without trusted infrastructure attestation;
- treat encryption as proof that residency restrictions do not apply;
- guarantee that network packets never transit an intermediate country;
- make an operator-supplied label cryptographic proof of physical presence;
- perform automatic legal classification of arbitrary payloads;
- silently de-identify or declassify data;
- make global queries legally permissible merely because storage is compliant;
- replace data-protection impact assessments, legal advice, contracts, or
  external audits;
- allow policy metadata to become the only route to understanding stored data;
- promise atomic cross-jurisdiction transactions.

## 6. Terminology

### 6.1 Jurisdiction

A legal or contractual authority whose rules may govern data storage,
processing, access, transfer, retention, or deletion.

A jurisdiction is not assumed to be identical to a country or cloud region.
One physical region may be subject to multiple jurisdictions, entities, and
contracts.

### 6.2 Residency

The physical or provider-defined location in which data is stored.

Residency answers “where are the bytes?” It does not by itself answer who may
process them, which entity controls them, or whether an export is lawful.

### 6.3 Processing domain

An authenticated environment allowed to decode, query, transform, index, or
otherwise process governed material.

Storage permission does not automatically imply processing permission.

### 6.4 Residency domain

A stable Residiuum identifier for an operator-defined set of trusted placement
and processing attributes.

Examples:

- `eu-regulated`;
- `de-health-primary`;
- `us-fedramp-high`;
- `customer-managed-datacenter-7`;
- `offline-legal-archive`.

The identifier is meaningful only relative to a versioned domain descriptor.

### 6.5 Jurisdiction policy

An immutable, versioned declarative policy defining permitted and forbidden
actions and domains for governed data.

### 6.6 Policy epoch

A monotonically increasing generation of a policy definition or assignment.

An epoch change does not retroactively move or relabel bytes. It initiates a
validated policy transition.

### 6.7 Policy binding

The tuple:

```text
(residency_domain_id, jurisdiction_policy_id, policy_epoch, policy_hash)
```

recorded with data and evidence.

### 6.8 Data class

An operator-defined classification such as `public`, `internal`,
`confidential`, `health`, `financial`, or `export-controlled`.

Residiuum treats data-class values as policy inputs. It does not infer them.

### 6.9 Placement claim

A statement that a node, medium, worker, account, key service, or failure
domain has particular location and control attributes.

### 6.10 Placement attestation

Evidence supporting a placement claim. Depending on the profile, this may be:

- a signed operator inventory;
- cloud account/region identity;
- workload identity;
- TPM/TEE attestation;
- datacenter inventory evidence;
- offline media custody evidence.

The profile must state what evidence is trusted.

### 6.11 Export

Any action that causes data or a governed derivative to become accessible in a
domain not already permitted by its effective policy.

Export includes copying, query results, transformations, logs, backups, and
support access—not only raw segment transfer.

### 6.12 Policy transition

An authorized, evidence-recorded change from one policy binding to another.

Changing a catalog field is not a policy transition.

## 7. Core invariants

### 7.1 Fail closed

If Residiuum cannot establish that an authoritative action is permitted, it MUST
deny the action.

Examples:

- unknown node location;
- expired attestation;
- missing policy definition;
- stale policy epoch;
- incompatible transaction members;
- unavailable approved replica domains;
- unclassified backup target.

The system MUST NOT silently fall back to an unrestricted or default domain.

### 7.2 Policy identity travels with data

Every authoritative frame MUST carry or independently derive its policy
binding without requiring the live control plane.

Loss of catalogs or placement directories must not erase:

- the policy identity that governed the frame;
- its origin domain;
- the policy epoch observed by the writer;
- evidence of approved transitions.

### 7.3 Placement is not identity

Moving bytes does not change their item, event, segment, partition, policy, or
provenance identity.

Placement records say where data should be. They do not redefine what policy
governs the data.

### 7.4 No policy change by relabeling

An operator MUST NOT loosen or replace a policy by editing metadata in place.

A policy transition requires:

1. authenticated authorization;
2. old and proposed bindings;
3. compatibility and export evaluation;
4. an explicit migration plan;
5. verified movement or transformation;
6. deletion or retention of old copies according to policy;
7. durable transition evidence;
8. a new assignment epoch.

### 7.5 Deny overrides allow

All deny constraints override allows.

Unknown or absent required attributes evaluate to `indeterminate`, which is a
denial for storage, replication, repair, backup, processing, export, and purge.

### 7.6 Derivatives inherit governance

Indexes, projections, query spill files, caches, logs containing values,
embeddings, summaries, repaired copies, decoded forms, and transformed outputs
inherit the source policy unless an explicit authorized transformation assigns
a new policy.

### 7.7 Encryption is not residency

Encrypted data remains governed data unless the active policy explicitly
states otherwise.

Encryption-key location and use are separately constrained.

### 7.8 Coverage includes policy evidence

A read, query, backup, repair, migration, or recovery report must state whether
policy-relevant coverage is:

- complete;
- incomplete;
- violating;
- indeterminate.

An unavailable compliant domain is not permission to use a forbidden domain.

### 7.9 Recovery does not bypass policy

Doctor, salvage, disaster reconstruction, and offline examination are
policy-governed operations.

Emergency recovery may use an explicit break-glass profile, but it MUST create
durable audit evidence and MUST NOT silently become ordinary operation.

### 7.10 Control plane is not the sole policy authority

The control plane coordinates current policy and placement. It MUST NOT be the
only surviving source for policy IDs, hashes, assignments, or transition
evidence attached to authoritative data.

## 8. Policy model

### 8.1 Immutable policy documents

Every policy version is immutable and content-addressed.

Conceptual Rust types:

```rust
pub struct JurisdictionPolicyId(String);
pub struct ResidencyDomainId(String);
pub struct PolicyEpoch(u64);
pub struct PolicyHash([u8; 32]);

pub struct PolicyBinding {
    pub residency_domain_id: ResidencyDomainId,
    pub policy_id: JurisdictionPolicyId,
    pub policy_epoch: PolicyEpoch,
    pub policy_hash: PolicyHash,
}
```

The policy hash covers a deterministic canonical encoding of the complete
policy document.

### 8.2 Declarative first profile

The first profile MUST use a bounded declarative schema rather than arbitrary
policy code.

Reasons:

- deterministic evaluation;
- stable cross-language behavior;
- safe offline examination;
- bounded execution;
- simpler compatibility and audit;
- no embedded policy runtime attack surface.

A future policy-language profile may be proposed separately.

### 8.3 Policy document

Conceptual shape:

```rust
pub struct JurisdictionPolicy {
    pub id: JurisdictionPolicyId,
    pub epoch: PolicyEpoch,
    pub description: String,
    pub data_classes: Vec<String>,

    pub storage: StoragePolicy,
    pub processing: ProcessingPolicy,
    pub replication: ReplicationPolicy,
    pub backup: BackupPolicy,
    pub export: ExportPolicy,
    pub retention: RetentionPolicy,
    pub encryption: EncryptionPolicy,
    pub observability: ObservabilityPolicy,
    pub recovery: RecoveryPolicy,

    pub created_by: PrincipalId,
    pub approved_by: Vec<PrincipalId>,
    pub effective_after: Option<Timestamp>,
    pub expires_after: Option<Timestamp>,
}
```

Timestamps control administrative activation windows only. They do not prove
transaction order, physical location, or data destruction.

### 8.4 Storage policy

Required fields:

- allowed residency-domain IDs;
- explicitly denied domain IDs;
- allowed/denied country and region claims;
- allowed/denied providers and accounts;
- allowed legal operators and contracting entities;
- allowed media classes;
- whether offline media is permitted;
- whether local developer storage is permitted;
- minimum attestation profile;
- maximum attestation age.

### 8.5 Processing policy

Required fields:

- allowed processing-domain IDs;
- allowed worker roles;
- whether payload decoding is permitted;
- whether indexing is permitted;
- whether query pushdown is required;
- whether plaintext temporary material is permitted;
- whether support/operator access is permitted;
- whether automated model processing is permitted;
- required execution attestation.

### 8.6 Replication policy

Required fields:

- minimum and maximum replica count;
- required independent failure domains;
- allowed voter and learner domains;
- whether replicas may span providers;
- whether asynchronous remote copies are permitted;
- whether convergent append is permitted;
- required durability and acknowledgement class.

### 8.7 Backup policy

Required fields:

- allowed backup domains and media classes;
- minimum independent copies;
- encryption/key-domain requirements;
- maximum backup age;
- restore authorization;
- retention and deletion requirements;
- whether third-party backup processors are permitted.

### 8.8 Export policy

Required fields:

- default deny/allow posture;
- allowed destination domains;
- allowed purpose codes;
- required approver roles;
- allowed transformation profiles;
- result policy assignment rules;
- maximum export size or record count;
- expiry and replay restrictions.

### 8.9 Retention policy

Required fields:

- minimum and maximum retention;
- legal-hold behavior;
- purge authorization threshold;
- purge evidence requirements;
- backup and replica purge scope;
- cryptographic-erasure allowance;
- retention start event;
- policy for prepared, conflicting, or unknown-commit evidence.

### 8.10 Encryption policy

Required fields:

- required encryption profile;
- allowed key-service domains;
- allowed key identifiers or key-policy selectors;
- key rotation interval;
- key-retention minimum;
- whether customer-held keys are required;
- whether plaintext memory processing is permitted.

### 8.11 Observability policy

Required fields:

- permitted log fields;
- metric-label restrictions;
- trace-sampling restrictions;
- allowed telemetry domains;
- payload/identifier redaction rules;
- audit retention and storage domain.

### 8.12 Recovery policy

Required fields:

- allowed recovery domains;
- break-glass requirements;
- minimum approvers;
- whether offline salvage is permitted;
- allowed recovered-output domains;
- quarantine requirements;
- audit and notification obligations.

## 9. Residency-domain model

### 9.1 Domain descriptor

Conceptual shape:

```rust
pub struct ResidencyDomain {
    pub id: ResidencyDomainId,
    pub epoch: u64,
    pub countries: Vec<String>,
    pub regions: Vec<String>,
    pub provider: Option<String>,
    pub provider_account: Option<String>,
    pub legal_operators: Vec<String>,
    pub contracting_entities: Vec<String>,
    pub trust_domains: Vec<String>,
    pub failure_domains: Vec<String>,
    pub capabilities: DomainCapabilities,
    pub attestation_profile: String,
    pub attestation: PlacementAttestation,
}
```

Country and region values SHOULD use published identifiers where available,
but policies MUST NOT assume geography alone captures legal jurisdiction.

### 9.2 Capabilities

Capabilities are explicit:

- authoritative storage;
- voting replica;
- non-voting replica;
- query processing;
- index building;
- backup;
- archive;
- repair source;
- repair destination;
- key service;
- telemetry;
- recovery/quarantine.

A domain allowed for storage is not automatically allowed for all
capabilities.

### 9.3 Node and media assignment

Every node and medium is assigned:

- one current domain descriptor;
- one attestation generation;
- one expiry;
- one stable cryptographic identity.

The control plane rejects membership when the domain or attestation is invalid.

### 9.4 Attestation trust

The implementation supports profiles:

1. **operator-asserted** — signed inventory; suitable for development and
   lower-assurance private deployments;
2. **provider-verified** — cloud/workload identity tied to account and region;
3. **hardware-attested** — trusted hardware evidence where available;
4. **offline-custody** — signed media custody and location records.

Receipts and diagnostics MUST identify the attestation profile. They MUST NOT
present operator-asserted location as hardware-proven location.

## 10. Policy assignment and inheritance

### 10.1 Assignment hierarchy

Policy may be configured at:

1. cluster default;
2. database/namespace;
3. collection or stream;
4. partition key;
5. individual item, when the profile permits it.

The effective policy is the most restrictive compatible result.

An explicit child policy may tighten a parent. It may loosen a parent only
through an authorized policy transition.

### 10.2 Initial implementation boundary

The first implementation SHOULD support:

- cluster default;
- collection policy;
- partition-key policy.

Per-item policy is deferred until segment grouping, indexing, and query
coverage can preserve it without accidental co-mingling or excessive
fragmentation.

### 10.3 Segment homogeneity

The first profile MUST bind each active and sealed segment to exactly one
effective policy binding.

Writer pools are keyed by:

```text
(store, partition, policy_binding, encryption_profile)
```

This prevents data with incompatible policies from sharing a segment whose
placement cannot satisfy every member.

### 10.4 Chunk inheritance

Payload chunks inherit the parent item’s effective policy.

Chunk placement may be separately optimized only within domains permitted by
the parent policy. The chunk manifest records the policy binding and placement
profile.

### 10.5 Derived-data inheritance

Derived structures inherit the source policy:

- primary and secondary indexes;
- catalogs containing governed identifiers;
- checkpoints and snapshots;
- query materializations;
- embeddings and summaries;
- temporary spill files;
- caches containing payloads;
- repair and reconstructed copies.

When one derived structure covers multiple policies, its placement must satisfy
their intersection. If no compatible domain exists, the structure must be
partitioned by policy or not built.

## 11. Policy compatibility and intersection

### 11.1 Deterministic evaluation

The pure policy engine exposes:

```rust
pub enum PolicyDecision {
    Permit(DecisionEvidence),
    Deny(DenyReason),
    Indeterminate(IndeterminateReason),
}

pub fn evaluate(
    action: PolicyAction,
    principal: &Principal,
    binding: &PolicyBinding,
    policy: &JurisdictionPolicy,
    source: Option<&ResidencyDomain>,
    destination: Option<&ResidencyDomain>,
    execution: Option<&ResidencyDomain>,
    context: &PolicyContext,
) -> PolicyDecision;
```

The same inputs always produce the same decision.

### 11.2 Policy actions

The first profile defines:

- `ingest`;
- `store`;
- `read`;
- `decode`;
- `query`;
- `index`;
- `replicate`;
- `elect_leader`;
- `repair_read`;
- `repair_write`;
- `rebalance`;
- `tier_copy`;
- `tier_move`;
- `backup`;
- `restore`;
- `compact`;
- `snapshot`;
- `cache`;
- `spill`;
- `export`;
- `transform`;
- `salvage`;
- `disaster_reconstruct`;
- `purge`;
- `audit_write`;
- `key_use`.

Unknown future actions are denied by older evaluators.

### 11.3 Compatibility

Two policy bindings are transaction-compatible only when:

- they resolve to the same policy binding; or
- the profile explicitly defines a deterministic compatible intersection;
- one eligible execution/placement domain satisfies the effective result;
- no deny rule conflicts;
- required key and attestation profiles are compatible.

The first transaction profile SHOULD require an exact policy-binding match.
This is easier to explain and safer to implement.

### 11.4 No accidental weakening

When multiple policies apply, Residiuum retains the contributing policy IDs and
decision evidence. It does not emit a simplified policy that loses a source
restriction.

## 12. Wire and evidence model

### 12.1 Envelope keys

The next draft wire revision should reserve deterministic envelope keys for:

| Proposed key | Field |
|---:|---|
| 31 | `residency_domain_id` |
| 32 | `jurisdiction_policy_id` |
| 33 | `policy_epoch` |
| 34 | `policy_hash` |
| 35 | `data_class` |
| 36 | `origin_domain_id` |
| 37 | `policy_transition_id` |
| 38 | `residency_evidence` |
| 39 | `export_authorization_id` |

Exact numeric assignments remain draft until `FORMAT_SPEC.md` is amended.
Unknown keys remain losslessly preserved.

### 12.2 Per-frame binding

Every authoritative item, event, chunk, transaction, repair, migration, and
purge frame carries its effective policy binding.

Segment-level descriptors may optimize repetition, but they are not the only
source of policy identity for an independently recovered frame.

### 12.3 Policy descriptor frame

The wire profile should reserve a core frame kind for a canonical policy
descriptor or policy snapshot.

It contains:

- immutable policy ID and epoch;
- canonical policy bytes;
- policy hash;
- creator and approver identities;
- signature profile;
- activation/expiry metadata;
- predecessor policy reference.

Stores periodically retain relevant policy descriptors with governed segments
so disaster reconstruction can interpret bindings without the original
control plane.

### 12.4 Domain descriptor frame

A domain descriptor evidence frame records:

- domain ID and generation;
- placement claims;
- capabilities;
- attestation profile and evidence reference;
- issuer;
- validity interval;
- signature.

Sensitive infrastructure details may be referenced through a stable encrypted
evidence object, but sufficient classification must survive for offline
recovery.

### 12.5 Policy transition frame

A transition frame records:

- transition ID;
- old and new policy bindings;
- affected item/segment/partition scope;
- action: tighten, loosen, relocate, transform, purge;
- authorization and approvers;
- source and destination domains;
- migration plan hash;
- completion frontier;
- unresolved copies or violations;
- audit reference.

### 12.6 Decision evidence

Receipts and migration records include:

- policy binding;
- policy action;
- evaluated source/destination/execution domains;
- policy decision code;
- policy-engine version;
- attestation generations;
- decision timestamp as diagnostic evidence;
- authenticated principal;
- authorization or export ID where applicable.

## 13. Control-plane architecture

### 13.1 Responsibilities

The control plane:

- stores immutable policy and domain registries;
- validates signatures and attestations;
- assigns effective policies;
- schedules eligible placement;
- coordinates transitions and exports;
- publishes policy epochs;
- records violations and quarantine;
- distributes revocation and expiry state.

### 13.2 Replication

Policy, domain, and transition registries are replicated through durable
consensus.

Policy updates are not ordinary mutable documents. A new epoch is a new
immutable object linked to its predecessor.

### 13.3 Cached enforcement

Nodes may cache verified policy/domain descriptors.

Each decision records the exact policy and attestation generation. Expired or
revoked cache entries fail closed.

### 13.4 Control-plane loss

On control-plane quorum loss:

- existing reads MAY continue when cached policy and attestation evidence
  remains valid and the policy permits degraded operation;
- new placements, exports, policy transitions, membership changes, repair
  destinations, and strong writes requiring fresh evidence MUST pause;
- local salvage follows the recovery policy embedded with the data;
- no operation may invent a default unrestricted policy.

### 13.5 Disaster reconstruction

Reconstruction:

1. inventories surviving frames and media;
2. extracts policy/domain/transition evidence;
3. verifies signatures and hashes;
4. groups data by effective policy binding;
5. quarantines unknown or conflicting assignments;
6. reconstructs only placements permitted by surviving policy evidence;
7. creates a new control-plane recovery generation;
8. records every override and unresolved uncertainty.

## 14. Placement and scheduling

### 14.1 Eligibility

A node is eligible for a replica only when:

- its identity is an authorized cluster member;
- its residency-domain attestation is valid;
- the policy permits authoritative storage there;
- its role is permitted;
- key-service requirements can be met;
- adding it satisfies or improves failure-domain requirements;
- no deny constraint applies.

### 14.2 Leader placement

Leader placement requires both storage and processing permission.

A node may retain an encrypted replica but be forbidden from leading or
decoding it.

### 14.3 Placement algorithm

For each partition/policy binding:

1. enumerate attested eligible domains;
2. filter by hard policy constraints;
3. select replicas satisfying required failure-domain diversity;
4. select a leader from processing-eligible replicas;
5. compute policy evidence for the plan;
6. commit the plan and policy epoch through consensus;
7. activate only after copies verify.

Cost, latency, and utilization are tie-breakers after hard constraints.

### 14.4 Unsatisfiable placement

If policy requires three replicas but only two eligible domains are available:

- the write fails with `ResidencyUnavailable` or
  `PolicyPlacementUnsatisfied`;
- the system does not reduce replica count or use a forbidden domain;
- diagnostics identify unsatisfied constraints without exposing sensitive
  policy details to unauthorized callers.

### 14.5 Domain revocation

When a domain expires, is revoked, or changes attributes:

- stop new writes, leadership, repair, and copies to the domain;
- mark affected placement `policy-violating` or `policy-indeterminate`;
- revoke processing authorization;
- retain bytes as quarantined evidence unless policy requires immediate purge;
- create an evacuation plan to eligible domains;
- alert operators;
- record the violation interval and remediation evidence.

Changing a label does not erase evidence that bytes existed in the old domain.

## 15. Ingest and collection APIs

### 15.1 Explicit policy selection

Conceptual API:

```rust
let records = db.collection_with_policy(
    "patients",
    CollectionOptions::new()
        .residency_domain("eu-health")
        .jurisdiction_policy("health-eu-v3"),
)?;
```

An SDK may use a configured collection default, but the resulting write receipt
always reports the effective binding.

### 15.2 Partition key

Partition key and item key are separate:

```rust
records.put_with(
    "observation-901",
    &value,
    PutOptions::new()
        .partition_key("patient-42")
        .policy_binding(binding),
)?;
```

All data requiring one transaction and residency boundary should share an
appropriate partition key.

### 15.3 Write validation

Before accepting a write:

1. authenticate and authorize the caller;
2. resolve the effective immutable policy;
3. validate data class and collection assignment;
4. validate partition-policy compatibility;
5. verify current placement satisfies policy;
6. verify requested durability can be achieved in eligible domains;
7. verify encryption/key policy;
8. bind the write to the current policy and placement epochs.

Failure occurs before authoritative append when possible.

### 15.4 Write receipt

The receipt includes:

```rust
pub struct JurisdictionReceipt {
    pub policy_binding: PolicyBinding,
    pub partition_id: Option<PartitionId>,
    pub placement_epoch: Option<PlacementEpoch>,
    pub eligible_replica_domains: Vec<ResidencyDomainId>,
    pub acknowledged_domains: Vec<ResidencyDomainId>,
    pub storage_compliant: bool,
    pub processing_compliant: bool,
    pub decision_id: [u8; 16],
}
```

Domain details are subject to caller authorization.

## 16. Transaction integration

### 16.1 Separate scopes

`TRANSACTIONS.md` defines transaction ordering scopes. Jurisdiction adds a
policy-compatibility scope.

A transaction commits only when every member:

- resolves to the same local store or cluster partition required by the
  transaction profile;
- has the exact same effective policy binding in the first jurisdiction
  profile;
- can be processed and replicated within eligible domains;
- uses compatible encryption/key policy.

### 16.2 Staging

Before commit, staged changes remain in an execution domain permitted by the
policy.

Client-side transaction buffering of governed plaintext is forbidden unless
the client execution domain is authorized. SDKs may instead stage encrypted
or server-side material.

### 16.3 Commit

At commit:

1. validate versions and transaction scope;
2. validate policy bindings and current policy epoch;
3. validate leader and replica domains;
4. replicate only to eligible domains;
5. establish transaction and policy decision evidence;
6. publish atomically;
7. return both transaction and jurisdiction receipts.

A policy epoch change during commit causes retry or a typed policy conflict. It
does not silently commit under a different policy.

### 16.4 Cross-jurisdiction workflows

Cross-domain work is an explicit export/workflow:

```text
source partition
  → authorized transform/export
  → destination policy assignment
  → destination partition
```

It is not one atomic transaction.

The workflow preserves partial progress, approvals, transformations, and
compensation.

## 17. Reads and queries

### 17.1 Read authorization

A read requires:

- caller authorization;
- an allowed processing domain;
- valid policy and domain evidence;
- permitted key use;
- sufficient query coverage.

Possession of storage bytes does not imply authorization to decode them.

### 17.2 Query pushdown

When policy forbids moving raw data to the coordinator:

- the query executes inside an approved processing domain;
- only policy-approved results or aggregates leave;
- worker and transformation evidence is recorded;
- the coordinator receives coverage and policy status.

### 17.3 Distributed queries

A distributed query reports:

- requested policy domains;
- processing domains used;
- partitions completed/unavailable/denied;
- indexes and temporary media used;
- export/transformation authorization;
- policy epochs evaluated;
- violations or indeterminate evidence.

### 17.4 Query results are governed data

Results inherit source policies unless an approved transformation profile
assigns a new policy.

Aggregates are not automatically declassified.

### 17.5 Caches and temporary files

Query caches, decoded pages, sort spill, and temporary files:

- use permitted domains;
- use required encryption;
- have bounded lifetime;
- are included in purge and incident scope;
- are auditable where policy requires.

## 18. Indexes and derived structures

### 18.1 Index placement

Indexes are placed only in domains permitted for both storage and indexing.

An index spanning several policies must either:

- reside in a domain satisfying every source policy; or
- be split into policy-homogeneous shards.

### 18.2 Global indexes

Global secondary indexes are distributed derived collections. Every posting
retains source policy identity or maps to a policy-homogeneous index
partition.

An index miss proves absence only when both data coverage and policy coverage
are complete.

### 18.3 Rebuild

Index rebuild workers and temporary material are policy evaluated. A general
cluster worker cannot rebuild governed data merely because it can reach the
segments.

### 18.4 Semantic and model-derived data

Embeddings, model outputs, labels, summaries, and extracted entities inherit
source governance. A policy may forbid specific model-processing profiles.

## 19. Replication, repair, and rebalance

### 19.1 Replication

Before frame transfer:

- source verifies the destination identity and domain;
- destination verifies source authorization;
- both evaluate the exact policy epoch;
- transport is mutually authenticated and encrypted;
- destination verifies frames before acknowledgement.

### 19.2 Repair

Repair must:

- select only permitted sources and destinations;
- preserve policy binding and provenance;
- verify reconstructed output;
- record source replicas/shards and domains;
- avoid using an unapproved temporary domain;
- quarantine conflicts;
- publish placement only after policy and integrity verification.

### 19.3 Rebalance

Every rebalance plan contains:

- policy binding and epoch;
- source and proposed destination domains;
- eligibility evidence;
- member/placement epoch;
- expected temporary copies;
- safety-window policy;
- cleanup obligations.

A policy change or attestation expiry pauses and revalidates the job.

### 19.4 Safety windows

Old replicas retained after movement remain governed copies. A safety window
cannot retain an old copy longer or in a domain forbidden by policy.

## 20. Tiering and object storage

### 20.1 Tier roots are domains

Every hot, warm, cold, and archive root is assigned a residency-domain
descriptor.

A path or `s3://` URI is not sufficient evidence of jurisdiction.

### 20.2 Native cloud identity

Native object backends should verify:

- provider account/project;
- bucket/container identity;
- configured region;
- endpoint;
- encryption key;
- object lock/retention state where relevant.

Filesystem mirrors inherit the local mount domain and do not prove the remote
provider location behind the mount.

### 20.3 Tier movement

Tier copy/move evaluates policy before transfer and again before activation.
Migration evidence includes source/destination domains and policy decisions.

### 20.4 Offline media

Offline media requires custody evidence:

- medium identity;
- storage location;
- custodian;
- seal/inventory state;
- movement history;
- last verification.

Unknown custody yields `policy-indeterminate`, not compliant.

## 21. Backups and restore

### 21.1 Backups are governed copies

Backup systems are not exempt from residency, encryption, retention, purge, or
access policy.

Backup manifests include policy bindings and domain evidence.

### 21.2 Incremental backup

An incremental backup must not combine data into a target that violates any
source policy. Backup streams are grouped by compatible policy.

### 21.3 Restore

Restore requires:

- authorized principal and purpose;
- eligible destination domain;
- verified backup policy evidence;
- new placement and recovery generation;
- preservation of original bindings and transition history.

### 21.4 Test restore

Restore drills use approved processing and temporary domains. “Test” is not a
policy bypass.

## 22. Export and transformation

### 22.1 Export authorization

An export authorization is immutable and bounded:

```rust
pub struct ExportAuthorization {
    pub id: String,
    pub source_policy: PolicyBinding,
    pub destination_domain: ResidencyDomainId,
    pub purpose: String,
    pub transformation_profile: Option<String>,
    pub approved_by: Vec<PrincipalId>,
    pub max_bytes: Option<u64>,
    pub max_items: Option<u64>,
    pub expires_at: Timestamp,
    pub nonce: [u8; 16],
}
```

### 22.2 Export workflow

1. Resolve source policy and coverage.
2. Authorize principal, destination, purpose, and volume.
3. Execute any transformation in an approved domain.
4. Verify output and transformation identity.
5. Assign the destination policy.
6. Write destination data through its normal ingest path.
7. Record source-to-output lineage.
8. Mark authorization usage and remaining allowance.

### 22.3 Declassification

Residiuum never infers that hashing, aggregation, tokenization, encryption, or
redaction removes governance.

Only a named approved transformation profile may assign a less restrictive
policy, and it must preserve evidence of the source policies and transform.

### 22.4 Query-result egress

Returning data to a client is an export when the client domain is outside the
source policy. Gateways must evaluate client execution domain and result
policy before delivery.

## 23. Retention, legal hold, and purge

### 23.1 Retention state

Every governed scope may be:

- active;
- retention-minimum;
- eligible-for-purge;
- legal-hold;
- purge-planned;
- purge-in-progress;
- purge-attested;
- purge-incomplete;
- policy-conflicting.

### 23.2 Legal hold

A legal hold:

- is immutable and versioned;
- identifies authority, scope, and reason;
- prevents ordinary lifecycle deletion;
- propagates to replicas, backups, indexes, and derived data;
- requires explicit authorized release.

### 23.3 Purge planning

Purge is a privileged workflow, not an item delete.

The plan inventories:

- authoritative frames;
- replicas and erasure shards;
- chunks;
- backups and snapshots;
- indexes and caches;
- temporary and derived material;
- offline media;
- encryption keys;
- unresolved or unavailable domains.

### 23.4 Purge execution

Purge:

1. validates authority and legal holds;
2. freezes new derived copies;
3. commits a purge plan and scope;
4. deletes or cryptographically erases eligible copies;
5. gathers acknowledgements by redundancy/domain;
6. records unavailable or failed copies;
7. emits a durable purge attestation;
8. reports complete or incomplete purge.

### 23.5 Purge attestation

The attestation identifies:

- what was targeted;
- why and under whose authority;
- policy and legal-hold state;
- copies/domains addressed;
- methods used;
- key destruction evidence where applicable;
- failures and unavailable media;
- completion status;
- signatures and audit references.

An attestation is evidence of actions, not proof that unknown copies do not
exist.

### 23.6 Damage evidence versus erasure rights

Residiuum’s conflict-preservation rule does not override authorized legal purge.
The system preserves conflict evidence until an authorized purge explicitly
requires removal.

## 24. Encryption and key jurisdiction

### 24.1 Key domains

Keys have their own residency/processing domains. A data domain may be allowed
to store ciphertext while forbidden from accessing plaintext keys.

### 24.2 Key use

Every decrypt/encrypt/rewrap operation evaluates:

- data policy;
- key policy;
- execution domain;
- principal;
- purpose;
- attestation.

### 24.3 Rotation

Rotation preserves:

- old/new key IDs;
- policy binding;
- algorithm/profile;
- authorization;
- completion and unavailable-copy evidence.

### 24.4 Crypto-shredding

Crypto-shredding is considered complete only under a policy that accepts it
and when every relevant wrapped key or independent decrypt capability is
addressed.

Destroying one KMS key reference is not sufficient when plaintext or other
keys/copies exist.

### 24.5 Lost keys

Lost or unavailable keys produce `encryption-unavailable`. They do not convert
data into purged, absent, corrupt, or jurisdiction-compliant state.

## 25. Salvage and emergency recovery

### 25.1 Doctor

Doctor reports:

- policy binding distribution;
- missing policy/domain descriptors;
- expired or conflicting evidence;
- misplaced segments;
- unauthorized derived state;
- unresolved transitions;
- purge/hold state;
- recommended quarantine/remediation.

### 25.2 Salvage

Salvage preserves:

- raw verified frames;
- policy and domain bindings;
- policy/domain descriptor evidence;
- transitions, exports, and purges;
- original physical provenance;
- recovery execution domain;
- uncertainty and violations.

### 25.3 Quarantine

Unknown-policy or violating material is placed in an approved quarantine
domain. It remains examinable only by authorized recovery roles.

Quarantine is not policy reassignment.

### 25.4 Break glass

Break-glass recovery requires:

- named emergency profile;
- strong authentication;
- minimum approvers;
- bounded scope and expiry;
- approved recovery domain;
- tamper-evident audit;
- post-event review;
- explicit output policy.

## 26. Observability and audit

### 26.1 Metrics

Required metrics:

- bytes/items/segments by policy and domain using bounded labels;
- permitted, denied, and indeterminate decisions;
- placement constraints unsatisfied;
- policy epoch conflicts;
- violating/quarantined copies;
- attestation expiry and revocation;
- export volume and failures;
- policy-aware rebalance/repair progress;
- purge and legal-hold state;
- backup/restore compliance;
- query policy coverage;
- key-domain violations.

Sensitive policy IDs may require stable pseudonymous metric labels.

### 26.2 Structured logs

Logs include:

- decision ID;
- action;
- stable reason code;
- policy ID/epoch;
- source/destination/execution domain IDs;
- principal;
- object scope;
- transaction/partition/placement identifiers;
- result.

Payloads and secrets are excluded.

### 26.3 Audit log

Audit records are:

- append-only and tamper-evident;
- independently retained under an audit policy;
- ordered within a declared scope;
- signed or integrity chained;
- exportable for external review;
- protected against ordinary administrator deletion.

Audit records themselves are governed data.

### 26.4 Decision explanation

Authorized operators can request a deterministic explanation:

```text
DENY rebalance partition=42
policy=health-eu-v3 epoch=7
destination=us-east-1
reason=destination_domain_not_allowed
decision_id=...
```

Explanations must not leak restricted topology or policy details to
unauthorized callers.

## 27. Stable errors and status

Add stable error codes:

- `policy_not_found`;
- `policy_hash_mismatch`;
- `policy_epoch_stale`;
- `policy_conflict`;
- `policy_denied`;
- `policy_indeterminate`;
- `residency_unavailable`;
- `processing_domain_denied`;
- `placement_unsatisfied`;
- `attestation_missing`;
- `attestation_expired`;
- `attestation_revoked`;
- `export_authorization_required`;
- `export_authorization_invalid`;
- `legal_hold_active`;
- `purge_incomplete`;
- `recovery_domain_denied`;
- `key_domain_denied`.

Status values:

- `compliant`;
- `compliant-degraded`;
- `migration-pending`;
- `policy-indeterminate`;
- `policy-violating`;
- `quarantined`;
- `purge-incomplete`.

No status may collapse missing evidence into compliant.

## 28. Operator configuration

Illustrative policy YAML:

```yaml
apiVersion: dingo.io/jurisdiction/v1alpha1
kind: JurisdictionPolicy
metadata:
  id: health-eu
  epoch: 3

dataClasses:
  - health

storage:
  allowedDomains: [eu-health-primary, eu-health-archive]
  deniedCountries: [US]
  minimumAttestation: provider-verified

processing:
  allowedDomains: [eu-health-primary]
  allowIndexing: true
  allowPlaintextTemporaryFiles: false
  allowSupportAccess: false

replication:
  minimumReplicas: 3
  requiredFailureDomains: 3
  allowCrossProvider: true

backup:
  allowedDomains: [eu-health-archive]
  minimumCopies: 1

export:
  default: deny
  allowedPurposes: [patient-request, regulator-order]
  minimumApprovers: 2

retention:
  minimumDays: 3650
  legalHoldEnabled: true
  purgeRequiresApprovers: 2

encryption:
  requiredProfile: aes-256-gcm-v1
  allowedKeyDomains: [eu-health-kms]

recovery:
  allowedDomains: [eu-health-recovery]
  breakGlassApprovers: 2
```

Illustrative domain YAML:

```yaml
apiVersion: dingo.io/jurisdiction/v1alpha1
kind: ResidencyDomain
metadata:
  id: eu-health-primary
  epoch: 12

location:
  countries: [DE, FR]
  provider: example-cloud
  account: health-production

legal:
  operators: [Example Health GmbH]
  contractingEntities: [Example Health GmbH]

capabilities:
  authoritativeStorage: true
  votingReplica: true
  queryProcessing: true
  indexing: true
  backup: false

attestation:
  profile: provider-verified
  evidence: workload-identity://...
  expiresAt: 2026-08-01T00:00:00Z
```

Actual serialization must be canonicalized independently of YAML formatting.

## 29. CLI and administration

Required commands:

```text
dingo policy validate FILE
dingo policy create FILE
dingo policy show POLICY[@EPOCH]
dingo policy diff OLD NEW
dingo policy assign COLLECTION --policy POLICY --domain DOMAIN
dingo policy plan-transition TARGET --to POLICY
dingo policy apply-transition PLAN

dingo domain register FILE
dingo domain attest DOMAIN
dingo domain revoke DOMAIN
dingo domain status DOMAIN

dingo jurisdiction status STORE|CLUSTER
dingo jurisdiction explain DECISION_ID
dingo jurisdiction violations
dingo jurisdiction quarantine TARGET

dingo export plan ...
dingo export apply PLAN

dingo hold create ...
dingo hold release ...
dingo purge plan ...
dingo purge apply PLAN
```

Rules:

- inspect/plan/apply are distinct;
- plans are immutable, hashed, and expire;
- JSON output is stable and versioned;
- destructive or loosening actions require explicit authorization;
- CLI never edits policy files in place as a substitute for transitions.

## 30. SDK surface

Conceptual types:

```rust
pub struct PolicyOptions {
    pub binding: PolicyBinding,
    pub data_class: Option<String>,
    pub purpose: Option<String>,
}

pub struct JurisdictionContext {
    pub execution_domain: ResidencyDomainId,
    pub purpose: Option<String>,
    pub export_authorization: Option<String>,
}

pub struct JurisdictionCoverage {
    pub policy_bindings: Vec<PolicyBinding>,
    pub processing_domains: Vec<ResidencyDomainId>,
    pub complete: bool,
    pub violations: Vec<PolicyViolation>,
    pub indeterminate: Vec<PolicyUncertainty>,
}
```

Every operation that may return governed data accepts or derives an execution
context. Every advanced result includes jurisdiction coverage.

## 31. Implementation architecture

### 31.1 New `dingo-policy` crate

Create a pure Rust crate with no storage or network I/O:

```text
crates/residiuum-policy/
  src/
    lib.rs
    ids.rs
    policy.rs
    domain.rs
    canonical.rs
    decision.rs
    evaluate.rs
    compatibility.rs
    errors.rs
```

Responsibilities:

- stable types;
- canonical serialization;
- policy hashing;
- deterministic evaluation;
- compatibility/intersection;
- decision reason codes;
- golden fixtures.

The crate must deny unknown actions/required attributes.

### 31.2 `residiuum-format`

Add:

- jurisdiction envelope keys;
- policy/domain/transition evidence frame kinds;
- deterministic codecs;
- diagnostic projection;
- unknown-version preservation;
- malformed/adversarial corpora.

### 31.3 `residiuum-store`

Add:

- store policy registry cache;
- policy-homogeneous active writer pools;
- per-frame policy binding;
- tier-domain descriptors;
- policy-aware compaction, checkpoint, salvage, backup, and purge;
- quarantine layout;
- policy-aware index/catalog metadata.

Suggested layout:

```text
store/
  policies/
    descriptors/
    domains/
    transitions/
    decisions/
  quarantine/
  audit/
```

These directories accelerate management. Essential policy identity remains in
framed evidence.

### 31.4 `residiuum-cluster`

Add:

- consensus-replicated policy/domain registries;
- node attestation state;
- constraint-aware placement scheduler;
- leader eligibility;
- policy-aware repair/rebalance;
- violation/quarantine state;
- policy epoch fencing;
- policy-aware coverage.

### 31.5 `residiuum-sdk`

Add:

- collection/partition policy configuration;
- jurisdiction contexts;
- receipts and coverage;
- stable errors;
- export/workflow API;
- backend parity.

### 31.6 `residiuum-cli`

Add policy/domain/transition/export/hold/purge commands and stable JSON.

### 31.7 `residiuum-examine` and SDA

Expose:

- policy binding;
- policy/domain descriptor evidence;
- transition history;
- decision evidence;
- compliance status;
- violations and uncertainty;
- holds, exports, and purge attestations.

SDA remains pure; the host supplies verified evidence.

## 32. Policy-decision algorithm

Reference algorithm:

```text
1. Load exact policy by (id, epoch).
2. Verify canonical hash equals binding.policy_hash.
3. Verify policy signature/approval profile.
4. Verify policy activation and non-revocation.
5. Resolve source, destination, and execution domains required by action.
6. Verify each descriptor hash, signature, attestation, and validity.
7. Verify principal authentication and action authorization.
8. Apply explicit deny rules.
9. Verify required allow constraints.
10. Verify replication, failure-domain, key, retention, and purpose constraints.
11. Produce Permit, Deny, or Indeterminate with deterministic reason codes.
12. Persist/audit the decision when required by policy.
```

Evaluation must be bounded by policy size, list lengths, and descriptor count.

## 33. Migration and policy changes

### 33.1 Tightening

A tighter policy:

- applies immediately to new operations after activation;
- may render existing placements violating;
- triggers migration/quarantine plans;
- does not claim existing forbidden copies vanished.

### 33.2 Loosening

Loosening requires:

- explicit classification as a loosening transition;
- required approvers;
- policy diff;
- export evaluation where newly permitted domains are involved;
- durable transition evidence.

### 33.3 Domain change

Moving a cloud account, datacenter, operator, or medium to a different domain
is a migration. Editing the domain descriptor does not retroactively move
bytes.

### 33.4 Rolling upgrades

Nodes that do not understand the active jurisdiction profile:

- may preserve unknown frames;
- may not lead, process, repair, or accept governed writes;
- may serve only explicitly permitted opaque-storage roles.

## 34. Failure behavior

### 34.1 Policy service unavailable

Use unexpired verified cache only where policy permits. Otherwise deny.

### 34.2 Attestation expires

Stop new governed work in that domain, mark status indeterminate, and begin
remediation.

### 34.3 Network partition

The quorum side may continue only using eligible domains and valid evidence.
Policy never relaxes to regain availability.

### 34.4 No compliant quorum

Strong writes pause with `ResidencyUnavailable` or
`DurabilityUnavailable`. Existing verified data remains readable only under
permitted read modes and processing domains.

### 34.5 Policy conflict

Preserve both verified policy/transition claims, quarantine affected material,
and refuse ordinary projection until an authorized resolution is recorded.

### 34.6 Misplaced bytes discovered

Do not erase evidence of the violation. Quarantine, restrict processing,
evaluate legal/policy obligations, move or purge through an authorized plan,
and retain audit evidence.

## 35. Security model

Threats include:

- administrator relabeling a forbidden domain;
- compromised node forging region claims;
- stale policy cache after revocation;
- unauthorized cross-domain repair;
- query spill to a default temp directory;
- logs leaking governed identifiers;
- backup agent copying to an unrestricted account;
- support personnel reading plaintext;
- policy downgrade during rolling upgrade;
- export replay;
- deletion of audit/transition evidence;
- salvage used to bypass normal authorization.

Controls:

- cryptographic node/workload identity;
- signed policy and domain descriptors;
- consensus-replicated registries;
- policy epoch fencing;
- mTLS between peers;
- least-privilege service identities;
- immutable plans and nonces;
- tamper-evident audit;
- dual control for loosening, export, break-glass, and purge;
- bounded parser/evaluator;
- secret/payload redaction;
- external security review.

## 36. Conformance tests

Every test reports separately:

- physical survival;
- logical commitment;
- query coverage;
- policy compliance;
- policy evidence completeness.

### 36.1 Ingest and placement

- permitted domain write succeeds;
- forbidden country/provider/account fails;
- unknown domain fails;
- expired/revoked attestation fails;
- insufficient compliant replicas fails without downgrade;
- leader storage allowed but processing denied;
- item and partition policy mismatch;
- stale policy epoch during write;
- memory/buffered/durable receipts report policy evidence.

### 36.2 Transactions

- exact policy match commits;
- mixed policy transaction fails before append;
- policy epoch changes during commit;
- leader failover preserves policy;
- retry returns original policy and transaction receipt;
- staged plaintext in forbidden client domain fails;
- cross-domain operation requires workflow/export.

### 36.3 Replication and repair

- destination becomes forbidden mid-copy;
- attestation expires after plan before activation;
- corrupt replica in allowed domain;
- healthy source in forbidden processing domain;
- repair temporary storage misconfigured;
- safety-window copy becomes forbidden;
- no eligible repair destination.

### 36.4 Rebalance

- interrupt at every persistent phase;
- policy change at every phase;
- destination revocation;
- source loss during compliant movement;
- joint membership cannot include forbidden voter;
- old copy cleanup failure is visible.

### 36.5 Tier and cloud

- local filesystem domain;
- native S3/GCS account/region verification;
- mirror does not claim remote provider proof;
- object bucket changes region/account;
- offline custody unknown;
- archive media movement;
- erasure shards across allowed/forbidden domains.

### 36.6 Queries and indexes

- coordinator forbidden from raw processing;
- pushdown returns allowed result;
- result egress requires export;
- spill directory forbidden;
- index worker forbidden;
- multi-policy index splits;
- missing policy partition yields incomplete coverage;
- aggregate is not automatically declassified.

### 36.7 Backup and restore

- forbidden backup target;
- incremental set with incompatible policies;
- expired backup attestation;
- restore into wrong domain;
- restore drill in forbidden temporary domain;
- backup purge incomplete.

### 36.8 Export

- valid bounded authorization;
- expired/replayed authorization;
- wrong purpose/destination;
- output exceeds allowance;
- transform fails;
- unauthorized declassification;
- lineage and destination policy preserved.

### 36.9 Retention and purge

- minimum retention blocks purge;
- legal hold blocks purge;
- dual approval required;
- offline copy prevents complete attestation;
- crypto-shred with alternate key copy;
- index/cache/backup included;
- purge interrupted at every phase;
- purge attestation survives catalog loss.

### 36.10 Recovery

- control plane destroyed;
- policy descriptor missing;
- conflicting policy transitions;
- salvage in allowed and forbidden recovery domains;
- break-glass flow;
- unknown future policy profile;
- node-local salvage reconstructs binding without catalogs.

### 36.11 Adversarial and fuzz

Fuzz:

- policy/domain canonical decoders;
- envelope fields;
- signatures and hashes;
- evaluator list/limit boundaries;
- policy transition chains;
- export authorizations;
- audit records;
- malformed descriptor graphs.

Property tests:

- deny always overrides allow;
- adding a restriction cannot broaden permitted domains;
- missing required attributes never produces permit;
- evaluation is deterministic;
- canonical encode/decode preserves hash;
- incompatible policies never share a first-profile transaction or segment.

## 37. Performance requirements

Jurisdiction enforcement must remain measurable and bounded.

Benchmark:

- policy-cache hit/miss;
- write decision overhead;
- leader/placement scheduling;
- transaction validation;
- query pushdown and policy coverage;
- index splitting by policy;
- repair/rebalance evaluation;
- policy transition planning;
- audit write overhead;
- large policy/domain registries.

Reports include p50/p95/p99, cache state, policy size, domain count, replica
count, transaction size, and audit mode.

Correctness is never traded for a faster permissive fallback.

## 38. Implementation phases

### Phase J0 — Specification and threat model

Deliver:

- accepted terminology and invariants;
- declarative policy schema;
- threat model;
- wire field allocation;
- canonical fixtures;
- legal/compliance review of claims.

Exit:

- independent reviewers can identify every governed copy path.

### Phase J1 — Pure policy engine

Deliver:

- `dingo-policy` crate;
- canonical encoding and hashing;
- policy/domain types;
- evaluator and reason codes;
- property and fuzz tests.

Exit:

- deterministic cross-platform golden corpus passes.

### Phase J2 — Store-level binding

Deliver:

- policy-homogeneous writer pools;
- per-frame bindings;
- policy/domain evidence frames;
- local tier-domain enforcement;
- doctor/examination projection;
- quarantine.

Exit:

- catalog loss cannot erase policy identity.

### Phase J3 — SDK and local transactions

Deliver:

- collection/partition policy assignment;
- jurisdiction receipts;
- transaction policy compatibility;
- stable errors;
- local policy-aware query/index behavior.

Exit:

- embedded conformance suite passes.

### Phase J4 — Cluster placement

Deliver:

- consensus registries;
- node/domain attestation;
- constraint-aware scheduling;
- leader eligibility;
- policy epoch fencing;
- coverage and health.

Exit:

- no cluster path can place or process governed data in a forbidden domain.

### Phase J5 — Repair, rebalance, tiers, and backup

Deliver:

- policy-aware replication/repair/rebalance;
- native cloud-domain verification;
- lifecycle and erasure integration;
- backup/restore enforcement;
- durable plans and resumability.

Exit:

- fault injection passes every movement path.

### Phase J6 — Export, retention, and purge

Deliver:

- export authorization/workflows;
- transformation lineage;
- legal holds;
- complete purge planning and attestations;
- key-domain integration.

Exit:

- cross-domain movement and deletion are fully governed and auditable.

### Phase J7 — Production qualification

Deliver:

- security audit;
- performance qualification;
- dashboards and alerts;
- runbooks;
- rolling upgrade/migration;
- external compliance-control mapping;
- release artifacts and operational drills.

Exit:

- every release gate in §40 passes.

## 39. Required specification changes

If accepted:

1. Add policy identity and independent recovery invariants to `OVERVIEW.md`.
2. Reserve jurisdiction fields and evidence frame kinds in `FORMAT_SPEC.md`.
3. Add collection/partition policy and error semantics to `DX_SPEC.md`.
4. Add domain-aware membership, placement, repair, and coverage to
   `CLUSTER_SPEC.md`.
5. Add policy compatibility and jurisdiction receipts to `TRANSACTIONS.md`.
6. Add production tasks and gates to `DEFECTS.md`.
7. Add jurisdiction examination fields to `SDA_PROFILE.md`.
8. Add implementation crates/modules to `ARCHITECTURE.md`.
9. Keep all affected profile labels draft until conformance and compatibility
   review.

## 40. Production release gates

Residiuum may claim the jurisdiction-aware profile only when:

- [ ] Every authoritative and derived copy path is inventoried.
- [ ] Every authoritative frame retains independent policy identity.
- [ ] Unknown policy/domain/attestation state fails closed.
- [ ] Segment, transaction, and partition policy compatibility is enforced.
- [ ] Placement, leadership, replication, repair, and rebalance are constrained.
- [ ] Tier, object store, backup, restore, and offline media are constrained.
- [ ] Query execution, indexes, spill, caches, logs, metrics, and traces are
      constrained.
- [ ] Cross-domain export is explicit, bounded, authorized, and auditable.
- [ ] Retention, legal hold, and purge cover replicas, backups, derived data,
      keys, and offline media.
- [ ] Control-plane destruction does not erase policy identity.
- [ ] Break-glass recovery is bounded and tamper-evident.
- [ ] Policy/domain schemas and wire encodings have compatibility guarantees.
- [ ] Fuzz, property, crash, network-partition, and movement tests pass.
- [ ] Independent security review has no unresolved critical/high finding.
- [ ] Operator runbooks and alerts exist for every violation/indeterminate
      state.
- [ ] Product documentation states assurance level and does not claim legal
      certification from technical enforcement alone.

## 41. Open decisions

Resolve before Phase J0 closes:

1. Exact canonical policy encoding.
2. Exact wire key and frame-kind assignments.
3. Initial attestation profiles and trust roots.
4. Standard identifiers for country, region, legal entity, and provider
   attributes.
5. Whether the first profile permits deterministic policy intersection or
   exact binding only.
6. Policy and decision evidence retention periods.
7. Domain descriptor confidentiality and offline reconstruction balance.
8. Whether clients may stage plaintext and how client execution domains are
   attested.
9. Query-result policy derivation and approved transformation profiles.
10. Legal-hold and purge authority model.
11. Crypto-shredding acceptance criteria.
12. Audit signing, chaining, and external anchoring.
13. Behavior for policy expiry during long offline archive periods.
14. How policy changes interact with convergent-append reconciliation.
15. How provider-region claims are verified for filesystem mirrors.

## 42. Recommendation

Adopt jurisdiction as a first-class architecture dimension now, before
transaction, cluster, tier, and wire profiles freeze.

The first implementation target should be deliberately narrow:

> Policy-homogeneous partitions and segments, fail-closed placement across
> attested residency domains, jurisdiction-aware transaction receipts, and
> evidence-preserving recovery.

Do not begin with a broad regulatory rules engine. Build a deterministic
declarative enforcement substrate that legal and compliance programs can map
onto.

The defining guarantee should be:

> Residiuum never moves, processes, repairs, backs up, or recovers governed data
> outside its declared policy without an explicit, authorized, auditable
> transition—and it never calls missing evidence compliant.
