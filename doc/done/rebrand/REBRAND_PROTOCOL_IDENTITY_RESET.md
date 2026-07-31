# Residiuum protocol identity reset

Date: 2026-07-31  
Status: **authorized pre-release hard reset**  
Compatibility policy: **none**

## 1. Decision

Residiuum is a new, unreleased product. No production installation, published
wire contract, or supported on-disk format depends on the former Dingo working
name.

Consequently, no former product identifier is frozen for compatibility. The
earlier `retain_legacy` decision is revoked in full.

Residiuum readers, writers, servers, clients, tools, fixtures, and qualification
profiles MUST emit and accept only the Residiuum identities defined by the
current source and specifications. No alias, dual-read path, compatibility
reader, or migration feature is required for former pre-release identities.

## 2. Reset scope

| Identity class | Former examples | Residiuum identity |
|---|---|---|
| Format magics | `DINGOFRM`, `DINGOEND`, D-prefixed component magics | `RESIDFRM`, `RESIDEND`, R-prefixed component magics |
| Profiles and policies | `dingo-heap-v1`, `dingo-rpc-v1`, `dingo-core-storage-v1` | `residiuum-heap-v1`, `residiuum-rpc-v1`, `residiuum-core-storage-v1` |
| Query language wire surface | `dql`, `dql_query`, `dql-*-v1` | `rql`, `rql_query`, `rql-*-v1` |
| Rule language wire surface | `dre`, `DRE-*`, `*-dre-*` | `rre`, `RRE-*`, `*-rre-*` |
| Identity URIs | `urn:dingo:cluster:*`, `urn:dingo:node:*` | `urn:residiuum:cluster:*`, `urn:residiuum:node:*` |
| Hash and MAC domains | former `dingo:` and `DINGODB-*` domains | `residiuum:` and `RESIDIUUM-*` domains |
| Internal subjects and metadata | former `__dingo_*`, `dingo-store-*` values | `__residiuum_*`, `residiuum-store-*` values |
| Media and examples | `*.dingo` | `*.residiuum` |
| MIME/application identifiers | former `application/dingo.*` values | `application/residiuum.*` values |

The frame and component magics retain their existing byte widths. This changes
identity, not structural layout.

## 3. Deliberate invalidation

The reset deliberately invalidates every pre-release artifact whose bytes,
hashes, authentication, or interpretation depend on a former identity,
including:

- test stores and segments;
- manifests, indexes, checkpoints, cursors, and continuation tokens;
- Heap keys, certificates, holder proofs, and authority artifacts;
- TLS identities and stored cluster metadata;
- backup, migration, scrub, and evidence artifacts;
- canonical query/rule artifacts and their hashes;
- golden vectors, fixtures, and qualification bundles.

They may be deleted and regenerated. A failure to read them is expected and is
not a compatibility defect.

## 4. Permitted historical references

Only these references may retain the former name:

1. rebrand history that explicitly labels it as the former working name;
2. the current repository URL or local checkout path while infrastructure still
   uses that name; and
3. website redirects whose sole purpose is forwarding obsolete public routes.

None of those references may appear as an accepted product, protocol, storage,
security, profile, API, CLI, environment, or qualification identity.

## 5. Qualification gate

This reset precedes Core Storage Qualification.

`CSQ-0` MUST register:

```text
residiuum-core-storage-v1
```

and the qualification command is:

```text
residiuum verify --profile residiuum-core-storage-v1 --level A2
```

CSQ evidence MUST be generated after this reset. Evidence derived from former
identities is inadmissible.

## 6. Acceptance

The reset is complete only when:

- active code, specifications, fixtures, scripts, and product documentation
  contain no former product/protocol identity;
- every identity-dependent golden vector has been regenerated;
- format, store, Heap, SDK, server, and CLI tests pass against the new values;
- document-link and website builds pass; and
- the repository records clearly state that no pre-release compatibility is
  promised.

