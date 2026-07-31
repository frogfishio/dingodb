# Residiuum documentation website specification

Status: **delivered specification v1.0; retained as implementation record**
Target: `https://docs.residiuumdb.org`
Companion and shared contract: [WEBSITE_SPEC.md](./WEBSITE_SPEC.md)
Source repository: `https://github.com/frogfishio/dingodb`

## 1. Decision

`docs.residiuumdb.org` is the canonical user, operator, reference, and
specification website for Residiuum.

Its primary job is successful task completion. Its secondary job is to make
the project’s unusual guarantees and maturity independently inspectable.

The documentation MUST provide two clearly separated experiences:

1. **Learn and operate** — concise, task-oriented documentation for people
   using Residiuum;
2. **Specifications** — normative and exploratory source documents for people
   implementing, reviewing, or researching Residiuum.

A new user MUST NOT need to learn frames, segments, Raft, or SDA internals to
store and query a local document. A reviewer MUST still be able to reach the
formal source and evidence for every important guarantee.

## 2. Requirement language and inherited rules

The terms MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

The documentation site inherits these sections of `WEBSITE_SPEC.md` without
modification:

- §5, Shared truth and status contract;
- §9, Visual and verbal system;
- §11.4–11.5, domains and security headers;
- §12, accessibility;
- §13, SEO and sharing;
- §14, analytics and privacy;
- the shared component semantics for status and claims.

Where this specification imposes a stricter documentation requirement, this
specification wins.

## 3. Audience and required journeys

### 3.1 Application developer

Must be able to:

1. determine whether Residiuum is appropriate;
2. install the released Rust SDK;
3. open a local store;
4. create a collection;
5. put/get/delete JSON and bytes;
6. filter and page results;
7. inspect history;
8. understand durability receipts and errors;
9. back up, inspect, and recover the store.

### 3.2 Operator

Must be able to:

1. identify the supported deployment profiles;
2. configure and validate a process;
3. understand acknowledgement and durability modes;
4. back up and restore;
5. scrub, diagnose, and salvage;
6. read health, metrics, and structured logs;
7. migrate formats safely;
8. find network, cluster, and cloud-tier limitations before deployment.

### 3.3 Evaluator or architect

Must be able to:

1. understand authority versus derived state;
2. understand independent survival and explicit holes;
3. inspect consistency, coverage, and acknowledgement semantics;
4. compare current features with intended designs;
5. reach normative specifications and conformance evidence.

### 3.4 Implementer or researcher

Must be able to:

1. browse stable, linkable specification pages;
2. distinguish normative, draft, exploratory, and historical documents;
3. render formal notation and diagrams correctly;
4. find companion specifications and implementation evidence;
5. link to a precise heading and source revision.

## 4. Documentation principles

1. **Task first.** User guides lead with the outcome and the smallest working
   path.
2. **Truth at point of use.** Experimental and design-only material is marked
   on the page and beside relevant links.
3. **One source.** Repository Markdown, code, and structured metadata are the
   source of truth; the site does not maintain silent copies.
4. **Evidence is reachable.** Guarantees link to specifications, tests, or
   disclosure artifacts.
5. **Damage is explicit.** Examples never turn partial or unknown results into
   empty complete success.
6. **Versions are visible.** Readers can tell whether a page applies to a
   release, the development branch, or a design.
7. **Ordinary paths stay ordinary.** Formal internals enrich the docs but do
   not overwhelm the first-use path.

## 5. Information architecture

### 5.1 Global navigation

Top navigation:

```text
Residiuum Docs | Learn | Guides | Operations | Reference | Specifications |
Status | [Version] | Search | GitHub
```

`Residiuum Docs` returns to the docs home. A separate wordmark link or footer
link returns to `residiuumdb.org`.

### 5.2 Canonical route tree

