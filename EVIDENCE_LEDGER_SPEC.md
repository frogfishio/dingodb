# Residiuum Evidence Ledger v1 specification

Status: normative design v1.0-draft; implementation not yet qualified

Profiles:

```text
dingo-evidence-ledger-v1
dingo-evidence-record-cbor-v1
dingo-evidence-export-v1
```

Normative companions:
[HEAP_SPEC.md](HEAP_SPEC.md),
[ATOMICS_SPEC.md](ATOMICS_SPEC.md),
[FORMAT_SPEC.md](FORMAT_SPEC.md),
[DATABASE_DOCTRINE.md](DATABASE_DOCTRINE.md), and
[doc/THREAT_MODEL.md](doc/THREAT_MODEL.md).

## 1. Decision

The **Residiuum Evidence Ledger** (DEL) is Residiuum's durable, append-only,
cryptographically verifiable record of security-sensitive and
administratively significant facts.

The ledger is not a log destination, telemetry transport, application event
stream, change-data-capture stream, or replacement for document history.

Residiuum has two deliberately separate operational channels:

| Channel | Contract |
|---|---|
| Ratatouille telemetry | bounded, asynchronous, high-volume, best effort, disposable |
| Residiuum Evidence Ledger | selective, durable, integrity-protected, independently examinable |

Telemetry failure MUST NOT change a database result. Failure to record evidence
classified as `required_atomic` MUST prevent the protected operation from
committing.

The product statement is:

> Ratatouille reports what Residiuum appears to be doing. The Residiuum Evidence
> Ledger proves what Residiuum accepted, rejected, or changed within its declared
> evidence coverage.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 3. Claims and non-claims

### 3.1 V1 claims

A conforming v1 implementation provides:

- exactly one independently ordered ledger per Heap;
- a separate deployment ledger for events that have no live Heap;
- deterministic record encoding;
- immutable record identity and sequence allocation;
- per-record signatures and predecessor hashes;
- signed checkpoints over bounded record ranges;
- atomic coupling between protected mutations and their required evidence;
- explicit gaps, damage, forks, truncation, retention cuts, and coverage;
- heap-confined read and export;
- evidence that remains verifiable when unrelated bytes are destroyed; and
- no dependency on Ratatouille, stdout, stderr, or file logging.

### 3.2 V1 does not claim

V1 does not claim that:

- evidence prevents an authorized operation;
- a timestamp establishes a universal order;
- a hash proves authorship;
- local media can resist an attacker who can delete every copy and every
  external or hardware anchor;
- a signature proves that the signed factual assertion was truthful;
- ordinary data history is retained merely because an evidence record refers
  to it; or
- a retained hash can reconstruct content that retention or damage removed.

The ledger proves that a named Residiuum evidence signer committed a canonical
assertion. The strength of rollback and deletion detection is stated by the
active anchoring profile.

### 3.3 Threats

V1 is designed to detect and report:

- bit rot, torn writes, overwritten regions, and punched holes;
- record insertion, deletion, reordering, replay, truncation, and fork;
- cross-Heap record substitution;
- canonical-encoding ambiguity and parser differentials;
- use of an unrecognized, expired, wrong-scope, or wrong-epoch signer;
- rollback relative to the configured local monotonic or external anchor;
- log injection through attacker-controlled strings;
- unbounded durable-write pressure from authentication failures; and
- evidence removal that lacks a valid retention cut.

Compromise of the active online ESK can forge new assertions and competing
tails within that key's authorized interval. It cannot alter a previously
external-witnessed checkpoint without detection or produce a certificate for a
different Heap. Compromise of the Heap master, recovery quorum, or every
configured anchor is outside the corresponding trust claim and MUST be stated
in incident reports rather than hidden by a successful signature check.

## 4. Scope model

### 4.1 Heap ledger

Every Heap has exactly one logical evidence ledger:

```text
LedgerId = BLAKE3-256(
  "DINGODB-EVIDENCE-LEDGER-V1"
  || 0x01
  || HeapId
)
```

Every record in that ledger contains exactly that `HeapId`. A Heap ledger
record MUST NOT name another Heap as a target, subordinate object, affected
scope, or data source. An opaque digest supplied by an external workflow is
permitted, but it MUST NOT be resolved into another Heap by the ledger.

The isolation theorem is:

```text
record ∈ Ledger(a)  =>  owner(record) = a

a != b  =>  Records(Ledger(a)) ∩ Records(Ledger(b)) = ∅
```

Ledger reads use an unforgeable `HeapCap<H>` and can construct only
`EvidenceReader<H>`. There is no wildcard Heap reader on the network data
plane.

### 4.2 Deployment ledger

One deployment ledger records facts that cannot belong to a live Heap, such as:

- failed Heap creation before identity publication;
- process-level evidence signer or anchor failure;
- attempted rollback of an unknown or purged Heap;
- local recovery-plane entry and exit; and
- deployment configuration changes that affect no single Heap.

```text
LedgerId = BLAKE3-256(
  "DINGODB-EVIDENCE-LEDGER-V1"
  || 0x02
  || DeploymentId
)
```

The deployment ledger MUST NOT contain application payloads, collection keys,
query results, or a mixed-Heap data-bearing result. A recovery operation that
affects several Heaps emits one non-data-bearing deployment event and one
separate event into every affected Heap ledger.

Deployment-ledger access exists only on the protected local administrative
plane. It is never authorized by a network HeapKey.

### 4.3 Genesis

A new Heap is not publishable or ready until its sequence-1 `LedgerGenesis`
record is durable. Genesis binds:

- `HeapId`;
- `DeploymentId`;
- `AuthorityEpoch`;
- the immutable Heap descriptor hash;
- the evidence profile;
- the initial evidence-signing certificate; and
- the initial retention and anchoring policies.

Failed creation before `HeapId` publication is recorded only in the deployment
ledger. Retry of the same creation operation either returns the same committed
genesis or records a distinct failed attempt; it never creates two sequence-1
records.

## 5. Evidence obligations

