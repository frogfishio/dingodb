# DingoDB Collection Contracts specification

Status: **Normative design v1.0-draft; not yet implemented**

Delivery position: **Unsequenced; this specification does not by itself alter
[NEXT_BUILD_PLAN.md](NEXT_BUILD_PLAN.md)**

Feature name: **Collection Contracts**

First qualified facility: **Collection Scoping**

Profiles:

```text
dingo-collection-contract-v1
dingo-collection-scope-v1
dingo-collection-scope-grant-v1
dingo-collection-contract-evidence-v1
```

Audience: collection, Heap authority, DQL, DRE, SDK, server, Atomics, index,
examination, recovery, and conformance implementers

Normative companions:
[HEAP_SPEC.md](HEAP_SPEC.md),
[DRE_SPEC.md](DRE_SPEC.md),
[DQL_SPEC.md](DQL_SPEC.md),
[DINGO_PREDICATE_SPEC.md](DINGO_PREDICATE_SPEC.md),
[ATOMICS_SPEC.md](ATOMICS_SPEC.md),
[DIRECT_ACCESS_SPEC.md](DIRECT_ACCESS_SPEC.md),
[ORDER_WAVELET_SPEC.md](ORDER_WAVELET_SPEC.md),
[DX_SPEC.md](DX_SPEC.md), and
[SDA_PROFILE.md](SDA_PROFILE.md)

## 1. Decision

DingoDB SHALL support a versioned **Collection Contract** attached to exactly
one collection.

A Collection Contract declares how that collection behaves through every
qualified access surface. It is not merely a document schema and is not an SDK
wrapper.

The governing product statement is:

> Define what a collection is once. DingoDB enforces it everywhere.

The first fully specified contract facility is **Collection Scoping**:

> Make “my records” the default execution domain. Make cross-scope access
> deliberate.

When Collection Scoping is active:

1. every record belongs to exactly one concrete scope;
2. ordinary omitted-scope access defaults to the caller's bound scope;
3. cross-scope read, update, and delete are explicit, independently granted
   operations;
4. cross-scope access is not an administrative role;
5. creation always requires one concrete bound scope;
6. creation from `Any`/`*` scope is structurally impossible;
7. no query, aggregation, join, index, cursor, Atomic, import, or alternate SDK
   path may bypass the effective scope.

## 2. Requirement language

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, MAY, and OPTIONAL are normative.

An implementation may claim only the conformance profiles it passes. The
presence of contract-shaped APIs does not establish conformance.

## 3. Why this exists

Most applications repeatedly implement the same collection behaviour around a
general-purpose database:

- “my records” filtering;
- public or cross-customer reads;
- system-owned identifiers and timestamps;
- lifecycle and recoverable deletion;
- document and transition rules;
- protected fields;
- history policy;
- safe aggregation and query limits.

When those rules live in application wrappers, every CRUD method, bulk path,
transaction path, query dialect, nested lookup, and new service must remember
to reproduce them. The common failure is not a sophisticated attack. It is one
forgotten predicate or one new path beneath the wrapper.

Collection Contracts move the behaviour to the collection boundary. Qualified
execution accepts only a plan that has been compiled under the active contract
revision.

## 4. Non-goals

Collection Contracts are not:

- Heap isolation;
- a replacement for HeapKeys;
- RBAC, users, groups, or mutable roles;
- a claim of physical separation between records in different scopes;
- arbitrary hooks, triggers, callbacks, or stored procedures;
- Turing-complete policy code;
- textual query rewriting performed by an SDK;
- a promise to eliminate all application vulnerabilities;
- protection against compromise of the DingoDB process or a key legitimately
  carrying cross-scope grants;
- a way to weaken DRE, Atomic, retention, damage, or coverage requirements.

Collection Scoping reduces the logical data-exposure surface of ordinary
application paths. It does not inherit the stronger non-crossing claim made
for distinct Heaps.

## 5. Terminology

### 5.1 Collection Contract

An immutable, versioned declarative artifact bound to:

```text
HeapId
CollectionId
ContractId
ContractRevision
source profile
semantic profile
canonical content hash
module identities
activation frontier
```

### 5.2 Scope key

An opaque, non-empty value identifying the one collection scope to which a
record belongs.

A scope key may represent a customer, tenant, account, user, project,
workspace, device, region, batch, publisher, or another application-defined
affiliation. The term does not imply human ownership.

### 5.3 Bound scope

One concrete scope:

```text
Bound(σ)
```

### 5.4 Any scope

The complete declared scope universe:

```text
Any
```

The source spelling MAY be `*`; the semantic name is `Any`.

### 5.5 Cross-scope operation

An ordinary operation deliberately executed with selector `Any`. It is not
called an administrative operation.

### 5.6 Scope grant

A cryptographically authenticated, per-operation maximum scope carried by the
Heap capability.

### 5.7 Scope selector

The scope requested for one operation:

```text
Bound(σ) | Any
```

### 5.8 Coordination scope

