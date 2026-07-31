# Residiuum main website specification

Status: **delivered specification v1.0; retained as implementation record**
Target: `https://residiuumdb.org`
Companion: [DOCS_SITE_SPEC.md](./DOCS_SITE_SPEC.md)
Product truth sources: [README.md](../../../README.md),
[USP.md](../../reference/product/USP.md), [doc/wip/status/CAPABILITY_MATRIX.md](../../wip/status/CAPABILITY_MATRIX.md),
[doc/done/programs/PRIME_TIME_PLAN.md](../programs/PRIME_TIME_PLAN.md)

## 1. Decision

`residiuumdb.org` is the product and project website for Residiuum.

Its job is to make a technically serious visitor understand Residiuum's category
in less than ten seconds, trust the project in less than two minutes, and reach
a working Rust example without having to reconstruct the product from its
architecture repository.

The category message is:

> Flexible documents. Mathematical guarantees. Serious speed.

The complete proposition is:

> A document-native database with database-owned mathematical truth, exact
> query mechanics, high-throughput indexed storage, and damage-local recovery.

The governing recovery rule remains:

> Put anything in. Damage it. Keep what survives.

Recovery is a foundation and stress property of the wider system. It MUST NOT
be presented as Residiuum's entire category.

The site MUST be compelling about the architecture and exact about maturity.
It MUST NOT present designs, scaffolds, or performance targets as shipped
capabilities.

## 2. Requirement language

The terms MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

“Launch” means the first public production deployment of the website, not a
claim that Residiuum itself is production-ready.

## 3. Website responsibilities

The main site MUST:

1. establish Residiuum as a mathematical document database rather than a niche
   salvage store;
2. explain the implemented SDA/ENR kernel and the RRE, Atomics, Direct Access,
   and Order Wavelet architecture with exact maturity labels;
3. demonstrate the engine's measured speed without disguising a diagnostic as
   a universal benchmark;
4. explain damage tolerance using a concrete visual model;
5. state what Residiuum is and who should use it today;
6. lead developers to a tested embedded quickstart;
7. expose capability maturity before a visitor makes a technical decision;
8. provide paths to documentation, source, security information, licensing,
   and project status;
9. establish Residiuum as a technically rigorous project rather than a generic
   database landing page;
10. make long-term product direction discoverable without confusing it with
   the current release.

The main site is not:

- the documentation site;
- a hosted-service signup funnel;
- a sales-lead form;
- a substitute for benchmark evidence;
- a release-status dashboard;
- a place to announce features merely because a specification exists.

## 4. Audience and visitor outcomes

### 4.1 Primary audience

- Rust developers embedding local storage;
- infrastructure and database engineers;
- builders storing irreplaceable application, device, research, archival, or
  creative data;
- technical evaluators dissatisfied with all-or-nothing recovery models.

### 4.2 Secondary audience

- contributors and researchers interested in recovery, formal data rules,
  examination, indexing, and distributed storage;
- application teams considering a future network deployment;
- people comparing Residiuum with SQLite, document stores, or embedded key-value
  stores.

### 4.3 Required outcomes

After the home page, a visitor SHOULD be able to answer:

- What is Residiuum?
- What makes its document and query model mathematically different?
- What is different about its failure model?
- What can I use today?
- What is still experimental or only designed?
- Why would I choose it?
- How do I run the smallest real example?

## 5. Shared truth and status contract

This section is normative for both Residiuum websites.

### 5.1 Source hierarchy

Website content MUST use this precedence when sources disagree:

1. `doc/wip/status/CAPABILITY_MATRIX.md` for present capability and maturity;
2. executable code and release metadata for current APIs and versions;
3. normative specifications for intended semantics;
4. `README.md` for project overview;
5. `USP.md` and roadmaps for thesis, targets, and direction.

A stronger claim MUST NOT be published merely because it appears in an older
page. Conflicts MUST fail content review and be resolved in the repository.

### 5.2 Public status vocabulary

Every material feature claim MUST have one of these labels:

| Label | Meaning | Public treatment |
|---|---|---|
| **Available** | Implemented in the named release and supported on the stated surface | May appear without a warning, but version and deployment scope MUST be clear |
| **Experimental** | Implemented and testable, but interfaces, formats, or guarantees may change | MUST carry an “Experimental” badge and link to limitations |
| **Development only** | Exists for development or evaluation and is not a supported deployment claim | MUST NOT be presented as a recommended production path |
| **Scaffold** | Types or partial machinery exist, but the capability is not operationally complete | MUST appear only on status or roadmap pages |
| **Design** | Specified or proposed but not implemented | MUST use future tense and a “Design” badge |
| **Deferred** | Intentionally not on the active delivery path | MUST appear only in roadmap or specification context |

The site MUST NOT use “shipped” as a maturity label because it does not express
fitness for use. “Available” always means “available within the explicitly
named scope.”

### 5.3 Current launch truth

The launch content MUST reflect at least the following:

| Surface | Launch label |
|---|---|
| Embedded single-node | Experimental / early access |
| Single-node TCP | Development only |
| In-process cluster | Integration-test harness |
| Multi-process network cluster | Experimental; not production |
| S3/GCS placement | Experimental filesystem mirror; not native cloud I/O |
| Erasure coding and lifecycle | Scaffold |
| Network and disk wire profiles | Draft where the capability matrix says draft |
| DDA, Order Wavelets, RRE, Atomics, and unimplemented Heap features | Design unless the capability matrix is updated with implementation evidence |

This table MUST be generated from, or checked against,
`doc/wip/status/CAPABILITY_MATRIX.md` before every release.

### 5.4 Claim rules

Approved launch language:

- “Flexible documents. Mathematical guarantees. Serious speed.”
- “A document database built around a deterministic mathematical kernel.”
- “PostgreSQL made the database responsible for truth. MongoDB made data shape
  flexible. Residiuum is designed to do both.”
- “DDA specifies exact access to result rank k without enumerating the
  preceding k − 1 matches,” when labelled Design.
- “Order Wavelets specify filtered ordering through exact conditional
  rank/select counts,” when labelled Design.
- “A database built to survive damage.”
- “Residiuum preserves independently verifiable data islands.”
- “Damage is reported as explicit holes rather than silently invalidating
  unrelated healthy material.”
- “Store JSON documents or opaque bytes through the Rust SDK.”
- “Embedded single-node is the strongest current path and remains experimental
  / early access.”
- “Indexes and catalogs are derived accelerators, not the only route back to
  surviving data.”
- “Designed for a fast hot path,” when adjacent copy does not imply a measured
  result.
- “Targets memory-store-class performance,” only on a roadmap or benchmark
  methodology page and explicitly as a target.
- An owner-observed numeric diagnostic MAY appear before a full artifact only
  when it is labelled “local diagnostic,” names the known hardware,
  concurrency, and record size, explicitly says that the full artifact is
  pending, and states that it is neither a benchmark nor an SLO.

Prohibited launch language until the corresponding release gate changes:

- “indestructible,” “unbreakable,” “cannot lose data,” or “always survives”;
- “Redis-fast,” “Redis-class,” “faster than MongoDB,” or any unqualified
  performance comparison;
- “production-ready,” “enterprise-ready,” or “battle-tested”;
- “production cluster,” “cloud-native object storage,” or “full transactions”;
- “secure by mathematics,” “mathematically impossible to breach,” or any
  security claim that confuses a formal model with an implementation proof;
- “authenticated continuation tokens” while DEF-097 remains open;
- a numeric reliability, retention, latency, throughput, or scale claim without
  a linked reproducible evidence artifact, except the explicitly provisional
  diagnostic form permitted above.

### 5.5 Claim record

Every non-trivial capability or comparative claim in site content MUST carry a
repository-side record with:

```yaml
id: claim.damage.explicit-holes
text: Damage is reported as explicit holes.
status: available
scope: recovery examination
source:
  - USP.md#3-the-defining-difference
  - doc/wip/status/CAPABILITY_MATRIX.md
verified_for: 0.2.0
last_verified: 2026-07-30
```