### 5.1 Obligation classes

Every registered evidence kind has exactly one obligation class:

| Class | Meaning | Failure semantics |
|---|---|---|
| `required_atomic` | accepted state transition and evidence are one commitment | operation does not commit |
| `required_before_reply` | fact is durable before success is returned | success is withheld; retry resolves by operation ID |
| `bounded_security` | denial/attempt evidence subject to admission aggregation | operation remains denied; loss/aggregation is explicit |
| `policy_optional` | enabled by Heap evidence policy | policy states whether failure closes or degrades |

An implementation MUST NOT silently downgrade an obligation class because the
ledger is slow, full, damaged, unavailable, or unanchored.

### 5.2 Atomic coupling

Let `D` be authoritative Heap state, `L` the Heap ledger, `m` a protected
mutation, and `e(m)` its canonical evidence record.

For every `required_atomic` operation:

```text
Commit(m) ⇔ Commit(D' = apply(D, m) ∧ L' = append(L, e(m)))
```

The following history is forbidden:

```text
Commit(D' = apply(D, m))
crash
append(L, e(m))
```

The evidence record is a closed member of the same Atomic plan described by
`ATOMICS_SPEC.md`. It receives the same commit position and decision outcome.
If the implementation has not qualified the required Atomic scope, it MUST
NOT expose the protected operation as qualified.

#### 5.2.1 Local authority and lifecycle coupling

Operations whose logical commit point is the protected Heap authority selector
use the authority protocol rather than pretending that authority files are an
ordinary collection.

The authority mutation event includes:

```text
evidence_ledger_id
evidence_sequence
evidence_id
evidence_record_hash
previous_evidence_record_hash
```

Under the existing local mutation lock, the implementation:

1. closes, signs, writes, verifies, and syncs the evidence record in a
   non-discoverable staging location;
2. writes and syncs the authority event containing the exact evidence binding;
3. writes and syncs the inactive authority-head slot;
4. advances the authority anchor;
5. atomically replaces and directory-syncs the authority selector, which is
   the logical commit point;
6. publishes only the staged evidence bytes whose hash matches the committed
   authority event;
7. advances the evidence head; and
8. returns the operation receipt.

A crash before step 5 leaves both changes uncommitted. A crash after step 5
causes startup recovery to publish the exact staged evidence before the Heap
becomes ready. Missing or different staged bytes make the Heap
`evidence_blocked`; recovery does not synthesize a replacement record.

Sequence reservation and authority revision allocation occur under the same
lock. An uncommitted staged sequence is reusable only when no committed
authority event names it. Once named by a committed event, it is never reused.

Lifecycle operations whose current qualified commit point is an Atomic use the
ordinary §5.2 path. A lifecycle implementation with a separate protected
control-document commit point MUST define the equivalent staged-byte binding
and pass the same crash matrix before qualification.

#### 5.2.2 Deployment-control coupling

A deployment-scoped protected mutation uses the same staged-evidence pattern:
the checksummed control document binds the next evidence sequence and record
hash, and its qualified atomic selector is the logical commit point. If a
deployment control surface cannot provide such a commit point, it cannot claim
`required_atomic`; it remains unavailable until the control protocol is
qualified.

### 5.3 Attempts and denials

An unauthenticated peer must not be able to force an unbounded durable write
rate. Authentication failures and authorization denials therefore use
`bounded_security`.

Events are aggregated by a bounded, non-secret classification key and time
window. An aggregate states:

- first and last observed time;
- count, saturated flag, and dropped count;
- public failure class;
- pseudonymized source classification when policy permits; and
- admission policy revision.

The ledger MUST NOT imply that an aggregate contains every attempt. Its
coverage field is `bounded_aggregate`, not `complete`.

### 5.4 Reads

Ordinary successful reads are not ledgered by default. A Heap MAY enable a
closed enhanced-audit policy for selected operations or collections.

When policy makes a read `required_before_reply`, Residiuum MUST durably append
the evidence before releasing the result. This cost is explicit and MUST NOT
be enabled through an unbounded request-supplied flag.

### 5.5 Evidence policy

Each Heap has one immutable active `EvidencePolicy` inside its
`HeapSecuritySnapshot`:

| Key | Field |
|---:|---|
| 1 | profile version = 1 |
| 2 | policy revision, nonzero uint |
| 3 | event overrides, sorted array |
| 4 | enhanced-read rules, sorted array |
| 5 | retention classes, canonical map |
| 6 | assertion encryption profile |
| 7 | anchoring profile |
| 8 | checkpoint maximum records |
| 9 | checkpoint maximum age seconds |
| 10 | bounded-security budget profile |

Event overrides may make `policy_optional` events stricter. They MUST NOT
downgrade a registry event from `required_atomic`, `required_before_reply`, or
`bounded_security`.

V1 defaults are:

```text
event overrides                 none
enhanced-read rules             none
retention                       retain_forever
assertion encryption            configured Heap evidence-encryption profile
anchoring                       local_signed
checkpoint maximum records      4096
checkpoint maximum age          60 seconds
denial aggregate window         60 seconds
active denial aggregate keys    1024 per process
```

Hard v1 maxima are 65,536 records or 86,400 seconds between checkpoints,
10 minutes per denial aggregate window, and 4,096 active denial aggregate
keys. A deployment may configure smaller values. Excess attacker-controlled
keys merge into one overflow aggregate rather than allocating more state.

Policy activation is `required_atomic`, advances `security_revision`, and
records both the before-policy and after-policy roots. An unknown policy field
or event code fails activation; it is never ignored as non-critical.

## 6. Event registry

Numeric event-kind codes are stable. Unknown codes are preserved but never
interpreted as a known event.

