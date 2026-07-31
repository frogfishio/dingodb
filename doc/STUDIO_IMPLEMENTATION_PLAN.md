# Residuum Studio implementation plan

Status: developer-ready work-package plan v1.0-draft

Date: 2026-07-30

Normative product contract:
[STUDIO_SPEC.md](../STUDIO_SPEC.md)

Companions:
[MASTER_DELIVERY_PLAN.md](../MASTER_DELIVERY_PLAN.md),
[PRODUCT_DELIVERY_ROADMAP.md](../PRODUCT_DELIVERY_ROADMAP.md),
[TELEMETRY_SPEC.md](../TELEMETRY_SPEC.md),
[EVIDENCE_LEDGER_SPEC.md](../EVIDENCE_LEDGER_SPEC.md),
[HEAP_SPEC.md](../HEAP_SPEC.md),
[RQL_SPEC.md](../RQL_SPEC.md), and
[DX_SPEC.md](../DX_SPEC.md).

## 1. Delivery rule

Studio is delivered as vertical, security-qualified slices.

Do not begin with a complete visual shell full of mocked database screens.
Each package exits with one real end-to-end path through:

```text
Angular view
Tauri IPC
Rust Studio core
ResiduumDB/telemetry/evidence source
honest result state
```

No package may add a generic IPC command, expose credentials to Angular, open a
wildcard Heap session, or use the legacy metrics/logging paths as its final
backend.

## 2. Repository and ownership

Planned paths:

```text
apps/dingo-studio/
crates/residuum-studio-core/
spec/studio/
scripts/verify-studio.sh
```

Ownership:

| Area | Path |
|---|---|
| Product/security contract | `STUDIO_SPEC.md` |
| Work packages/status | `doc/STUDIO_IMPLEMENTATION_PLAN.md` |
| Machine IPC/topic/view schemas | `spec/studio/` |
| Framework shell | `apps/dingo-studio/` |
| Reusable Rust orchestration | `crates/residuum-studio-core/` |
| ResiduumDB protocol semantics | existing legacy `dingo-*` crates/specs |
| CI qualification | `scripts/verify-studio.sh` |

## 3. Frozen baseline

Initial lock:

```text
Tauri          2.11.5
Angular        22.x exact patch in lockfile
TypeScript     Angular-22-supported exact version
Dart Sass      exact supported version in lockfile
Rust           workspace rust-version/toolchain
Node           one Angular-22-supported LTS line
package tool   repository-selected one, lockfile required
```

No floating dependency ranges in release artifacts. TEL/STUDIO security
updates follow `STUDIO_SPEC.md` §34.4.

## 4. Package map

| ID | Package | Release |
|---|---|---|
| DST-000 | contract, schemas, fixtures, skeleton | foundation |
| DST-001 | Tauri security shell | foundation |
| DST-002 | Rust core and IPC transport | foundation |
| DST-003 | credential vault and connection manager | S1 |
| DST-004 | immutable Heap workspace | S1 |
| DST-005 | collection and record explorer | S1 |
| DST-006 | document/bytes/history/damage inspector | S1 |
| DST-007 | RQL editor and execution | S2 |
| DST-008 | cursor, direct-rank, explain, SQL import, SDA lab | S2 |
| DST-009 | Ratatouille ingestion and telemetry dashboard | S3 |
| DST-010 | Evidence Ledger live/offline workspace | S3 |
| DST-011 | index and operational jobs | S3 |
| DST-012 | lifecycle and high-impact confirmation | S3 |
| DST-013 | RRE, contracts, relationships, Atomics | S4 |
| DST-014 | cluster workspace | S5 |
| DST-015 | packaging, update, accessibility, qualification | release |

## 5. DST-000 — Contract and skeleton

### Entry

- `STUDIO_SPEC.md` accepted as the product boundary.
- Framework baseline selected.

### Deliverables

- `apps/dingo-studio` Tauri + Angular + SCSS skeleton;
- `crates/residuum-studio-core`;
- `spec/studio/ipc-v1.json`;
- `spec/studio/commands-v1.json`;
- `spec/studio/errors-v1.json`;
- complete encoding of the closed command and event registry from
  `STUDIO_SPEC.md`;
- shared Rust/TypeScript fixture generator;
- locked dependency manifests;
- license notices;
- `scripts/verify-studio.sh`; and
- CI jobs for Rust, Angular, Tauri, schema drift, and security lint.