The implementation MAY store these records in one YAML file or in page
frontmatter. CI MUST reject unknown claim IDs, expired `verified_for` versions,
and a rendered status stronger than the record.

## 6. Information architecture

### 6.1 Canonical routes

| Route | Purpose | Primary action |
|---|---|---|
| `/` | Explain, establish trust, and convert to first use | Get started |
| `/survival/` | Explain independent survival, holes, salvage, and limits | Read recovery guide |
| `/how-it-works/` | Concise architecture for evaluators | Explore specifications |
| `/use-cases/` | Describe good and poor fits | Evaluate fit |
| `/status/` | Current capability and maturity matrix | See known limitations |
| `/benchmarks/` | Reproducible evidence and methodology | Run benchmarks |
| `/roadmap/` | Product direction, clearly separated from release truth | Follow progress |
| `/security/` | Security posture and disclosure path | Report a vulnerability |
| `/project/` | Source, licenses, contributing, governance, releases | View source |
| `/privacy/` | Website privacy statement | None |

The launch MAY omit `/benchmarks/` only if no benchmark package has been
published. In that case every performance link MUST go to a short status block
on `/status/` that says no public comparative result is currently claimed.

### 6.2 Global navigation

Desktop:

```text
Residiuum | Why Residiuum | How it works | Status | Docs | GitHub | Get started
```

Mobile MUST expose the same destinations in a keyboard-operable menu.

`Why Residiuum` links to `/survival/`. `Docs` opens `docs.residiuumdb.org`. `GitHub`
links to `https://github.com/frogfishio/dingodb`. `Get started` links to the
current embedded quickstart on the documentation site.

The navigation MUST NOT contain inactive menu items.

### 6.3 Footer

The footer MUST include:

- Docs;
- Status and known limitations;
- Roadmap;
- Security;
- Source and contributing;
- licensing;
- privacy;
- current website build revision;
- current documented Residiuum release;
- “Residiuum is experimental software” at launch.

## 7. Home page content specification

The home page MUST follow this narrative order:

1. mathematical document-database category;
2. current engineering performance signal;
3. implemented kernel versus normative designs;
4. survival as a cross-cutting system property;
5. current capability and maturity;
6. ordinary Rust developer experience;
7. delivery direction.

### 7.1 Hero

Recommended launch copy:

```text
DOCUMENT-NATIVE · MATHEMATICAL CORE · DAMAGE-TOLERANT

Flexible documents.
Mathematical guarantees.
Serious speed.

Residiuum is a document database built around a deterministic mathematical
kernel—exact query semantics, database-owned invariants, direct ranked access,
and ordering by counted structure rather than offset scans and hope.

[Explore the system] [Run the Rust quickstart]
```

The hero MUST contain an adjacent engineering signal:

```text
≈350 MB/s
indexed ingest · 4 KiB writes · four workers · MacBook Air M4
chaos active

Owner-observed local diagnostic; not a cross-system benchmark or SLO.
```

The numeric signal MUST be removed or replaced if its full qualification is
found inaccurate. The release value MUST come from workspace or release
metadata, not hard-coded page copy.

### 7.2 Product quadrant and mathematical system

The next section MUST establish:

```text
PostgreSQL made the database responsible for truth.
MongoDB made data shape flexible.
Residiuum is designed to do both.
```

It MUST then distinguish:

- SDA/ENR deterministic kernel — Available;
- shipped RQL subset and dialect compilation — Experimental;
- bounded resume-key cursors — Available;
- RRE and Atomics — Design;
- Residiuum Direct Access — Design;
- Residiuum Order Wavelets — Design.

The full RQL/RRE/DDA/DOW architecture MAY lead the product story even when some
profiles are designs, provided the status appears adjacent to each proposition.

### 7.3 Performance evidence

The home page MUST explain that the measured value includes live indexing and
coexists with chaos punches. It MUST also state that a comparative result
requires the complete benchmark disclosure contract.

### 7.4 Survival visual

The primary visual is the “damaged CD-ROM” model supplied by the product
owner:

1. an intact data surface is visibly divided into small, independent regions;
2. several regions are punched out or abraded;
3. destroyed regions become labelled `HOLE`;
4. all undamaged regions remain visibly readable and verified;
5. an accompanying sentence states that the visual explains the failure model,
   not a literal optical-disc storage format.