The Key, LocalHeap, or Partition serialization domain defined by
`ATOMICS_SPEC.md`. It is unrelated to a Collection Scope except that both bind
the same Heap. Specifications and code MUST NOT use one type for both.

## 6. Contract architecture

A Collection Contract is the composition:

```text
CollectionContract =
    ScopePolicy
  × StateRules
  × TransitionRules
  × SystemFieldPolicy
  × LifecyclePolicy
  × DisclosurePolicy
  × ProtectionPolicy
  × HistoryPolicy
  × QueryEffectPolicy
```

The modules have separate semantics:

| Module | Purpose |
|---|---|
| Scope policy | Which record scopes each operation may address |
| State rules | Valid document states, expressed by DRE |
| Transition rules | Valid before/after transitions, expressed by DRE |
| System fields | Database-generated, application-immutable metadata |
| Lifecycle | Finite states such as live/deleted/archived |
| Disclosure | Which fields may be returned under a capability |
| Protection | Encryption and permitted query/index operations |
| History | Version visibility and retention policy |
| Query effects | Finite admission and resource limits |

`dingo-collection-contract-v1` qualifies the container, identity, activation,
and enforcement seam.

`dingo-collection-scope-v1` qualifies the complete scoping semantics in this
document.

The remaining modules are typed composition slots. Where a normative companion
exists, that companion governs its semantics. A slot without a qualified
profile is design-only and MUST fail activation; it is not advertised merely
because the container can name it.

## 7. Collection-only boundary

A contract belongs to one `(HeapId, CollectionId)` pair:

\[
\Gamma_C =
(\operatorname{HeapId},\operatorname{CollectionId},
 \operatorname{ContractRevision})
\]

It is not a Heap-wide default and cannot silently attach to another collection
with the same name.

Two collections may use byte-identical source while retaining different
artifact identities:

\[
C_1\ne C_2
\Longrightarrow
\operatorname{Artifact}(C_1)\ne\operatorname{Artifact}(C_2)
\]

A collection without an active contract keeps its existing declared behaviour.
There is no implicit deployment-global contract.

Cross-collection operations evaluate the contract of every participating
collection independently.

## 8. Contract source

The initial human surface is declarative and visually compatible with DQL and
DRE:

```text
contract yellow_pages_v1 for listings {
    scope {
        create: bound
        read:   any
        update: bound
        delete: bound
    }

    rules listing_rules_v1
}
```

An illustrative larger contract is:

```text
contract customer_records_v1 for customers {
    scope {
        create: bound
        read:   bound
        update: bound
        delete: bound
    }

    system {
        id:         generated
        created_at: commit_time on create
        updated_at: commit_time on change
    }

    lifecycle {
        states: live, deleted
        default: live
        delete:  live -> deleted
        restore: deleted -> live
        purge:   deleted after 90d
    }

    protect $.identity {
        encryption: heap_field_v1
        query: forbidden
    }

    rules customer_rules_v1

    history {
        retain: 15y
    }

    query {
        allow: read, filter, project, group
        deny: external_write, user_code
        max_output: 1000
    }
}
```

The larger example reserves the intended composition. Each named module still
requires its own qualified profile.

Unknown modules, clauses, enum values, or critical fields fail compilation.
They are never ignored as annotations.

## 9. Canonical contract

Source compiles to a canonical, finite Contract IR.

Compilation returns:

```text
ContractCompilation =
    Verified(artifact, verification_record)
  | Refused(ordered_diagnostics)
```

The artifact contains:

```text
profile
HeapId
CollectionId
ContractId
ContractRevision
source hash
semantic profile versions
canonical module IR
DRE artifact identities
required Atomic scope
required rights-registry version
dependency set
cost bounds
artifact hash
```

Compilation is deterministic:

\[
\operatorname{Compile}(S,C,H)=
\operatorname{Compile}(S,C,H)
\]

for the same canonical source, collection identity, Heap identity, and
compiler/semantic profiles.

The verifier is independent of the parser and source-level normalization. It
recomputes identities, bounds, module compatibility, and forbidden
combinations from the canonical artifact.

Persistent bytes MUST NOT be emitted until the canonical CBOR field labels,
domain separators, limits, and compatibility matrix are frozen.

## 10. Scope key model

### 10.1 Type

The semantic type is:

```rust
pub struct CollectionScopeKey(Vec<u8>);
```

Qualified v1 requires:

- length `1..=256` bytes;
- exact byte equality;
- no implicit trimming, case folding, Unicode normalization, numeric
  conversion, or locale interpretation;
- canonical byte-string encoding;
- a distinct type from HeapId, CollectionId, document key, and Atomic scope.

SDKs MAY provide an explicit UTF-8 constructor. Its UTF-8 bytes are the scope
key; the server performs no further normalization.

### 10.2 Authoritative location

The scope key is authoritative record-envelope metadata, not an
application-writable JSON field.

Every authoritative create, replacement, tombstone, version, recovery
projection, derived index member, and Atomic member carries or inherits the
same scope key.

DQL and SDA expose it as system metadata:

```text
@scope
```

