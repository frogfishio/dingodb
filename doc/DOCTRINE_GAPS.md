# ResiduumDB Doctrine Gap Analysis

Status: Current-state assessment  
Date: 2026-07-28  
Normative target: [DATABASE_DOCTRINE.md](../DATABASE_DOCTRINE.md)  
Truth source for shipped capability:
[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md)

## 1. Purpose

The doctrine defines what ResiduumDB intends to mean as a database. This document
states which parts exist today, which are partial, and which are absent.

It prevents a doctrine document from being mistaken for a shipped capability
claim.

Status labels:

- **present** — implemented and covered by in-tree evidence for the named
  maturity profile;
- **partial** — useful mechanism exists but does not satisfy the complete
  doctrine;
- **scaffold** — types, policy evaluator, or test seam exists without the
  operational product;
- **absent** — no supported implementation;
- **external** — delegated to the operating system or deployment environment.

## 2. Executive assessment

ResiduumDB already has an unusually strong integrity and recovery substrate.

The largest doctrine gap is confidentiality and lifecycle authority:

- transport security exists;
- native encryption at rest and key lifecycle do not;
- logical delete exists;
- governed retention, holds, TTL execution, and proven purge do not;
- full single-node backup exists;
- incremental, encrypted, remote, and cluster-consistent backup do not;
- role-based service authorization exists;
- self-contained HeapKey authority and strong multi-tenant isolation do not.

The correct near-term program is not “add encryption” in isolation. It is:

1. define policy documents and state machines;
2. implement key-provider and envelope-encryption foundations;
3. make lifecycle plan/apply durable;
4. make holds and purge coverage authoritative;
5. integrate backup, restore, audit, and readiness with the same policy model.

## 3. Capability map