| Code | Wire name | Default obligation |
|---:|---|---|
| 1 | `ledger_genesis` | `required_atomic` |
| 2 | `ledger_signer_rotate` | `required_atomic` |
| 3 | `ledger_policy_change` | `required_atomic` |
| 5 | `ledger_retention_cut` | `required_atomic` |
| 100 | `heap_rename` | `required_atomic` |
| 101 | `heap_state_change` | `required_atomic` |
| 102 | `heap_retire` | `required_atomic` |
| 103 | `heap_purge_begin` | `required_atomic` |
| 104 | `heap_purge_result` | `required_atomic` |
| 105 | `heap_takeover` | `required_atomic` |
| 200 | `heapkey_issue` | `required_atomic` |
| 201 | `heapkey_first_use` | `required_before_reply` |
| 202 | `authority_cycle` | `required_atomic` |
| 203 | `authority_grace_change` | `required_atomic` |
| 204 | `authority_blacklist_change` | `required_atomic` |
| 205 | `access_policy_change` | `required_atomic` |
| 206 | `authorization_denial` | `bounded_security` |
| 207 | `authentication_failure` | `bounded_security` |
| 208 | `scope_override_mutation` | `policy_optional` |
| 209 | `scope_override_read` | `policy_optional` |
| 300 | `data_key_create` | `required_atomic` |
| 301 | `data_key_rotate` | `required_atomic` |
| 302 | `data_key_destroy` | `required_atomic` |
| 303 | `evidence_data_key_rotate` | `required_atomic` |
| 400 | `hold_place` | `required_atomic` |
| 401 | `hold_release` | `required_atomic` |
| 402 | `retention_policy_change` | `required_atomic` |
| 410 | `backup_create` | `required_before_reply` |
| 411 | `restore_import` | `required_atomic` |
| 412 | `export_create` | `required_before_reply` |
| 420 | `scrub_result` | `required_before_reply` |
| 421 | `damage_discovered` | `required_before_reply` |
| 422 | `repair_result` | `required_atomic` |
| 423 | `salvage_result` | `required_before_reply` |
| 424 | `migration_result` | `required_atomic` |
| 500 | `dre_revision_change` | `required_atomic` |
| 501 | `relationship_revision_change` | `required_atomic` |
| 502 | `collection_contract_change` | `required_atomic` |
| 503 | `index_definition_change` | `required_atomic` |
| 510 | `atomic_privileged_outcome` | `required_atomic` |
| 600 | `configuration_change` | `required_atomic` |
| 601 | `readiness_security_failure` | `bounded_security` |
| 602 | `rollback_rejected` | `bounded_security` |
| 603 | `recovery_plane_use` | `required_before_reply` |

Codes `4`, `6–99`, unused values inside assigned ranges, and `604–65535` are
reserved. New events require a registry revision, threat analysis, declared
payload schema, obligation class, retention class, and redaction analysis.

Checkpoint creation is signed structural evidence, not an event record and not
an application-visible state change. A forced checkpoint is completed before
the requesting high-impact operation returns, as specified in §10.1.

Scope-override reads and mutations can be normal application access patterns
and are not ledgered by default. A Heap may promote either closed event kind
when its exposure policy requires durable use evidence. Changes to the
collection scope contract itself are always covered by
`collection_contract_change`.

Ordinary CRUD, successful queries, latency, cache activity, throughput, and
resource measurements are telemetry. They MUST NOT enter the ledger unless a
closed policy explicitly promotes a bounded event class.

## 7. Canonical record

### 7.1 Primitive identities

```text
EvidenceId       = 16 opaque bytes
EvidenceSequence = monotonically increasing nonzero u64
EvidenceHash     = BLAKE3-256(domain || canonical_record_without_signature)
OperationId      = 16 or 32 opaque bytes, according to the originating protocol
```

Sequences are allocated independently per ledger, are never reused, and do
not reset after restore, compaction, signer rotation, or authority cycling.
Exhaustion fails closed.

`EvidenceId` is generated from a cryptographically secure random source.
Replay of the same operation ID returns the existing outcome and MUST NOT
append a semantically different record.

### 7.2 Deterministic CBOR body

Persistent v1 uses deterministic CBOR under the repository canonical-CBOR
profile. The record is the following numeric-key map:

| Key | Field | Encoding |
|---:|---|---|
| 1 | profile version | uint = 1 |
| 2 | ledger ID | bstr(32) |
| 3 | scope kind | 1 Heap, 2 deployment |
| 4 | scope ID | bstr(16) |
| 5 | sequence | nonzero uint |
| 6 | evidence ID | bstr(16) |
| 7 | event kind | uint16 |
| 8 | obligation class | 1–4 in §5.1 order |
| 9 | outcome | 1 accepted, 2 denied, 3 failed, 4 partial, 5 observed |
| 10 | operation ID | bstr(16/32) or null |
| 11 | commit position | nonzero uint or null |
| 12 | authority epoch | nonzero uint or null |
| 13 | authority generation | nonzero uint or null |
| 14 | authority revision | nonzero uint or null |
| 15 | security revision | nonzero uint or null |
| 16 | actor | canonical map §7.3 |
| 17 | target | canonical map §7.4 |
| 18 | assertion envelope | canonical map §7.7 |
| 19 | before root | bstr(32) or null |
| 20 | after root | bstr(32) or null |
| 21 | observed Unix nanoseconds | signed integer or null |
| 22 | trusted time floor | Unix seconds uint or null |
| 23 | ordering evidence | canonical map §7.5 |
| 24 | coverage | canonical map §7.6 |
| 25 | previous record hash | bstr(32) |
| 26 | signer ID | bstr(16) |
| 27 | signature algorithm | uint = 1 (Ed25519) |
| 28 | signer certificate | canonical certificate bytes |
| 29 | signature | bstr(64) |

Sequence 1 uses 32 zero bytes for `previous record hash`. Every other record
names the immediately preceding canonical record hash.

The signature input is:

```text
"DINGODB-EVIDENCE-RECORD-V1"
|| deterministic_cbor(map keys 1..28, with key 29 absent)
```

Hashing the complete stored record uses:

```text
BLAKE3-256(
  "DINGODB-EVIDENCE-RECORD-HASH-V1"
  || deterministic_cbor(map keys 1..29)
)
```

