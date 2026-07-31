# Medusa Durability Fabric

Normative cluster companion:
[CLUSTER_SPEC.md](CLUSTER_SPEC.md).

Formal assurance companion:
[FORMAL_ASSURANCE_SPEC.md](../formal-assurance/FORMAL_ASSURANCE_SPEC.md),
particularly the cluster theorem family `FAS-8`.

Status: Architecture specification v0.1  
Scope: Distributed data dispersal, protection compilation, placement,
availability evidence, commitment witnesses, verification and repair

## 1. Definition

The **Medusa Durability Fabric** is Residiuum's distributed durability layer.
It compiles each write and sealed storage region into a verified strategy for
dispersal, commitment, placement, protection, survival and repair while
preserving independently examinable Residiuum frames as authoritative truth.

The short name is **Medusa**. `MDF` MUST NOT be used as the public abbreviation;
that abbreviation is already strongly associated with an unrelated database
file format.

The product vocabulary is:

> Hydra finds. Chimera stores. Medusa survives.

The three mechanisms have distinct authority:

| Mechanism | Compiles | Authority boundary |
|---|---|---|
| Hydra | How immutable data is found | Derived index; never payload authority |
| Chimera | How data is represented locally | Derived layout; frames remain salvage truth |
| Medusa | How data is distributed, committed, protected and repaired | Protection evidence; never changes frame meaning |

## 2. Design objective

Conventional replicated databases commonly bind three jobs to one leader and
one replicated-log operation:

1. transfer the payload;
2. establish its durable availability; and
3. decide its logical order.

That coupling makes the leader a bandwidth bottleneck and makes data movement
part of the consensus critical path. Medusa separates the jobs:

```text
data plane       = disperse and verify authoritative frames or coded regions
evidence plane   = prove that an admitted recoverable set is durably present
ordering plane   = order a compact commitment through partition consensus
repair plane     = continuously preserve the declared failure envelope
```

The separation is architectural, not semantic. A write is acknowledged as
committed only when the selected consistency and durability contracts are both
satisfied.

Medusa MUST improve the speed/durability frontier without weakening:

- frame self-description and independent examination;
- Heap noninterference;
- generation-exact reads;
- consensus fencing and logical commitment;
- explicit holes, conflicts and coverage;
- idempotent retry; or
- catalog-independent salvage.

## 3. Non-goals

Medusa is not:

- a replacement for per-partition consensus;
- a claim that arbitrary destruction is survivable;
- a requirement to erasure-code hot or tiny values;
- permission to make a derived sidecar authoritative;
- permission to acknowledge volatile buffering as durable storage;
- a global cluster lock or global ordering service;
- a backup, retention or legal-hold policy by itself; or
- a Byzantine-fault claim unless the selected profile explicitly provides and
  proves Byzantine assumptions and thresholds.

“Indestructible” always means survival within a declared failure envelope plus
honest, bounded and independently examinable degradation outside it.

## 4. Terms

**Medusa Protection Profile (MPP)**  
The complete, immutable description of how one value, batch, segment or coding
region is dispersed, placed, committed, verified and repaired.

**Protection region**  
The bounded unit independently protected by replication or coding. Damage to
one region MUST NOT make healthy regions undecodable.

**Systematic fragment**  
A directly readable fragment containing original authoritative Residiuum
frames rather than parity-only material.

**Parity fragment**  
Derived reconstruction material authenticated to one protection region.

**Durable fragment attestation (DFA)**  
An authenticated statement that a named node durably persisted and verified a
specific fragment under a specific epoch and protection profile.

**Medusa Availability Certificate (`MACert`)**  
Evidence that a sufficient set of durable fragment attestations satisfies the
profile's reconstruction and failure-domain rules. `MACert` deliberately
avoids the established cryptographic abbreviation `MAC`.

**Medusa Commit Receipt (MCR)**  
Portable evidence binding the payload commitment, availability certificate,
partition order, term, placement epoch and consistency result.

**Failure envelope**  
The explicitly modeled family of simultaneous failures the profile promises to
tolerate.

**Witness checkpoint**  
A compact independently retained commitment root. It contains no application
payload and grants no write, repair or management authority.

## 5. Governing invariants

