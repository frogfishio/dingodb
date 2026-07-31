# Residiuum Ingress specification

Status: **normative design v0.1-draft**

Scope: hostile-network client ingress for the native stateful protocol and
HTTPS JSON data API. This specification defines no management surface.

## 1. Decision

Residiuum SHALL provide a separate, hardened `residiuum-ingress` process.

Ingress is an untrusted, non-authoritative data-plane bridge:

```text
client authority
      ↓
Residiuum Ingress
      ↓
the same or less authority
      ↓
authoritative partition
```

Its governing property is authority non-amplification:

\[
Authority(Forward(r)) \subseteq Authority(Client(r))
\]

Ingress serves exactly two public protocol families:

1. the native, stateful Residiuum client protocol; and
2. an HTTPS JSON data API.

It verifies, admits, routes and streams. It is not a database node, control
plane, application server, generic reverse proxy or administrative gateway.

## 2. Physically absent authority

The ingress binary MUST contain no code, route, opcode or dependency capable
of:

- creating, deleting or configuring a Heap or collection;
- issuing, rotating or recovering master/application credentials;
- changing cluster membership, voters, placement, terms or replication;
- sealing, unsealing or changing the seal profile;
- invoking repair, salvage mutation, forced recovery or evidence deletion;
- reading or opening Residiuum storage directories;
- administering the Evidence Ledger;
- running plugins, scripts, commands, arbitrary upstream requests or files;
- exposing debug consoles, dynamic routes or generic execute-command methods.

These capabilities are absent, not disabled by policy. A feature flag MUST NOT
compile them into an ingress release binary.

The build has a closed dependency allowlist. CI SHALL reject dependencies on
storage, cluster-control, repair, key-ceremony, administrative or evidence-
administration crates. It SHALL also inspect exported symbols, opcode tables,
HTTP routes and binary features against a frozen ingress manifest.

## 3. Ingress identity

Ingress has a narrow cluster-issued identity granting only:

```text
open an authenticated forwarding channel
receive signed routing/policy snapshots
submit an independently authenticated client request
relay the resulting response and evidence
emit bounded telemetry and audit events
```

The ingress identity grants no data operation. Every destination independently
validates the original client capability and request proof. An assertion that
“Ingress approved this request” is insufficient authority.

Ingress holds only:

- its narrow transport identity;
- public capability-verification material;
- authenticated epoch/blacklist and admission snapshots;
- a derived partition-routing cache;
- ephemeral sessions; and
- TLS/ACME operational material.

It holds no master key, cluster signing root, membership vote, repair authority
or authoritative payload.

## 4. Deployment and network boundary

The normal public topology is:

```text
hostile network
      ↓
one or more Residiuum Ingress processes
      ↓ authenticated private data-plane channels
Residiuum partition leaders/eligible replicas
```

Storage and control nodes remain private. An external WAF, CDN, DDoS service or
L4/L7 load balancer MAY precede Ingress, but none is required for routing or
database correctness.

Ingress exposes public data listeners only. Its health/readiness listener is
loopback or a separately protected operational network and reveals no Heap,
partition, topology or customer identity.

## 5. Closed public operation set

The native and JSON surfaces map to one canonical data-plane operation set:

```text
get
create
put
patch
delete
query
continue
subscribe
cancel
ping
```

An implementation profile may postpone an operation, but cannot add a generic
command escape. Unknown methods, routes, versions and opcodes are rejected
before body decoding.

Collection and Heap lifecycle operations are management and never enter this
set. `query`, `continue` and `subscribe` operate on data under the client’s
capability; they confer no schema, policy or cluster authority.

## 6. HTTPS JSON data API

The closed route families are:

```text
POST   /v1/collections/{collection}/documents
PUT    /v1/collections/{collection}/documents/{key}
GET    /v1/collections/{collection}/documents/{key}
HEAD   /v1/collections/{collection}/documents/{key}
PATCH  /v1/collections/{collection}/documents/{key}
DELETE /v1/collections/{collection}/documents/{key}
POST   /v1/collections/{collection}/query
POST   /v1/continuations/resume
POST   /v1/continuations/cancel
```

The Heap comes from the cryptographically bound capability and MUST agree with
every authenticated request field. It is not selected by an untrusted header.

Mutating requests require one canonical idempotency identity. JSON objects
reject duplicate property names, invalid Unicode and non-profile numeric
values. Patch semantics use one frozen Residiuum patch profile; accepting
multiple ambiguous patch dialects is forbidden.

