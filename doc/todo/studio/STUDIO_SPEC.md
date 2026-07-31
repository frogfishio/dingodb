# Residiuum Studio v1 specification

Status: normative product and architecture design v1.0-draft; implementation
not yet qualified

Product name: **Residiuum Studio**

Profiles:

```text
residiuum-studio-v1
residiuum-studio-ipc-v1
residiuum-studio-workspace-v1
residiuum-studio-telemetry-v1
residiuum-studio-evidence-v1
```

Implementation baseline:

- Rust workspace toolchain;
- [Tauri 2.11.5](https://tauri.app/release/tauri/all-versions/);
- [Angular 22](https://angular.dev/reference/releases);
- TypeScript version supported by Angular 22;
- SCSS using the current supported Dart Sass toolchain; and
- Residiuum protocols and SDKs from the same repository revision.

Dependency versions are exactly pinned in release builds. Updating Tauri,
Angular, the webview/runtime, or any security-sensitive plugin requires the
qualification subset in §34.4.

Normative companions:
[HEAP_SPEC.md](../../wip/heap/HEAP_SPEC.md),
[DX_SPEC.md](../../reference/product/DX_SPEC.md),
[RQL_SPEC.md](../../wip/query/RQL_SPEC.md),
[SDA_PROFILE.md](../../reference/query/SDA_PROFILE.md),
[COLLECTION_CONTRACT_SPEC.md](../rre/COLLECTION_CONTRACT_SPEC.md),
[RRE_SPEC.md](../rre/RRE_SPEC.md),
[ATOMICS_SPEC.md](../atomics/ATOMICS_SPEC.md),
[DIRECT_ACCESS_SPEC.md](../direct-access/DIRECT_ACCESS_SPEC.md),
[TELEMETRY_SPEC.md](../telemetry/TELEMETRY_SPEC.md),
[EVIDENCE_LEDGER_SPEC.md](../evidence/EVIDENCE_LEDGER_SPEC.md), and
[doc/todo/studio/STUDIO_IMPLEMENTATION_PLAN.md](./STUDIO_IMPLEMENTATION_PLAN.md).

## 1. Decision

Residiuum Studio is the first-party desktop environment for developing,
examining, operating, and understanding Residiuum.

It is not a web admin page wrapped in a desktop shell. It is a capability-bound
database IDE with:

- Heap-confined live data workspaces;
- RQL, SQL-ish import, and SDA examination;
- document, bytes, history, damage, and coverage views;
- RRE, collection-contract, relationship, and Atomic tooling;
- Ratatouille telemetry dashboards and live tail;
- Residiuum Evidence Ledger inspection and offline verification;
- index, scrub, backup, retention, tier, and lifecycle operations; and
- cluster topology after the cluster management profile is qualified.

The product promise is:

> Every Residiuum guarantee should be visible, every uncertainty should remain
> visible, and no convenience feature may create an authority path that the
> database itself forbids.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 3. Product boundaries

### 3.1 Studio is four clients

Studio keeps four logically independent channels:

```text
Residiuum Studio
    ├── Heap data/control ─── Residiuum qualified RPC + HeapKey
    ├── telemetry ─────────── Ratatouille collector/gateway
    ├── evidence ──────────── Heap-bound AuditRead or offline export
    └── examination ───────── read-only store/package/SDA sources
```

Channels have separate credentials, connection state, errors, retention, and
trust indicators. Failure or compromise of one channel MUST NOT silently
upgrade authority in another.

### 3.2 Studio is not an authority ceremony tool

Studio MUST NOT:

- accept, import, store, display, transmit, paste, or request a Heap master
  private key;
- cycle Heap master authority;
- generate a new master key;
- execute recovery-quorum ceremonies;
- mutate the protected authority store;
- expose an SSH wrapper that accepts master-key material; or
- turn a remote Studio connection into the protected local recovery plane.

When an operation requires a local authority ceremony, Studio displays:

- why it cannot perform the operation;
- the immutable `HeapId`;
- the required local command/runbook name;
- the expected evidence kind; and
- how to refresh the Studio session after completion.

It never asks for the old or new master key.

### 3.3 No global data connection

A live data session is bound to exactly one:

```text
endpoint
DeploymentId
HeapId
AuthorityEpoch
HeapKey certificate fingerprint
holder key
rights/constraints snapshot
collection-scope grants
```

There is no deployment-wide data session, wildcard Heap session, “show all
records” superuser connection, or client-side Heap filter.

Studio may show several Heaps in separate workspaces only by holding separate
valid Heap sessions. It MUST NOT merge their records, query results, caches,
cursors, history, evidence, or exports.

### 3.4 Existing CLI

The interactive `residiuum console` remains a terminal product. Studio does not
replace or embed its line-oriented implementation. Both products reuse the
same SDK/protocol and language contracts.

### 3.5 Threat model

Studio MUST assume that every external input is hostile, including:

- documents, field names, collection names, RQL results, and error text;
- telemetry dimensions, trace identifiers, and diagnostic messages;
- Evidence Ledger entries and offline evidence packages;
- SDA bytes, damaged segments, manifests, and repair reports;
- connection profiles and imported credentials;
- clipboard contents, dragged files, and user-selected paths;
- a remote endpoint impersonating a legitimate deployment;
- a compromised telemetry collector or evidence source; and
- URLs embedded in data or documentation.

The protected assets are:

1. Heap credentials and client private keys;
2. the exact Heap and authority epoch to which a workspace is bound;
3. document and evidence contents;
4. operator intent for destructive actions;
5. local profile metadata; and
6. the integrity of Studio releases.

The v1 trust boundary is:

```text
untrusted database / telemetry / package data
                    |
                    v
        Rust parsers and bounded adapters
                    |
        typed, schema-checked Studio IPC
                    |
                    v
          unprivileged Angular renderer
```

Angular MUST NOT parse private-key material, raw evidence archives, or
unrestricted SDA input directly. Rust bounds input sizes, validates formats,
and returns typed projections.

Studio does not claim to protect secrets from an already-compromised operating
system account, kernel, debugger, or process-memory reader. It MUST still
minimize secret lifetime, prevent routine renderer exposure, redact
diagnostics, and use the operating-system credential vault.

Remote endpoint identity MUST be authenticated before any credential is sent.
Certificate failure, deployment-identity disagreement, Heap mismatch, or
authority-epoch mismatch MUST fail closed. Studio MUST never offer a
“continue anyway” action for these conditions.

## 4. Supported operating modes

### 4.1 Live Heap mode

Primary mode. Studio connects through the qualified Residiuum RPC protocol using a
HeapKey and holder proof.

Permitted features are derived from actual rights, constraints, collection
scope grants, server capability status, and Heap administrative state.

### 4.2 Live telemetry mode

Studio consumes Ratatouille NDJSON through:

1. a local loopback Ratatouille-compatible receiver owned by Studio; or
2. an authenticated collector/gateway tail API qualified for the deployment.

Studio does not scrape Residiuum's legacy `metrics` RPC in a qualified profile.
The local receiver is memory-bounded and writes no telemetry files.

Ratatouille's Rust relay is plain TCP/HTTP. A remote Residiuum server therefore
sends to its protected local sidecar; Studio reaches the collector through an
authenticated encrypted gateway. Studio MUST NOT encourage plain remote relay
traffic.

### 4.3 Offline evidence mode

Studio opens an immutable Residiuum Evidence Ledger export package and invokes the
same Rust verification kernel as `residiuum evidence verify`.

No live Residiuum or credential is required. The UI preserves the independent
verification axes:

```text
framing
ownership
signatures
continuity
material
anchoring
retention
```

### 4.4 Offline examination mode

Studio may examine:

- read-only store copies;
- segments;
- salvage packages;
- backup manifests;
- SDA examination units; and
- damage/coverage reports.

Offline examination uses `residiuum-examine` and `residiuum-format`. It MUST NOT open a
live store as a second writer, repair media implicitly, rebuild catalogs merely
to make the UI look complete, or suppress unsupported/damaged units.

### 4.5 Local development mode

A later qualified development profile MAY launch a disposable loopback Residiuum
server with a newly generated development Heap. It:

- uses an explicit user-selected directory;
- never opens an already served store;
- generates development-only credentials;
- clearly labels the process and durability profile;
- stops through bounded drain; and
- does not expose master authority through Angular.

This mode is not required for Studio v1 initial release.

## 5. Technology architecture

### 5.1 Repository shape

```text
apps/residiuum-studio/
    package.json
    angular.json
    src/
        app/
        styles/
        assets/
    src-tauri/
        Cargo.toml
        tauri.conf.json
        capabilities/
        src/
    schemas/
        ipc/
        fixtures/
    tests/
        e2e/
        security/
```

Recommended Rust crate boundary:

```text
crates/residiuum-studio-core/     # session, query, telemetry, evidence, settings
apps/residiuum-studio/src-tauri/  # Tauri application and IPC adapter
```

`residiuum-studio-core` contains no webview types. The Tauri crate contains no
database semantics.

### 5.2 Dependency direction

```text
Angular UI
    ↓ closed residiuum-studio-ipc-v1
Tauri IPC adapter
    ↓
residiuum-studio-core
    ├── residiuum-sdk / residiuum-client
    ├── residiuum-examine / residiuum-format / residiuum-sda
    ├── residiuum-heap
    └── Evidence and telemetry adapters
```

Angular MUST NOT speak the Residiuum wire protocol, open telemetry sockets, parse
private keys, read evidence packages directly, or access the filesystem
outside closed Tauri commands.

### 5.3 Licensing

Residiuum Studio is an AGPL-3.0-or-later networked product. Pure protocol,
examination, and SDK dependencies retain their existing repository licenses.
No Studio dependency may reverse the repository's permissive → MPL → AGPL
dependency direction.

## 6. Tauri security profile

### 6.1 Webview

Release builds:

- load only packaged local Angular assets;
- allow no remote page navigation;
- use a strict Content Security Policy;
- disable production devtools;
- reject inline script and `eval`;
- allow no arbitrary custom protocol target;
- expose no remote-origin IPC capability;
- use exact Tauri capability manifests; and
- open external documentation links through an explicit, validated command.

The UI never renders untrusted document content as HTML. JSON, strings, error
details, RQL/SDA, and telemetry are text nodes or escaped editor models.

### 6.2 Plugins

No plugin is enabled merely for convenience.

Forbidden by default:

- unrestricted shell/process execution;
- arbitrary filesystem access;
- arbitrary HTTP;
- clipboard read;
- global shortcuts;
- opener with unchecked URLs; and
- remote updater configuration.

Each enabled plugin has:

- exact command allowlist;
- path/URL scope where applicable;
- threat-model entry;
- automated capability test; and
- pinned version.

### 6.3 IPC

The IPC surface is a closed numeric/string registry under
`residiuum-studio-ipc-v1`. There is no generic:

```text
execute
invoke_sdk
run_command
query_raw_server
read_file
write_file
http_request
```

Every command validates:

- schema version;
- session/resource handle;
- maximum encoded size;
- enum values;
- paths through a scoped path type;
- cancellation token;
- deadline; and
- caller window identity.

Unknown fields and commands are rejected.

### 6.4 Large data transfer

Tauri invoke responses are limited to bounded metadata and small pages.
Large results use:

- cursor-paged pulls; or
- bounded Tauri channels carrying typed chunks.

The Rust side owns backpressure. Closing a tab cancels its outstanding work and
releases cursors, buffers, and package handles.

### 6.5 Closed IPC protocol

Studio IPC is a versioned product protocol, not an internal convenience API.

Every request has:

```text
StudioRequest = {
    version: 1,
    request_id: RequestId,
    command: StudioCommand,
    payload: CommandPayload
}
```

Every terminal response is exactly one of:

```text
StudioSuccess = {
    version: 1,
    request_id: RequestId,
    ok: true,
    result: CommandResult
}

StudioFailure = {
    version: 1,
    request_id: RequestId,
    ok: false,
    error: {
        code: StudioErrorCode,
        message: RedactedText,
        retryable: bool,
        state_may_have_changed: bool,
        evidence_ref?: EvidenceRef
    }
}
```

`state_may_have_changed` is mandatory. If a mutating request loses its response
after submission and the core cannot prove non-execution, it is true. The UI
then reconciles state and MUST NOT silently retry.

The closed v1 command registry is:

| Family | Commands |
|---|---|
| profiles | `profile.list`, `profile.get`, `profile.save`, `profile.delete` |
| credentials | `credential.import`, `credential.describe`, `credential.forget` |
| sessions | `session.open`, `session.describe`, `session.refresh`, `session.close` |
| Heap | `heap.summary`, `heap.capabilities`, `heap.health` |
| collections | `collection.list`, `collection.describe`, `collection.stats` |
| records | `record.page.open`, `record.page.next`, `record.page.close`, `record.get`, `record.put`, `record.delete`, `record.history`, `record.damage` |
| RQL | `rql.validate`, `rql.explain`, `rql.run`, `rql.rank`, `rql.cancel` |
| translators | `sqlish.translate`, `jsonschema.translate` |
| SDA | `sda.open`, `sda.inspect`, `sda.evaluate`, `sda.close` |
| rules | `rre.list`, `rre.validate`, `rre.impact`, `rre.activate` |
| contracts | `contract.list`, `contract.describe`, `contract.preview`, `contract.activate` |
| Atomics | `atomic.preview`, `atomic.submit`, `atomic.status`, `atomic.cancel` |
| telemetry | `telemetry.source.open`, `telemetry.source.describe`, `telemetry.batch.next`, `telemetry.dashboard.snapshot`, `telemetry.source.close` |
| evidence | `evidence.page`, `evidence.entry.get`, `evidence.package.open`, `evidence.package.verify.next`, `evidence.package.report`, `evidence.package.close` |
| indexes | `index.list`, `index.preview`, `index.create`, `index.rebuild`, `index.drop` |
| jobs | `job.list`, `job.get`, `job.cancel`, `job.retry` |
| lifecycle | `lifecycle.policy.get`, `lifecycle.policy.preview`, `lifecycle.policy.activate`, `hold.list`, `hold.place`, `hold.release`, `purge.preview`, `purge.submit` |
| cluster | `cluster.summary`, `cluster.nodes`, `cluster.placement`, `cluster.operation.preview`, `cluster.operation.submit` |
| local application | `settings.get`, `settings.update`, `dialog.pick.package`, `external_link.open`, `diagnostics.snapshot` |

This table reserves protocol names; it does not make an unfinished server
operation available. A command is disabled at compile time and absent from the
renderer capability document until its work package and qualification tests
pass.

There are no commands resembling `shell`, `exec`, `eval`, `read_file`,
`write_file`, `http_request`, `invoke_raw`, or `rpc_raw`.

The only asynchronous event names permitted in v1 are:

```text
channel.chunk
channel.end
channel.error
session.changed
job.changed
```

Each event carries a typed opaque handle, sequence number, and bounded payload.
Unknown commands, fields, enum values, and event names are rejected.
Protocol schemas and generated Rust/TypeScript types live under `spec/studio/`;
handwritten duplicate transport types are forbidden.

## 7. Secret and credential handling

### 7.1 Key ownership

HeapKey certificates and holder private keys live only in Rust memory and the
operating-system credential vault when persistence is explicitly requested.

Angular receives:

```text
credential_ref
certificate_fingerprint
HeapId
rights summary
constraint summary
expiry
```

It never receives certificate bytes, holder private-key bytes, TLS exporter,
collector credential, evidence decryption key, or master-key material.

### 7.2 Credential import

Import accepts only supported credential-package formats through a native
file-picker scoped to the selected file. Rust:

1. reads bounded bytes;
2. parses strictly;
3. verifies internal identity and algorithms;
4. obtains holder possession;
5. displays a non-secret summary;
6. asks whether to store in the OS vault; and
7. zeroizes transient secret bytes.

Drag-and-drop credential import is disabled in v1.

### 7.3 Session memory

Secret-bearing values:

- are held in zeroizing types;
- are never serializable into Studio settings;
- do not implement debug display;
- are excluded from panic, telemetry, and support bundles;
- are cleared on disconnect/expiry;
- are not copied to clipboard; and
- never cross IPC.

### 7.4 Clipboard

Copying ordinary displayed data is explicit. Copying sensitive identifiers,
evidence assertions, or document bodies shows a visual privacy indicator.

Studio never reads the clipboard. It does not automatically clear the system
clipboard because that can destroy unrelated user data; instead it warns that
copied material has left Studio's protection.

## 8. Session and workspace model

### 8.1 Handles

Rust issues random opaque handles:

```text
StudioConnectionId
StudioHeapSessionId
StudioQueryId
StudioPageHandle
StudioTelemetrySourceId
StudioEvidencePackageId
StudioJobId
```

Handles are process-local, unguessable, non-persistent, and bound to the
creating window. A handle of one kind cannot be used as another.

### 8.2 Heap workspace

A workspace is immutable in Heap identity:

```text
StudioHeapWorkspace {
    session_id
    endpoint_ref
    deployment_id
    heap_id
    authority_epoch
    certificate_fingerprint
    rights
    constraints
    scope_grants
    server_capabilities
    heap_state
}
```

Authority, policy, or state revision change triggers capability refresh.
Failure closes or degrades affected tabs; Studio never keeps an old capability
alive for visual convenience.

### 8.3 Multiple Heaps

Multiple Heap workspaces may be open. The UI gives each a stable color marker
derived from a local workspace palette, not Heap data.

The following are never shared across Heap workspaces:

- query tabs;
- active collection;
- editor buffers associated with records;
- cursors;
- result caches;
- history;
- evidence pages;
- exports;
- mutation drafts;
- undo state; and
- scope selections.

Copy/paste between workspaces is a user-mediated export/import action and never
an internal cross-Heap operation.

### 8.4 Reconnect

Reconnect performs a fresh challenge and holder proof. It verifies expected
`DeploymentId`, `HeapId`, and `AuthorityEpoch`.

Identity mismatch opens a new connection confirmation; it never silently
retargets an existing workspace.

## 9. Information architecture

### 9.1 Application shell

The desktop shell has:

```text
connection rail
workspace tabs
object navigator
primary editor/result area
context inspector
bottom operational panel
status bar
command palette
```

The shell remains usable at 1280×720 and scales to multiple high-density
monitors.

### 9.2 Primary navigation

Per Heap:

```text
Overview
Collections
Query
Indexes
Rules & Contracts
Evidence
Operations
```

Deployment/operator connections additionally expose:

```text
Telemetry
Instances
Cluster          # only when qualified and authorized
```

Offline packages open in isolated workspaces with no mutation commands.

### 9.3 Persistent identity strip

Every live workspace continuously shows:

- Heap name as a mutable label;
- shortened immutable `HeapId`;
- endpoint/deployment reference;
- authority epoch;
- connection state;
- Heap administrative state;
- scope mode (`bound`, `any`, or not scoped);
- credential expiry; and
- incomplete/degraded indicators.

The name is never shown without the immutable identity nearby.

## 10. Connection manager

The connection manager supports:

- endpoint profiles without embedded secrets;
- credential references;
- expected deployment and Heap identity;
- TLS trust profile;
- connect timeout;
- display label and local color;
- telemetry source association; and
- last successful non-secret capability summary.

It MUST NOT support:

- master-key entry;
- global administrator username/password;
- disabling identity verification;
- accepting invalid TLS certificates in release builds;
- embedding credentials in URLs; or
- storing raw keys in Angular/local storage.

Connection test returns bounded stages:

```text
transport
TLS
protocol
Heap authentication
holder proof
identity binding
capability/rights
readiness
```

It does not expose sensitive internal denial causes.

## 11. Overview workspace

The Heap overview presents:

- immutable identity and current label;
- state and revisions;
- credential rights/constraints/expiry;
- collection/index counts visible to the capability;
- current coverage and tier availability;
- readiness;
- recent permitted operations/jobs;
- Evidence Ledger health;
- associated telemetry health; and
- limitations/capability maturity.

Values are labeled:

```text
live
sampled at <time>
derived
partial
unavailable
unsupported
```

Missing telemetry is never displayed as zero.

## 12. Collection explorer

### 12.1 Navigator

The navigator lists only collections returned inside the current Heap
capability. It supports:

- name and immutable `CollectionId`;
- contract/scope indicator;
- RRE revision;
- index status summary;
- collection state; and
- local text filtering of the already authorized list.

The filter does not query other Heaps or infer hidden names.

### 12.2 Record list

Record lists:

- use opaque continuation cursors;
- request bounded pages;
- virtualize rendered rows;
- show coverage with every page;
- preserve declared order;
- never synthesize offset pagination;
- expose `complete`, `partial`, and `uncertain`; and
- allow immediate cancellation.

Default page size is 100. Configurable range is 10–1000, subject to server
constraints.

### 12.3 Scope

For a scoped collection, the explorer shows the effective selector:

```text
scope bound <local pseudonymous display>
scope any
```

`scope any` is available only when the capability and contract permit the
operation. Studio cannot remove the engine-owned scope predicate.

Creation is disabled in `scope any`; the UI explains that cross-scope sessions
have R/U/D authority but cannot create without a bound scope.

### 12.4 Search

Explorer filters compile to typed RQL or supported SDK filters. Studio shows:

- generated RQL;
- parameters separately from text;
- consistency;
- coverage policy;
- order;
- budget; and
- whether an index/direct-access path is expected.

It never concatenates user values into source.

## 13. Document and bytes editor

### 13.1 Representations

An item can be viewed as:

- structured tree;
- canonical/pretty JSON where valid;
- raw UTF-8 text;
- hexadecimal bytes;
- SDA value;
- envelope/provenance;
- history;
- integrity/coverage; and
- diff against another surviving version.

Invalid UTF-8, malformed JSON, unknown codecs, encrypted-unavailable content,
and partial chunks remain viewable through their surviving representations.

### 13.2 Editing

Editing is explicit and never autosaves.

A mutation draft binds:

```text
HeapId
CollectionId
record identity
before version/content hash
contract revision
RRE revision
scope
representation
```

Save uses compare-version/content preconditions where available. Conflict opens
a three-way comparison; Studio does not overwrite silently.

### 13.3 Validation

Before submission Studio MAY perform local syntax and RRE previews, but labels
them `preview`. Only the server's admitted result is authoritative.

The save result displays:

- committed outcome;
- durability requested/achieved;
- version/event identity in shortened form;
- rule/relationship outcome;
- coverage;
- receipt; and
- linked evidence reference when applicable.

### 13.4 Delete

A routine single-record delete is one explicit command without a modal
confirmation. It displays the exact target and resulting receipt.

Bulk delete, cross-scope mutation, collection retirement, Heap lifecycle,
purge, hold release, and similarly high-impact actions use the confirmation
protocol in §26.

## 14. History and damage

History is a first-class timeline, not a hidden tab.

It shows:

- version/event order;
- commit/durability evidence;
- tombstones;
- surviving material state;
- content/envelope hashes;
- source/repair provenance;
- holes and uncertain intervals; and
- coverage.

A damaged interval is rendered as an explicit object between surviving
islands. The UI MUST NOT draw an unbroken line through it.

The visual language is:

```text
verified complete     solid
verified partial      hatched
missing/hole          open gap
uncertain dependency  dotted
conflicting           red split
unsupported codec     neutral blocked
encrypted unavailable locked
```

Color is never the sole indicator.

## 15. RQL workbench

### 15.1 Editor

The RQL editor provides:

- syntax highlighting from the frozen grammar;
- diagnostics with exact spans;
- parameter editor;
- collection/field completion from authorized metadata only;
- formatting;
- query tabs bound to one Heap;
- cancellation;
- history stored locally only when enabled; and
- snippets containing no credentials or result data by default.

The editor labels the shipped language subset honestly. Unsupported future RQL
syntax is not silently sent through another semantics.

### 15.2 Execution

Execution closes:

```text
Heap
sources
scope
parameters
predicate/projection
order
page size
cursor
consistency
coverage
budget
deadline
```

Results show the canonical plan identity and coverage. Empty under incomplete
coverage is not presented as “no records.”

### 15.3 Paging and direct access

Studio retains opaque cursor bytes only in Rust. Angular receives a page handle
and navigation availability.

Supported controls:

```text
next page
previous locally retained page
go to exact rank
show rank/coverage proof
restart query
```

There is no arbitrary offset control. “Go to row 100001” uses qualified Direct
Access or explains why exact rank is unavailable.

### 15.4 Explain

Explain separates:

- semantic plan;
- physical strategy;
- index/direct-access use;
- work limits;
- order;
- scope;
- consistency;
- expected coverage;
- actual coverage;
- scanned/matched/returned counts; and
- timing by bounded phase when available.

Explain never reveals another Heap's structure or hidden collection names.

## 16. SQL-ish import

Studio offers SQL/SQL-ish+ as an import editor:

```text
SQL-ish input
    ↓ compile
canonical RQL
    ↓ inspect/approve
execute
```

It displays unsupported or lossy constructs explicitly. It never presents the
importer as a SQL engine.

Generated RQL is editable and becomes the executed source of record.

## 17. SDA laboratory

SDA mode supports:

- raw SDA/ENR source;
- examination-unit selection;
- bounded host inputs;
- result tree and canonical representation;
- carrier/absence/`Null` distinctions;
- `Fail` values;
- holes and coverage;
- deterministic re-run; and
- copy/export of the program separately from data.

SDA stays pure. Studio performs I/O, decoding, tier staging, decryption, and
resource limits before evaluation as required by `SDA_PROFILE.md`.

The lab can inspect offline packages without a live Heap.

## 18. Rules, contracts, and relationships

### 18.1 RRE

The RRE editor provides:

- grammar-aware source editing;
- typed AST/normalized form;
- human explanation;
- mathematical predicate rendering;
- example evaluator;
- JSON Schema import preview;
- revision diff;
- activation impact scan;
- violation samples only when authorized; and
- activation receipt/evidence.

Studio does not invent JavaScript callbacks, arbitrary scripts, or
Turing-complete extensions.

### 18.2 Collection contracts

Contract views cover:

- scope mode and field;
- operation classes;
- bound/any grants;
- create restrictions;
- immutable ownership metadata;
- relationship declarations;
- unique constraints;
- revision/history; and
- derived-structure impact.

Cross-scope capability is described as a deliberate application access mode,
not “administrator.”

### 18.3 Relationships

Relationships are shown as a directed graph and exact declarative rules:

```text
source collection/field
target collection/key
required/optional
on delete restrict
cardinality
coordination scope
qualification status
```

The graph is explanatory. The canonical closed rule remains the authority.

## 19. Atomic workbench

Studio visualizes a closed Atomic plan:

```text
AtomicId reference
Heap and coordination scope
read set
predicates
members
RRE/relationship consequences
limits
prepare root
decision
commit position
material coverage
```

It supports:

- plan preview where the server exposes one;
- submission of typed bounded plans;
- outcome resolution after disconnect;
- retry using the same operation identity;
- distinction between `not_committed`, `unknown_commit`, and conflicting
  evidence; and
- SDA examination of surviving Atomic evidence.

Studio never offers interactive lock-holding transactions or cross-Heap
Atomics.

## 20. Telemetry workspace

### 20.1 Source

Telemetry comes exclusively from the `residiuum-telemetry-v1` Ratatouille stream
or a qualified collector view. Studio does not create a parallel metrics/log
protocol.

It validates:

- Ratatouille outer envelope;
- fixed Residiuum topic;
- one string argument;
- Residiuum telemetry message schema;
- source deployment reference;
- boot/sample/topic sequence;
- cardinality;
- maximum message size; and
- clock/counter continuity.

Malformed or unknown messages are counted and isolated, not rendered as
trusted fields.

### 20.2 Live retention

The built-in receiver holds a bounded memory ring:

```text
default messages       20,000
maximum messages       100,000
default decoded bytes  64 MiB
maximum decoded bytes  256 MiB
overflow               discard oldest
```

It writes no telemetry files. Historical telemetry is the collector's
responsibility.

### 20.3 Dashboards

Required dashboard groups:

- overview/readiness;
- throughput and latency;
- admission and errors;
- storage write/read/sync/amplification;
- query work/coverage/direct access;
- indexes and cache;
- damage/scrub/salvage/repair;
- backup/retention/tiering/purge;
- RRE/Atomics;
- Evidence Ledger;
- cluster; and
- telemetry's own drops/queue/connectivity.

Charts:

- show units;
- identify boot/reset boundaries;
- never interpolate across missing samples;
- visualize dropped intervals;
- distinguish zero from unavailable;
- use fixed metric registries;
- bound visible series; and
- allow raw validated message inspection.

### 20.4 Live tail

Live tail is a diagnostic view over transitions and exemplars, not a fake
complete access log.

It prominently states:

```text
best effort
sampled
may contain gaps
not audit evidence
```

Filters run locally over fixed fields. They do not alter Residiuum's producer
filter or request retransmission.

### 20.5 Alerts

Studio evaluates optional local alert rules over validated snapshots. Alert
state is ephemeral unless the external collector persists it.

Alerts never become Evidence Ledger records merely because Studio displayed
them.

## 21. Evidence workspace

### 21.1 Live evidence

Live evidence uses operation 143 (`audit_read`) through a Heap session with
`AuditRead`. The UI calls the feature **Evidence**.

Pages:

- use immutable sequence cursors;
- show retention frontier;
- preserve missing/damaged intervals;
- show material and continuity independently;
- redact assertion/actor fields according to projection policy; and
- never combine Heap ledgers.

### 21.2 Record view

The record inspector shows:

- sequence and Evidence ID reference;
- event kind and obligation;
- outcome;
- actor projection;
- target projection;
- before/after roots;
- operation/commit references;
- time and ordering evidence;
- coverage;
- predecessor continuity;
- signer certificate status;
- checkpoint/anchor coverage; and
- assertion state (`visible`, `redacted`, or `encrypted_unavailable`).

### 21.3 Offline verification

Offline package verification:

- runs entirely in Rust;
- streams bounded results to Angular;
- continues after holes;
- can operate without decryption keys;
- never labels partial verification as valid-complete;
- provides a machine-readable report export; and
- does not mutate the package.

## 22. Index workspace

Studio supports:

- list and inspect;
- create from typed definition;
- impact/space estimate when available;
- build/rebuild progress;
- ready/stale/partial/failed state;
- lag and coverage;
- drop;
- linked RQL usage; and
- receipts/evidence.

Index names and definitions come only from the current Heap. A stale index is
never presented as healthy because queries happened to return results.

## 23. Operations workspace

### 23.1 Jobs

Long operations use durable job identity where supported:

```text
requested
admitted
running
paused/waiting
completed
failed
partial
unknown
```

Closing Studio never implies job cancellation. Reconnect resolves status by
job/operation identity.

### 23.2 Operations

The workspace covers qualified:

- scrub;
- salvage/export;
- backup;
- restore import;
- compaction;
- index maintenance;
- tier move;
- retention status;
- hold placement/release;
- Heap suspend/resume/read-only/active;
- retirement;
- purge;
- data-key status/rotation; and
- recovery examination.

Studio exposes an operation only when:

- the server advertises it active;
- the session has the required right;
- constraints admit the target;
- Heap state permits it; and
- the Studio profile has implemented its exact receipt/outcome model.

The server remains authoritative; disabled UI is not authorization.

### 23.3 Damage-aware operations

Scrub, salvage, repair, restore, and purge views always show:

- declared scope;
- examined/managed coverage;
- unavailable domains;
- surviving/removed units;
- conflicts;
- partial outcomes;
- receipts/evidence; and
- whether the operation changed authoritative material.

## 24. Cluster workspace

Cluster UI is unavailable until the cluster management contract is qualified.

When enabled it shows bounded:

- nodes and roles;
- partitions;
- leader/quorum state;
- placement;
- replica health and lag;
- coverage;
- snapshot/repair/rebalance jobs;
- authority epoch/term context; and
- incomplete or conflicting observations.

The graph does not infer health from network reachability alone. It labels the
source and sample time of every state.

Cross-Heap data inspection remains prohibited even for a cluster operator.

## 25. Rights-driven UI

Studio derives affordances from:

```text
server capability
Heap right
HeapKey constraint
collection scope grant
Heap state
resource/admission policy
Studio implementation maturity
```

Every command declares its required operation ID and right in a machine-readable
registry. UI menus are generated from that registry.

Rules:

- hidden versus disabled is a UX decision, never security;
- unavailable commands explain which non-secret condition is missing;
- an old rights cache cannot authorize a call;
- server denial refreshes capability state;
- unknown/reserved rights are not treated as administrator; and
- Studio never sends an operation through a broader command as fallback.

## 26. High-impact confirmation

A high-impact action uses a closed confirmation object:

```text
operation
HeapId
target IDs
declared coverage
expected state/revision
irreversibility
required right
evidence obligation
operation ID
```

The UI:

1. obtains a server preview when supported;
2. shows immutable IDs, not names alone;
3. requires a deliberate hold-to-confirm or exact target phrase;
4. expires the confirmation after 60 seconds or any revision change;
5. submits the same operation ID and preview root;
6. displays exact committed/partial/unknown outcome; and
7. links the receipt/evidence.

Confirmation is protection against user error, not authorization.

Purge, retirement, hold release, cross-scope bulk mutation, force
reconfiguration, and destructive recovery always use this flow.

## 27. Local settings and persistence

Studio may persist:

- window/layout preferences;
- theme and accessibility settings;
- endpoint display profiles;
- credential references, never credentials;
- telemetry collector profiles without inline secrets;
- saved RQL/SDA snippets when enabled;
- recent non-secret package paths;
- workspace layout; and
- UI feature preferences.

Studio does not persist by default:

- query results;
- documents;
- history bodies;
- telemetry messages;
- evidence assertions;
- decrypted package contents;
- credentials;
- clipboard history; or
- server error detail.

Settings are versioned, bounded, atomically replaced, and stored in the OS
application-data directory with user-only permissions. Sensitive references
are resolved through the OS credential vault.

A “Forget workspace” operation removes settings and credential references
after showing exact scope. It does not delete database data.

## 28. SCSS design system

### 28.1 Goals

The visual system is:

- dense enough for database work;
- calm under high information volume;
- explicit about danger and uncertainty;
- keyboard-first;
- accessible without sacrificing professional density; and
- recognizably Residiuum rather than a generic component-library theme.

### 28.2 Structure

```text
src/styles/
    _tokens.scss
    _themes.scss
    _typography.scss
    _density.scss
    _layout.scss
    _states.scss
    _editor.scss
    _charts.scss
    _utilities.scss
    styles.scss
```

Components consume semantic CSS custom properties generated from SCSS:

```text
--residiuum-surface-*
--residiuum-text-*
--residiuum-border-*
--residiuum-accent-*
--residiuum-state-complete
--residiuum-state-partial
--residiuum-state-uncertain
--residiuum-state-conflicting
--residiuum-state-danger
--residiuum-focus
--residiuum-density-*
```

No component hard-codes semantic state colors.

### 28.3 Themes and density

Required:

- dark;
- light;
- high contrast;
- compact density;
- comfortable density; and
- operating-system reduced-motion support.

Theme/density changes do not reload workspaces.

### 28.4 Accessibility

Studio targets WCAG 2.2 AA for application UI:

- full keyboard navigation;
- visible focus;
- semantic labels/roles;
- screen-reader status announcements;
- non-color state indicators;
- minimum contrast;
- scalable text;
- reduced motion;
- table/grid navigation; and
- accessible chart summaries.

## 29. Angular architecture

### 29.1 Feature domains

```text
app/core/
app/shell/
app/connections/
app/workspaces/
app/explorer/
app/query/
app/sda/
app/rules/
app/atomics/
app/telemetry/
app/evidence/
app/indexes/
app/operations/
app/cluster/
app/shared/
```

Features are lazy loaded where practical.

### 29.2 State

Angular state uses typed services/signals and immutable view models.

Rules:

- Rust session handles are the authority;
- no secret in frontend state;
- no generic global bag;
- Heap identity is part of every Heap-bound view-model type;
- selectors cannot merge workspaces;
- route parameters are not authority;
- data pages are bounded; and
- component destruction cancels subscriptions/work.

### 29.3 Editors and tables

Editor and grid dependencies require explicit selection records covering:

- license;
- bundle size;
- keyboard/accessibility;
- virtualized large-data performance;
- CSP compatibility;
- no remote asset loading; and
- maintenance status.

No editor executes document content.

## 30. Error and uncertainty UX

Studio has independent states:

```text
success complete
success partial
not committed
unknown commit
denied
unavailable
unsupported
damaged
conflicting
cancelled locally
```

It MUST NOT collapse them into generic green/red toast messages.

Errors display:

- stable public code;
- retryability;
- affected channel;
- operation ID reference where safe;
- whether state may have changed;
- next safe action; and
- evidence/status link where available.

Internal causes and arbitrary server strings are not rendered unless received
through a separately authorized diagnostic projection.

## 31. Performance and resource budgets

Studio itself must remain usable while inspecting a database under stress.

Default bounds:

```text
record page                    100
maximum record page            1000
locally retained result pages  5 per query tab
maximum open query tabs        32 per Heap workspace
telemetry live messages        §20.2
Angular rendered table rows    virtualized
IPC message                    1 MiB
stream chunk                   256 KiB
offline package worker memory  configurable, default 512 MiB
```

Targets on reference hardware:

```text
cold app interactive               <= 2.5 s
workspace switch                   <= 100 ms without network
telemetry chart update             <= 250 ms at 10 s snapshots
scroll frame                       no sustained < 50 fps
UI main-thread long task           no routine task > 50 ms
idle CPU with no live telemetry    < 1%
```

Large decode, verification, diff, chart reduction, and package work runs in
Rust/background workers. Angular never receives millions of raw rows merely
to discard them.

## 32. Updates and release security

Release packages:

- are reproducible where the platform permits;
- are code-signed;
- publish checksums and provenance;
- use a signed update manifest;
- never accept invalid update TLS/certificates;
- do not update while a high-impact operation is awaiting confirmation; and
- display installed Studio and Residiuum protocol versions.

Auto-update is opt-in until the updater threat model and rollback policy are
qualified.

The application supports macOS, Windows, and Linux only when each platform
passes the same security/IPC/credential-vault suite.

## 33. Telemetry about Studio

Studio does not silently send product analytics to Residiuum, Frogfish, or any
third party.

An optional future Studio-self telemetry profile requires separate explicit
consent and specification. It cannot reuse connected database telemetry or
document/evidence content.

Local UI diagnostics are bounded in memory and visible to the user. They are
not written as rolling log files.

## 34. Qualification

### 34.1 Functional

1. connect to a qualified Heap and verify identity;
2. open two Heaps and prove workspace/cursor/cache non-interference;
3. browse JSON, bytes, malformed, partial, encrypted-unavailable, and damaged
   records;
4. edit with compare-version conflict handling;
5. execute RQL with parameters, coverage, cursor, cancellation, and explain;
6. import SQL-ish into visible RQL;
7. evaluate SDA over live and offline examination units;
8. inspect RRE/contracts/relationships/Atomics without semantics drift;
9. ingest and render every telemetry topic with gaps/resets/drops;
10. read and offline-verify Evidence Ledger material;
11. operate each implemented job through retry/unknown/partial outcomes; and
12. render all supported rights/state combinations.

### 34.2 Security

1. prove no master-key input or IPC path exists;
2. prove Angular never receives holder or collector secret bytes;
3. IPC unknown command/field/type/size/window attacks;
4. CSP, navigation, remote-origin, custom-protocol, and XSS tests;
5. malicious JSON/HTML/SVG/text/telemetry/evidence/package content;
6. credential import fuzzing and zeroization checks;
7. session-handle guessing, type confusion, reuse, and cross-window theft;
8. cross-Heap cache/result/cursor/export attacks;
9. path traversal, symlink, special-file, and package-bomb attacks;
10. clipboard, screenshot, support-bundle, panic, and crash-report secret tests;
11. TLS/identity mismatch and authority-cycle reconnect;
12. updater signature/rollback/invalid-TLS tests; and
13. dependency/license/vulnerability scanning.

### 34.3 Performance and resilience

1. million-row logical result through cursor/virtualization without loading it;
2. 100,000-message telemetry ring at maximum byte bound;
3. collector disconnect/reconnect and malformed flood;
4. slow/dead Residiuum server with responsive cancellation/UI;
5. large offline evidence package with holes;
6. memory stability across 24-hour telemetry/query session;
7. 32 tabs and multiple Heap workspaces;
8. OS suspend/resume and network transition;
9. forced Rust command panic isolated from secret output; and
10. crash/restart with settings recovery and no persisted result data.

### 34.4 Framework update subset

Every Tauri/Angular/webview/plugin major or security-sensitive update reruns:

- IPC/capability tests;
- CSP/navigation/XSS corpus;
- credential-vault boundary;
- package/updater verification;
- deep-link/path handling;
- e2e connection/query/telemetry/evidence smoke;
- bundle/license scan; and
- startup/memory benchmarks.

## 35. Product releases

### Studio S1 — Explorer

Outcome:

> Connect to one Heap, inspect identity, collections, records, bytes, history,
> damage, and coverage without violating Heap confinement.

### Studio S2 — Workbench

Outcome:

> Write and execute RQL, inspect generated plans/SDA, navigate by cursors and
> exact rank, and edit documents with conflict-safe receipts.

### Studio S3 — Operations

Outcome:

> Observe live Ratatouille telemetry, inspect Evidence Ledger records, and run
> qualified index/scrub/backup/lifecycle operations with honest outcomes.

### Studio S4 — Integrity

Outcome:

> Design and inspect RRE, collection contracts, relationships, and Atomic plans
> through their canonical mathematical contracts.

### Studio S5 — Cluster

Outcome:

> Operate a qualified Residiuum cluster without weakening Heap data isolation.

## 36. Completion definition

Residiuum Studio v1 is complete only when:

- S1 through S4 exit their implementation gates;
- the Tauri/Angular/Rust boundary passes §34;
- every live workspace is immutable in Heap identity;
- master authority is structurally absent;
- data, telemetry, evidence, and examination channels remain separate;
- partial/damaged/unknown outcomes are never prettified into completeness;
- large data is cursor-paged and virtualized;
- no credential or payload crosses into logs, telemetry, settings, or
  untrusted HTML;
- every dangerous operation uses exact rights, preview, confirmation, receipt,
  and evidence semantics;
- Studio works with telemetry disconnected;
- accessibility and platform release gates pass; and
- capability status is published honestly.