A future contract may expose a read-only payload mirror, but the mirror is not
authoritative and cannot replace envelope metadata.

### 10.3 Uniqueness

For every live or historical record \(d\) in a scoped collection:

\[
\exists!\sigma\in\Omega_C:
\operatorname{ScopeKey}(d)=\sigma
\]

There is no null, absent, empty, wildcard, set-valued, or damaged-but-assumed
scope key.

### 10.4 Immutability

For every ordinary transition preserving record identity:

\[
\operatorname{ScopeKey}(d_{\mathrm{after}})
=
\operatorname{ScopeKey}(d_{\mathrm{before}})
\]

Moving a record between scopes is not an update. V1 provides no ordinary move
operation. A future migration profile may model a move as an explicit
create-new/delete-old Atomic with new identity and evidence.

## 11. Operation-specific scope policy

Scope is not one property applied identically to all CRUD operations.

For a collection contract \(\Gamma\):

\[
P_\Gamma:
\{C,R,U,D\}\rightarrow\{\operatorname{BoundOnly},
\operatorname{CrossScopeAllowed}\}
\]

Creation is always `BoundOnly`; `CrossScopeAllowed` is not in the creation
grammar.

The source:

```text
scope {
    create: bound
    read:   any
    update: bound
    delete: bound
}
```

means:

- new records originate in one concrete scope;
- reads may deliberately span every scope;
- updates default to and may be limited to the bound scope;
- deletes default to and may be limited to the bound scope.

The declared value is a maximum, not a default selector. `read:any` permits an
explicit cross-scope read; it does not cause an omitted selector to become
`Any`.

The words `owner`, `administrator`, `user`, and `role` do not occur in the
semantic model.

## 12. Stateless capability grants

### 12.1 Separation from Heap rights

Heap rights remain coarse:

```text
Read
Write
ReadHistory
...
```

Collection scope grants are critical constraints that only narrow those
rights. They never create a Heap right.

Admission requires both:

```text
required Heap right
AND
effective Collection Scope Grant
AND
active Collection Contract policy
AND
requested selector
```

### 12.2 Grant shape

For one collection:

```text
CollectionScopeGrant {
    collection_id
    bound_scope_key?
    create: deny | bound
    read:   deny | bound | any
    update: deny | bound | any
    delete: deny | bound | any
}
```

Rules:

1. `create:any` has no encoding and is always invalid.
2. Any `bound` entry requires exactly one `bound_scope_key`.
3. `bound_scope_key` does not grant an operation by itself.
4. `read:any` does not imply update or delete.
5. `update:any` does not imply read.
6. `delete:any` does not imply read or update.
7. no grant implies contract administration, Heap administration, or key
   issuance;
8. duplicate collection grants are rejected;
9. grants sort by immutable CollectionId;
10. unknown modes or fields fail closed.

The existing `dingo-heap-v1` constraint registry is frozen and does not
contain this constraint. Remote qualification therefore requires a versioned
HeapKey constraint-registry amendment or successor profile. Implementations
MUST NOT reinterpret an existing v1 constraint kind.

### 12.3 Zero-lookup rule

The signed HeapKey contains every scope grant. Capability construction decodes
it once into resident immutable state.

The ordinary data hot path performs:

```text
memory-only Heap capability check
memory-only collection grant check
memory-only active contract check
```

It performs no user, role, group, ownership, or permission database lookup.

The active contract is collection semantics, not a mutable human authorization
record. Contract activation is versioned and resident.

### 12.4 Operation mapping

The scope operation classes refine, but do not replace, Heap rights:

| Collection operation | Scope class | Minimum Heap right |
|---|---|---|
| create/add/create-if-absent | `C` | `Write` |
| get/find/count/scan/DQL/ordinary inspect | `R` | `Read` |
| history read | `R` | `ReadHistory` |
| replace/update/restore-existing | `U` | `Write` |
| logical delete | `D` | `Write` |
| upsert | `U` and conditional `C` | `Write` |

An operation with several effects requires every applicable class. The
operation registry supplies the coarse Heap right; the canonical operation
plan supplies the C/R/U/D classification.

Backup, export, recovery, contract activation, index administration, Heap
lifecycle, and physical purge retain their separate Heap rights and are not
reclassified as ordinary C/R/U/D. Their public outputs and any restored
ordinary collection remain subject to the rules in this specification.

### 12.5 Contract administration

Contract installation, validation, activation, replacement, and retirement
require a future independent `ContractAdmin` Heap right and a CollectionId
constraint. `ContractAdmin` grants no C/R/U/D, ordinary read, history read,
backup, recovery, or key issuance.

A contract operation also requires every module-specific administration right
named by its companions. For example, attaching or replacing DRE rules
requires both `ContractAdmin` and the qualified DRE rule-administration right.
The union is checked before validation reads or publication.

The existing rights bitmap and operation registry remain frozen until their
versioned amendment allocates this right and its operation IDs. Implementations
MUST NOT overload `HeapAdmin`, `PolicyAdmin`, or `Write`.