### Required decisions

- exact Node/package-manager version;
- supported initial OS matrix;
- code-signing development/release separation;
- editor/grid/chart dependency selections; and
- OS credential-vault adapter interface.

### Exit

- packaged empty shell launches on every initial platform;
- no remote network request;
- no unrestricted Tauri capability;
- Rust and TypeScript round-trip all IPC golden fixtures;
- a renderer command is exposed only when it is registered and its owning work
  package enables it;
- unknown fields/commands reject; and
- dependency/license scan passes.

## 6. DST-001 — Tauri security shell

### Deliverables

- strict CSP;
- local-only asset protocol;
- navigation blocker;
- external-link validator;
- exact capability manifests;
- release devtools policy;
- per-window identity;
- panic boundary with redacted error;
- scoped native file picker; and
- capability regression tests.

### Tests

- remote navigation;
- `javascript:`/`data:`/malformed URLs;
- iframe and injected HTML;
- remote-origin IPC;
- custom protocol traversal;
- command invocation from wrong window;
- arbitrary path/open/shell attempts; and
- CSP snapshot.

### Exit

Security suite proves Angular can invoke only the closed DST-000 command
registry from the packaged local origin.

## 7. DST-002 — Rust core and IPC

### Deliverables

- opaque typed handle registry;
- cancellation/deadline registry;
- bounded channel abstraction;
- error/outcome vocabulary;
- task ownership by window/workspace;
- graceful app shutdown;
- generated TypeScript view models;
- no-secret `Debug`/serialization tests; and
- in-memory fake sources for e2e.

### Tests

- handle guessing/type confusion/reuse;
- cross-window and cross-workspace access;
- command size/depth limits;
- cancellation races;
- closed-window cleanup;
- Rust panic/error conversion;
- chunk backpressure; and
- 24-hour handle churn model.

### Exit

One fake bounded collection page streams Rust → Tauri → Angular and cancels
without leaked handles or tasks.

## 8. DST-003 — Credentials and connection manager

### Depends

DST-001, DST-002, qualified `connect_heap`.

### Deliverables

- OS vault abstraction;
- credential import/parser;
- zeroizing secret holder;
- endpoint/TLS profile;
- connection-stage diagnostics;
- expected identity pinning;
- fresh holder proof on reconnect;
- credential expiry display;
- “forget connection” flow; and
- explicit absence of master-key commands/types.

### Tests

- malformed/oversize credential packages;
- wrong Heap/deployment/epoch;
- invalid TLS;
- expired/cycled/blacklisted credential;
- possession failure;
- vault locked/unavailable;
- sentinel secret corpus through IPC/settings/panic;
- memory cleanup; and
- source scan proving no master-key input command.

### Exit

Studio connects to one Heap and Angular receives only the non-secret session
summary.

## 9. DST-004 — Immutable Heap workspace

### Deliverables

- workspace shell/identity strip;
- Heap-capability model;
- rights/constraint/scope matrix;
- capability-driven navigation;
- revision refresh;
- multi-workspace isolation;
- reconnect/identity-change handling;
- status bar; and
- command palette filtered by capability.

### Tests

- two Heaps with equal collection names/keys;
- stale capability after cycle/policy/state change;
- workspace route tampering;
- cross-Heap handle/cursor/cache attempts;
- suspended/read-only/retired states; and
- unknown rights/operations.

### Exit

Two simultaneously open Heaps cannot exchange any internal handle or result,
and every screen continuously displays immutable Heap identity.

## 10. DST-005 — Collection/record explorer

### Depends

DST-004 and active list/open/get/scan/find operations.

### Deliverables

- collection navigator;
- cursor-paged record table;
- virtualization;
- coverage/status strip;
- bound/any scope visualization;
- local authorized-list filtering;
- typed search builder;
- generated RQL preview;
- page cancellation; and
- bounded page cache.

### Tests

- 1,000,000 logical rows without full materialization;
- complete/partial/empty-incomplete pages;
- cursor invalidation;
- no offset emulation;
- scoped collection bound/any/create rules;
- unknown/malformed record key representations; and
- server resource-limit outcomes.

### Exit

One Heap can browse collections and arbitrarily large logical result sets with
bounded memory and honest coverage.

## 11. DST-006 — Inspector/editor/history/damage

### Deliverables