No floating-point value, indefinite-length item, duplicate key, non-canonical
integer, arbitrary map key, or unregistered event payload field is accepted.
Exact event assertion schemas MUST land in
`spec/evidence/events-v1.json` before implementation milestone DEL-1.

### 7.3 Actor

The actor map is:

| Key | Field |
|---:|---|
| 1 | actor kind: 1 HeapKey, 2 local operator, 3 internal engine, 4 peer |
| 2 | certificate fingerprint bstr(32) or null |
| 3 | holder-public-key fingerprint bstr(32) or null |
| 4 | local credential fingerprint bstr(32) or null |
| 5 | node/deployment identity bstr(16) or null |
| 6 | encoded right used uint64 or null |

Raw certificates, private/public keys, bearer material, usernames supplied by
an untrusted client, and network tokens MUST NOT be stored. Human identity is
an application assertion unless authenticated by a separately named local
operator profile.

### 7.4 Target

The target map is:

| Key | Field |
|---:|---|
| 1 | Heap ID bstr(16) or null for deployment scope |
| 2 | collection ID bstr(16) or null |
| 3 | stream ID bstr(16) or null |
| 4 | object identity digest bstr(32) or null |
| 5 | observed Heap name text or null |

Names are historical context, never authority. User-controlled names are
length-bounded, normalized only by their owning specification, and encoded as
data rather than interpolated text.

### 7.5 Ordering evidence

The ordering map is:

| Key | Field |
|---:|---|
| 1 | ledger sequence |
| 2 | Heap commit position or null |
| 3 | partition ID bstr(16) or null |
| 4 | partition term uint or null |
| 5 | partition position uint or null |
| 6 | writer-local monotonic sample uint or null |

Ledger sequence is total order only within one ledger. Commit position is total
order only within its qualified Heap coordination scope. Wall-clock fields are
evidence, not order.

### 7.6 Coverage

The coverage map is:

| Key | Field |
|---:|---|
| 1 | mode: 1 complete, 2 bounded aggregate, 3 sampled, 4 partial |
| 2 | observed count uint |
| 3 | dropped count uint |
| 4 | saturated boolean |
| 5 | policy revision uint |

`complete` is valid only when the event contract makes completeness
demonstrable. Damage discovered after commitment changes examination coverage;
it does not rewrite the original record.

### 7.7 Assertion envelope

The assertion envelope is:

| Key | Field |
|---:|---|
| 1 | mode: 1 plaintext, 2 encrypted |
| 2 | event schema version |
| 3 | stored assertion bytes |
| 4 | plaintext assertion hash bstr(32) |
| 5 | evidence data-key ID bstr(16) or null |
| 6 | encryption metadata canonical map or null |

In plaintext mode, field 3 is the deterministic-CBOR encoding of the
registered event assertion and fields 5–6 are null. In encrypted mode, field 3
is ciphertext and fields 5–6 identify the frozen encryption profile. The
plaintext assertion hash is:

```text
BLAKE3-256(
  "DINGODB-EVIDENCE-ASSERTION-V1"
  || event_kind_u16_be
  || canonical_plaintext_assertion
)
```

Encryption occurs before the outer record is signed. Core identity, ordering,
coverage, signer certificate, ciphertext, and ciphertext metadata remain
signature-verifiable without the decryption key. An implementation MUST NOT
encrypt by applying a second whole-frame transform that hides the fields
required for offline signature and coverage verification.

## 8. Signing and key separation

### 8.1 Evidence signing key

Every Heap uses an online Evidence Signing Key (ESK) distinct from:

- the offline Heap master key;
- issued HeapKey holder keys;
- payload data-encryption keys; and
- transport TLS keys.

The ESK is Ed25519 in v1. Its certificate binds:

```text
HeapId
AuthorityEpoch
SignerId
public key
valid-from ledger sequence
optional valid-through ledger sequence
evidence profile
```

The certificate is deterministic CBOR:

| Key | Field |
|---:|---|
| 1 | certificate version = 1 |
| 2 | scope kind: 1 Heap, 2 deployment |
| 3 | scope ID bstr(16) |
| 4 | authority epoch uint or null |
| 5 | signer ID bstr(16) |
| 6 | Ed25519 public key bstr(32) |
| 7 | valid-from sequence, nonzero uint |
| 8 | valid-through sequence uint or null |
| 9 | profile text = `dingo-evidence-ledger-v1` |
| 10 | issuer kind: 1 Heap master, 2 deployment evidence root |
| 11 | issuer fingerprint bstr(32) |
| 12 | signature algorithm = 1 |
| 13 | issuer signature bstr(64) |

The issuer signature input is:

```text
"DINGODB-EVIDENCE-SIGNER-CERT-V1"
|| deterministic_cbor(map keys 1..12, with key 13 absent)
```

For a Heap ledger, the certificate is signed by the current Heap master during
a protected local ceremony and becomes usable only through an authority-chain
event. For a deployment ledger, the signer is certified by a deployment
evidence root whose public key is pinned in the protected deployment anchor.
Creation and rotation of that root require the protected local recovery plane
and are themselves deployment-ledger events. After a recovery quorum advances
Heap authority, the recovered current Heap master signs the new evidence
signer certificate; the separate authority event carries the quorum proof.

The ESK private key MUST be non-exportable when the configured provider
supports it.

The complete signer certificate is embedded in every record. This deliberate
bounded duplication means a surviving record contains the public key and
master-signed binding needed to validate its record signature without locating
an earlier signer-registry record. Authority-root material is included in an
export/backup verification package and MAY be pinned independently.

The deployment ledger has a separately certified deployment evidence signer.
A key is never reused across Heap and deployment ledgers.

### 8.2 Rotation

Rotation appends a `ledger_signer_rotate` record signed by the old signer and
binds the new certificate. The first record signed by the new key names the
rotation record as predecessor.

Emergency rotation without the old key requires the Heap recovery ceremony.
The resulting record states the discontinuity and recovery evidence hash.
Verification reports `authorized_discontinuity`; it MUST NOT manufacture an
unbroken old-key signature chain.