### 5.1 Frame authority

Authoritative logical meaning resides in verified Residiuum frames. Medusa MAY
add manifests, parity, placement records, certificates and witnesses. Losing
all such derived structures MUST NOT make a surviving systematic frame
meaningless or prevent SDA examination of that frame.

### 5.2 Commitment requires order and availability

For value or batch `x` in epoch `e` under protection profile `P`:

```text
Committed(x, e)
    => Ordered(H(x), e)
       AND CertifiedAvailable(H(x), P, e)
```

Neither a consensus position without certified payload availability nor
durably stored payload without ordering evidence is a committed write.

### 5.3 Admissible reconstruction

Let:

- `G` be the generator matrix for a coded protection region;
- `k` be the number of independent fragments required for reconstruction;
- `Phi(P)` be every failure set admitted by profile `P`; and
- `A` be the fragments covered by valid durable attestations.

The profile is admissible only if:

```text
for every S in Phi(P): rank(G[A \ S]) >= k
```

For full replication this reduces to requiring at least one surviving verified
replica after every admitted failure set.

Node counts alone are insufficient. Evaluation operates on physical and
administrative failure domains, including process, device, host, rack, zone,
region, provider, credential cohort and software cohort where declared.

### 5.4 No false completeness

Failure beyond the envelope MUST produce explicit degradation:

- `complete` — every requested region is verified and readable;
- `reconstructed` — complete only after verified reconstruction;
- `partial` — healthy regions are returned and holes are identified;
- `unavailable` — enough identity survives to report the missing material;
- `conflicting` — incompatible verified evidence survives; or
- `unknown-commit` — physical bytes survive without sufficient commitment
  evidence.

Medusa MUST NOT convert any of these states into absence, an empty value or an
unqualified successful read.

### 5.5 Bounded damage

Coding MUST operate over independently verifiable bounded protection regions.
An implementation MUST publish the maximum region size. Losing or corrupting
one region MUST NOT require decoding unrelated regions or invalidate their
coverage.

### 5.6 Heap noninterference

Every Medusa identifier, attestation, certificate, placement decision,
fragment lookup and repair operation MUST be cryptographically bound to the
Heap identity. A fragment from Heap `A` cannot satisfy availability or
reconstruction for Heap `B`.

## 6. Medusa Protection Profile

A profile is conceptually:

```text
MPP = (
    version,
    authority_format,
    write_path,
    code,
    k,
    m,
    region_size,
    failure_envelope,
    placement_constraints,
    attestation_policy,
    witness_policy,
    repair_policy,
    read_policy
)
```

The canonical profile bytes are hashed. Every fragment, certificate and commit
receipt identifies the profile version and hash. Unknown mandatory profile
fields fail closed.

Profiles are versioned, consensus-committed cluster policy. Clients MAY request
a named durability class, but clients cannot weaken a Heap's minimum profile.

### 6.1 Required built-in profile families

| Family | Intended use | Protection |
|---|---|---|
| `medusa-local` | Development only | One durable local frame; no replicated claim |
| `medusa-hot-r3` | Small/hot ordinary values | Three exact systematic replicas across hosts |
| `medusa-zone-r3` | Zone-tolerant hot data | Three exact replicas across three zones |
| `medusa-warm-lrc` | Sealed warm regions | Systematic locally reconstructable coding |
| `medusa-archive-ec` | Large cold immutable regions | Systematic erasure coding across declared domains |
| `medusa-critical` | Highest declared protection | Geographic systematic copies, coded archive and independent witnesses |

The names describe policy families, not frozen parameter values. A receipt
always reports the resolved parameters.

## 7. Protection compilation

The compiler takes:

```text
compile(
    logical_size,
    frame_shape,
    mutability,
    temperature,
    operation_kind,
    consistency_requirement,
    minimum_durability,
    topology,
    measured_device_and_network_cost,
    current_risk_state
) -> MPP
```

Selection MUST be deterministic for the same committed policy, topology view
and classified input. The decision and its inputs are observable.

The compiler MAY choose among:

- direct replicated commit;
- pipelined replicated commit;
- certified parallel dispersal;
- systematic local-reconstruction coding;
- systematic erasure coding;
- geographic replication plus coded archive; and
- offline or provider-independent witness/checkpoint publication.