| Doctrine area | Current state | Status | Required next boundary |
|---|---|---|---|
| Embedded process boundary | Exclusive writer lock; filesystem ownership assumed | **present** for current embedded profile | Document filesystem permissions and unsupported network filesystems in product setup |
| TLS | TLS 1.3, mTLS peer identity, non-loopback bind gate | **present**, network surfaces still development/experimental | Operational certificate lifecycle, revocation, rotation drill, external review |
| Authentication | Shared tokens and TLS peer identity | **partial** | Per-heap master authority, self-contained holder-bound HeapKeys, local issuance and cycling |
| Authorization | Role/privilege sets; separate salvage, purge, force-reconfigure | **partial** | HeapKey rights matrix, heap-bound capabilities, zero-lookup hot path, complete-path audit |
| Audit | Hash-chained in-process audit records | **partial** | Durable sink, independent retention, truncation detection across restart, export/rotation |
| Admission | Rate, auth failure, connection churn, expensive-operation budgets | **present** for current server profile | Per-tenant budgets and production soak |
| Native at-rest encryption | Wire flag and doctrinal placeholders only | **absent** | Versioned AEAD profile, encrypted envelopes/bodies/indexes, key IDs, recovery behavior |
| Filesystem encryption | Host responsibility | **external** | Deployment detection/reporting and explicit exception recording |
| Client-confidential encryption | Opaque bytes can be stored | **scaffold** | SDK encryption profile, exposed-metadata contract, query limitations |
| Key providers | Secret refs exist for configuration | **absent** for data encryption | Provider-neutral KMS/keystore trait, local protected provider, failure semantics |
| Key hierarchy | None | **absent** | KEK/DEK hierarchy, wrapped-key envelope, blast-radius profiles |
| Key rotation | TLS reload exists; data keys do not | **absent** | Rewrap jobs, coverage tracking, decrypt-only state, compromise rotation |
| Key destruction | No data-key dependency inventory | **absent** | Destroy plan/apply, key dependency scan, crypto-erasure evidence |
| Logical delete | Tombstone/history model | **present** | Product wording must continue to distinguish delete from purge |
| TTL/expiration | No supported item expiration scheduler | **absent** | Policy schema, time-safety rules, asynchronous bounded executor |
| Tier lifecycle | Pure `LifecyclePolicy` evaluator and explicit transfers | **scaffold** | Durable scheduler, resumable jobs, plan/apply, holds and copy-safety |
| Retention | Compaction has retention-hold state and recorded horizons | **partial** | Store/collection/item policy hierarchy and enforcement |
| Governance retention | No immutable policy enforcement | **absent** | Separately authorized override model and audit |
| Compliance retention | No non-bypassable ResiduumDB profile | **absent** | Frozen policy semantics, enforcement, qualification; no compliance claim before audit |
| Legal/investigation holds | No general hold registry | **absent** | Stable hold identity, scope, authority, propagation across lifecycle and backup |
| Purge | Privilege and confirmation scaffolding; no complete purge engine | **scaffold** | Managed-domain inventory, replicas/tiers/backups coverage, purge attestation |
| Secure erasure | None | **absent** | Honest managed-copy/crypto-erasure profiles; never claim overwrite-based flash erasure |
| Full backup/restore | Verified full single-node package and restore | **present** for single-node cut | Restore drills, independent destination policy |
| Incremental backup | Not shipped | **absent** | Chain manifests, retention, chain verification and compaction |
| Backup encryption | Not shipped | **absent** | Independent backup KEK, key recovery procedure |
| Remote backup | Not shipped natively | **absent** | Resumable object-store target and conditional integrity |
| PITR | Event history exists; no supported PITR product | **partial substrate** | Base/frontier/replay protocol, commitment and key coverage |
| Cluster-consistent backup | Not shipped | **absent** | Partition frontier coordination and topology-aware restore |
| Integrity scrub | Bounded single-node scrub, findings, pause/resume | **present** for single-node cut | Background scheduling, tier/replica integration, repair coupling |
| Replica repair | In-process anti-entropy and repair evidence | **partial/experimental** | Multi-process proof and production network qualification |
| Native object stores | Filesystem mirrors only | **scaffold** | Native S3/GCS APIs, retries, range reads, multipart safety |
| Erasure coding | Manifest/types only | **scaffold** | Reviewed codec, shard placement, reconstruction and repair |
| Data classification | No portable policy labels | **absent** | Persisted classification registry and default policy resolution |
| Logical heap namespaces | One store/server exposes one flat collection namespace; URL path label is informational | **absent** | Heap identity, heap-bound APIs, HeapKey authority, complete-path isolation, recovery attribution |
| Multi-tenancy | Shared process and collection namespace | **absent as strong isolation** | Separate-store guidance first; qualified heap isolation and quotas later |
| Record-level security | Not shipped | **absent** | Full-path policy enforcement before any isolation claim |
| Capacity safety | Query/admission budgets; no complete disk-full doctrine | **partial** | Reserve margins, write rejection, read-only recovery state, alerts |
| Configuration | Versioned validation, secret refs, redaction | **present/partial** | Atomic audited live reload and doctrine policy documents |
| Logging | Structured NDJSON with redaction | **present/partial** | Client parity, audit separation, support-bundle policy |
| Metrics/health | Liveness/readiness/detail and process metrics | **partial** | Key, encryption, retention, hold, backup, purge, tier and capacity metrics |
| Migration | Phased wire migration with evidence | **present** for single-node cut | Encryption-profile migration and mixed-key/version qualification |
| Long-term compatibility | Draft wire and runbook | **partial** | Wire freeze, golden archives, retained readers, periodic clean-room tests |
| Incident process | Threat model first cut | **partial** | Published vulnerability policy, external audit, incident runbook and exercises |

## 4. Important current truths

### 4.1 Data is not encrypted at rest by ResiduumDB today

The encrypted frame flag is format vocabulary, not an implemented
confidentiality feature.

Operators requiring at-rest protection today must use filesystem/volume
encryption or encrypt payloads before ResiduumDB receives them.

Indexes, catalogs, logs, backups, and temporary files must be included in that
external protection boundary.

### 4.2 Delete is not purge

Current deletion changes live state and retains history.

It must not be described as physical erasure or privacy deletion across
segments, backups, replicas, or tiers.

### 4.3 Backup exists, but the backup program is incomplete

The single-node full backup is real and verified.

The following remain separate product work:

- incremental chains;
- encrypted packages;
- remote destinations;
- cluster coordination;
- retention integration;
- automated restore drills;
- RPO/RTO evidence.

### 4.4 Lifecycle is not automated protection yet

Tier policy evaluation and segment transfer mechanisms exist, but a durable
scheduler respecting holds, backup state, repair state, and last-copy safety
does not.

### 4.5 Authorization is not yet tenant isolation