The visual MUST be understandable without animation. With motion enabled, the
damage MAY occur progressively and surviving regions MAY resolve to
`VERIFIED`. With `prefers-reduced-motion`, only the final static state appears.

Required accessible description:

> A storage surface with several missing regions. Missing regions are reported
> as holes; intact regions on both sides remain independently readable.

### 7.5 System promises

The system promises are:

1. **One deterministic kernel**
   Query, rule, examination, and evidence surfaces share exact semantics.

2. **Documents with database-owned truth**
   RRE and Atomics specify finite invariants and admitted transitions without
   forcing document shape into relational tables.

3. **Position and order are mathematical operations**
   DDA and Order Wavelets specify direct ranks and counted ordering.

4. **Healthy pieces survive**
   Local corruption is contained by independently verifiable storage units;
   SDA can examine the resulting evidence.

Massive retention appears as a supporting proposition under these cards:

> Keep arbitrary material now. Preserve enough structure to investigate what
> remains years later.

This MUST NOT imply that native cloud archive tiers or a fifteen-year
compatibility guarantee are available today.

### 7.6 What works today

This section MUST be fed by structured capability data and show:

- embedded open/create;
- JSON and byte put/get/delete;
- deterministic SDA/ENR execution;
- shipped RQL subset and dialect compilation;
- bounded cursor paging;
- filters and secondary indexes;
- per-key history;
- backup, verified restore, scrub, and salvage/examination surfaces;
- current limitations and experimental label.

Network, clustering, archive-tier, and design-only capabilities MUST appear
under a separate “What is being built” heading.

Each item MUST display its deployment scope and status badge. The section MUST
link to `/status/`.

### 7.7 Real quickstart

The page MUST show an executable Rust example based on
`crates/residiuum-sdk/README.md`, not illustrative pseudocode.

Minimum example:

```toml
[dependencies]
residiuum-sdk = "0.2"
```

```rust
use residiuum_sdk::{json, Residiuum, Filter};

let mut db = Residiuum::open("./app.dingo")?;
let mut users = db.collection("users")?;

users.put("user-42", &json!({
    "name": "Alice",
    "status": "active"
}))?;

let active = users.find(&Filter::field("status").eq("active"))?;
```

The actual launch snippet MUST compile in CI against the displayed release.
The site MUST provide copy controls, expected result, cleanup instructions, and
a link to the complete quickstart. Snippet lines MAY be adjusted to keep them
executable; the repository test is authoritative.

### 7.8 How Residiuum differs

Use a factual decision table, not a winner-takes-all comparison:

| Need | Recommended direction |
|---|---|
| Mature SQL transactions, constraints, and broad tooling | Use PostgreSQL or SQLite |
| Mature general-purpose document database and operational ecosystem | Use MongoDB |
| Embedded arbitrary data with Residiuum’s explicit damage/salvage model | Evaluate Residiuum |
| Production network cluster today | Do not choose Residiuum yet |

Comparisons MUST describe workload fit, not attack other products.

### 7.9 Architecture teaser

Show this compact model:

```text
Application API / RQL
          ↓
Authoritative self-verifying records
          ↓
Derived, rebuildable catalogs and indexes
          ↓
SDA examination and recovery evidence
```

Heaps, RRE, Atomics, Direct Access, and Order Wavelets MAY appear as linked
research/design cards. Each MUST carry its current public status.

### 7.10 Final action

```text
Put something important in Residiuum.

Start with one local file and a Rust application.

[Run the quickstart] [Read the architecture]
```

## 8. Secondary page requirements

### 8.1 Survival

`/survival/` MUST explain:

- the ordinary all-or-nothing failure model;
- independent survival units;
- verification and explicit holes;
- authoritative versus derived state;
- catalog-independent salvage;
- what cannot be recovered;
- how replication, backup, and salvage differ;
- a runnable “damage then examine” demonstration when the repository has a
  stable operator journey for it.

