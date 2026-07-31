# Residiuum rebrand plan

Status: **non-website rebrand complete through REB-12; website Phase 4 is
owner-managed and still open; Phase 5 is complete for non-web surfaces and
awaits the websites**

Scope: product identity, documentation, implementation identifiers,
compatibility, release notes, and websites

Canonical product name: **Residiuum**

Canonical short name: **Residiuum**

## 1. Purpose

This document is the authoritative plan for the transition from the former
DingoDB working name to Residiuum.

The migration is deliberately phased. A completed phase does not authorize the
next phase, and a documentation rename does not imply that an implementation
identifier has shipped. Phase 2 is now explicitly authorized. Its work remains
subject to the compatibility classes and gates in this document; in particular,
persisted, wire, cryptographic, and historical identifiers are not ordinary
mechanical renames.

Living Phase 2 inventory (REB-1): [doc/REBRAND_INVENTORY.md](doc/REBRAND_INVENTORY.md).  
Class C freeze (REB-5): [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).  
User-facing migration draft (REB-6): [REBRAND_CHANGELOG.md](REBRAND_CHANGELOG.md).

Documentation MUST distinguish the product identity from literal technical
identifiers that still exist in the implementation.

## 2. Canonical terminology

| Former documentation term | Canonical term |
|---|---|
| DingoDB | Residiuum |
| Dingo | Residiuum, when referring to the product |
| Dingo Query Language (DQL) | Residiuum Query Language (RQL) |
| Dingo Rule Expression (DRE) | Residiuum Rule Expression (RRE) |
| Dingo Rules | Residiuum Rules |
| Dingo Predicate | Residiuum Predicate |
| Dingo Studio | Residiuum Studio |
| Dingo Evidence Ledger | Residiuum Evidence Ledger |
| Dingo Direct Access | Residiuum Direct Access |
| Dingo Order Wavelet | Residiuum Order Wavelet |
| `dingodb.org` | `residiuumdb.org` |
| `docs.dingodb.org` | `docs.residiuumdb.org` |

The RQL and RRE names describe the canonical language identities. Lowercase
implementation spellings such as `dql`, `dql_query`, source filenames, work
package identifiers, and compatibility profiles remain literal until their
separate implementation migration is approved.

## 3. Literal legacy identifiers

Markdown MUST preserve a legacy identifier when changing it would make an
example, command, path, protocol statement, compatibility claim, test vector,
or source reference false. After Phase 2 Class A/B renames, **current**
implementation identity is Residiuum-named; the remaining **literal legacy**
identifiers include, without limitation:

- Class C wire/on-disk facts: `.dingo` store files; `dingo-*-v1` profiles;
  frame magics `DINGOFRM` / `DINGOEND`; `urn:dingo:…`;
  `__dingo_snapshot_base__`; content-types such as `application/dingo.heap-*`;
  every `DINGODB-*` cryptographic domain separator (see
  [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md));
- Class D history: git history, release tags, remote
  `github.com/frogfishio/dingodb`, historical work-package ids;
- Phase 4 surfaces: local dirs `web/dingodb.org`, marketing copy, old routes;
- Historical docs that deliberately show pre-Phase-2 names as migration
  before/after examples (e.g. [REBRAND_CHANGELOG.md](REBRAND_CHANGELOG.md)).

**Current (post–Phase 2) Class B identity (do not reverse):** packages
`residiuum-*`, type `Residiuum` / `Residiuum::open`, CLI `residiuum` /
`residiuum-sda`, URI `residiuum://`, env `RESIDIUUM_*`.

When readers could mistake a literal identifier for the current brand,
documentation SHOULD label it **legacy technical identifier** on first relevant
use. No document may claim that an implementation rename has shipped merely
because its product terminology has changed.

## 4. Naming form

Use **Residiuum** for the product. Do not append “DB” to the product name.

Do not use the incorrect spellings “Residuum” or “ResiduumDB.” The `db` suffix
belongs only to the public domain names. Do not abbreviate the product itself
to “RDB”; that abbreviation is already overloaded. RQL and RRE are the
canonical language abbreviations.