Pure local compilation and verification of a source file require no data
right. Validating existing collection contents additionally uses the protected
activation protocol; it does not manufacture an ordinary Read capability.

### 12.6 Activation effect on keys

Once a scope contract becomes active, an ordinary HeapKey without a matching
valid `CollectionScopeGrant` cannot access that collection.

Activation tooling MUST report which presented/tested application-key profiles
would become unusable. It cannot weaken the contract to preserve an old
unscoped key.

Master-key recycling, blacklist/grace handling, expiry, holder proof, and
security-revision invalidation remain governed by `HEAP_SPEC.md`. Cycling Heap
authority makes old collection grants inert with their containing HeapKeys.

### 12.7 Useful capability shapes

Private records:

```text
bound_scope_key = "bob"
C: bound
R: bound
U: bound
D: bound
```

Yellow Pages publisher:

```text
bound_scope_key = "bob"
C: bound
R: any
U: bound
D: bound
```

Public directory reader:

```text
bound_scope_key = absent
C: deny
R: any
U: deny
D: deny
```

Cross-scope maintenance worker:

```text
bound_scope_key = absent
C: deny
R: any
U: any
D: any
```

The last capability may be used by moderation, reporting, cleanup, indexing,
fraud detection, integration, or administration. The capability itself has no
role label.

## 13. Selector resolution

### 13.1 Requested selectors

The request selects:

```text
Bound(σ)
Any
Omitted
```

`Omitted` is resolved before planning:

- if the grant carries a bound scope admitted for the operation, resolve to
  that bound scope;
- otherwise reject with `collection_scope_required`;
- never resolve omission to `Any`.

Cross-scope access is always explicit.

### 13.2 Effective domain

Let:

- \(D_P(op)\) be the domain permitted by the active contract;
- \(D_G(op)\) be the domain granted by the capability;
- \(D_Q(op)\) be the requested selector domain.

The effective domain is:

\[
D_{\mathrm{effective}}(op)
=D_P(op)\cap D_G(op)\cap D_Q(op)
\]

The request is admitted only when its requested domain does not attempt to
widen either the contract or the capability:

\[
D_Q(op)\subseteq D_P(op)
\quad\land\quad
D_Q(op)\subseteq D_G(op)
\]

An empty intersection fails before reading or mutation.

### 13.3 No predicate override

The effective scope predicate is an engine-owned plan node:

\[
\operatorname{InScope}_{D}(d)
\equiv
\operatorname{ScopeKey}(d)\in D
\]

It is not inserted into caller-controlled DQL source and cannot be removed,
negated, shadowed, renamed, projected away, or replaced by the caller.

## 14. CRUD semantics

### 14.1 Create

Creation has the type:

\[
\operatorname{Create}:
\operatorname{Bound}(\sigma)\times Payload
\rightarrow Record_\sigma
\]

There is no:

\[
\operatorname{Create}:
\operatorname{Any}\times Payload
\rightarrow Record
\]

Create admission requires:

```text
Heap Write right
contract create policy = bound
capability create grant = bound
requested/effective selector = Bound(grant.bound_scope_key)
valid payload and DRE rules
```

DingoDB assigns the scope envelope metadata. The payload cannot supply,
override, or infer it.

A HeapKey issuance request containing `create:any` is rejected. An SDK type,
wire decoder, batch decoder, or Atomic plan containing cross-scope creation is
rejected before effects.

### 14.2 Read

For caller predicate \(Q\) and effective domain \(D\):

\[
\operatorname{Read}_{\Gamma,k,D}(Q,S)
=
\{d\in S\mid Q(d)\land\operatorname{InScope}_D(d)\}
\]

This applies to:

- get;
- get bytes/payload;
- find;
- count;
- scan;
- DQL and supported SDA data queries;
- history;
- aggregation;
- Direct Access;
- Order Wavelets;
- text, vector, and geospatial retrieval when implemented.

A get of a known document key outside the effective bound scope returns the
same public hidden-object result as an absent key. Authorized diagnostics MAY
record the distinction.

### 14.3 Update

For every affected record:

\[
\operatorname{InScope}_{D}(d_{\mathrm{before}})
\]

must hold at the Atomic serialization point, and:

\[
\operatorname{ScopeKey}(d_{\mathrm{after}})
=
\operatorname{ScopeKey}(d_{\mathrm{before}})
\]

Predicate update evaluates the effective scope before selecting members.
Cross-scope update requires an explicit `Any` selector, `update:any`, and a
contract permitting it.

### 14.4 Delete

Logical delete applies only to records in the effective domain at the Atomic
serialization point.

The tombstone preserves the authoritative scope key. Restore, when supported,
restores into the same scope and is classified as update.

Cross-scope delete requires an explicit `Any` selector, `delete:any`, a
contract permitting it, and all normal work/member bounds.

Physical purge remains a separate Heap lifecycle operation. It does not create
an ordinary cross-scope CRUD surface.

### 14.5 Put and upsert

`put` is outcome-ambiguous because it may create or update.

On a scoped collection:

- a known-existing conditional replacement is `U`;
- create-if-absent is `C` and requires `Bound`;
- an upsert requires both the update grant over selected existing records and
  a concrete bound create grant for the absent branch;
- `Any` upsert is prohibited;
- the operation determines existence and validates the correct branch at the
  Atomic serialization point;
- it cannot race from an admitted update into an unauthorized create.

SDKs SHOULD prefer explicit `create` and version-conditional `replace`.

## 15. Yellow Pages example

Contract:

```text
contract yellow_pages_v1 for listings {
    scope {
        create: bound
        read:   any
        update: bound
        delete: bound
    }
}
```

Publisher capability:

```text
collection = listings
bound_scope_key = "bob"
C: bound
R: any
U: bound
D: bound
```

Bob may:

```text
create a listing in scope "bob"
read listings across every scope deliberately
update listings in scope "bob"
delete listings in scope "bob"
```

Bob may not:

```text
create a listing in scope "alice"
create while using selector Any
update or delete Alice's listing
change a listing's scope
```

A public search component may hold only:

```text
R: any
```

This is ordinary application work. No administrative identity or mutable role
is involved.

## 16. DQL surface

### 16.1 Default bound query

When a capability contains a bound scope:

```text
from listings
where category = $category
```

defaults to:

```text
scope bound
```

The bound value is supplied by the authenticated capability, not by a string
literal in query source.

### 16.2 Explicit cross-scope query

```text
from listings
scope any
where category = $category
```

requires `read:any` and a contract permitting cross-scope read.

### 16.3 Explicit bound query

The source spelling:

```text
scope bound
```

means the capability's bound scope. V1 DQL does not permit an arbitrary scope
literal to replace the authenticated bound value.

### 16.4 Parameters and plan identity

Scope selection is not an ordinary query parameter.

The canonical plan identity binds:

```text
HeapId
CollectionId
ContractRevision
scope profile
selector kind
bound ScopeKey hash when bound
operation class
authority generation
ordinary DQL plan and parameter hashes
coverage/read view
```

Prepared plans, cursors, selection artifacts, caches, and result reuse cannot
cross any of those identities.

## 17. Rust SDK surface

The preferred API uses distinct handle types:

```rust
let mine = listings.bound_scope();
let directory = listings.cross_scope()?;

mine.create(posting).await?;
mine.replace(id, version, changes).await?;
mine.delete(id).await?;

let plumbers = directory
    .query("category = $category")
    .bind("category", "plumber")
    .run()
    .await?;
```

Conceptually:

```rust
pub struct BoundScope;
pub struct CrossScope;

pub struct ScopedCollection<S> {
    collection: CollectionHandle,
    scope: S,
}

impl ScopedCollection<BoundScope> {
    pub async fn create<T>(&self, value: T) -> Result<CreateReceipt>;
}

impl<S> ScopedCollection<S> {
    pub async fn query(&self, source: &str) -> Result<QueryBuilder<S>>;
}
```

`ScopedCollection<CrossScope>` has no `create` method.

Update and delete methods appear only when the capability grant and contract
shape are known statically; otherwise the SDK performs local early rejection
and the server remains authoritative.

The raw unscoped collection handle cannot execute ordinary data operations
once a scope contract is active. It can only derive an admitted scoped handle
or perform separately authorized contract administration.

## 18. Policy closure

### 18.1 Rule

Every logical source introduced by an operation is evaluated under the active
contract of that source collection.

For sources \(C_1,\ldots,C_n\):

\[
\operatorname{Plan}
=
\operatorname{Compile}
\left(
Q,
\bigwedge_{i=1}^{n}\Gamma_{C_i},
k
\right)
\]

No source inherits, drops, or guesses another source's scope.

### 18.2 Nested and aggregate execution

Scope enforcement applies recursively to:

- subqueries;
- joins/lookups;
- unions;
- graph traversal;
- facets;
- correlated queries;
- computed aggregates;
- materialized selection artifacts.

An engine MUST NOT attempt Mongo-style textual insertion into a finite list of
known pipeline stages. It compiles each logical source to a contract-bound
plan node. Unknown source-producing operators are refused.

### 18.3 Cross-collection binding

If two collections use semantically related scope keys, v1 may bind both to
the caller's authenticated bound key only when both contracts declare the same
scope profile and the capability contains matching grants.

Otherwise each source requires an explicit admitted selector. Equal bytes do
not imply equal meaning across collections.

### 18.4 Derived structures

Every derived structure binds the contract revision and scope domain:

```text
HeapId
CollectionId
ContractRevision
scope profile
ScopeKey or Any domain identity
read view/frontier
coverage
content hash
```

This includes indexes, predicate bitmaps, rank maps, Order Wavelets, caches,
prepared queries, cursors, and selection artifacts.

## 19. DRE composition

DRE answers:

> Is this document state or transition valid?

The Collection Contract answers:

> Under which collection semantics may this operation observe or change it?

For a proposed transition:

\[
\operatorname{CommitAllowed}
\iff
\operatorname{HeapAuthorized}
\land
\operatorname{ScopeAdmitted}
\land
\operatorname{DREValid}
\land
\operatorname{AtomicValid}
\land
\operatorname{LifecycleValid}
\]

DRE rules cannot read mutable user/role state. They may read the canonical
operation scope and authenticated capability claims only through explicitly
typed, finite contract inputs.

A DRE rule cannot widen scope. A scope clause cannot waive a DRE violation.

## 20. Atomics composition

Collection scope and Atomic coordination scope are orthogonal.

Every Atomic member binds:

```text
HeapId
CollectionId
ContractRevision
CRUD operation class
effective scope selector
record ScopeKey
```

Before prepare, the planner proves every member is in the admitted effective
domain.

At the serialization point it revalidates:

- active contract revision;
- capability and scope grant;
- existence-dependent C/U branch;
- current record ScopeKey;
- DRE and lifecycle rules;
- all ordinary Atomic witnesses.

A LocalHeap Atomic MAY update or delete records from several Collection Scopes
only when every member is admitted by an explicit cross-scope grant.

It cannot use cross-scope selection to create a member. Created members always
name one bound ScopeKey admitted by a bound create grant.

## 21. Direct Access and ordering

The scope predicate participates in exact membership.

For caller predicate \(Q\) and effective domain \(D\):

\[
M_{\mathrm{effective}}(d)
=
M_Q(d)\land M_{\mathrm{scope},D}(d)
\]

Direct Access may claim exact rank only when the scope bitmap/domain, ordinary
predicate membership, order, view, and coverage are exact.

A bound cursor cannot continue as `Any`; an `Any` cursor cannot continue as
bound. Cursor authentication binds the scope domain and ContractRevision.

Damage to scope membership evidence causes explicit incomplete/refused
coverage. DingoDB never treats unknown scope as matching the caller.

## 22. Contract activation and replacement

### 22.1 Empty collection

A contract may activate immediately on an empty collection after artifact
verification and durable publication.

### 22.2 Existing collection

Activating Collection Scoping on a non-empty collection requires an explicit
total migration mapping:

\[
\operatorname{Assign}:
\operatorname{RecordId}\rightarrow\operatorname{ScopeKey}
\]

Activation performs:

```text
barrier
capture complete frontier
validate complete collection coverage
assign exactly one scope to every existing record
replay concurrent changes under prospective contract
serialize final validation
publish ContractRevision
```

It refuses:

- incomplete coverage;
- unassigned records;
- duplicate/ambiguous assignments;
- empty or oversized scope keys;
- records changed without a prospective scope;
- damaged authoritative data whose scope cannot be established.

There is no inferred owner, default wildcard, filename-derived scope, or
best-effort activation.

### 22.3 Replacement

Contract replacement creates a new immutable revision.

The new revision is validated against current live state at an explicit
frontier. Writes crossing the activation barrier are evaluated under exactly
one revision and record which one.

Replacement cannot make existing scope keys mutable or reinterpret their
bytes. A change of scope-key semantics requires an explicit migration.

### 22.4 Retirement

Old contract artifacts and verification evidence remain available while
referenced by:

- retained history;
- Atomic evidence;
- live cursors/read views;
- backups;
- recovery or audit retention.

They are not executable for new operations after retirement.

## 23. Other contract modules

### 23.1 System fields

The contract may declare system-owned values such as:

```text
generated record ID
created commit position/time
updated commit position/time
origin ScopeKey
```

Application writes cannot set or replace system-owned fields. Values depending
on commit state are produced inside the Atomic transition, not by an SDK clock.

### 23.2 Lifecycle

Lifecycle is a finite state machine. Recoverable deletion is one profile, not a
magic `_deleted` predicate:

```text
live -> deleted -> purged
deleted -> live
```

The default readable state is part of the contract and applies through policy
closure. Tombstones preserve ScopeKey.

### 23.3 Disclosure and protection

Protected paths are declared by exact canonical paths, never name prefixes.

The protection profile determines:

- encryption and key domain;
- authenticated associated data;
- plaintext disclosure;
- permitted predicates;
- permitted indexes;
- sort/aggregation eligibility;
- rotation, backup, restore, and damage behaviour.

Until a protection profile is qualified, a contract naming it cannot activate.

### 23.4 History

History policy declares retention and visibility. Scope enforcement applies to
history reads using the record's historical immutable ScopeKey.

History retention cannot exceed legal/Heap policy ceilings and cannot weaken a
hold.

### 23.5 Query effects

A finite effect system may classify:

```text
read
filter
project
join
group
sort
materialize
external_write
user_code
```

Admission requires:

\[
\operatorname{Effects}(Q)
\subseteq
\operatorname{AllowedEffects}(\Gamma,k)
\]

Limits on stages, output, work, memory, or duration are explicit bounds. There
is no arbitrary callback.

### 23.6 Ingest modes

Unknown-field behaviour is explicit:

```text
reject
preserve
```

A future `strip` transform must return a receipt naming every removed path.
Silent stripping is not part of v1.