### 8.3 Encryption at rest

Evidence confidentiality uses an Evidence Data Key domain distinct from
application payload keys. Destroying a payload key MUST NOT make retained
security evidence unverifiable.

Signatures and hashes cover the stored assertion representation, including
ciphertext. An examiner without the evidence decryption key can still verify
framing, ownership, signatures, sequence, and ciphertext integrity, but
reports assertion contents as `encrypted_unavailable`.

Key identifiers and algorithms are declared by the active encryption profile.
Nonces and associated-data construction MUST be unique and frame-local under
`FORMAT_SPEC.md`; the ledger does not invent a second encryption envelope.

## 9. Physical survival profile

### 9.1 Frame allocation

The following core frame kinds are allocated:

| Value | Kind |
|---:|---|
| 14 | `EvidenceRecord` |
| 15 | `EvidenceCheckpoint` |
| 16 | `EvidenceRetentionCut` |

Every frame uses envelope key 31 `heap_id` and ownership profile 1 for Heap
scope. Deployment frames use envelope key 37 `deployment_id`, omit `heap_id`,
and set ownership profile 2. They MUST NOT be admitted through a `HeapStore`.

Heap-scoped SubjectV2 metadata keys are:

```text
EvidenceRecord:       0x03 || evidence_sequence_u64_be
EvidenceCheckpoint:   0x04 || checkpoint_end_sequence_u64_be
EvidenceRetentionCut: 0x05 || first_retained_sequence_u64_be
```

The frame `event_id` equals `EvidenceId` for a record and the checkpoint/cut ID
for structural evidence.

Deployment evidence uses SubjectV3:

```text
offset  size  field
0       1     version = 0x03
1       16    DeploymentId
17      1     object kind = 0x03 deployment evidence
18      16    all zero
34      2     key length, unsigned big-endian
36      N     metadata key
```

The metadata keys use the same `0x03`, `0x04`, and `0x05` subtype encodings
above. `N` is exactly 9. A SubjectV3 frame is accepted only by the protected
deployment-evidence store; ordinary stores, Heap stores, and network data-plane
paths reject it.

### 9.2 Independent survival

Every evidence record is a complete self-framed SDA-examinable unit with:

- `HeapId` ownership;
- ledger and record identity;
- sequence;
- predecessor hash;
- canonical body hash;
- signer identity; and
- signature.

Therefore, if bytes in the middle of a ledger are destroyed, a later healthy
record remains individually authenticatable. Examination reports the broken
continuity interval rather than rejecting the surviving suffix.

The required rule is:

```text
valid(record_n) does not require readable(record_n-1)
```

The predecessor hash proves continuity when the predecessor survives; it is
not a decode dependency.

### 9.3 Placement

Evidence MAY share physical devices with application data, but qualified
profiles MUST:

- place evidence in separately discoverable segments;
- use independent segment rotation and retention;
- include evidence segments in backup manifests;
- prevent payload compaction from rewriting evidence;
- preserve retained evidence through Heap payload purge; and
- report the media domains covered by each checkpoint and backup.

Evidence storage location is not a security boundary. Frame ownership and
signature validation remain authoritative.

## 10. Checkpoints and anchors

### 10.1 Checkpoint

A checkpoint covers a contiguous bounded range and contains:

| Key | Field |
|---:|---|
| 1 | version = 1 |
| 2 | ledger ID bstr(32) |
| 3 | checkpoint ID bstr(16) |
| 4 | first sequence |
| 5 | last sequence |
| 6 | first record hash bstr(32) |
| 7 | last record hash bstr(32) |
| 8 | ordered record Merkle root bstr(32) |
| 9 | record count |
| 10 | previous checkpoint hash bstr(32) |
| 11 | signer ID bstr(16) |
| 12 | observed Unix nanoseconds signed integer or null |
| 13 | signer certificate canonical bytes |
| 14 | Ed25519 signature bstr(64) |

The signature input and checkpoint hash are:

```text
"DINGODB-EVIDENCE-CHECKPOINT-V1"
|| deterministic_cbor(map keys 1..13, with key 14 absent)

BLAKE3-256(
  "DINGODB-EVIDENCE-CHECKPOINT-HASH-V1"
  || deterministic_cbor(map keys 1..14)
)
```

The first checkpoint uses 32 zero bytes for `previous checkpoint hash`.
Subsequent checkpoints name the preceding complete checkpoint hash. A
checkpoint range begins exactly after the preceding range; overlap, gaps, and
out-of-order ranges are invalid unless explicitly explained by a retention cut
or damage report.

The ordered Merkle tree uses:

```text
leaf = BLAKE3-256("DINGODB-EVIDENCE-MERKLE-LEAF-V1" || sequence_be || record_hash)
node = BLAKE3-256("DINGODB-EVIDENCE-MERKLE-NODE-V1" || left || right)
```

An odd final node is paired with itself. Empty checkpoints are forbidden.
Checkpoints do not replace per-record signatures.

Implementations checkpoint after a configurable count or duration. Both
limits are bounded by the selected deployment profile. High-impact operations
MAY force a checkpoint before their receipt is released.

### 10.2 Anchoring profiles

Every deployment declares one of:

| Profile | Protection |
|---|---|
| `local_signed` | detects mutation/fork; complete local deletion or rollback may be undetectable |
| `local_monotonic` | checkpoint head bound to qualified monotonic local anchor |
| `external_witnessed` | checkpoint hash acknowledged by an independent witness |

The active profile appears in genesis, policy changes, exports, health detail,
and verification reports.

The system MUST NOT advertise rollback resistance stronger than its profile.
External witnessing is not on the ordinary mutation critical path; only
operations whose policy explicitly requires a witnessed checkpoint wait for
one.

### 10.3 Head

The active ledger head is a two-slot, checksummed, atomically selected control
document containing:

| Key | Field |
|---:|---|
| 1 | version = 1 |
| 2 | ledger ID bstr(32) |
| 3 | last sequence |
| 4 | last record hash bstr(32) |
| 5 | last checkpoint hash bstr(32) or null |
| 6 | signer ID bstr(16) |
| 7 | head revision, nonzero uint |