Continuation material is supplied in a signed header or body, never in a URL
where infrastructure logs and caches commonly retain it.

No catch-all route, directory serving, dynamic extension or upstream URL is
permitted.

## 7. Native stateful protocol

The native protocol supports long-lived authenticated connections,
multiplexed bounded streams, streaming bodies/responses, prepared data plans,
continuations and cancellation.

Its opcode table is generated from the same closed operation registry as §5.
Frames are length-delimited, versioned and bounded before allocation. Unknown
critical fields and unsupported versions fail closed.

Native payloads are forwarded without deserialize/reserialize when admission
and routing can be established from the authenticated envelope. Optional
application-layer payload encryption MAY keep bodies opaque to Ingress.

## 8. Assurance profiles

Every operation selects one of four composable assurance profiles.

### 8.1 `transport`

TLS plus a short-lived, audience-bound, Heap-bound bearer capability. This is
the ordinary low-friction server API.

### 8.2 `proof_of_possession`

The capability binds a client public key. Every request is signed over the
canonical operation, target, content digest, capability identity, request ID,
deadline and security-critical metadata.

For HTTP, the profile uses
[HTTP Message Signatures (RFC 9421)](https://www.rfc-editor.org/rfc/rfc9421.html)
and [Digest Fields (RFC 9530)](https://www.rfc-editor.org/rfc/rfc9530.html).
If semantic JSON rather than exact transmitted bytes is signed, the selected
profile uses [JCS (RFC 8785)](https://www.rfc-editor.org/rfc/rfc8785.html) with
published errata and frozen vectors. Exact-byte signing is preferred where it
does not harm interoperability.

Ingress may relay a valid signed request but cannot alter or manufacture one
without the client private key.

### 8.3 `commit_receipt`

The authoritative partition returns a signed receipt binding the request to
its outcome and commitment evidence. Ingress can relay but cannot issue or
upgrade an authoritative receipt.

### 8.4 `ledger_evidence`

The receipt is bound to a signed Residiuum Evidence Ledger checkpoint and
carries or references an independently verifiable inclusion proof.

The ordinary profile remains simple. Higher assurance is requested through a
canonical native field or HTTP `Residiuum-Assurance` field and may be required
by Heap policy.

## 9. Canonical signed request

The signed request envelope binds at minimum:

```text
protocol/domain/version
cluster and Heap audience
collection/key or query target
operation and conditional predicates
content/representation digest
capability digest and client key identity
canonical idempotency/request identity
nonce where required
issued time and absolute deadline
requested read/durability/assurance profile
```

A signature from another protocol, cluster, Heap, operation or version domain
is invalid. Ingress and destination use the same golden vectors and reject
ambiguous encodings.

## 10. Replay and idempotency

Ingress MAY reject obvious replay from a bounded cache, but its cache is not
authoritative. The destination partition enforces replay/idempotency semantics
at the point of effect.

Retries through a different ingress or node use the same identity and produce
the original committed result, one newly committed effect, or an honest
indeterminate outcome. They MUST NOT duplicate a logical mutation.

Expired requests, mismatched body digests, reused identities with different
operations, invalid nonces and invalid signatures fail before database work.

## 11. Authoritative receipt

A `commit_receipt` contains at minimum:

```text
receipt format/version and cryptographic domain
cluster identity/generation
request and capability digests
client key and canonical request identities
Heap, collection, key and operation
result and logical generation/value digest
partition, term, log position and placement epoch
durability/commit-evidence digest
declared time authority and issued time
Evidence Ledger position/root when requested
signing key identity and cluster signature
```

Receipt meanings are exact:

- a write receipt proves the named operation and value digest committed;
- a read receipt proves the returned observation/value digest at the stated
  view/frontier and coverage;
- an absence receipt requires `absent_proved` and its authoritative frontier;
- a delete receipt proves logical deletion/tombstone commitment, not physical
  erasure of every historical byte; and
- a partial, damaged, unknown or unavailable result cannot be promoted to a
  complete/absence receipt.

Receipt verification is offline and independent of Ingress or a running
database:

```text
residiuum verify-receipt <receipt> [--request <request>]
residiuum verify-ledger-proof <receipt> <checkpoint>
```

## 12. Non-repudiation claim boundary

Residiuum claims cryptographic attribution and independently verifiable
commitment evidence, not metaphysical proof of a human act.

The claim is conditional on disclosed assumptions including private-key
custody at signing time, cluster-signing-key protection, algorithm security,
revocation/epoch state and the named time authority. A compromised client key
limits attribution exactly as documented.

## 13. Honest HTTP outcomes

HTTP status does not replace the Residiuum observation algebra. Every response
contains or maps unambiguously to one of:

```text
complete
absent_proved
partial
damaged
unknown
unauthorized
unavailable
conflicting
```

`404` is permitted only for proved absence. A timeout, missing partition,
unavailable archive tier, incomplete index or unreadable evidence cannot become
`404` or an empty complete query.

## 14. Admission order and resource bounds

Ingress performs the cheapest safe rejection first:

```text
socket/concurrency budget
→ TLS handshake limits
→ frame/header/method/route bounds
→ capability structure and signature
→ epoch/blacklist snapshot
→ proof-of-possession and replay/deadline checks
→ Heap/capability quota and query-cost admission
→ bounded decompression/body parsing
→ route and stream with backpressure
```

Every allocation and queue has a declared bound. Profiles specify maximum
headers, body/frame size, JSON depth, decompression ratio, streams, request
lifetime, response buffering, query cost and per-capability/Heap concurrency.
One Heap cannot consume another Heap’s reserved admission envelope.

## 15. TLS and ACME

Ingress uses a mature, pinned TLS implementation; it does not implement
cryptographic primitives or TLS itself. It supports modern TLS, optional mutual
TLS, protected certificate injection and ACME issuance/renewal.

ACME profiles cover standalone HTTP/TLS validation, DNS validation for
multi-ingress deployments and operator-provided certificates. Multi-ingress
issuance is coordinated to prevent validation races and issuance-rate abuse.
Certificate rotation does not drop accepted connections outside declared drain
behavior.

ACME authority manages only the public transport identity. It grants no Heap,
cluster or administrative authority.

## 16. Routing

Ingress follows the load-balancer-free routing contract in
[CLUSTER_SPEC.md](CLUSTER_SPEC.md#13-routing-without-a-mandatory-load-balancer).
It discovers several private data-plane nodes, caches authenticated partition
routes, forwards directly, refreshes stale epochs and uses canonical request
identities on retry.

Multiple ingress processes are independent and replaceable. DNS/SRV and client
failover MAY provide discovery without an external load balancer.

## 17. Process containment

Production Ingress runs unprivileged with no database mounts, no shell/child
process permission, a read-only filesystem where possible, bounded memory and
descriptors, a strict syscall sandbox and outbound access only to declared
data-plane/ACME/telemetry destinations.

It emits high-volume operational telemetry through the Residiuum telemetry
channel and sends named security/audit events to the Evidence Ledger. It does
not synchronously log request bodies, credentials or secret material.

## 18. Compromise boundary

Assume the entire ingress process is compromised. The attacker may disrupt or
observe plaintext traffic handled by it, consume its bounded resources and lie
about availability.

The attacker MUST NOT thereby be able to issue capabilities, alter a signed
operation, cross Heap authority, join consensus, change placement, become a
replica, invoke repair, access database files or perform management. The
ingress TLS/transport identity MUST NOT be accepted as client authority.

Native application-layer encryption may narrow plaintext exposure. The JSON
API necessarily processes plaintext unless a separately specified encrypted-
JSON profile is used.

Volumetric attacks that saturate the network are outside a single process’s
defeat guarantee; optional upstream DDoS protection remains compatible.

## 19. Formal and test obligations

The Formal Assurance Spine SHALL include ingress authority non-amplification,
Heap confinement, request-domain separation, destination revalidation, replay
safety and receipt unforgeability under named cryptographic assumptions.

Conformance includes:

1. dependency, symbol, route and opcode denylist enforcement;
2. parser fuzzing and state-machine model checking for both protocols;
3. cross-protocol signature and replay attempts;
4. signed-request mutation of every bound field;
5. destination rejection after a malicious ingress claims authorization;
6. Heap/epoch/audience confusion attempts;
7. duplicate, reordered and conflicting idempotency identities;
8. malformed/deep/oversized/compression-bomb JSON and native frames;
9. slowloris, stream floods, cancellation and backpressure;
10. stale routing, ingress loss after commit and retry through another ingress;
11. forged, altered, stale and wrong-cluster receipts;
12. ledger-inclusion proof verification and mutation;
13. certificate issuance, renewal, expiry and multi-ingress races;
14. resource isolation between hostile and healthy Heaps; and
15. operation with no external proxy or load balancer.

An ingress release is accepted only when the closed binary manifest, negative
authority tests, protocol corpus, resource bounds, formal obligations and
hostile-network qualification all pass.