- JSON/tree/text/hex/SDA representations;
- malformed/opaque/encrypted-unavailable views;
- edit draft identity;
- compare-version save;
- conflict diff;
- delete receipt;
- history timeline;
- holes/partial/conflicting visual grammar;
- envelope/provenance view; and
- accessible non-color state indicators.

### Tests

- JSON, bytes, invalid UTF-8, large chunked, partial and unknown codec;
- optimistic conflict;
- RRE rejection;
- durability mismatch;
- tombstone/history holes;
- no HTML/script execution from values;
- huge scalar truncation/expand; and
- single delete versus bulk-action boundary.

### Exit

S1 is eligible to close when DST-003–006 and their combined e2e suite pass.

## 12. DST-007 — RQL workbench

### Deliverables

- grammar-derived editor support;
- subset/version indicator;
- parameter editor;
- completion from authorized metadata;
- execution/cancellation;
- result table/tree/raw views;
- consistency/coverage/budget controls;
- saved snippets without results;
- query error/unknown state; and
- canonical plan summary.

### Tests

- parameter injection attempts;
- RQL subset mismatch;
- missing versus null;
- cancellation/deadline;
- budget exhaustion;
- partial coverage;
- empty incomplete result;
- scope confinement;
- saved-snippet secret scan; and
- large result paging.

### Exit

Canonical RQL executes without values concatenated into source and always
displays coverage.

## 13. DST-008 — Advanced query and SDA

### Deliverables

- opaque cursor navigation;
- previous locally retained page;
- Direct Access exact-rank control;
- explain;
- SQL-ish → RQL import/diff;
- raw SDA laboratory;
- live/offline examination-unit picker;
- deterministic rerun; and
- query/plan/SDA export without credentials.

### Tests

- cursor/filter/scope/revision mismatch;
- exact-rank available/unavailable/damaged;
- no loop-to-offset fallback;
- lossy SQL import warning;
- SDA `None`/`Null`/`Fail`;
- evaluator limits;
- offline holes/coverage; and
- generated RQL approval/execution identity.

### Exit

S2 closes when DST-007–008 pass and the workbench remains responsive under the
maximum page/tab budget.

## 14. DST-009 — Ratatouille telemetry

### Depends

`TELEMETRY_SPEC.md` TEL-0 through required server sources.

### Deliverables

- loopback TCP/HTTP-compatible Ratatouille receiver;
- qualified collector/gateway adapter;
- outer envelope and ResiduumDB message validator;
- bounded memory ring;
- source/boot/sequence/gap tracking;
- topic store;
- dashboard reducers;
- required charts;
- live transition/exemplar tail;
- telemetry self-health; and
- no telemetry persistence.

### Tests

- every golden topic/message;
- malformed NDJSON/JSON and oversize input;
- unknown topic/field;
- wrong source;
- gap/reset/clock rollback/counter decrease;
- 100,000 messages / 256 MiB bound;
- overflow and UI drop indication;
- disconnect/reconnect;
- secret sentinel rendering; and
- plain remote transport warning/block.

### Exit

Studio displays current telemetry and explicit gaps without using stdout logs,
files, or the ResiduumDB metrics RPC.

## 15. DST-010 — Evidence

### Depends

DEL read/export/verifier packages.

### Deliverables

- live `AuditRead` paging;
- sequence/gap timeline;
- record inspector;
- signer/checkpoint/anchor state;
- retention cuts;
- redacted/encrypted assertions;
- immutable package opener;
- streaming offline verifier;
- verification report export; and
- linkage from operation receipts.

### Tests

- wrong Heap `AuditRead`;
- damaged middle with healthy suffix;
- fork/truncation;
- signer rotation and missing authority proof;
- retention cuts;
- encrypted unavailable;
- package traversal/bomb/symlink/special files;
- no package mutation; and
- no false valid-complete.

### Exit

Evidence results match the CLI verifier golden corpus exactly.

## 16. DST-011 — Indexes and jobs

### Deliverables

- index list/create/drop/rebuild;
- state/lag/coverage/progress;
- job identity/status/reconnect;
- scrub, backup, salvage/export, compaction, and tier jobs as their server
  operations qualify;
- receipt/evidence links; and
- unknown/partial outcomes.

### Tests

- stale/partial/failed index;
- job survives Studio closure;
- same operation retry;
- progress unavailable versus zero;
- permission/state/constraint denial;
- partial backup/scrub/salvage; and
- cancellation semantics per job.