The compiler MUST NOT select a path whose measured or proven preconditions are
false. Unsupported accelerators fall back to a conforming conservative path.

### 7.1 Default selection doctrine

- Tiny and latency-critical values prefer exact replication.
- Large values and batches prefer parallel dispersal when the saved leader
  bandwidth exceeds certificate overhead.
- Open mutable regions remain replicated.
- Coding begins only at an immutable generation boundary.
- Warm data prefers systematic locally repairable codes.
- Cold data may use wider systematic erasure codes.
- Highly critical data combines independent geographic representations rather
  than relying on one larger local code.
- Encoding across Heap, partition, retention or encryption boundaries is
  forbidden.

There MUST be no fixed public size threshold claimed as universally optimal.
The implementation publishes its default thresholds and the measurements used
to tune them.

## 8. Certified dispersal protocol

### 8.1 Preparation

The accepting node:

1. authenticates the operation and resolves Heap authority;
2. canonicalises the operation and assigns its idempotency identity;
3. produces authoritative frames and `payload_root`;
4. freezes the applicable MPP and placement epoch;
5. divides large material into bounded protection regions; and
6. selects destinations satisfying the failure-domain constraints.

### 8.2 Parallel durable placement

The coordinator sends systematic frames and, where selected, parity fragments
to destinations concurrently. A destination acknowledges only after:

- authenticating the sender and profile;
- validating Heap, partition, generation, region and fragment identities;
- verifying the fragment hash and region commitment;
- passing structural frame validation for systematic material;
- crossing the declared durable persistence boundary; and
- recording sufficient replay information to make the attestation idempotent.

Volatile receipt, kernel buffering without the selected durability boundary,
or a hash that was not independently recomputed cannot produce a DFA.

### 8.3 Availability certification

The coordinator forms an availability certificate only from distinct valid
attestations that satisfy:

- reconstruction rank;
- placement and anti-affinity constraints;
- current node admission and placement epochs;
- the selected durability class; and
- every failure set in the declared envelope.

A certificate MUST identify at least:

```text
protocol_version
heap_id
partition_id
operation_id
payload_root
profile_hash
placement_epoch
region_commitments
attesting_nodes_or_aggregate
attested_fragment_set
durability_boundary
certificate_authentication
```

Implementations SHOULD aggregate attestations by authenticated Merkle batches
or an equivalent mechanism so public-key operations are not required per frame.
Batching MUST retain a verifiable inclusion path for each committed operation.

### 8.4 Compact ordering

After availability certification, partition consensus orders a compact commit
descriptor binding:

```text
operation_id
payload_root
availability_certificate_hash
profile_hash
placement_epoch
```

Consensus members MUST validate the certificate against committed membership,
policy and placement state before commitment. Consensus does not need to carry
the full payload when the selected protocol proves its availability.

### 8.5 Commit receipt

The client receives success only after both availability and ordering are
established. The MCR reports:

- logical identity and payload root;
- Heap and partition identity;
- term and committed position;
- placement epoch;
- resolved MPP and failure envelope;
- availability-certificate hash;
- completeness and durability result; and
- authentication sufficient for later verification.

An ambiguous response is retried using the same operation identity. Retry MUST
return the original result or a typed conflict; it MUST NOT create a second
logical write.

## 9. Fast paths and fallback

Medusa is adaptive rather than dogmatic.

### 9.1 Small-write path

An exact-frame replicated-log path MAY carry tiny writes when it is faster. Its
acknowledgement MUST provide the same semantic fields and satisfy the same
failure-envelope predicate as a certificate-based path.

### 9.2 Large-write path

Large values and batches SHOULD use parallel dispersal so one partition leader
does not serially proxy all payload bytes. The leader or sequencer orders only
the compact descriptor after it verifies the certificate.

### 9.3 Conservative fallback

If coding, aggregation, direct routing, synchronized clocks, accelerated I/O or
another optional optimization is unavailable or outside its proved operating
bounds, Medusa falls back to exact replication and the baseline consensus
protocol. It MUST NOT silently weaken durability to preserve latency.

## 10. Coding and physical format