## 24. Damage and coverage

Collection Contracts obey DingoDB's governing recovery rule:

> What is gone is gone. What remains still lives.

For scoped ordinary execution:

- a record with an unreadable or absent authoritative ScopeKey is not assigned
  to the caller;
- complete-domain reads become incomplete/refused when missing evidence could
  have contained matching records;
- survivors mode, when explicitly requested and supported, includes only
  records with verified surviving scope evidence;
- update/delete never acts on a record whose scope membership is unknown;
- contract artifact damage prevents new ordinary execution under that
  revision;
- SDA remains able to report surviving records, contract fragments, scope
  evidence, and holes.

Salvage may recover a record with unknown scope as examinable evidence. It
cannot silently publish it into an ordinary scoped collection.

## 25. Backup, restore, import, and recovery

Backups include:

```text
contract source/artifact/revision
activation and verification evidence
scope key envelope metadata
scope grant profile requirements, excluding secret key material
derived-structure rebuild metadata
```

Restore preserves scope keys when restoring the same logical collection
identity under the applicable restore profile.

Import into a different collection evaluates every imported record under the
destination contract. It requires explicit scope assignment and cannot carry
source authority or capability grants.

Recovery and maintenance paths may examine bytes outside ordinary CRUD scope
under their separate Heap rights. They do not create an uncontracted ordinary
data handle.

## 26. Examination and evidence

SDA SHALL expose, subject to examination authority:

```text
contract identity and revision
canonical artifact hash
module profiles and status
activation frontier
verification record
scope coverage
per-record surviving ScopeKey
scope-key damage/absence
derived artifact bindings
Atomic contract/scope evidence
contract holes and uncertainty
```

Ordinary query receipts SHOULD expose:

```text
ContractRevision
selector kind: bound | any
hashed bound ScopeKey when disclosure is inappropriate
coverage
read view/frontier
cross_scope: true | false
```

Secret key material and unauthorized raw ScopeKeys never appear in diagnostics.

## 27. Exposure-surface claim

Let \(P\) be all ordinary application access paths and:

\[
X=\{p\in P\mid p\text{ explicitly requests an admitted Any selector}\}
\]

Without database-owned confinement, any path that forgets its application
predicate may become cross-scope:

\[
\operatorname{PotentialCrossScopeSurface}=P
\]

With qualified Collection Scoping:

\[
\operatorname{CrossScopeSurface}=X
\]

If ten percent of application paths deliberately use `Any`, the design has
reduced the logical cross-scope exposure surface by approximately ninety
percent. That percentage is an application measurement, not a universal
database benchmark.

The permitted claim is:

> Code cannot access another collection scope merely because it forgot a
> filter. Cross-scope access must be both cryptographically granted and
> explicit in the operation.

The prohibited claims include:

- “scopes are physically isolated like Heaps”;
- “Collection Scoping makes an application secure”;
- “no side channel exists between scopes”;
- “Any-scope keys are harmless”;
- “scope replaces authorization.”

## 28. Logical confinement theorem

For a qualified bound operation with effective scope \(\sigma\), every returned
or mutated ordinary record \(d\) satisfies:

\[
\operatorname{ScopeKey}(d)=\sigma
\]

For a query:

\[
\forall d\in
\operatorname{Read}_{\Gamma,k,\{\sigma\}}(Q,S),
\quad
\operatorname{ScopeKey}(d)=\sigma
\]

For update/delete:

\[
\forall d\in\operatorname{Affected},
\quad
\operatorname{ScopeKey}(d)=\sigma
\]

This theorem assumes:

- the Heap capability is valid;
- the contract compiler/verifier and enforcement kernel conform;
- authoritative scope evidence is readable;
- no unqualified legacy/raw bypass is in the claimed profile.

It does not assert physical noninterference, identical timing, identical
resource contention, or completeness under undeclared damage.

## 29. Stable errors

At minimum:

```text
collection_contract_missing
collection_contract_invalid
collection_contract_revision_stale
collection_contract_profile_unsupported
collection_contract_activation_incomplete
collection_contract_migration_required

collection_scope_required
collection_scope_key_invalid
collection_scope_key_missing
collection_scope_key_immutable
collection_scope_grant_missing
collection_scope_bound_mismatch
collection_scope_cross_denied
collection_scope_create_requires_bound
collection_scope_create_any_invalid
collection_scope_hidden
collection_scope_coverage_incomplete
collection_scope_profile_unsupported
```

Public hidden-object responses do not reveal whether a record exists outside
the effective scope. Authorized examination may expose the cause.

## 30. Bounds

Qualified v1 freezes at least:

```text
maximum canonical contract source bytes
maximum Contract IR bytes
maximum modules per contract
maximum DRE artifacts per contract
maximum scope key bytes = 256
maximum collection scope grants per HeapKey
maximum contract revisions retained without external policy
maximum activation scan/replay work per attempt
maximum query effect entries and numeric limits
maximum diagnostics and violation records
```

Every decoder validates bounds before allocation. Every activation and
cross-scope bulk mutation has an explicit work/member budget.