```text
/
├── getting-started/
│   ├── choose-residiuum/
│   ├── rust/
│   ├── cli/
│   ├── first-collection/
│   └── next-steps/
├── guides/
│   ├── collections/
│   ├── json-and-bytes/
│   ├── filtering/
│   ├── rql/
│   ├── raw-sda/
│   ├── indexes/
│   ├── pagination-and-ordering/
│   ├── history/
│   ├── large-payloads/
│   ├── backup-and-restore/
│   ├── scrub-and-salvage/
│   └── remote-development/
├── concepts/
│   ├── data-model/
│   ├── authoritative-and-derived/
│   ├── damage-holes-and-coverage/
│   ├── durability-and-receipts/
│   ├── heaps/
│   ├── sda/
│   ├── rql/
│   ├── rre/
│   └── atomics/
├── operations/
│   ├── deployment-profiles/
│   ├── configuration/
│   ├── durability/
│   ├── backup-restore/
│   ├── scrub/
│   ├── salvage/
│   ├── migration/
│   ├── logs/
│   ├── health-and-metrics/
│   ├── security/
│   ├── single-node-server/
│   └── clustering/
├── reference/
│   ├── rust-sdk/
│   ├── cli/
│   ├── configuration/
│   ├── errors/
│   ├── receipts-and-evidence/
│   ├── rql/
│   ├── sda/
│   ├── wire/
│   └── compatibility/
├── specifications/
│   ├── index/
│   └── <stable-specification-slug>/
├── status/
│   ├── capabilities/
│   ├── known-limitations/
│   ├── compatibility/
│   ├── benchmark-disclosure/
│   └── roadmap/
├── contributing/
│   ├── documentation/
│   ├── development/
│   └── conformance/
└── versions/
    └── <release>/
```

The exact number of guide pages MAY change. The top-level groups, separation of
guides from specifications, and stable route purposes MUST remain.

### 5.3 Documentation home

The home page MUST include, in order:

1. a global experimental/early-access notice;
2. “Start here” cards for Rust embedded and CLI;
3. an “I need to…” task grid:
   - store JSON or bytes;
   - find and page documents;
   - understand durability;
   - back up or restore;
   - inspect damage and salvage;
   - evaluate remote or cluster deployment;
4. current release and documentation version;
5. current deployment-profile summary;
6. links to concepts and normative specifications;
7. a conspicuous known-limitations link.

The docs home MUST NOT begin with a full architecture essay.

## 6. Content classes and page contract

### 6.1 Content classes

| Class | Purpose | Style |
|---|---|---|
| Tutorial | Guided learning from a clean state | Linear and complete |
| How-to guide | Complete a specific real task | Outcome-first steps |
| Concept | Explain a model or design decision | Explanatory |
| Reference | Exact API, command, syntax, config, or behavior | Factual and exhaustive |
| Operation | Safely run, diagnose, recover, or migrate | Preconditions, verification, rollback |
| Specification | Normative or exploratory source contract | Source-faithful |
| Status | Capability, maturity, compatibility, or limitation truth | Generated/evidence-linked |

Every page MUST declare exactly one class.

### 6.2 Required frontmatter

Every indexed page MUST validate this schema:

```yaml
title: Open a local store
description: Create and reopen an embedded Residiuum store from Rust.
class: tutorial
status: experimental
applies_to:
  product: "0.2"
  surface: embedded-single-node
source:
  path: crates/residiuum-sdk/README.md
  revision: generated
last_verified: 2026-07-30
owners:
  - sdk
keywords:
  - rust
  - embedded
claim_ids: []
```

Rules:

- `status` MUST use the vocabulary in `WEBSITE_SPEC.md` §5.2, with additional
  specification document states described in §9 below;
- `last_verified` older than the configured freshness interval MUST generate a
  warning and MUST fail release for getting-started, operations, security, and
  reference pages;
- `source.path` MUST resolve in the pinned repository revision;
- `applies_to.surface` MUST be a known deployment profile;
- generated API pages MAY use a specialized schema;
- design-only pages MUST NOT declare an available product surface.

