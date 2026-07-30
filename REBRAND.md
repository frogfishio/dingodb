# ResiduumDB documentation identity

Status: normative for Markdown documentation  
Scope: documentation-only rebrand  
Canonical product name: **ResiduumDB**  
Canonical short name: **Residuum**

## 1. Purpose

This document controls the documentation transition from the former DingoDB
working name to ResiduumDB. It does not rename Rust packages, source symbols,
executables, environment variables, URI schemes, persistent files, wire
profiles, cryptographic domains, test vectors, or repository directories.

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
- the current `dingo` executable and `dingo-sda` executable;
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
unchanged during this documentation-only phase because those directories have
not yet been renamed.

The docs-site content filenames and routes containing `dql`, `dre`, or
`choose-dingodb` also remain as legacy route identifiers in this phase. Their
visible titles and link labels use RQL, RRE, and ResiduumDB. Renaming those
routes requires coordinated changes to non-Markdown navigation and migration
manifests and belongs to the later website migration.

## 6. Completion rule

The Markdown phase is complete when:

1. normative prose uses the canonical terminology;
2. normative Markdown specification names and visible link labels use RQL,
   RRE, and Residuum names;
3. all local Markdown links resolve;
4. every remaining Dingo-branded occurrence is a literal technical identifier,
   historical statement, local path, or explicit compatibility note; and
5. no Rust or non-Markdown implementation artifact has been changed by this
   phase.
