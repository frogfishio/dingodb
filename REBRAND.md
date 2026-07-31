# ResiduumDB rebrand plan

Status: **active; Phase 0 and Phase 1 complete; Phase 2 wholesale repository
naming in progress**

Scope: product identity, documentation, implementation identifiers,
compatibility, release notes, and websites

Canonical product name: **ResiduumDB**

Canonical short name: **Residuum**

## 1. Purpose

This document is the authoritative plan for the transition from the former
DingoDB working name to ResiduumDB.

The migration is deliberately phased. A completed phase does not authorize the
next phase, and a documentation rename does not imply that an implementation
identifier has shipped. Phase 2 is now explicitly authorized. Its work remains
subject to the compatibility classes and gates in this document; in particular,
persisted, wire, cryptographic, and historical identifiers are not ordinary
mechanical renames.

Living Phase 2 inventory (REB-1): [doc/REBRAND_INVENTORY.md](doc/REBRAND_INVENTORY.md).

Documentation MUST distinguish the product identity from literal technical
identifiers that still exist in the implementation.

## 2. Canonical terminology

| Former documentation term | Canonical term |
|---|---|
| DingoDB | ResiduumDB |
| Dingo | Residuum, when referring to the product |
| Dingo Query Language (DQL) | Residuum Query Language (RQL) |
| Dingo Rule Expression (DRE) | Residuum Rule Expression (RRE) |
| Dingo Rules | Residuum Rules |
| Dingo Predicate | Residuum Predicate |
| Dingo Studio | Residuum Studio |
| Dingo Evidence Ledger | Residuum Evidence Ledger |
| Dingo Direct Access | Residuum Direct Access |
| Dingo Order Wavelet | Residuum Order Wavelet |
| `dingodb.org` | `residuumdb.org` |
| `docs.dingodb.org` | `docs.residuumdb.org` |

The RQL and RRE names describe the canonical language identities. Lowercase
implementation spellings such as `dql`, `dql_query`, source filenames, work
package identifiers, and compatibility profiles remain literal until their
separate implementation migration is approved.

## 3. Literal legacy identifiers

Markdown MUST preserve a legacy identifier when changing it would make an
example, command, path, protocol statement, compatibility claim, test vector,
or source reference false. This includes, without limitation:

- Rust packages and import paths beginning with `dingo-`;
- Rust crate paths beginning with `dingo_`;
- the current Rust type `Dingo` and expressions such as `Dingo::open`;
- the current `dingo` executable and `residuum-sda` executable;
- the `dingo://` URI scheme;
- `DINGO_*` environment variables;
- `.dingo` store files;
- `dingo-*-v1` wire and persistence profiles;
- `DINGOFRM` and every `DINGODB-*` cryptographic domain separator;
- historical evidence, accepted test output, package names, filesystem paths,
  work-package identifiers, and repository URLs that have not yet moved.

When readers could mistake a literal identifier for the current brand,
documentation SHOULD label it **legacy technical identifier** on first relevant
use. No document may claim that an implementation rename has shipped merely
because its product terminology has changed.

## 4. Naming form

Use **ResiduumDB** for the first product mention in a document and whenever the
database category matters. Use **Residuum** afterward when natural.

Do not use the misspelling “Residiuum.” Do not abbreviate the product itself to
“RDB”; that abbreviation is already overloaded. RQL and RRE are the canonical
language abbreviations.

## 5. Domain policy

The canonical public hosts are:

- `https://residuumdb.org`
- `https://docs.residuumdb.org`

References to local repository directories such as `web/dingodb.org` remain
unchanged until the Phase 4 website migration because those directories have
not yet been renamed.

The docs-site content filenames and routes containing `dql`, `dre`, or
`choose-dingodb` also remain as legacy route identifiers until Phase 4. Their
visible titles and link labels use RQL, RRE, and ResiduumDB. Renaming those
routes requires coordinated changes to non-Markdown navigation and migration
manifests and belongs to the later website migration.

## 6. Completion rule

Phase 1, the Markdown phase, is complete when:

1. normative prose uses the canonical terminology;
2. normative Markdown specification names and visible link labels use RQL,
   RRE, and Residuum names;