It MUST explicitly state that damage tolerance reduces blast radius; it does
not make missing or overwritten bytes recoverable.

### 8.2 How it works

`/how-it-works/` MUST be an evaluator overview, not a copy of the normative
specifications. It MUST cover:

- collection and key application model;
- JSON and opaque bytes;
- immutable/self-verifying storage units;
- derived indexes and catalogs;
- history;
- query surfaces: Rust filters, RQL, and raw SDA;
- durability acknowledgement modes;
- recovery evidence and coverage;
- current deployment profiles.

Every section links to the relevant docs or specification.

### 8.3 Use cases

`/use-cases/` MUST include “good fit now,” “evaluate carefully,” and “not yet”
lists. At launch:

Good fit to evaluate:

- embedded Rust applications;
- irreplaceable local records and blobs;
- systems that value inspectable partial recovery;
- experimental/research workloads exploring formal data handling.

Not yet:

- a drop-in mature SQL database;
- a production MongoDB replacement;
- a public multi-tenant service requiring mature network operations;
- a production multi-node cluster;
- a native object-store archive platform.

### 8.4 Status

`/status/` MUST render the capability matrix in a user-oriented form and link
to the full repository evidence. It MUST show:

- documented release and date;
- deployment profiles;
- capability statuses;
- known material limitations;
- compatibility labels;
- benchmark status;
- definition of each maturity label.

The page MUST be generated from structured data checked against
`doc/wip/status/CAPABILITY_MATRIX.md`; hand-maintained duplicate truth is not acceptable.

### 8.5 Benchmarks

When enabled, `/benchmarks/` MUST state:

- exact source revision;
- hardware and operating system;
- dataset and operation distribution;
- concurrency;
- record sizes;
- warm/cold state;
- durability acknowledgement mode;
- error bars or repeated-run distribution;
- commands and raw results;
- competing system versions and equivalently configured durability;
- known omissions.

A chart without downloadable methodology and raw results MUST NOT be
published.

### 8.6 Roadmap

`/roadmap/` MUST organize direction by maturity gates, not dates that the
project has not committed to. It SHOULD show:

- embedded early-access hardening;
- single-node server maturity;
- cluster qualification;
- archive/native object-store work;
- future search: text, vector, geospatial;
- design work: Heaps, RRE, Atomics, DDA, Order Wavelets.

Roadmap cards MUST NOT use the “Available” badge.

### 8.7 Security

`/security/` MUST include:

- supported versions, or a statement that no production-supported line exists;
- vulnerability reporting route from repository security policy;
- response expectations only if the project can meet them;
- dependency and release-signing posture;
- concise threat-model links;
- a warning not to submit vulnerabilities through public issues.

No email address or response SLA may be invented by the website team.

### 8.8 Project

`/project/` MUST link to:

- GitHub repository;
- release artifacts;
- `CONTRIBUTING.md`;
- code of conduct if one exists;
- crate-specific licensing explanation;
- changelog or release notes;
- project governance or maintainer information if published.

The site MUST NOT describe the whole workspace under a single license when
crate licenses differ.

## 9. Visual and verbal system

### 9.1 Direction

The visual language is “technical field instrument meets damaged archival
media.” It should feel precise, resilient, and a little physical.

Avoid:

- generic purple/blue SaaS gradients;
- a glowing cylinder as the primary database image;
- fake dashboards;
- fake customer logos or testimonials;
- aggressive “Mongo killer” language;
- a cartoon mascot as the main product identity.

### 9.2 Palette

Initial design tokens:

```css
--paper: #f3f0e7;
--paper-raised: #fffdf7;
--ink: #121410;
--ink-muted: #555b52;
--line: #c9c5b9;
--signal: #d94f27;
--verified: #247451;
--warning: #8a5b00;
--hole: #777970;
--code-bg: #171a16;
--code-ink: #eff4e9;
```

Designers MAY tune values, but the semantic token names and purposes MUST
remain. Final combinations MUST pass WCAG 2.2 AA. Status MUST never be encoded
by color alone.

### 9.3 Type

- Primary: IBM Plex Sans, self-hosted.
- Technical/code: IBM Plex Mono, self-hosted.
- System fallbacks MUST be supplied.
- A page MUST remain usable if web fonts fail.