### 6.3 Page chrome

Every content page MUST show:

- breadcrumb;
- title and description;
- status badge;
- applies-to version and surface;
- last verified date;
- left navigation;
- on-page table of contents where useful;
- previous/next links within its section;
- “Edit this page” source link;
- “Report a docs issue” link prefilled with page and version;
- source revision link;
- related pages.

On mobile, navigation becomes a drawer and the on-page table of contents
becomes a collapsible “On this page” control.

## 7. Getting-started contract

### 7.1 Rust quickstart

The canonical quickstart MUST start from an empty directory and include:

1. prerequisites and supported Rust version;
2. project creation;
3. `residiuum-sdk` dependency;
4. complete compilable source;
5. first run and expected output;
6. second run proving reopen/persistence;
7. one filter query;
8. where the store is created;
9. cleanup;
10. next links for durability, backup, and recovery.

The snippet MUST be extracted from or exercised by a repository test. CI MUST
compile it against the displayed crate version and MSRV.

It MUST state:

- embedded single-node maturity;
- default durability semantics;
- effective licensing of the dependency path;
- that enabling cluster changes the dependency/license graph.

### 7.2 CLI quickstart

The CLI quickstart MUST:

- install a versioned release through a supported method;
- show exact version output;
- create/open a temporary store;
- put and get data;
- run doctor/inspection;
- clean up safely.

It MUST NOT tell readers to pipe an unauthenticated network installer into a
shell unless the project deliberately publishes and documents that mechanism.

### 7.3 Executable examples

Every executable example MUST declare one of:

- `tested`: run in CI;
- `compiled`: compiled but not executed;
- `illustrative`: deliberately incomplete.

Getting-started examples MUST be `tested`. Copy controls MUST copy only usable
code, not prompts or diff markers.

## 8. Guides, reference, and operations

### 8.1 Query documentation

The site MUST distinguish:

- Rust `Filter` and query builder;
- RQL human query language;
- SQL/Mongo/JSON dialect input where supported;
- raw SDA examination;
- multi-collection enrichment/join surfaces;
- design-only DDA and Order Wavelet behavior.

Documentation MUST NOT imply that the RQL v1 design is fully implemented when
the current user guide describes a smaller surface.

Pagination and ordering documentation MUST explain:

- current cursor/page APIs;
- deterministic order requirements;
- cursor binding and invalidation;
- filters and frozen read-view implications;
- partial coverage;
- current continuation-token integrity limitations, including DEF-097 where
  applicable;
- DDA and Order Wavelets as design until implementation evidence changes.

### 8.2 Damage and recovery documentation

The recovery journey MUST keep these terms distinct:

- backup;
- restore;
- replication;
- scrub;
- repair;
- salvage;
- export;
- hole;
- partial result;
- unknown commit;
- coverage.

Every destructive demonstration MUST operate only on a newly created temporary
store and MUST identify the exact target before changing it. A copy/paste
example MUST NOT risk a broad home or workspace path.

### 8.3 Operations page template

Every operator procedure MUST include:

```text
Outcome
Applies to / maturity
Risk level
Prerequisites
Pre-flight checks
Procedure
Verification
Rollback or recovery
Failure meanings
Related evidence
```

If an operation has no rollback, the page MUST say so before the procedure.

### 8.4 Reference generation

Reference pages SHOULD be generated where possible:

- Rust API from rustdoc;
- CLI command and flag tables from binary help metadata;
- configuration keys from the configuration schema;
- stable error codes from source definitions;
- profile/version constants from code;
- RQL/SDA grammar from their authoritative grammar or conformance source.

Hand-written explanation MAY surround generated reference but MUST NOT silently
duplicate generated tables.

Rustdoc SHOULD be published at:

```text
/api/rust/<crate-version>/
```

It SHOULD share navigation and design tokens, or open with a clear route back
to the reference index.

## 9. Specification publishing

### 9.1 Source fidelity

