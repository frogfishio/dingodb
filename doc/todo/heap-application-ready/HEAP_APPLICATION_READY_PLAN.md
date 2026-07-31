# Heap Application Ready work plan

Status: developer-ready v1.0

Program release: P1

Normative source: [HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md)

Living qualification queue: [HEAP_NEXT_TASKS.md](../../done/programs/HEAP_NEXT_TASKS.md)

## 1. Outcome

P1 is complete when this journey works from a clean machine:

```text
create deployment
create Heap "acme"
create Heap "beta"
issue separate CRUD application keys
start the qualified Heap server
connect to each Heap
create collection "users" in both
put the same key with different values
query, page, inspect history, and manage an index
prove neither key can observe the other Heap
blacklist one application key
cycle authority and prove all old keys inert
back up and restore one Heap without transferring authority
retire the restored/source Heap safely
```

The journey MUST use no legacy flat SDK, raw Store access, shared token, or
RBAC lookup.

## 2. Frozen public shape

### 2.1 Rust

Target application surface:

```rust
let deployment = Residiuum::open_deployment("./data")?;

let acme = deployment.open_heap(acme_key)?;
let users = acme.create_collection("users")?;

users.put("user-42", &json!({ "name": "Alice" }))?;
let value = users.get("user-42")?;
```

Administration is separate:

```rust
let local = HeapAuthority::open_local("./data", master_key)?;
let created = local.create_heap(CreateHeap::named("acme"))?;
let key = local.issue_key(
    created.heap_id(),
    IssueKey::new(Rights::READ | Rights::WRITE),
)?;
```

`HeapAuthority` is illustrative naming for the package. Implementers MAY reuse
the existing `residiuum-authority` types, but MUST preserve the separation:

- ordinary `Residiuum`/`RemoteHeap` handles cannot wield master authority;
- network data connections cannot execute master-key ceremonies;
- local authority APIs are not linked into the qualified data server.

### 2.2 CLI

Target commands:

```text
dingo heap create STORE --name acme --master-key-out FILE
dingo heap list STORE
dingo heap inspect STORE --heap acme

dingo heap key issue STORE --heap acme --master-key FILE \
  --rights read,write --out FILE
dingo heap key inspect FILE
dingo heap key blacklist STORE --heap acme --master-key FILE \
  --certificate FILE
dingo heap key cycle STORE --heap acme --master-key FILE \
  --new-master-key-out FILE [--grace 48h]

dingo heap collection create STORE --heap acme --key FILE users
dingo heap backup STORE --heap acme --key FILE --output PACKAGE
dingo heap restore PACKAGE --store STORE --new-name recovered-acme
dingo heap retire STORE --heap acme --master-key FILE
```

Command spelling MAY change once, during `HAR-2`. Command meaning may not.

Secret material MUST default to an explicit file, created atomically with
owner-only permissions. It MUST NOT print to stdout, logs, shell history, or
JSON output. An explicit dangerous development flag MAY print a test key.

## 3. Rights

P1 uses existing rights:

| Operation | Required right |
|---|---|
| open/list/get/scan/find/history | `Read` |
| put/delete | `Write` |
| create/list/drop/rebuild index | `IndexAdmin` as specified |
| create collection | `HeapAdmin` |
| ordinary Heap metadata inspection | `Read` with declassification limits |
| local issue/blacklist/cycle/retire | master authority; never a network right |

No new role or RBAC table is introduced.

## 4. Work packages

### HAR-0 — Truth cleanup

Purpose: make repository evidence agree before adding surface.

Work:

- update `scripts/check_kani_heap.sh` to require
  `VERUS_PROOFS_CONNECTED=true`;
- run Kani and Verus jobs together;
- remove stale statements that Verus/Kani remain open;
- preserve `qualified=false` because CPR-005 remains open;
- update `HEAP_NEXT_TASKS.md` so external review is not the first code package;
- add P1 package IDs to the status scoreboard.

Evidence: Unit, Model.

Exit:

```text
scripts/check_kani_heap.sh
scripts/check_verus_heap.sh
scripts/check_heap_architecture.sh
```

agree on the same flags and CI is green.