Body copy SHOULD use a comfortable 65–75 character measure. Marketing headings
MUST not use all caps except short eyebrow labels.

### 9.4 Voice

Voice is plain, rigorous, confident, and slightly irreverent. It MAY say
“damage it” and “keep the weird data.” It MUST become exact whenever it
describes guarantees.

Prefer:

- “Here is what survives.”
- “Here is what is unknown.”
- “Here is the evidence.”

Avoid:

- “revolutionary,” “next-generation,” “web-scale,” and “AI-powered”;
- “trust us” claims;
- unexplained mathematical prestige language;
- swearing in primary website copy.

## 10. Component contract

The implementation MUST provide reusable components for:

- header and mobile navigation;
- footer;
- `StatusBadge`;
- `CapabilityCard`;
- `Claim`;
- `CodeExample`;
- `Callout`;
- `EvidenceLink`;
- `SurvivalDiagram`;
- comparison table;
- release banner;
- SEO metadata;
- accessible external-link indicator.

`StatusBadge` MUST accept only the vocabulary in §5.2. Unknown status values
MUST fail the build.

`Claim` MUST require a valid claim record ID for capability, performance,
reliability, security, or comparative assertions.

## 11. Technical implementation

### 11.1 Recommended stack

The reference implementation is:

- TypeScript;
- Astro in static-output mode;
- Markdown/MDX for editorial pages;
- a shared design-token package with the docs site;
- no client framework for static content;
- small isolated interactive islands only where necessary.

Another stack is acceptable only if it meets every output, performance,
accessibility, content-validation, and maintainability requirement in this
specification.

### 11.2 Repository layout

Recommended:

```text
website/
  apps/
    main/
    docs/
  packages/
    design/
    content-schema/
    capability-data/
  scripts/
```

The website MAY live in a separate repository. If so, its build MUST pin the
Residiuum source revision used for capability data, snippets, and release
metadata.

### 11.3 Content data

Navigation, capability status, release metadata, claims, and redirects MUST be
structured data, not duplicated literals across components.

Content files MUST have:

```yaml
title:
description:
status:
last_verified:
claim_ids: []
```

### 11.4 Domains

- `https://residiuumdb.org` is canonical.
- `https://www.residiuumdb.org/*` MUST permanently redirect to the same path on
  `https://residiuumdb.org`.
- `https://docs.residiuumdb.org` hosts documentation.
- HTTP MUST redirect to HTTPS.
- Preview deployments MUST be `noindex`.

DNS SHOULD enable DNSSEC where supported. CAA SHOULD restrict certificate
issuers. HSTS SHOULD be enabled after both production hosts and redirects have
been validated; preload MUST not be enabled casually.

### 11.5 Security headers

Production SHOULD send:

- a restrictive Content Security Policy;
- `X-Content-Type-Options: nosniff`;
- `Referrer-Policy: strict-origin-when-cross-origin`;
- `Permissions-Policy` disabling unused browser capabilities;
- frame protection through CSP `frame-ancestors`;
- HSTS after domain validation.

Third-party scripts MUST be justified, pinned where possible, and included in
the CSP. User-controlled HTML MUST NOT be rendered.

### 11.6 Performance budgets

Measured at the 75th percentile on mobile:

| Metric | Budget |
|---|---|
| LCP | ≤ 2.0 s |
| INP | ≤ 200 ms |
| CLS | ≤ 0.05 |
| Initial page JavaScript | ≤ 100 KB compressed |
| Initial CSS | ≤ 40 KB compressed |
| Home page transfer, excluding cached fonts | ≤ 750 KB |

Images MUST use explicit dimensions and responsive AVIF/WebP with a fallback.
Fonts MUST be subset and self-hosted. No autoplay video is permitted. The
primary story and navigation MUST work with JavaScript disabled.

## 12. Accessibility

The site MUST meet WCAG 2.2 AA.

In particular:

- all functions are keyboard operable;
- focus is visible and never obscured;
- a skip link is present;
- headings form a valid hierarchy;
- navigation exposes current location;
- dialog/menu focus is managed correctly;
- diagrams have a concise alternative and an adjacent detailed explanation;
- animation obeys reduced-motion preferences;
- code blocks can scroll without trapping keyboard focus;
- touch targets meet minimum size guidance;
- status includes text/icons, not only color;
- zoom to 200% and reflow at 320 CSS pixels remain usable.

Automated checks do not replace a manual keyboard and screen-reader pass.

## 13. SEO and sharing

Every public page MUST have:

- unique title and description;
- canonical URL;
- Open Graph and social-card metadata;
- meaningful share image;
- indexability setting;
- breadcrumb data where appropriate.

The build MUST generate `sitemap.xml` and `robots.txt`.

Use structured data only when factually valid. `SoftwareSourceCode` is
appropriate for the repository. Do not invent ratings, price, organization
facts, or availability.

Redirects MUST preserve inbound links when routes change. A useful branded 404
MUST offer search/docs, status, and home links.

## 14. Analytics and privacy

The preferred launch is no analytics. If product measurement is required, use
a self-hosted or privacy-preserving, cookieless system and collect only:

- page view;
- get-started click;
- documentation click;
- source/release click;
- code-copy success;
- survival-demo completion.

Do not collect typed search queries on the main site, IP-derived profiles,
fingerprints, or cross-site identity. Do not load advertising pixels.

If no non-essential cookies or local storage are used, do not display a fake
cookie-consent banner. `/privacy/` MUST accurately describe the deployed
configuration.

## 15. Build, review, and release

Every pull request MUST run:

1. static build;
2. type and schema validation;
3. claim/status validation;
4. internal and external link checks;
5. snippet compilation;
6. HTML validation;
7. accessibility automation;
8. Lighthouse budget checks on representative pages;
9. spelling with a Residiuum terminology allowlist;
10. screenshot regression at desktop and mobile breakpoints.

Each pull request SHOULD receive a temporary preview URL. Preview pages MUST
not be indexed.

A website release MUST record:

- website source revision;
- Residiuum source revision;
- documented Residiuum version;
- generation time;
- content schema version.

## 16. Launch deliverables

Required:

- all routes in §6.1 except the conditional benchmark page;
- shared header/footer and complete responsive behavior;
- home-page narrative in §7;
- capability/status data pipeline;
- tested embedded quickstart;
- survival visual and accessible explanation;
- security, privacy, licensing, and project pages;
- metadata, sitemap, redirects, 404, and social image;
- analytics disabled or privacy configuration documented;
- automated quality gates;
- a content-owner runbook for version/status updates.

Not required for launch:

- blog;
- newsletter;
- hosted playground;
- account system;
- sales form;
- localization;
- live service-status product;
- customer logos or case studies.

## 17. Acceptance criteria

The main website is ready when:

1. a first-time visitor can identify Residiuum’s survival proposition in a
  five-second comprehension test as a mathematical document database rather
  than only a resilient embedded store;
2. the visitor can identify “experimental / early access” without opening a
   secondary page;
3. every capability shown on the home page has a valid status and evidence
   source;
4. a clean Rust project can run the displayed quickstart against the displayed
   version;
5. no prohibited claim in §5.4 appears;
6. all navigation works with keyboard and without JavaScript;
7. mobile layouts work at 320, 375, 768, and 1280 CSS-pixel widths;
8. WCAG 2.2 AA automated and manual checks pass;
9. the budgets in §11.6 pass in CI or an explicit waiver is recorded;
10. `www`, HTTP, canonical, sitemap, preview `noindex`, and 404 behavior are
    verified in production;
11. the website revision and documented product revision are visible;
12. a release owner can update version and maturity without editing the same
    fact in multiple pages.

## 18. Product-owner content required before launch

The developers MUST obtain or confirm:

- final wordmark and favicon assets;
- repository vulnerability-reporting route;
- whether any analytics is desired;
- maintainers/governance copy that may be made public;
- social-card attribution and licensing for any external imagery;
- release artifact URL policy.

Missing inputs MUST use an explicit “not yet published” state. Developers MUST
not invent them.