### 10.1 Systematic requirement

Every generally supported coded profile MUST be systematic unless a separate
archival profile explicitly declares otherwise. Healthy systematic fragments
remain ordinary, directly examinable Residiuum material.

### 10.2 Region manifest

Every fragment carries or is paired with a self-describing descriptor binding:

- Heap, partition, segment and generation;
- protection-region identity and byte/frame range;
- code family, parameters and fragment coordinate;
- canonical profile hash;
- systematic or parity role;
- payload and fragment integrity roots;
- encryption domain and key generation where applicable;
- format version; and
- enough information to locate sibling fragments by authenticated search.

Destruction of the central directory MUST NOT erase fragment identity.

### 10.3 No giant-stripe dependency

An implementation MUST NOT require whole-segment reconstruction merely to read
one healthy frame. Region sizing is bounded by policy and disclosed in receipts
or segment descriptors. Regions SHOULD align to frame and chunk boundaries.

### 10.4 Recompilation

Changing protection is a generation-safe operation:

1. compile the destination MPP;
2. construct and verify the complete destination representation;
3. obtain its availability certificate;
4. consensus-commit the new protection generation;
5. retain the previous generation for the safety window; and
6. reclaim it only after independent verification and policy authorization.

In-place destruction of the only admitted representation is forbidden.

## 11. Placement and correlated failure

Placement is compiled jointly with protection. “Three nodes” is not a failure
model.

The placement solver consumes authenticated topology labels and MUST support:

- device, host, rack, zone and region anti-affinity;
- provider and administrative-domain separation;
- credential and encryption-key cohort separation;
- software-build cohort separation during rolling upgrades;
- capacity and repair-bandwidth limits;
- latency locality; and
- copyset/codingset-style control of correlated-loss probability and blast
  radius.

The solver's objective function MUST disclose its trade-offs among:

- probability of any data-loss event;
- expected unrecoverable bytes;
- maximum credible blast radius;
- read and commit latency;
- repair fan-in, bandwidth and time;
- storage amplification; and
- topology imbalance.

Unverified node-supplied topology labels cannot establish a strong placement
claim. The cluster reports the weaker achieved envelope until labels are
attested or administratively admitted.

## 12. Witness checkpoints

Witnesses preserve compact logical-commit evidence independently of ordinary
payload and control-plane replicas.

A checkpoint binds:

```text
cluster_and_recovery_generation
heap_id
partition_id
term_and_committed_frontier
segment_or_interval_commitment_root
profile_and_placement_epochs
previous_checkpoint_root
witness_policy
witness_authentication
```

Witnesses:

- store no application payload;
- cannot create, order, repair, delete or decrypt data;
- grant no cluster membership or management authority;
- MAY be placed in another region, provider or offline retention system; and
- are used to distinguish committed, prepared, conflicting and unknown-commit
  surviving evidence during catastrophe recovery.

Witness absence does not make a healthy frame physically unreadable. A profile
that promises portable commitment proof MUST report degraded evidence until its
witness policy is satisfied.

## 13. Verification, scrubbing and repair

### 13.1 Hierarchical verification

Medusa narrows damage using authenticated hierarchy:

```text
partition -> segment -> protection region -> frame/chunk -> fragment
```

Anti-entropy exchanges roots before requesting detailed inventories. Equality
is established by verified identity and cryptographic commitment, never by
file name, length, modification time or catalog generation alone.

### 13.2 Repair source selection

Repair chooses sources using integrity, commitment, generation and placement
evidence. A newer timestamp alone never outranks verified committed material.

Repair MUST:

- reconstruct only the affected bounded regions;
- independently verify reconstructed frames;
- preserve conflicting surviving evidence;
- write to a new generation or destination;
- obtain replacement durable attestations;
- commit changed placement before retiring old material; and
- record a durable repair receipt.

Locally reconstructable codes SHOULD use local parity for ordinary single
fragment loss. Wider reconstruction is reserved for failures that exceed the
local group.

### 13.3 Repair debt

The cluster continuously calculates repair debt from the difference between
the promised and currently achieved envelopes. Admission control MAY slow new
writes when continued ingestion would make repair debt unsafe.