### HAR-1 — Collection creation

Purpose: remove the offline/manual provisioning gap.

Owned areas:

- `spec/heap/operations-v1.json`;
- `spec/heap/rpc-v1/collection_create.*.json`;
- accepted/rejected fixtures;
- `residiuum-store::heap::catalog::create_object`;
- `residiuum-server::heap_dispatch`;
- `residiuum-sdk::RemoteHeap` and embedded Heap handle.

Contract:

```text
collection_create {
  operation_id
  canonical_name
}
→ {
  collection_id
  canonical_name
  descriptor_hash
  receipt
}
```

Rules:

- operation code is `106`;
- requires `HeapAdmin`;
- name normalization occurs once under the Heap profile;
- duplicate same-name/same-operation retry returns the original result;
- duplicate name with different operation returns stable conflict;
- the collection ID is immutable and Heap-bound;
- no other Heap's names affect the result or timing class beyond the qualified
  metadata profile;
- publication is staged then atomically made discoverable;
- a crash cannot expose an uncommitted partial descriptor.

Required tests:

- create/list/open/use;
- idempotent retry;
- duplicate conflict;
- wrong right;
- same name in two Heaps;
- foreign Heap key;
- failpoint before/after publication;
- rebuild catalog after deletion;
- RPC golden fixtures.

Evidence: Unit, Isolation, Crash, Journey.

Exit: `RemoteHeap::create_collection` and embedded equivalent pass the same
conformance corpus.

### HAR-2 — Local Heap creation ceremony

Purpose: create a Heap without hand-editing catalogs or using test fixtures.

Owned areas:

- `residiuum-authority::ceremony`;
- `residiuum-store::heap::catalog`;
- new CLI `heap create/list/inspect`;
- SDK/admin façade only if it remains local.

Creation phases:

```text
validate target and name
generate HeapId and staging ID
create master key locally
stage Heap descriptor
commit authority genesis bound to staged descriptor hash
publish descriptor and authority head
write creation receipt
return Heap identity and master-key file
```

Crash outcomes:

- before authority commit: no discoverable Heap;
- after authority commit/before publication: restart completes publication or
  enters the specified failed-creation tombstone path;
- after publication: retry returns the original Heap.

Security:

- refuse symlink/path traversal under authority root;
- master key file mode is owner-only;
- no secret in process logs;
- master-key bytes never enter store frames or qualified server memory;
- creation requires local filesystem authority;
- same name does not imply same identity.

Required tests:

- clean create;
- crash at every phase;
- same operation retry;
- conflicting create;
- bad permissions;
- catalog deletion/rebuild;
- two Heaps with same collection/key names;
- master-key absence proves ordinary data server still starts but cannot mutate
  authority.

Evidence: Unit, Isolation, Crash, Journey.

Exit: a clean checkout can create a usable Heap through CLI only.

### HAR-3 — Application-key lifecycle

Purpose: complete the CA-like authority experience.

Operations:

- issue key;
- inspect public claims;
- blacklist certificate fingerprint;
- cycle generation with no grace;
- cycle generation with bounded grace;
- resolve status;
- export only explicitly authorized public material.

Rules:

- issuance specifies exact rights, constraints, audience, and expiry;
- issued key is self-contained for request admission;
- blacklist is part of the resident immutable security snapshot;
- hard cycle invalidates every earlier generation;
- grace admits only the immediately previous generation until its trusted-time
  deadline;
- blacklisting takes precedence over grace;
- mutation builds and validates a complete next snapshot before publication;
- every security-barrier operation waits for server reload acknowledgement when
  the server is running;
- retry uses the same operation identity and returns the same receipt.

Required tests:

- rights matrix;
- issuer and audience mismatch;
- blacklist hit/miss;
- cycle all-old-inert;
- grace boundary;
- blacklist during grace;
- restart during grace;
- rollback/forked head;
- clock uncertainty fail-closed;
- no Heap existence leak;
- no network master operation.

Evidence: Unit, Property, Isolation, Crash, Model, Journey.

Exit: the CLI can reproduce the complete issue → use → blacklist/cycle journey.

### HAR-4 — Qualified remote posture