Root and `doc/` specifications remain authoritative repository files. The docs
site MUST render from those sources or from a mechanically traceable
transformation. It MUST NOT maintain editorial copies.

Each rendered specification MUST show:

- original repository path;
- source revision;
- original status line;
- rendered document state;
- companions;
- table of contents;
- link to raw source;
- link to implementation/capability evidence where available.

### 9.2 Specification states

Document state is separate from product capability status:

| Document state | Meaning |
|---|---|
| **Normative** | Defines a conformance contract for its stated version |
| **Draft** | Intended to become normative; still changeable |
| **Exploratory** | Records a proposal or research direction |
| **Historical** | Retained for reference; not current |

A page MAY therefore say:

```text
Document: Normative design v1.0-draft
Product capability: Design — not implemented
```

The UI MUST show both axes and MUST never convert “normative design” into
“available.”

### 9.3 Stable specification URLs

Specification URLs MUST use stable semantic slugs, for example:

```text
/specifications/heaps/
/specifications/rre/
/specifications/atomics/
/specifications/direct-access/
/specifications/order-wavelets/
```

Renamed source files MUST redirect old public URLs. Heading anchors SHOULD
remain stable; breaking anchors require a redirect/alias map.

### 9.4 Mathematics and diagrams

The build MUST support:

- server-rendered KaTeX (preferred) for mathematical notation;
- fenced Mermaid or repository diagram sources rendered to safe static SVG;
- syntax highlighting for RQL, RRE, SDA, Rust, JSON, TOML, and shell;
- accessible text descriptions for diagrams;
- horizontal overflow for large formulas and tables without page overflow.

Client-side arbitrary script execution from Markdown is prohibited.

## 10. Repository content mapping

The launch migration MUST use a reviewed manifest. Initial mapping:

| Repository source | Docs destination |
|---|---|
| `README.md` | Docs home summary and getting-started entry points |
| `crates/residiuum-sdk/README.md` | Rust SDK quickstart/reference source |
| `DX_SPEC.md` | Concepts plus specification |
| `doc/RQL/USER_GUIDE.md` | `/guides/rql/` |
| RQL/SDA grammar and manuals | `/reference/rql/`, `/reference/sda/`, related guides |
| `doc/wip/status/CAPABILITY_MATRIX.md` | `/status/capabilities/` |
| `doc/reference/operations/BENCHMARK_DISCLOSURE.md` | `/status/benchmark-disclosure/` |
| backup/scrub/migration/runbook documents | `/operations/` |
| root `*_SPEC.md` files | `/specifications/` |
| root `*_PROPOSAL.md` files | `/specifications/` with Exploratory state |
| `FUTURE_ROADMAP.md` and prime-time strategy | Curated `/status/roadmap/` |
| crate API documentation | `/api/rust/<version>/` |

Internal defect-program prose MUST not be dumped into the primary user
navigation. Material user-visible limitations MUST be summarized on
`/status/known-limitations/` and link to public source evidence where
appropriate.

The migration manifest MUST record:

```yaml
source:
destination:
mode: render | transform | generate | summarize
owner:
status:
```

`transform` and `summarize` modes require a freshness check against the source.

## 11. Versioning and release documentation

### 11.1 URL policy

`https://docs.residiuumdb.org/` serves documentation for the latest published
release line.

- `/next/` serves documentation built from the main development branch.
- `/versions/<major.minor>/` preserves the final documentation for a published
  minor line.
- Patch releases MAY share a minor-line documentation set if the API and
  behavior are unchanged.
- Specification semantic URLs remain unversioned but expose version/state
  history; frozen normative revisions MAY additionally live under a versioned
  archive.

The version selector MUST distinguish:

```text
0.2 (current release)
Next (unreleased)
Archived versions
```

### 11.2 Canonical and search behavior

