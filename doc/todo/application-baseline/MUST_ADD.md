# Residiuum immediate Must-Add program

Status: **normative execution list v1.0-draft — developer ready**

Program: `APB` — Application Baseline

Date: 2026-07-31

Authority:

- [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md)
- [PRODUCT_DEFICIENCIES.md](../../reference/product/PRODUCT_DEFICIENCIES.md)
- [DX_SPEC.md](../../reference/product/DX_SPEC.md)
- [doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](./CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md)

## 1. Decision

Immediately after:

```text
DEF-098…DEF-104 accepted
→ CSQ-0…CSQ-12 accepted
```

Residiuum executes this Must-Add program.

Its purpose is narrow:

> Close the missing ordinary application APIs before RRE, Atomics, Direct
> Access, search, archive, or cluster expansion becomes the principal product
> program.

This list is intentionally smaller than the full deficiency register.

## 2. Entry and exit

Entry:

- `CSQ-12 = accept`;
- DEF-098–DEF-104 are accepted;
- the evidence bundle verifies; and
- APP-0/APP-1 completed work is reconciled rather than discarded.

Exit:

```text
APB-0…APB-12 = accept
and
residiuum-application-baseline-v1 verifies locally and remotely
```

Until exit, Residiuum may claim qualified core storage but not a complete
ordinary document-database application surface.

## 3. The Must-Add list

| Order | Package | Must add | Primary deficiencies |
|---:|---|---|---|
| 0 | `APB-0` | Freeze the complete application contract and outcome mappings | all baseline |
| 1 | `APB-1` | One Heap-bound backend-neutral client | PD-001 |
| 2 | `APB-2` | Conditional create/replace/delete, add, and upsert | PD-002, PD-004 |
| 3 | `APB-3` | Collection describe/rename/retire/restore/purge and capability discovery | PD-011, PD-023 |
| 4 | `APB-4` | Document-path lookup and atomic single-document mutation | PD-003 |
| 5 | `APB-5` | Bounded streaming bulk mutation with per-item truth | PD-005 |
| 6 | `APB-6` | Stable bounded read views | PD-008 |
| 7 | `APB-7` | RQL Application Core, builder, explain, paging, and remote parity | PD-009 |
| 8 | `APB-8` | Coverage-aware exists/count/distinct/group/numeric aggregates | PD-010 |
| 9 | `APB-9` | Resumable at-least-once change feed | PD-012 |
| 10 | `APB-10` | Streaming resumable import/export | PD-013 |
| 11 | `APB-11` | Official application test kit and shared backend conformance | PD-030 |
| 12 | `APB-12` | Packaged application qualification and compatibility evidence | baseline gate |

DEF-099 and DEF-100 already implement the underlying historical-recovery and
coverage-aware scan contracts before CSQ. `APB-1`, `APB-2`, and `APB-7` must
surface them coherently; they are not reimplemented.

Labor sequence from APB-0 through **query** and **atomics** (including pure risk
lanes and product stages):  
[APB_QUERY_ATOMICS_SEQUENCE.md](./APB_QUERY_ATOMICS_SEQUENCE.md).

## 4. APB-0 — Contract closure

Depends: `CSQ-12`, accepted APP-0/APP-1 evidence

Deliver:

```text
spec/app/baseline-v1/
  operations-v1.json
  outcomes-v1.json
  projections-v1.json
  capabilities-v1.schema.json
  protocol schemas
  canonical fixtures
```

Work:

- amend the existing APP contract rather than create a competing API;
- register every APB operation and stable error;
- define exact local/remote outcome parity;
- bind DEF-099/100 recovery and scan types;
- freeze version, receipt, operation ID, coverage, read-view, change-checkpoint,
  job, and continuation types;
- declare compatibility and deprecation rules for legacy SDK paths; and
- add compile fixtures for the complete public Rust surface.

Exit:

- no APB implementer must invent a public type or semantic choice;
- every reachable lower outcome has a total public projection;
- the wire operation registry contains no unexplained reserved APB operation;
  and
- cross-Heap composition is impossible by construction.

## 5. APB-1 — Unified Heap-bound client

Depends: `APB-0`, qualified collection provisioning

Deliver:

```rust
HeapClient
CollectionClient
IndexManager
History/Recovery clients
embedded and remote adapters
```

Rules:

- one application source changes only its constructor between embedded and
  remote;
- collection handles bind immutable `HeapId` and `CollectionId`;
- no raw wire JSON, tuple result, filesystem path, or caller-supplied Heap ID;
- synchronous v1 remains acceptable; async is not allowed to distort this
  contract; and
- semantic outcomes, receipts, coverage, and errors are identical across
  backends.

Exit: shared compile and behavior suites pass both backends.