## 5. Domain policy

The canonical public hosts are:

- `https://residiuumdb.org`
- `https://docs.residiuumdb.org`

References to local repository directories such as `web/dingodb.org` remain
unchanged until the Phase 4 website migration because those directories have
not yet been renamed.

The docs-site content filenames and routes containing `dql`, `dre`, or
`choose-dingodb` also remain as legacy route identifiers until Phase 4. Their
visible titles and link labels use RQL, RRE, and Residiuum. Renaming those
routes requires coordinated changes to non-Markdown navigation and migration
manifests and belongs to the later website migration.

## 6. Completion rule

Phase 1, the Markdown phase, is complete when:

1. normative prose uses the canonical terminology;
2. normative Markdown specification names and visible link labels use RQL,
   RRE, and Residiuum names;
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
| 1. Documentation identity | Codex | **complete** | Establish Residiuum, RQL, RRE, renamed normative Markdown, and the legacy-identifier rule |
| 2. Wholesale repository naming | Codex | **complete through REB-12** | Class A/B renames + Class C freeze + RQL surface + residual audit + docs + workspace verify |
| 3. Rust realignment | principal | **complete for rebrand scope** | Workspace compiles and the full workspace suite is green under Residiuum names |
| 4. Website and route migration | principal | owner-managed; open | Rename website directories, routes, navigation, domains, metadata, and deployment configuration |
| 5. Final audit | Codex | **non-web complete; website remainder open** | Non-web repository is audited; repeat against both websites after Phase 4 |

Phase 0 was declared complete and Phase 2 was authorized by the principal on
2026-07-31. Phase 2 mechanical renames, RQL public-surface completion (REB-8),
Class C re-audit (REB-9), public-identity residual fixes (REB-10), and REB-12
workspace verification are complete. The non-web rebrand is closed. Website
work remains the independently managed Phase 4, followed by the website portion
of the Phase 5 audit.

## 8. Phase 2 change surface

Phase 2 is repository-wide, not a blind search-and-replace. It includes:

- Rust public types and constructors such as `Residiuum` and `Residiuum::open`;
- Cargo package, crate, feature, and import names such as `residiuum-sdk` and
  `residiuum_sdk`;
- executables and commands such as `residiuum` and `residiuum-sda`;
- the `residiuum://` URI scheme;
- `RESIDIUUM_*` environment variables (hard-break from former `DINGO_*`);
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
| `Dingo` | `Residiuum` |
| `Dingo::open` | `Residiuum::open` |
| `dingo-*` package/binary prefix | `residiuum-*` |
| `dingo_*` Rust crate/module prefix | `residiuum_*` |
| `dingo` CLI | `residiuum` |
| `dingo://` | `residiuum://` |
| `DINGO_*` | `RESIDIUUM_*` |
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
  and redirects use Residiuum;
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

## 14. Remaining implementation handoff

This section is the authoritative handoff for completing the rebrand. Execute
the work packages sequentially. Do not restart the rebrand, run an unrestricted
brand replacement, or allow multiple models to edit the same surface
concurrently.

### 14.1 Canonical identity

The product name is **Residiuum**.

The spelling `residiuumdb` is not a product name. It is permitted only as part
of a web domain, including:

- `residiuumdb.com`;
- `residiuumdb.org`; and
- `docs.residiuumdb.org`.

The spellings `Residuum`, `ResiduumDB`, and `residuum-*` are incorrect,
unreleased intermediate names and have no compatibility status.

### 14.2 REB-8 — complete the interrupted RQL public-surface rename

**Labor status (2026-07-31): complete → board `in_review`.** Implementers
continued from the partial tree rather than restarting the rename.

Prior interrupted state (now resolved):

- `crates/residiuum-sdk/src/dialects/dql.rs` had been moved to `rql.rs`;
- several `Dql` / `dql` public symbols had become `Rql` / `rql`;
- some frozen profile **values** had accidentally been changed to `rql-*`
  (restored to `dql-*`); and
- `crates/residiuum-sdk/src/lib.rs` exported the obsolete
  `DQL_APP_CORE_PROFILE` symbol (now exports `RQL_APP_CORE_PROFILE` only).