- current release pages are canonical;
- `/next/` pages MUST display an “unreleased” banner and SHOULD be `noindex`;
- archived pages are indexable only if they clearly display their old version;
- search defaults to the version currently being viewed;
- cross-version results MUST show their version.

### 11.3 Version truth

Release, SDK API, wire, protocol, cluster, and conformance labels MUST be
generated from code or structured capability metadata where possible. The UI
MUST not collapse those different labels into one “version.”

## 12. Search

The default implementation SHOULD use a static, local index such as Pagefind.
It MUST not require a hosted search provider.

Search requirements:

- open with `/` when focus is not in an input;
- open with `Cmd/Ctrl+K`;
- keyboard navigation and screen-reader announcements;
- index title, description, headings, body, keywords, status, surface, and
  version;
- filters for current release, design/specification, operations, and API;
- show status and version on every result;
- exclude navigation chrome, duplicated source blocks, preview content, and
  non-canonical copies;
- no search query may be transmitted externally by default.

Zero-result pages MUST offer terminology, status, and GitHub issue links.

## 13. Design and components

The docs site MUST use the shared Residiuum colors, typography, status vocabulary,
header primitives, footer, and code style from `WEBSITE_SPEC.md`.

Documentation-specific components:

- `DocsHeader`;
- `SidebarTree`;
- `TableOfContents`;
- `VersionSelector`;
- `SearchDialog`;
- `PageMetadata`;
- `MaturityBanner`;
- `SpecState`;
- `SurfaceBadge`;
- `CodeGroup`;
- `TestedCode`;
- `ExpectedOutput`;
- `Procedure`;
- `RiskCallout`;
- `CoverageResult`;
- `MathBlock`;
- `Diagram`;
- `SourceLink`;
- `RelatedPages`.

At desktop width, use a left navigation column, readable center column, and
optional right table of contents. The center text measure SHOULD remain under
80 characters. Large reference tables MAY use a wider layout.

Status banners MUST be compact but impossible to miss. Persistent warnings
MUST not consume so much vertical space that the documentation becomes
unusable.

## 14. Technical implementation

### 14.1 Recommended stack

The reference implementation is:

- Astro Starlight or an equivalent Astro static documentation system;
- TypeScript;
- shared packages with the main website;
- Markdown/MDX with a strict component allowlist;
- Pagefind for local static search;
- Shiki for build-time highlighting;
- KaTeX for build-time mathematics;
- build-time Mermaid-to-SVG rendering;
- generated rustdoc for Rust API reference.

The site SHOULD use the same monorepo and preview/release pipeline as the main
site. A different system is acceptable only if all requirements are retained.

### 14.2 Performance budgets

Measured at the 75th percentile on mobile:

| Metric | Budget |
|---|---|
| LCP | ≤ 2.0 s |
| INP | ≤ 200 ms |
| CLS | ≤ 0.05 |
| Initial documentation JavaScript | ≤ 150 KB compressed |
| Initial CSS | ≤ 50 KB compressed |

Search code and index MUST load on first search intent, not on every initial
page load. Mathematics, diagrams, and highlighting SHOULD be rendered at build
time.

### 14.3 Content security

The documentation renderer MUST:

- reject unsafe raw HTML by default;
- allow only reviewed MDX components;
- sanitize generated SVG;
- prohibit inline event handlers and arbitrary scripts;
- escape code/output;
- scan committed examples for likely credentials and secrets;
- prevent user-controlled query strings from becoming unsanitized HTML.

## 15. Documentation CI

Every pull request MUST run:

1. static site build;
2. frontmatter/schema validation;
3. capability and claim-status validation;
4. source-path and pinned-revision validation;
5. internal link, anchor, image, and redirect checks;
6. external link reporting with controlled failure policy;
7. tested example execution;
8. compile-only example validation;
9. CLI output/reference drift checks;
10. config/error/profile generation drift checks;
11. spell/style check with project terminology;
12. duplicate canonical content detection;
13. accessibility automation;
14. HTML validation;
15. Lighthouse performance budgets;
16. desktop and mobile screenshot regression;
17. search index smoke tests;
18. secret scanning.