3. all local Markdown links resolve;
4. every remaining Dingo-branded occurrence is a literal technical identifier,
   historical statement, local path, or explicit compatibility note; and
5. no Rust or non-Markdown implementation artifact has been changed by this
   phase.

Phase 1 met this rule on 2026-07-31. Existing implementation names remain
intentionally visible where documentation must describe reality.

## 7. Migration sequence and ownership

The required order is:

| Phase | Owner | State | Work |
|---|---|---|---|
| 0. Defect stabilization | active defect developers | **complete** | Current storage defects addressed before rebrand churn |
| 1. Documentation identity | Codex | **complete** | Establish ResiduumDB, RQL, RRE, renamed normative Markdown, and the legacy-identifier rule |
| 2. Wholesale repository naming | Codex | **in progress** | Rename implementation-facing product identifiers and write the migration changelog |
| 3. Rust realignment | principal | not started | Resolve, compile, and semantically realign Rust after the mechanical identity change |
| 4. Website and route migration | principal | not started | Rename website directories, routes, navigation, domains, metadata, and deployment configuration |
| 5. Final audit | Codex | not started | Review the entire repository and websites for correctness, compatibility, stale branding, broken references, and release readiness |

Phase 0 was declared complete and Phase 2 was declared in progress by the
principal on 2026-07-31. That declaration opens the phase; it does not waive
the compatibility analysis required below.

## 8. Phase 2 change surface

Phase 2 is repository-wide, not a blind search-and-replace. It includes:

- Rust public types and constructors such as `Dingo` and `Dingo::open`;
- Cargo package, crate, feature, and import names such as `residuum-sdk` and
  `residuum_sdk`;
- executables and commands such as `dingo` and `residuum-sda`;
- the `dingo://` URI scheme;
- `DINGO_*` environment variables;
- product-owned configuration keys, service names, telemetry names, fixtures,
  examples, scripts, and generated-package metadata;
- internal repository directories whose names are not being reserved for the
  website phase;
- RQL/RRE implementation symbols still carrying DQL/DRE names; and
- every affected Markdown example and reference after the implementation
  spelling becomes true.

Phase 2 must also create a user-facing migration changelog. That changelog must
state every breaking rename, compatibility alias, removal schedule, store or
protocol implication, and concrete before/after command or code example.

## 9. Compatibility classes

Every candidate rename MUST be classified before it is changed.

### 9.1 Class A — source-local identity

Private modules, variables, comments, test names, and non-persisted internal
symbols may normally be renamed directly, followed by compilation and tests.

### 9.2 Class B — public interface identity

Public Rust APIs, crates, executables, command names, environment variables,
configuration keys, and URI schemes are breaking interfaces. Each needs an
explicit choice:

1. hard break;
2. temporary alias with a removal version; or
3. permanent compatibility spelling.

The choice and its test obligation MUST appear in the changelog.

### 9.3 Class C — persisted, wire, or cryptographic identity

The following are protocol facts, not cosmetic branding:

- `.dingo` paths or file conventions;
- `dingo-*-v1` persistence and wire profile identifiers;
- `DINGOFRM` magic bytes;
- `DINGODB-*` cryptographic domain separators;
- serialized enum names, capability identifiers, token claims, and proof
  fixtures; and
- accepted golden vectors or historical evidence derived from those values.

Class C identifiers MUST NOT be mechanically overwritten. Each requires one of:

- retention as a legacy compatibility identifier;
- a new versioned identifier plus a reader/negotiation path for the old value;
- an explicit one-way migration with rollback and recovery behavior; or
- a deliberate compatibility break recorded in the format/protocol
  specification and changelog.

Cryptographic domain separators are especially sensitive: changing one creates
a new cryptographic domain and may invalidate existing keys, signatures,
proofs, fixtures, or stored authority. That effect must be designed, not
discovered during compilation.

### 9.4 Class D — immutable history

Released version tags, accepted evidence, archived benchmark output, historical
repository references, old package coordinates, and quotations remain
unchanged. Current documentation may annotate them as former names, but must
not rewrite history.