## 31. Conformance

### CC-0 — Contract identity

- artifact binds HeapId and CollectionId;
- canonical source/IR/hash fixtures pass;
- foreign collection/Heap substitution fails;
- unknown critical fields fail;
- activation revision is immutable and examinable.

### CS-1 — Scope algebra

- exhaustive `deny/bound/any` intersections;
- omission never resolves to Any;
- `create:any` is unrepresentable and rejected in hostile encodings;
- bound mismatches fail;
- rights and scope grants remain independent.

### CS-2 — CRUD

- create assigns exactly one scope;
- application cannot write or change ScopeKey;
- read/update/delete touch only effective scope;
- Any R/U/D works only when both contract and grant permit it;
- get outside bound scope does not disclose existence;
- put/upsert race cannot create through an update grant.

### CS-3 — Policy closure

- DQL, count, scan, history, aggregation, nested sources, and every qualified
  SDK path preserve scope;
- an unknown source-producing operator is refused;
- prepared plan, cache, cursor, DDA, and DOW artifacts reject scope mismatch;
- raw/legacy paths are absent from the qualified binary or explicitly outside
  the claim.

### CS-4 — Atomics and concurrency

- concurrent create/update/delete matches the semantic oracle;
- scope is revalidated at serialization;
- cross-scope bulk work cannot introduce create members;
- crash recovery preserves member ScopeKeys and ContractRevision;
- retries cannot change scope or operation class.

### CS-5 — Activation and damage

- empty activation;
- complete migration of existing records;
- concurrent activation replay;
- missing/ambiguous assignment refusal;
- damaged scope/contract evidence;
- backup/restore/import/salvage;
- survivors never masquerade as complete.

### CS-6 — Two-scope adversarial suite

For scopes `A` and `B`, generate arbitrary documents and operations. Under a
bound `A` capability:

- no returned record has scope `B`;
- no `B` version or tombstone changes;
- `B` existence does not alter ordinary object-found/object-hidden shape;
- injected predicates cannot remove the scope node;
- nested queries cannot reach `B`;
- indexes/cursors from Any or `B` are rejected.

### CS-7 — Yellow Pages journey

```text
create scoped listings collection
activate yellow_pages_v1
issue Bob: C bound / R any / U bound / D bound
issue Alice: C bound / R any / U bound / D bound
Bob and Alice create listings
Bob deliberately reads both through Any
Bob cannot update/delete Alice
Bob cannot create through Any
read-only Any service reads both
rotate Heap authority and prove old grants inert
examine contract, scopes, and evidence through SDA
```

## 32. Initial implementation ownership

Recommended vertical slice:

```text
crates/dingo-contract/
  source.rs
  ast.rs
  canonical.rs
  artifact.rs
  verify.rs
  scope.rs
  grant.rs
  algebra.rs
  limits.rs
  oracle.rs

crates/dingo-heap/
  collection_scope_constraint.rs

crates/dingo-store/
  contract_catalog.rs
  scoped_record.rs
  contract_activation.rs

crates/dingo-sdk/
  contract.rs
  scoped_collection.rs

crates/dingo-server/
  contract.rs
  scope_admission.rs

crates/dingo-examine/
  contract.rs
```

The pure contract crate contains no filesystem, clock, network, callbacks,
global state, or store dependency.

The storage adapter owns the authoritative enforcement seam. SDK typing is
early error prevention, not the security or correctness boundary.

## 33. Implementation sequence

### CCT-0 — Semantic oracle and profiles

- canonical contract identity;
- scope algebra;
- CRUD semantic oracle;
- limits and hostile fixtures;
- Heap constraint-profile amendment design.

### CCT-1 — Scoped record envelope

- authoritative ScopeKey metadata;
- create/read/update/delete integration;
- immutable scope law;
- history/tombstone propagation;
- damage states.

### CCT-2 — Capability grants

- signed per-collection C/R/U/D scope grants;
- local decode and intersection;
- issuance rejection for `create:any`;
- security-revision and authority-cycle behavior.

### CCT-3 — Query policy closure

- DQL scope node;
- scans/counts/history;
- aggregate/nested source propagation;
- index/cache/cursor bindings;
- complete/survivors behavior.

### CCT-4 — Activation and recovery

- empty activation;
- existing-record migration;
- concurrent barrier/replay;
- backup/restore/import/salvage;
- SDA examination.

### CCT-5 — Public DX and qualification

- Rust typestate handles;
- CLI contract compile/verify/activate/status;
- Yellow Pages journey;
- two-scope adversarial suite;
- capability status and documentation.

Other contract modules follow their companion specifications and receive
separate conformance rows.

## 34. Final invariant

The collection invariant is:

> Every record originates in exactly one concrete collection scope and remains
> in that scope. Each operation independently declares which scopes it may
> address. Bound access is the default; cross-scope access is explicit,
> cryptographically granted, and never a source of new records.

The engineering invariant is:

> No qualified operation reaches collection data except through the active,
> verified Collection Contract.