### Exit

Every shipped job has an exact durable outcome model; no progress bar is
fabricated from elapsed time.

## 17. DST-012 — Lifecycle and confirmation

### Deliverables

- read-only/active/suspend/resume;
- hold place/release;
- retire/purge;
- data-key status/rotate;
- operation preview;
- high-impact confirmation object;
- revision/expiry invalidation;
- unknown-commit resolution;
- purge coverage and tombstone result; and
- local ceremony guidance with no master input.

### Tests

- stale preview;
- wrong Heap/name reuse;
- confirmation expiry;
- incomplete purge;
- hold-blocked action;
- disconnected-after-submit;
- double submit/idempotency;
- master-key sentinel/source scan; and
- evidence unavailable/fail-closed.

### Exit

S3 closes when DST-009–012 pass for every operation Studio advertises.

## 18. DST-013 — Mathematical integrity UX

### Deliverables

- RRE editor/AST/math view;
- example evaluator;
- JSON Schema import;
- activation impact;
- collection-contract editor;
- scope grant visualizer;
- relationship graph/rules;
- Atomic plan and outcome viewer; and
- canonical source/receipt/evidence linkage.

### Tests

- editor/runtime semantic golden corpus;
- no script/Turing-complete escape;
- RRE revision conflict;
- activation with invalid existing data;
- scope `Any` cannot create;
- relationship restrict;
- Atomic unknown/conflicting/material-partial; and
- canonical hashes match server/tooling.

### Exit

S4 closes only when every visual representation round-trips to the canonical
machine contract or is labeled explanatory/non-authoritative.

## 19. DST-014 — Cluster

### Entry

Qualified cluster management operations and telemetry exist.

### Deliverables

- topology;
- node/partition/leader/quorum;
- replica lag/coverage;
- placement;
- repair/rebalance/snapshot jobs;
- transition timeline; and
- bounded topology telemetry association.

### Tests

- split/partition/stale leader;
- identity reuse;
- missing/offline/partial observation;
- no reachability-as-health inference;
- multi-Heap data non-interference; and
- operation permission/evidence.

### Exit

S5 closes only with the cluster's own qualification gate.

## 20. DST-015 — Release qualification

### Deliverables

- macOS/Windows/Linux packages for platforms passing qualification;
- code signing/notarization;
- SBOM;
- checksums/provenance;
- signed update manifest or updater disabled;
- accessibility report;
- performance/memory report;
- security test report;
- capability matrix/docs; and
- release runbook.

### Gates

- `STUDIO_SPEC.md` §34 passes;
- no critical/high unresolved security finding;
- no master-key path;
- no secret reaches Angular/settings/diagnostics;
- S1–S4 e2e passes;
- cold start/resource targets pass;
- dependency/license scan passes; and
- published claims match actual packages.

## 21. CI structure

Required jobs:

```text
studio-schema
studio-rust
studio-angular
studio-tauri
studio-ipc-security
studio-e2e-fake
studio-e2e-dingo
studio-telemetry-corpus
studio-evidence-corpus
studio-accessibility
studio-package-<platform>
studio-dependency-audit
studio-performance
```

Pull requests touching framework capability, IPC, credentials, navigation,
package opening, updater, or external links automatically run the security
subset.

## 22. Work order

Critical path:

```text
DST-000
   ├── DST-001
   └── DST-002
          ↓
       DST-003
          ↓
       DST-004
          ↓
       DST-005 → DST-006          S1
          ↓
       DST-007 → DST-008          S2
          ├── DST-009
          ├── DST-010
          └── DST-011 → DST-012   S3
                         ↓
                      DST-013     S4
                         ↓
                      DST-014     S5

DST-015 qualifies each release.
```

DST-009 and DST-010 may proceed in parallel after DST-004 when their server
contracts are ready. They do not block Explorer S1.

## 23. Immediate handoff

The first development ticket is **DST-000**.

It may create only:

- the locked Tauri/Angular/SCSS skeleton;
- the Rust core crate;
- machine-readable IPC/command/error registries;
- fixture generation;
- security-minimal capabilities;
- verification script; and
- CI.

It MUST NOT create fake production CRUD, a generic Tauri command, master-key
input, unrestricted filesystem/network/shell plugins, or a broad UI component
catalog before one vertical connection slice exists.