### 15.1 Documentation-specific lint rules

CI MUST fail when:

- a Design page uses present-tense availability claims;
- an Experimental page lacks a limitation link;
- an operations page lacks verification;
- a destructive operation lacks risk and target safeguards;
- a current-release page references an unreleased API without a badge;
- a code block marked `tested` is not registered in the test harness;
- an unknown status, surface, version, RQL/RRE/SDA language tag, or claim ID is
  used;
- “transaction” is used for Atomics without explaining the semantic boundary;
- “database” is used for Heap where the product terminology requires Heap;
- a generated reference is manually duplicated and has drifted.

## 16. Documentation governance

Each top-level section MUST have an owner role. Ownership MAY be a team label
rather than a named individual.

Release process:

1. freeze the Residiuum source revision;
2. generate release/API/capability data;
3. run snippets against release artifacts;
4. resolve stale critical pages;
5. build the versioned archive;
6. deploy the archive;
7. switch current canonical docs;
8. verify links, search, redirects, and source revision;
9. record the docs build revision.

Content freshness:

- getting started, security, operations, status, and API/reference pages MUST
  be verified for each minor release;
- concepts SHOULD be reviewed when companion specifications change;
- design specifications update automatically from source but their rendered
  status MUST still be checked;
- stale pages MUST show a visible warning in development previews and block
  release where specified in §6.2.

## 17. Launch deliverables

Required:

- documentation home;
- Rust and CLI getting-started journeys;
- core data guides for JSON, bytes, filters, indexes, pagination, and history;
- durability, backup/restore, scrub, and salvage operations;
- deployment-profile, capability, compatibility, and limitations pages;
- Rust SDK, CLI, configuration, errors, RQL, and SDA reference entry points;
- rendered specification index and stable source-backed spec pages;
- version selector with current and Next;
- local search;
- edit/report/source links;
- accessible responsive navigation;
- generated sitemap, redirects, 404, and metadata;
- all CI gates in §15;
- documentation release runbook.

May follow launch:

- translations;
- interactive playground;
- embedded runnable WebAssembly examples;
- hosted API search;
- annotations/discussion;
- personalized or role-based navigation.

## 18. Acceptance criteria

The documentation site is ready when:

1. a new Rust user can complete open, put, query, close, and reopen from a clean
   machine using only the quickstart;
2. every quickstart command and code sample passes against the displayed
   release;
3. experimental/development/design status is visible before the relevant
   instructions;
4. a user evaluating a network cluster sees “not production” before deployment
   steps;
5. a user can distinguish RQL, raw SDA, RRE, and Atomics and see what is
   implemented;
6. a user can complete backup, restore, scrub, and safe salvage journeys with
   pre-flight and verification steps;
7. every published normative/design specification has a stable URL, source
   revision, document state, and separate product capability status;
8. current, Next, and archived documentation cannot be mistaken for one
   another;
9. search returns status- and version-labelled results and works entirely
   without an external provider;
10. edit and issue links identify the exact source page/version;
11. WCAG 2.2 AA automated and manual checks pass;
12. mobile and desktop layouts, code copy, math, diagrams, tables, and deep
    anchors are verified;
13. performance budgets pass or an explicit reviewed waiver is recorded;
14. source changes that invalidate claims, snippets, generated reference, or
    page freshness fail CI;
15. one release command/process produces a revision-stamped, reversible static
    deployment.

## 19. Explicit non-goals

The docs site MUST NOT:

- hide maturity to make the project appear larger;
- turn every internal planning document into beginner navigation;
- require a JavaScript application runtime to read core content;
- add authentication for public documentation;
- become the source of truth for APIs or specifications already owned by code
  or repository documents;
- present formal designs as “mathematically proven software” without proof
  artifacts and implementation conformance evidence;
- use a conversational assistant as the only way to find documentation.