## 6. APB-2 — Safe single-key mutations

Depends: `APB-1`

Deliver:

```rust
create(key, value, options)
replace(key, value, if_version, options)
delete_with(key, if_version, if_present, options)
add(value, key_profile, options)
upsert(key, value, options)
```

Rules:

- version test plus mutation is one Key Atomic;
- proven absence is required for `create`;
- incomplete coverage cannot satisfy absence;
- generated keys are returned and profile-labelled;
- `upsert` reports inserted versus replaced;
- stable operation IDs make qualified retries idempotent;
- every receipt reports achieved durability and exact new version; and
- exact historical/current inspection from DEF-099 is available beside these
  mutations.

Exit: concurrent lost-update, crash, retry, damage, and local/remote parity
matrices pass.

## 7. APB-3 — Collection lifecycle and capabilities

Depends: `APB-1`, qualified Heap administration

Deliver:

```rust
describe_collection
rename_collection
retire_collection
restore_retired_collection
plan_purge_collection
purge_collection
heap.capabilities
collection.capabilities
```

Rules:

- rename changes a name binding, never immutable identity;
- stale handles/cursors cannot alias a reused name;
- retirement preserves read/history/examination under explicit policy;
- purge is privileged, planned, evidenced, retention-aware, and high friction;
- capabilities report effective limits and supported semantic profiles; and
- unsupported options reject before effect.

Exit: lifecycle crash/retry, two-Heap isolation, name reuse, backup, cursor,
and capability-negotiation tests pass.

## 8. APB-4 — Document-path operations

Depends: `APB-2`

Deliver:

```rust
lookup(key, [Get(path), Exists(path)], options)
mutate(key, [Set, Remove, Increment, ArrayAppend, Test], options)
```

Rules:

- all paths observe one exact document version;
- all mutations commit or none commit;
- optional `if_version` prevents lost updates;
- arithmetic and path traversal are bounded and checked;
- binary documents reject JSON mutation;
- the future RRE hook validates the proposed final value;
- physical full-document rewrite is initially allowed; client-side
  read/modify/write is not required; and
- receipt reports old/new versions and per-operation results.

Exit: model, concurrency, crash, large-document, path-fuzz, and mutation tests
pass.

## 9. APB-5 — Bounded bulk mutation

Depends: `APB-2`, `APB-4`

Deliver:

```rust
bulk_write(iterator, BulkOptions) -> BulkResultStream
```

Required operations:

```text
create / replace / mutate / delete / add / upsert
```

Every input produces exactly one:

```text
committed(receipt)
rejected(error)
uncertain(recovery_handle)
not_attempted(reason)
```

Rules:

- bounded input, concurrency, memory, queue, and output pages;
- ordered and unordered modes;
- item-level operation IDs;
- no implied multi-key atomicity;
- an optional Atomic scope compiles to the future Atomics engine and rejects
  as unavailable until that profile exists; and
- partial transport failure remains recoverable per item.

Exit: million-item bounded-memory, mixed-failure, response-loss, retry, and
backend-parity campaigns pass.

## 10. APB-6 — Stable read views

Depends: `APB-1`, `APB-3`

Deliver:

```rust
heap.read_view(options) -> ReadView
```

A view binds:

```text
Heap identity
authoritative frontier and coverage
query/rule/index semantic versions
expiry
retention/resource budget
```

Rules:

- queries, pages, counts, exports, and watch bootstrap can share one view;
- mutation does not silently change its observation;
- reclamation is pinned only within declared budget;
- unsupported/expired views fail explicitly; and
- generation-fenced restart cursors remain available for callers that do not
  require a snapshot.

Exit: mutation-between-pages, compaction, tier movement, expiry, resource, and
reopen tests pass.

## 11. APB-7 — RQL Application Core

Depends: `APB-1`, `APB-6`, existing APP-4/APP-5 compiler work

Deliver:

```rust
collection.query()
collection.rql(source, parameters, options)
collection.explain_rql(...)
```

Required:

- canonical predicate/plan;
- projection, scalar ordering, limit, bounded page, and continuation;
- complete-by-default coverage;
- budgets, deadline, and cancellation;
- index-versus-scan correctness;
- authenticated Heap/collection/view/plan/parameter-bound cursors; and
- embedded/remote parity.

Exit: builder and RQL compile to the same plan and all pages reconcile with the
independent complete-scan oracle.

## 12. APB-8 — Aggregate baseline

Depends: `APB-7`

Deliver:

```text
exists
count
distinct
group_count
numeric min/max/sum/average
```

Every result reports coverage, read view, work, bounds, and precision/overflow
policy. Incomplete coverage cannot produce an exact result. Approximation uses
a different result type with an explicit error bound.