## 10. Provisional public-name map

These are the intended targets for Phase 2. “Provisional” means implementation
work must confirm collisions and compatibility behavior before adoption.

| Current implementation name | Intended name |
|---|---|
| `Dingo` | `Residuum` |
| `Dingo::open` | `Residuum::open` |
| `dingo-*` package/binary prefix | `residuum-*` |
| `dingo_*` Rust crate/module prefix | `residuum_*` |
| `dingo` CLI | `residuum` |
| `dingo://` | `residuum://` |
| `DINGO_*` | `RESIDUUM_*` |
| DQL implementation names | RQL equivalents |
| DRE implementation names | RRE equivalents |

This table does not authorize changing Class C identifiers. Store suffixes,
wire-profile strings, magic bytes, and cryptographic domains require separate
decisions during the Phase 2 inventory.

## 11. Phase gates

### 11.1 Entry gate for Phase 2

All of the following are required:

1. the active defect set is complete or deliberately parked;
2. its evidence has been accepted;
3. no defect branch or agent is still writing the files to be renamed;
4. the baseline builds and its required qualification tests pass;
5. the working tree state is recorded so unrelated user changes can be
   preserved; and
6. the principal explicitly authorizes Phase 2.

### 11.2 Exit gate for Phase 2

Phase 2 is ready for Rust realignment only when:

1. the full rename inventory has a disposition for every occurrence;
2. all intended Class A and Class B names have moved;
3. every Class C name has an explicit keep, version, migrate, or break decision;
4. no accidental mixed-brand public surface remains;
5. generated metadata and lockfile consequences are identified;
6. the migration changelog exists; and
7. known compilation failures caused by the mechanical rename are recorded for
   Phase 3 rather than concealed.

### 11.3 Exit gate for the complete rebrand

The rebrand is complete only after:

- Rust builds and the required storage, Heap, protocol, security, and
  qualification suites pass;
- old aliases behave exactly as documented;
- new stores and supported old stores open according to policy;
- wire negotiation and credentials behave according to the Class C decisions;
- package metadata and published names are coherent;
- website routes, canonical URLs, navigation, search metadata, code samples,
  and redirects use ResiduumDB;
- repository-wide stale-name searches contain only approved legacy or
  historical occurrences; and
- the final review records no unexplained Dingo, DQL, or DRE identity.

## 12. Required migration changelog

Phase 2 creates `REBRAND_CHANGELOG.md`. At minimum it must contain:

1. the reason for the rename and the effective release;
2. a complete old-to-new public-name table;
3. Rust dependency and import migration;
4. API/type migration;
5. CLI and executable migration;
6. URI, environment, and configuration migration;
7. RQL and RRE naming migration;
8. storage-format and on-disk compatibility;
9. wire-protocol and cluster compatibility;
10. HeapKey, token, proof, and cryptographic-domain consequences;
11. compatibility aliases and their removal policy;
12. operational upgrade and rollback procedure;
13. website/domain changes and redirects; and
14. known intentionally retained legacy identifiers.

The changelog must describe shipped behavior, not anticipated behavior. It may
be drafted during Phase 2, but it becomes normative only after Phase 3 and the
relevant compatibility tests are complete.

## 13. Final audit method

The Phase 5 review must examine:

- filenames, directory names, source symbols, package manifests, lockfiles,
  scripts, CI, release automation, fixtures, examples, and generated metadata;
- human prose separately from literal implementation identifiers;
- case, separator, and abbreviation variants of DingoDB, Dingo, DQL, and DRE;
- public API and package usability from a clean consumer project;
- old and new command, URI, environment, and configuration behavior;
- new-store, old-store, mixed-version, backup, restore, and rollback journeys;
- wire and cluster interoperability where promised;
- credentials, signatures, golden vectors, and formal-verification evidence;
- both websites, their redirects, canonical metadata, search indexes, social
  metadata, and downloadable examples; and
- the final changelog against the behavior actually demonstrated by tests.

The audit result must classify every remaining former-name occurrence as
approved compatibility, immutable history, or a defect. “Probably harmless”
is not a valid disposition.