The following public names MUST become RQL names:

- `BuiltinDialect::Dql` → `BuiltinDialect::Rql`;
- dialect identifier `"dql"` → `"rql"`;
- `compile_dql` → `compile_rql`;
- `DqlProgram` → `RqlProgram`;
- `parse_dql` → `parse_rql`;
- public method `dql` → `rql`;
- `explain_dql` → `explain_rql`;
- `DQL_APP_CORE_PROFILE` symbol → `RQL_APP_CORE_PROFILE`;
- `DQL_PLAN_PROFILE` symbol → `RQL_PLAN_PROFILE`;
- visible console terminology DQL → RQL; and
- source-document links `DQL_SPEC.md` → `RQL_SPEC.md`.

The obsolete public dialect aliases `"dql"` and `"dingo-ql"` SHOULD be removed
unless an existing normative compatibility decision explicitly requires them.

The following serialized values are frozen compatibility identifiers and MUST
remain unchanged:

- `dql-app-core-v1`;
- `dql-plan-v1`;
- `dql-plan-encoding-v1`;
- `dingo:dql-plan-v1:canonical-v1`;
- wire operation and fixture names such as `dql_query.*`;
- serialized error identifiers such as `dql_feature_unavailable`;
- accepted profile fields under `spec/app/v1/`; and
- accepted wire fixtures under `spec/heap/`.

The intended distinction is:

```rust
pub const RQL_APP_CORE_PROFILE: &str = "dql-app-core-v1";
pub const RQL_PLAN_PROFILE: &str = "dql-plan-v1";
```

In `crates/residiuum-sdk/src/app_v1.rs`, restore the frozen values
`dql-app-core-v1` and `dql-plan-v1`, including their assertions. In
`crates/residiuum-sdk/src/lib.rs`, export `RQL_APP_CORE_PROFILE` and remove the
stale `DQL_APP_CORE_PROFILE` export. Tests must use the new Rust symbol names
while continuing to expect the frozen serialized values.

REB-8 acceptance (observed):

```text
cargo check --workspace                    # exit 0
cargo test -p residiuum-sdk                # exit 0
cargo test -p residiuum-cli --test console # exit 0 (1/1)
rg '\bDql\b|compile_dql|parse_dql|explain_dql|DQL_APP_CORE_PROFILE|DQL_PLAN_PROFILE' crates
# empty
```

Shipped pattern:

```rust
pub const RQL_APP_CORE_PROFILE: &str = "dql-app-core-v1";
pub const RQL_PLAN_PROFILE: &str = "dql-plan-v1";
```

Remaining lowercase `dql` occurrences are frozen profiles, wire identifiers,
fixtures, error identifiers, or historical compatibility statements.

### 14.3 REB-9 — Class C compatibility audit

Inspect every changed literal against
[doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md). In particular,
preserve:

- `DINGOFRM` and `DINGOEND`;
- `dingo-*-v1` profiles;
- `dingo-store-9`;
- persisted cluster labels;
- `urn:dingo:*`;
- `application/dingo.*`;
- `DINGODB-*` cryptographic domains;
- `dingo:` hash domains;
- `.dingo`;
- `__dingo_snapshot_base__`; and
- the frozen DQL profile and wire identifiers listed in REB-8.

An earlier mechanical pass accidentally renamed some Class C values. The known
store, Heap, server, SDA, and cluster cases have been restored, including the
store readers that recognize the `dingo-store-*` metadata family. REB-9 must
prove that no additional Class C value escaped.

**Labor status (2026-07-31): complete → board `in_review`.** Greps confirmed
magics, `dingo-*-v1` profiles, `dingo-store-*`, URNs, `application/dingo.*`,
`DINGODB-*` domains, `__dingo_snapshot_base__`, and frozen DQL profile strings
remain. No Class C reverts required after REB-8 profile restore.

REB-9 acceptance (observed):

```text
cargo test -p residiuum-format  # exit 0
cargo test -p residiuum-heap    # exit 0
cargo test -p residiuum-store   # exit 0 (default features)
cargo test -p residiuum-cluster # exit 0
```