Repair scheduling prioritises impending irrecoverability rather than raw byte
count or oldest timestamp.

## 14. Crash and partition behavior

| Failure point | Required outcome |
|---|---|
| Before any DFA | No commitment; staged material is reclaimable |
| After some DFAs but before `MACert` | No commitment; valid fragments remain discoverable staging evidence |
| After `MACert` but before consensus proposal | Available but uncommitted; retry may resume with the same operation identity |
| During consensus | Baseline consensus rules decide; payload is not duplicated logically |
| After consensus commit but before reply | Idempotent retry returns the committed MCR |
| Coordinator/leader loss | Another authorized coordinator validates surviving evidence and resumes |
| Directory loss | Rebuild from self-identifying fragments, certificates, logs and witnesses |
| Failures inside envelope | Reads continue directly or by verified reconstruction |
| Failures outside envelope | Healthy regions survive; holes and commitment uncertainty are explicit |

Staged uncommitted fragments are garbage-collected only after an authenticated
expiry and negative commitment check. Expiry MUST account for maximum retry,
recovery and consensus uncertainty windows.

## 15. Security

All Medusa protocol messages are bound to:

- cluster and recovery generation;
- Heap identity;
- node identity and admission epoch;
- partition, placement and protection epochs;
- operation and fragment identity; and
- protocol version and purpose domain.

The design MUST prevent:

- cross-Heap fragment substitution;
- old-epoch attestation replay;
- one physical fragment counting as multiple independent failure domains;
- parity poisoning;
- fabricated durable-boundary claims;
- a witness becoming write authority;
- repair from an unauthenticated or merely newer source; and
- downgrade to a weaker profile without committed authorization.

Encryption is applied at a boundary compatible with verification and coding.
The selected construction MUST specify whether coding occurs over ciphertext
or plaintext, how nonces and associated data are bound, and how reconstruction
avoids nonce reuse. Plaintext MUST NOT be exposed to nodes outside the
authorized encryption domain merely to produce parity.

## 16. Observability

Medusa exposes bounded telemetry for:

- selected path and compiler reason;
- payload versus ordering bytes;
- dispersal fan-out and completion latency;
- durable-boundary latency per destination class;
- certificate construction and verification latency;
- protection/profile distribution;
- achieved versus promised failure envelope;
- repair debt, repair fan-in, bytes and time;
- reconstruction reads and degraded reads;
- correlated placement violations;
- staging garbage and age; and
- witness lag.

Telemetry is never authority. Commit receipts, repair receipts and witness
checkpoints requiring non-repudiation belong in the Evidence Ledger.

## 17. Formal assurance obligations

The Medusa model is admitted through the Formal Assurance Spine. At minimum it
MUST model and prove, under explicit assumptions:

1. **Commit availability:** a committed descriptor implies an admitted
   reconstructable attested set at commitment.
2. **Heap separation:** evidence or fragments from one Heap cannot satisfy
   another Heap's predicate.
3. **Epoch fencing:** stale membership, placement and profile evidence cannot
   certify a current write.
4. **Reconstruction integrity:** admitted fragments reconstruct only the
   committed region root or fail verification.
5. **Generation-safe transition:** protection recompilation never retires the
   last admitted generation before the replacement commits.
6. **Idempotent recovery:** coordinator failure and retry cannot create two
   logical commitments for one operation identity.
7. **Witness non-authority:** witness material cannot authorize data mutation
   or membership change.
8. **Bounded degradation:** failures outside the envelope cannot cause healthy
   regions to be reported as absent or corrupt merely because siblings failed.

Liveness, latency and durability probability claims remain conditional on
their network, storage, independence and cryptographic assumptions.

## 18. Conformance and destructive testing

Every implementation profile MUST test:

1. crash at every persistent transition in Section 14;
2. every fragment-loss combination up to the declared envelope;
3. sampled combinations immediately beyond the envelope;
4. bit corruption in systematic, parity, manifest and certificate material;
5. duplicate, missing, swapped and cross-generation fragments;
6. cross-Heap and cross-partition substitution;
7. attestation replay across every epoch type;
8. false topology labels and collapsed failure domains;
9. leader and dispersal-coordinator loss under sustained writes;
10. network partition before and after availability certification;
11. directory and control-plane destruction followed by reconstruction;
12. witness loss, staleness, conflict and independent recovery;
13. interrupted protection recompilation at every durable step;
14. repair during concurrent read, rebalance and further failure;
15. staged-fragment collection racing delayed retry;
16. systematic direct read with missing parity;
17. partial read with holes beyond the failure envelope;
18. fallback from every optional accelerated path;
19. skew, hot partitions and slow/dishonest storage destinations; and
20. long-running scrub and repair-debt saturation.

Tests verify physical survival, logical commitment, reconstruction integrity,
Heap isolation and reported coverage as independent properties.

Performance qualification compares at least:

- baseline exact leader replication;
- pipelined exact replication;
- certified parallel dispersal;
- full replication versus each coded profile;
- healthy direct reads versus reconstruction reads; and
- foreground throughput with and without repair pressure.

No path becomes default merely because it wins a synthetic throughput test.
It must satisfy its proof, crash, recovery and tail-latency gates.

## 19. Delivery packages

Implementation is staged without changing the final architecture:

### MED-0 — Model and evidence vocabulary

- Freeze canonical MPP, DFA, `MACert`, MCR and witness schemas.
- Define failure-domain algebra and achieved-envelope reporting.
- Add formal models and deterministic golden vectors.

### MED-1 — Exact-replication compatibility profile

- Express current exact replication as `medusa-hot-r3`.
- Emit complete durability receipts.
- Establish profile compiler, telemetry and conservative fallback.

### MED-2 — Certified parallel dispersal

- Implement large-value/batch staging, attestations and compact ordering.
- Prove idempotent crash recovery and staging collection.
- Qualify leader-bandwidth and tail-latency improvements.

### MED-3 — Hierarchical integrity and bounded repair

- Add protection-region commitments and region-local anti-entropy.
- Implement repair debt and evidence-bearing repair receipts.

### MED-4 — Systematic LRC and archival coding

- Implement warm LRC then cold erasure profiles.
- Qualify reconstruction, partial survival and generation-safe recompilation.

### MED-5 — Independent witnesses and geographic profiles

- Add witness checkpoints and catastrophic commitment reconstruction.
- Compose geographic systematic copies with coded archives.

### MED-6 — Adaptive optimisation

- Tune selection with Performance Qualification evidence.
- Add further consensus or I/O fast paths only behind proved preconditions and
  conservative fallback.

Each package requires its own work plan, threat-model delta, proof obligations,
test matrix, benchmark disclosure and rollback boundary before admission.

## 20. Research basis

Medusa adapts, rather than blindly reproduces, several established ideas:

- [Narwhal and Tusk](https://arxiv.org/abs/2105.11827): separate reliable data
  dissemination and availability from compact consensus ordering.
- [Erasure Coding in Windows Azure Storage](https://www.usenix.org/conference/atc12/technical-sessions/presentation/huang):
  local reconstruction codes reduce ordinary repair I/O and bandwidth.
- [Copysets](https://www.usenix.org/conference/atc13/technical-sessions/presentation/cidon):
  replica-group selection materially changes correlated-loss probability.
- [EPaxos](https://www.cs.cmu.edu/~dga/papers/epaxos-sosp2013-abstract.html)
  and [Tempo](https://arxiv.org/abs/2104.01142): leaderless and
  dependency-aware ordering provide later protocol-profile research paths.
- [Nezha](https://www.vldb.org/pvldb/vol16/p629-geng.pdf): synchronized-clock
  ordering demonstrates a deployable acceleration with explicit timing
  assumptions.
- [Calvin](https://dsf.berkeley.edu/cs286/papers/calvin-sigmod2012.pdf):
  deterministic ordering can remove substantial coordination from distributed
  transactional execution.

These works motivate mechanisms. Residiuum conformance depends on this
specification's invariants, its formal models and executable evidence—not on a
paper citation or performance number obtained under a different threat model.

## 21. Governing principle

> Consensus decides. Medusa proves the bytes can survive the decision.

Inside the declared failure envelope, Medusa reconstructs the committed value.
Outside it, every healthy systematic frame continues to speak for itself and
every hole remains explicit.