Exit: scan/index/partition differential and arithmetic boundary suites pass.

## 13. APB-9 — Change feed

Depends: `APB-2`, `APB-6`

Deliver:

```rust
collection.watch(options) -> ChangeStream
```

First-profile guarantee:

```text
at-least-once
ordered within declared scope
published after declared durability
resumable within retention
gaps and replays explicit
```

Events bind Heap, collection, event/item identity, operation kind, durability
position, optional version references, coverage, and an opaque checkpoint.

Exit: subscribe/bootstrap race, reconnect, duplicate, checkpoint expiry,
compaction, retention, damage, and backend-parity suites pass.

## 14. APB-10 — Import/export

Depends: `APB-3`, `APB-5`, `APB-6`

Deliver:

```text
JSON / JSONL
raw bytes and directory trees
SDA/evidence export
CSV through explicit mapping
```

Rules:

- streaming, bounded, cancellable, and resumable;
- per-item truth and checkpoint;
- explicit rule/read-view/collection binding;
- no implicit skipping, coercion, overwrite, or validation bypass;
- opaque preservation option for unsupported material; and
- local/remote parity where the deployment advertises the format.

Exit: larger-than-RAM, interruption/resume, malformed input, collision policy,
partial coverage, and round-trip journeys pass.

## 15. APB-11 — Application test kit

Depends: begins with `APB-1`; completes after `APB-10`

Deliver:

```text
temporary isolated Heap
deterministic clock/key/operation-ID sources
fault/crash child harness
fixture import
coverage/damage/version assertions
shared embedded/remote behavior runner
```

The kit exposes supported fault scenarios without importing private engine
modules.

Exit: an external consumer crate can test retries, version conflicts, partial
coverage, lock contention, watch replay, and recovery using public packages.

## 16. APB-12 — Application qualification

Depends: every `APB-0` through `APB-11` package and the qualified remote Heap
posture

Deliver:

```text
residiuum verify --profile residiuum-application-baseline-v1 --level A2
```

Mandatory packaged journey:

```text
create Heap and collection
→ conditional and document-path mutations
→ bounded bulk
→ query/page/count under a read view
→ kill/reopen
→ enumerate around a partial document
→ recover an exact historical version
→ resume a watch
→ export/import
→ retire/restore collection
→ verify local/remote semantic parity
```

Exit:

- every APB claim has invariant/oracle/suite evidence;
- no forbidden semantic collapse survives;
- public examples use only the baseline façade;
- compatibility fixtures are frozen;
- capability documentation matches; and
- no unexplained skip, flake, infrastructure failure, or unsupported backend
  branch remains.

## 17. Dependency graph

```text
CSQ-12
  ↓
APB-0
  ↓
APB-1 ───────────────┬──────────────→ APB-11
  ├→ APB-2 → APB-4 → APB-5 ─┐
  ├→ APB-3 ──────────────────┼→ APB-10
  └→ APB-6 → APB-7 → APB-8  │
             └──────→ APB-9  │
all APB-0…11 + remote Heap ──┴→ APB-12
```

`APB-11` grows alongside the implementation. It does not wait until the end.

## 18. Existing-work mapping

| Existing package/work | APB use |
|---|---|
| APP-0 | input fixtures and contract history for APB-0 |
| APP-1 / HAR-1 | collection provisioning dependency |
| APP-2 | implementation core of APB-1 |
| APP-3 | CRUD/history/index parity feeding APB-1/2 |
| APP-4/5 | predicate/plan/compiler feeding APB-7 |
| APP-6/7 | query, cursor, and remote execution feeding APB-7 |
| APP-8 | absorbed into the broader APB-12 journey |
| DEF-099 | recovery API authority used by APB-1/2 |
| DEF-100 | coverage-aware scan authority used by APB-1/7 |

No accepted code or fixture is thrown away. APB closes omissions and produces
one final baseline qualification.

## 19. What is not in the immediate program

The following remain mandatory later, but do not expand APB:

```text
RRE document rules
LocalHeap Atomics
referential integrity
constraint-grade unique indexes
retention/legal hold
unified administrative jobs
async Rust and Node.js
SQL/JSON Schema cross-compilers
text/vector/geospatial
archive and cluster production profiles
```

They resume only after APB-12 unless the master plan explicitly permits pure,
non-surface preparation.

## 20. Non-negotiable result

When this program exits, an application developer no longer needs to invent:

```text
an ORM-like wrapper
CAS conventions
JSON patch races
bulk retry bookkeeping
snapshot conventions
pagination truth
change polling
import/export checkpointing
collection deletion rituals
backend-specific branches
```

That is the exact point at which Residiuum stops being an exceptional storage
engine with a promising API and becomes a complete ordinary document database
ready for its mathematical extensions.