Current role and privilege controls are valuable legacy service protections.

They do not yet establish hostile-tenant isolation through every history,
query, index, ENR, SDA, export, recovery, and diagnostic path.

They are not the target heap security model. The target is the self-contained
per-heap authority defined in `HEAP_SPEC.md`: ResiduumDB authorizes cryptographic
system keys, while applications authorize people above it.

Separate stores and operating-system/process boundaries remain the honest
recommendation for mutually hostile tenants.

## 5. Recommended implementation order

### Doctrine Phase 0 — Freeze vocabulary and policy envelopes

Add versioned representations for:

- classification;
- retention;
- hold;
- encryption profile;
- key reference and state;
- lifecycle job;
- purge plan and attestation;
- backup policy;
- deployment profile.

Exit condition:

- every document has deterministic encoding and compatibility rules;
- configuration validation can reject contradictions without performing data
  mutation.

### Doctrine Phase 1 — Key-provider foundation

Implement:

- provider-neutral KEK operations;
- local protected provider;
- test provider;
- wrapped DEK representation;
- provider health/readiness;
- zero-plaintext-fallback behavior;
- key inventory and dependency reporting.

Exit condition:

- a store can prove whether its encrypted data is recoverable without exposing
  plaintext keys.

### Doctrine Phase 2 — Independent native encryption

Implement:

- AEAD per independently recoverable frame/chunk unit;
- cleartext recovery header and published leakage;
- private envelope fields;
- encrypted indexes/catalogs or explicit rebuild-after-unlock mode;
- ciphertext salvage;
- encrypted compaction and migration;
- encrypted backup baseline.

Exit condition:

- arbitrary middle damage still permits later ciphertext recovery;
- missing keys classify as `encrypted-unavailable`;
- backup restore and cloning cannot cause nonce reuse.

### Doctrine Phase 3 — Rotation and key lifecycle

Implement:

- new-write key activation;
- KEK rewrap;
- DEK rotation;
- decrypt-only old keys;
- compromise workflow;
- destruction plan/apply;
- coverage and audit.

Exit condition:

- no old key is retired before every required managed object is covered or
  explicitly accepted as unavailable.

### Doctrine Phase 4 — Retention, holds, TTL, and purge

Implement:

- policy precedence;
- safe time evaluation;
- durable scheduler;
- governance override;
- holds;
- managed-copy inventory;
- purge attestation;
- incomplete coverage.

Exit condition:

- restart at every lifecycle phase is safe;
- no hold is bypassed;
- no offline tier produces a false purge success;
- the last required copy cannot be deleted.

### Doctrine Phase 5 — Recovery product

Implement:

- incremental and encrypted backup;
- remote targets;
- PITR;
- cluster-consistent frontiers;
- automated restore drills;
- RPO/RTO reporting;
- key-recovery exercises.

Exit condition:

- released binaries can execute documented disaster journeys in clean
  environments.

### Doctrine Phase 6 — Service identity and tenancy

Implement:

- per-heap master authority;
- local-only HeapKey issuance and authority cycling;
- self-contained holder-bound HeapKeys;
- zero-lookup established-channel authorization;
- hard invalidation and optional bounded grace with a resident blacklist;
- per-tenant quotas;
- complete-path authorization tests;
- separate key domains;
- external identity integration above ResiduumDB where applications require it.

Exit condition:

- the claimed tenancy boundary is demonstrated across all data and
  administrative paths.

## 6. Release-language constraints

Until the relevant phases land:

- say “supports TLS,” not “encrypted database”;
- say “logical delete,” not “secure delete”;
- say “full single-node backup,” not “complete backup solution”;
- say “lifecycle policy scaffold,” not “automatic lifecycle management”;
- say “role-based service authorization,” not “multi-tenant isolation”;
- say “filesystem cloud mirror,” not “native S3/GCS backend”;
- say “retention design,” not “WORM-compliant storage”;
- say “can preserve data for long periods,” not “guaranteed fifteen-year
  readability.”

## 7. Completion rule

A doctrine capability is complete only when it has:

1. normative semantics;
2. versioned persistent representation;
3. implementation;
4. hostile and crash testing;
5. operational workflow;
6. observability;
7. backup/restore interaction;
8. upgrade/migration behavior;
9. capability-matrix entry;
10. truthful public wording.