Purpose: make the HeapKey path the normal Residiuum server.

Work:

- make HeapKey TLS listener the default Heap-serving profile;
- require explicit `--legacy-token-server` or equivalent for legacy service;
- prohibit qualified and legacy global access from sharing a process/store;
- validate config before bind;
- ensure process help and errors label the legacy path non-qualified;
- preserve public health endpoints only under the closed declassification
  registry.

Required tests:

- default config starts HeapKey listener;
- absent/invalid HeapKey receives indistinguishable reject;
- token cannot authenticate to qualified listener;
- qualified key cannot authenticate to legacy listener;
- config cannot co-host both paths;
- TLS exporter replay fails;
- admission, rate, timeout, and shutdown behavior;
- remote parity for every active P1 operation.

Evidence: Isolation, Journey, Performance.

Exit: every public remote tutorial uses `connect_heap`; shared-token examples
are in a legacy appendix only.

### HAR-5 — Heap operations

Purpose: make one Heap independently operable.

P1 operations:

- heap-scoped backup;
- payload-only restore to a new Heap identity;
- inspect backup manifest;
- retire;
- purge planning/status where already qualified;
- scrub/salvage classification by Heap;
- lifecycle status.

Rules:

- backup never exports master authority;
- restore creates a new Heap ID unless the separate DR retain-ID ceremony is
  explicitly invoked locally;
- cursors, capabilities, and application keys from the source are inert on the
  restored Heap;
- incomplete purge remains `retired`;
- foreign-Heap units are excluded and reported;
- unknown/corrupt ownership is quarantined, not guessed.

Required tests:

- backup/restore one of two co-resident Heaps;
- source keys fail against restored Heap;
- restored Heap receives new application keys;
- damage and mixed-Heap frames;
- unavailable media during purge;
- retention/legal-hold refusal;
- repeated operation identity.

Evidence: Isolation, Crash, Damage, Journey.

Exit: P1 can demonstrate backup/restore without any cross-Heap authority
transfer.

### HAR-6 — Ordinary SDK and CLI journey

Purpose: remove architecture knowledge from normal application use.

Deliver:

- one Rust quickstart;
- one CLI quickstart;
- one remote quickstart;
- stable error codes;
- receipts with operation ID and achieved durability;
- key-safe configuration loading;
- examples for indexes, history, pagination, blacklist, and cycle;
- compatibility migration note for flat stores.

The ordinary application type MUST be Heap-bound. It MUST be impossible to
obtain a global collection handle from it.

Required journey fixture:

```text
tests/journeys/heap_application_ready/
```

It runs from an empty temporary directory and emits a machine-readable report.

Evidence: Journey, Isolation.

Exit: a developer follows public docs without importing a legacy feature.

### HAR-7 — Release evidence

Purpose: decide whether P1 may be labelled Application Ready.

Required:

- all HAR package gates green;
- `verify-heap.sh full`;
- Kani and Verus green;
- two-Heap differential suite;
- crash matrix;
- fuzz budget;
- bounded latency disclosure for admission;
- public limitations;
- capability matrix and website/docs updates.

P1 does not require a paid review. Without CPR-005 it MUST say:

> Heap isolation is implemented, machine-checked, adversarially tested, and
> awaiting independent review.

It MUST NOT say “independently reviewed” or set the H6 qualified claim.

## 5. Package ownership

| Area | Primary crate |
|---|---|
| IDs, rights, pure admission | `residiuum-heap` |
| local master ceremony | `residiuum-authority` |
| descriptors, lifecycle, backup | `residiuum-store` |
| Heap protocol and listener | `residiuum-server` |
| ordinary embedded/remote API | `residiuum-sdk` |
| human administration | `residiuum-cli` / local authority binary |
| fixtures and qualification | `spec/heap`, `verification/heap-verus` |

No package may bypass its lower-level owner by reproducing authority logic.

## 6. Explicitly deferred

- independent review procurement;
- cluster Heap placement/qualification;
- cross-Heap administration transactions;
- hosted control plane;
- billing;
- broad multi-user RBAC;
- cascade delete;
- text/vector/geospatial indexes;
- direct ranked access.