Each slot is `{1: payload_bstr, 2: SHA-256(payload_bstr)}` in deterministic
CBOR. The selector and directory synchronization use the same qualified
two-slot publication rules as the Heap authority head. The head is a recovery
accelerator and local rollback anchor according to §10.2; signed records and
checkpoints remain authoritative evidence.

Startup validates both slots, the selector, the signer registry, and all
available evidence needed to connect the selected head. A valid successor
beyond the selected head is an orphan until deterministic recovery either
publishes it or quarantines it. Equal sequence with unequal valid record hashes
is a fork and makes protected operations unavailable.

## 11. Crash and retry protocol

Required failpoints:

```text
before_evidence_prepare
after_evidence_prepare
after_state_member
after_evidence_member
before_atomic_decision
after_atomic_decision
before_evidence_publish
after_evidence_publish
before_evidence_head
after_evidence_head
before_reply
```

Allowed restart outcomes for `required_atomic`:

- no valid Atomic decision: state and evidence are not visible;
- committed decision, both members healthy: state and evidence are visible;
- committed decision, evidence material damaged later: state remains committed
  and examination reports missing/partial evidence;
- conflicting valid decisions or evidence heads: Heap enters evidence-blocked
  readiness and does not guess;
- replay with the same operation ID: returns the original outcome;
- replay with the same operation ID and different canonical request root:
  `evidence_operation_conflict`.

The implementation MUST never compensate for missing required evidence by
writing a new record that claims to be the original. A recovery/repair record
may describe the loss and provenance of any restored bytes.

## 12. Availability and backpressure

The Evidence Ledger is deliberately not lossy. It therefore has different
backpressure semantics from Ratatouille.

When required evidence cannot be durably committed:

- the protected mutation fails with `EvidenceUnavailable`;
- ordinary unaudited reads MAY continue;
- enhanced-audit reads fail before returning data;
- Heap readiness detail reports `evidence_blocked`;
- Ratatouille MAY report the condition but cannot clear it; and
- no file/stdout fallback is attempted.

`bounded_security` writers use fixed memory, CPU, cardinality, and durable-rate
budgets. Exhaustion increments an in-memory aggregate that is flushed when
capacity returns. If that aggregate is itself lost in a crash, the next
record cannot claim complete coverage.

Storage quotas reserve space for ledger closure, retention cuts, damage
records, and purge results. Application writes MUST NOT be permitted to consume
this reserve.

## 13. Retention, holds, and purge

### 13.1 Default

V1 defaults to `retain_forever`. A shorter policy requires an explicit
`retention_policy_change` evidence record.

Evidence policies classify event kinds independently. A policy cannot shorten
retention below an active legal hold, recovery dependency, signer-certificate
dependency, unexpired backup contract, or the minimum required by another
normative Residiuum profile.

### 13.2 Retention cut

Physical removal is not an ordinary delete. Before removing an eligible closed
range, Residiuum commits a kind-5 `ledger_retention_cut` record and a structural
cut frame:

| Key | Field |
|---:|---|
| 1 | version = 1 |
| 2 | cut ID bstr(16) |
| 3 | ledger ID bstr(32) |
| 4 | first removed sequence |
| 5 | last removed sequence |
| 6 | ordered record Merkle root bstr(32) |
| 7 | governing policy revision |
| 8 | hold-evaluation root bstr(32) |
| 9 | authorizing evidence-record hash bstr(32) |
| 10 | previous retention-cut hash bstr(32) |
| 11 | signer ID bstr(16) |
| 12 | signer certificate canonical bytes |
| 13 | Ed25519 signature bstr(64) |

The signature input and cut hash are:

```text
"DINGODB-EVIDENCE-RETENTION-CUT-V1"
|| deterministic_cbor(map keys 1..12, with key 13 absent)

BLAKE3-256(
  "DINGODB-EVIDENCE-RETENTION-CUT-HASH-V1"
  || deterministic_cbor(map keys 1..13)
)
```

The cut and its authorizing record commit before physical removal begins.
The first cut uses 32 zero bytes for `previous retention-cut hash`.

The cut preserves proof that a specific committed range existed while
deliberately no longer preserving its individual assertions. Verification
reports `retained_by_commitment`, never `complete`.

The cut record and signer certificates needed to verify it are retained for
the lifetime of the Heap identity.

### 13.3 Heap purge

Payload purge does not silently purge the ledger. After Heap purge, the ledger
retains at minimum:

- genesis;
- authority and signer transitions;
- hold and retention decisions;
- purge plan/result and coverage;
- retention cuts;
- damage, repair, recovery, and takeover evidence; and
- permanent identity tombstone linkage.

Evidence-key destruction is a separate high-impact ceremony. It may render
assertion contents unreadable but MUST preserve ciphertext, signatures,
checkpoints, key-destruction evidence, and declared coverage unless a stronger
lawful-erasure profile explicitly authorizes physical removal.

## 14. Query API

Existing Heap operation 143 retains its frozen v1 wire name `audit_read`.
The public SDK method and documentation call the facility `evidence_read`;
qualified dispatch continues to authorize numeric operation 143 with
`AuditRead`.

```text
evidence_read {
    after_sequence?       # exclusive; null means ledger start/retention frontier
    limit                 # 1..=1000, default 100
    event_kinds?          # sorted unique, maximum 64
    outcomes?             # sorted unique
    operation_id?         # exact match
    collection_id?        # exact match inside current Heap
    observed_from_ns?
    observed_to_ns?
    include_assertion     # default false
}
```

Response:

```text
EvidencePage {
    heap_id
    ledger_id
    records
    next_after_sequence?
    retention_frontier
    examined_through_sequence
    material_coverage
    continuity_coverage
    anchor_profile
    checkpoint_hash?
}
```

Pagination is keyset pagination over immutable sequence, never offset
pagination. A cursor is bound to:

```text
HeapId
LedgerId
filter root
last returned sequence
policy revision
```

Changing the filter invalidates the cursor. Missing/damaged ranges are returned
as explicit coverage intervals and do not cause later healthy records to
disappear.

`AuditRead` is necessary but not sufficient to reveal encrypted assertions or
sensitive actor fields. Projection policy may return hashes in place of those
fields. Responses never contain another Heap's records.

## 15. Export and independent verification

An evidence export is immutable and contains:

```text
manifest.cbor
records/
checkpoints/
retention-cuts/
signer-certificates/
authority-proof/
anchor-receipts/
coverage-map.cbor
```

The canonical manifest binds every included object by path-independent type,
identity, length, and BLAKE3-256 hash. It states:

- Heap or deployment scope;
- requested and observed sequence interval;
- missing, damaged, encrypted-unavailable, and retention-cut intervals;
- checkpoint and anchor coverage;
- signer and authority dependencies;
- export time evidence; and
- exporter version/profile.

`dingo evidence verify <package>` MUST operate offline and return independent
axes:

```text
framing:       valid | partial | invalid
ownership:     valid | conflicting | unknown
signatures:    valid | partial | invalid | key_unavailable
continuity:    complete | gaps | fork | truncated | unknown
material:      complete | partial | missing | encrypted_unavailable
anchoring:     local_signed | local_monotonic | external_witnessed | unverifiable
retention:     complete | retained_by_commitment | policy_conflict
```

The tool MUST continue after a damaged interval, resynchronize through SDA,
and verify later independently healthy records. It MUST NOT collapse
`partial`, `missing`, or `unknown` into success.

## 16. Backup, restore, recovery, and clustering

### 16.1 Backup

A Heap backup manifest includes the ledger head, all required records and cuts
through that head, signer certificates, and anchor receipts. A backup that
omits ledger material is a payload-only backup and MUST say so.

### 16.2 Restore to a new Heap identity

Payload-only restore creates a new `HeapId` and a new ledger. Its genesis
contains the source backup manifest hash as inert provenance. Source evidence
is not relabeled or imported as if produced by the new Heap.

### 16.3 Same-identity disaster recovery

Same-identity takeover preserves the ledger, advances `AuthorityEpoch`, rotates
the evidence signer, and appends `heap_takeover`. If the old tail cannot be
recovered, the new record starts an authorized discontinuity naming exact
coverage and recovery evidence.

### 16.4 Cluster

For a qualified strong partition, required evidence is a member of the same
consensus proposal as the protected state change. Followers do not invent
independent Heap-ledger sequence numbers.

Leadership change preserves operation-ID deduplication, commit position,
evidence sequence, and signer authority. A partitioned or stale leader cannot
produce an accepted checkpoint for a later authority epoch.

Node-local authentication failures belong to bounded deployment/node evidence,
not a fabricated globally complete Heap order.

## 17. Data minimization

The ledger MUST NOT contain by default:

- document bodies or query results;
- raw collection keys;
- HeapKey certificates or holder public keys;
- secrets, passwords, tokens, TLS exporters, private keys, or key plaintext;
- complete RQL/SDA text;
- unrestricted client strings;
- network addresses without an explicit privacy policy; or
- telemetry measurements.

Where identity is needed, use a domain-separated digest:

```text
BLAKE3-256(
  "DINGODB-EVIDENCE-IDENTITY-V1"
  || HeapId
  || kind
  || canonical_identity_bytes
)
```

Digestability does not make low-entropy values anonymous. Policies MUST state
dictionary-attack exposure and MAY use a protected keyed digest profile.

Event schemas define maximum encoded size. V1 record bodies MUST NOT exceed
64 KiB. Larger reports are stored as separately framed, heap-owned evidence
artifacts and referenced by content hash and coverage; they are never embedded
without bound.

## 18. Errors and health

Public errors:

```text
evidence_unavailable
evidence_operation_conflict
evidence_policy_denied
evidence_cursor_invalid
evidence_coverage_incomplete
evidence_encrypted_unavailable
evidence_export_incomplete
```

Protected local diagnostics additionally distinguish:

```text
evidence_head_corrupt
evidence_fork
evidence_signature_invalid
evidence_signer_unknown
evidence_sequence_gap
evidence_anchor_stale
evidence_reserve_exhausted
evidence_retention_conflict
```

Heap readiness exposes only a Boolean publicly. Authorized detail reports:

- ledger head sequence and checkpoint age;
- active signer and certificate validity;
- reserve capacity;
- anchoring profile and last anchor status;
- damage/fork/gap state;
- pending bounded aggregates; and
- whether protected writes and enhanced-audit reads are admitted.

Metrics and Ratatouille events MAY mirror these states. They are not evidence.

## 19. Rust boundary

The implementation separates decision, canonicalization, persistence, and
telemetry:

```rust
pub struct EvidencePlan<H> {
    heap: HeapBrand<H>,
    operation_id: OperationId,
    event: ClosedEvidenceEvent,
    obligation: EvidenceObligation,
}

pub trait EvidencePlanner<H> {
    fn close(
        &self,
        cap: &HeapCap<H>,
        operation: &AuthorizedOperation,
    ) -> Result<Option<EvidencePlan<H>>, EvidenceError>;
}

pub trait EvidenceAtomicMember<H> {
    fn prepare_member(
        &mut self,
        plan: EvidencePlan<H>,
    ) -> Result<PreparedEvidenceMember<H>, EvidenceError>;
}

pub trait EvidenceReader<H> {
    fn page(
        &self,
        cap: &HeapCap<H>,
        request: EvidenceRead,
    ) -> Result<EvidencePage, EvidenceError>;
}

pub trait EvidenceVerifier {
    fn verify(
        &self,
        source: &mut dyn EvidenceSource,
    ) -> EvidenceVerificationReport;
}
```

`ClosedEvidenceEvent` has no generic JSON map, callback, format string, script,
or user-selected event name. Only registry-generated typed variants can enter
canonicalization.