Hygiene performed during REB-9: `residiuum-store` integration tests that import
public `Store` were given `required-features = ["legacy-raw-store"]` so default
`cargo test -p residiuum-store` compiles cleanly. Optional
`--features legacy-raw-store` still surfaces one pre-existing DEF-013 catalog
failure (not a Class C escape).

### 14.4 REB-10 — public identity residual audit

The required public identity is:

| Surface | Required identity |
|---|---|
| Product | `Residiuum` |
| Rust entry type | `Residiuum` |
| Constructor | `Residiuum::open` |
| Cargo packages | `residiuum-*` |
| Rust imports | `residiuum_*` |
| Main CLI | `residiuum` |
| SDA CLI | `residiuum-sda` |
| Client URI | `residiuum://` |
| Environment | `RESIDIUUM_*` |
| Query language | RQL |
| Rule language | RRE |

Search for and classify every remaining spelling of `Residuum`,
`ResiduumDB`, `residuum`, `DingoDB`, product-facing `Dingo`, public
`dingo_*`, `dingo://`, public `DINGO_*`, DQL, and DRE. Every remaining former
name must be approved Class C compatibility, immutable history, an explicitly
deferred website occurrence, or a defect.

Do not modify `web/` during REB-10. Website migration remains Phase 4.

**Labor status (2026-07-31): complete → board `in_review`.** Defects fixed
without touching `web/`:

| Former public identity | Now |
|---|---|
| `DingoDeployment` | `ResidiuumDeployment` |
| `DingoConfigFile` | `ResidiuumConfigFile` |
| CLI product strings / prompt `dingo` | `residiuum` |
| Cargo `keywords = ["dingodb", …]` | `["residiuum", …]` |
| Stale media error `DINGO_{}_ROOT` text | `RESIDIUUM_{}_ROOT` |

Class C and history retained as above. Wrong intermediate spellings `Residuum` /
`residuum-*` appear only as deliberate forbidden-form documentation.

### 14.5 REB-11 — documentation and changelog reconciliation

Reconcile this document, [REBRAND_CHANGELOG.md](REBRAND_CHANGELOG.md), and
[doc/REBRAND_INVENTORY.md](doc/REBRAND_INVENTORY.md) with implemented reality.
They must record:

- Residiuum as the exact product name;
- `Residuum` as an incorrect, unreleased intermediate spelling;
- `residiuumdb` as domain-only;
- the completed Class A/B renames;
- deliberate retention of Class C identifiers;
- RQL public names paired with frozen DQL profile and wire values;
- explicit deferral of the websites; and
- only test evidence that was actually observed.

All local Markdown links must resolve after reconciliation.

**Labor status (2026-07-31): complete → board `in_review`.** Docs and changelog
reconciled; REB-12 workspace evidence follows.

### 14.6 REB-12 — final verification

After REB-8 through REB-11 are complete, run:

```text
cargo check --workspace
cargo test --workspace
```

The full workspace suite previously exposed a test-harness race in
`hp006_heap_migration`: parallel cases shared the process-wide failpoint
registry. The failpoint-using cases now take a common test mutex, and the test
passes under its normal parallel configuration.

Do not apply repository-wide automatic formatting during this work. The
repository has substantial pre-existing formatting differences; doing so would
create a large unrelated diff. Formatting must be limited to files materially
changed by the relevant work package.

REB-12 is complete only when the workspace build and test suite pass, the
residual audit has no unexplained occurrence, and the evidence is recorded in
the rebrand changelog. Website work and the post-website Phase 5 audit remain
separate.

**Labor status (2026-07-31): complete → board `in_review`.** Observed:

```text
cargo check --workspace   # exit 0
cargo test --workspace    # exit 0 (~1265 tests passed, 0 failed)
```

Evidence recorded in [REBRAND_CHANGELOG.md](REBRAND_CHANGELOG.md) §15. Incidental
DEF-013 durable collection-catalog frontier fix landed under `residiuum-store`
so the workspace suite is green.