No evidence API accepts `Option<HeapId>` for a Heap-bound operation. No
implementation crate exposes an unchecked constructor for `EvidenceReader<H>`
or a way to change its brand.

Ratatouille adapters consume committed evidence notifications after commit.
The dependency direction is:

```text
operation -> evidence decision/commit -> optional telemetry notification
```

Evidence code never calls telemetry in order to complete a commit.

## 20. Verification properties

The pure verification kernel MUST establish at least:

```text
P1 Scope:
  verify(record, Ledger(a)) = valid => record.scope_id = a

P2 Sequence uniqueness:
  valid(r1) ∧ valid(r2) ∧
  r1.ledger_id = r2.ledger_id ∧ r1.sequence = r2.sequence
  => hash(r1) = hash(r2), otherwise fork

P3 Continuity:
  r2.sequence = r1.sequence + 1 ∧ complete_interval(r1, r2)
  => r2.previous_record_hash = hash(r1)

P4 Independent survival:
  verify_signature(rn) does not require bytes(rn-1)

P5 Atomic evidence:
  visible(protected_mutation) => committed(required_evidence)

P6 Confinement:
  Exec(S, Cap(a), evidence_op) changes no Ledger(b), b != a

P7 Retention honesty:
  removed(range) => valid_retention_cut(range) ∨ damage_reported(range)

P8 Telemetry independence:
  telemetry_state cannot change evidence decision or committed outcome
```

P1, P2, P4, P6, and canonical encoding/signature verification are candidates
for Verus/Kani connection. P5 is a refinement obligation against the Atomic
commit implementation. Crash and media-loss tests remain necessary; proof of
the pure kernel does not prove filesystem or consensus behavior.

## 21. Qualification tests

A profile cannot claim `dingo-evidence-ledger-v1` until all pass:

1. golden canonical-CBOR and signature vectors;
2. malformed/non-canonical/adversarial decoder corpus;
3. operation-ID identical replay and conflicting replay;
4. every failpoint in §11 across every `required_atomic` event family;
5. evidence-media full, read-only, short-write, torn-write, and fsync failure;
6. payload mutation cannot commit when required evidence fails;
7. telemetry disabled, blocked, disconnected, and overflowing with no effect
   on evidence outcomes;
8. cross-Heap API, iterator, cursor, export, backup, salvage, and cache attacks;
9. signer rotation, lost signer, expired certificate, and recovery rotation;
10. record deletion, insertion, reordering, replay, truncation, and fork;
11. holes through record and checkpoint segments followed by SDA recovery;
12. retained suffix verification with missing middle records;
13. retention cut with and without holds and recovery dependencies;
14. payload purge while minimum evidence remains verifiable;
15. evidence-key destruction with ciphertext verification;
16. backup/restore-to-new-ID and same-ID takeover;
17. bounded denial flood with honest dropped/aggregate coverage;
18. offline export verification with no running Residiuum;
19. concurrency/sequence linearizability;
20. cluster leader change and stale-leader rejection for any cluster claim;
21. fuzzing of record, checkpoint, cut, certificate, cursor, and manifest
    parsers; and
22. external review of claim language and cryptographic construction.

## 22. Work packages

| ID | Deliverable | Depends on | Exit evidence |
|---|---|---|---|
| DEL-0 | machine-readable registries, CBOR schemas, vectors | spec freeze | cross-language golden vectors |
| DEL-1 | `dingo-evidence` pure types, canonicalizer, verifier | DEL-0 | property/fuzz tests |
| DEL-2 | EvidenceRecord/Checkpoint/Cut format support + SDA | DEL-1 | hole/salvage corpus |
| DEL-3 | per-Heap store, head, reserve, recovery | DEL-2 | crash matrix |
| DEL-4 | signer certificates, rotation, provider interface | DEL-1, authority | crypto vectors + ceremonies |
| DEL-5 | Atomic coupling for mandatory mutations | DEL-3, Atomics | failpoint proof corpus |
| DEL-6 | bounded denial aggregation and policy engine | DEL-3 | flood tests |
| DEL-7 | heap-confined read/cursor/export API | DEL-3 | isolation tests |
| DEL-8 | retention, holds, purge, backup, restore | DEL-3–7 | lifecycle matrix |
| DEL-9 | offline verifier and SDA examination UX | DEL-2, 4, 8 | damaged-package fixtures |
| DEL-10 | cluster integration | qualified cluster/Atomics | partition histories |
| DEL-11 | formal connection and qualification | DEL-0–10 as claimed | proof + review bundle |

DEL-0 through DEL-9 are required for a qualified single-node claim. DEL-10 is
required only for a cluster claim. No milestone may replace durable evidence
with the existing in-memory `HeapAuthAuditLog`.

## 23. Compatibility and migration

The existing bounded in-memory `HeapAuthAuditLog` is diagnostic state. On DEL
activation it becomes a source for `bounded_security` aggregation or is
removed. Its historical contents are not imported and MUST NOT be represented
as durable evidence.

Existing administrative receipts remain valid domain receipts. Where their
operation becomes ledger-protected, the receipt adds:

```text
evidence_id
evidence_sequence
evidence_hash
checkpoint_hash?
```

A receipt is a caller-facing reference to evidence, not a duplicate evidence
record.

Before activation, the server reports:

```text
evidence_ledger = unavailable
evidence_profile = none
```

It MUST NOT describe in-memory diagnostics, structured logs, Ratatouille
events, history frames, or Atomic evidence as the Residiuum Evidence Ledger.

## 24. Completion definition

The Evidence Ledger is development-complete for qualified single-node use only
when:

- DEL-0 through DEL-9 are complete;
- all §21 applicable tests pass in CI;
- all mandatory event producers use typed closed schemas;
- every `required_atomic` producer is connected to the Atomic commit path;
- heap isolation qualification includes ledger read/export/recovery paths;
- offline verification survives arbitrary holes with honest coverage;
- documentation states the configured anchoring and retention profiles; and
- the runtime contains no path that calls files, stdout, stderr, or
  Ratatouille as a substitute for required evidence.
