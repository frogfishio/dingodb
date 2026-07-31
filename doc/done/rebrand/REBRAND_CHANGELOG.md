# Residiuum rebrand changelog (Phase 2)

Status: **Phase 2 implementation complete through REB-12 (2026-07-31);
unreleased pending a release tag**
Normative plan: [REBRAND.md](./REBRAND.md). Inventory: [doc/done/rebrand/REBRAND_INVENTORY.md](./REBRAND_INVENTORY.md).
Protocol identity reset:
[doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md](./REBRAND_PROTOCOL_IDENTITY_RESET.md).

**Product name:** Residiuum only. **`Residuum` / `ResiduumDB` / `residuum-*`**
are incorrect unreleased intermediate spellings with no compatibility status.
**`residiuumdb`** is domain-only (`residiuumdb.org`, `docs.residiuumdb.org`).
Websites (`web/`) Phase 4 is implemented in-repo (Feature WEB); see §13.

## 1. Reason and effective release

**Why:** The working name DingoDB collides with an existing product. The
canonical product identity is **Residiuum**.

**Pre-release spelling correction:** An internal rebrand pass briefly used
`Residuum` / `residuum-*`. Those names were never released and have no
compatibility status. The canonical spelling contains the second `i`:
`Residiuum` / `residiuum-*`.

**What this cut does:** A complete pre-release identity hard break for crates,
public APIs, CLI, URI scheme, environment variables, profiles, magics, storage
metadata, wire identities, query/rule identities, URNs, MIME identifiers, and
cryptographic domains.

Effective release: **unreleased** until principal tags; apply when upgrading
from a pre-Phase-2 tree.

## 2. Public name table (hard break — no aliases)

| Former | New |
|--------|-----|
| Cargo packages `dingo-*` | `residiuum-*` |
| Crate dirs `crates/dingo-*` | `crates/residiuum-*` |
| Published `dingo-sda` (dir `crates/sda-core`) | `residiuum-sda` |
| `dingo-sda-cli` / bin `dingo-sda` | `residiuum-sda-cli` / bin `residiuum-sda` |
| Type `Dingo` | `Residiuum` |
| `Dingo::open` / `connect` / `open_deployment` / … | `Residiuum::…` |
| Type `DingoDeployment` | `ResidiuumDeployment` |
| Type `DingoConfigFile` | `ResidiuumConfigFile` |
| CLI binary `dingo` | `residiuum` |
| URI `dingo://host:port[/label]` | `residiuum://host:port[/label]` |
| Env `DINGO_*` (token, fuzz, quality, crash matrix, …) | `RESIDIUUM_*` |
| `parse_dingo_url` / `ParsedDingoUrl` | `parse_residiuum_url` / `ParsedResidiuumUrl` |
| Cargo package keywords `dingodb` | `residiuum` |

**Compatibility aliases:** none in this cut.

## 3. Rust dependency and import migration

**Cargo.toml (consumer):**

```toml
# before
dingo-sdk = { path = "path/to/crates/dingo-sdk" }
# or
dingo-sdk = { version = "0.2.0", ... }

# after
residiuum-sdk = { path = "path/to/crates/residiuum-sdk" }
# or
residiuum-sdk = { version = "0.2.0", ... }
```

**Rust imports:**

```rust
// before
use dingo_sdk::{Dingo, Filter, json};

// after
use residiuum_sdk::{Residiuum, Filter, json};
```

Workspace members and `[workspace.dependencies]` use `residiuum-*` paths.

## 4. API / type migration

| Before | After |
|--------|-------|
| `Dingo::open(path)` | `Residiuum::open(path)` |
| `Dingo::connect(url)` | `Residiuum::connect(url)` |
| `Dingo::open_deployment` / `create_deployment` | same methods on `Residiuum` → `ResidiuumDeployment` |
| `Dingo::connect_heap` | `Residiuum::connect_heap` |
| `DingoDeployment` | `ResidiuumDeployment` |
| `DingoConfigFile` | `ResidiuumConfigFile` |

Module path `dingo_sdk::dingo` → `residiuum_sdk::residiuum` (usually use crate root re-export).

## 5. CLI and executables

| Before | After |
|--------|-------|
| `dingo serve` | `residiuum serve` |
| `dingo serve-cluster` | `residiuum serve-cluster` |
| `dingo doctor` / `config` / `migrate` / `scrub` / … | `residiuum …` |
| `dingo-sda` | `residiuum-sda` |
| `dingo-testrig` | `residiuum-testrig` |
| `dingo-authority` | `residiuum-authority` |
| `dingo-cluster-multiproc-child` | `residiuum-cluster-multiproc-child` |
| `dingo-store-crash-child` | `residiuum-store-crash-child` |

Build: `cargo build -p residiuum-cli --bin residiuum`.

## 6. URI, environment, configuration

| Before | After |
|--------|-------|
| `dingo://127.0.0.1:7434/app` | `residiuum://127.0.0.1:7434/app` |
| `DINGO_TOKEN` | `RESIDIUUM_TOKEN` (`DEFAULT_TOKEN_ENV`) |
| `DINGO_FUZZ_*`, `DINGO_QUALITY_*`, `DINGO_CRASH_MATRIX_FULL`, … | `RESIDIUUM_*` equivalents |
| `DINGO_MP_*` (multiproc child harness) | `RESIDIUUM_MP_*` |
| `DINGO_CRASH_*` (crash-matrix child) | `RESIDIUUM_CRASH_*` |
| `DINGO_S3_ROOT` / `DINGO_GS_ROOT` / `DINGO_AWS_*` / `DINGO_KMS_*` | `RESIDIUUM_*` equivalents |
| `env:DINGO_TOKEN` secret refs | `env:RESIDIUUM_TOKEN` |

Config files that referenced old env names must be updated.

## 7. RQL / RRE naming

**Product / language identity:** Residiuum Query Language (**RQL**) and
Residiuum Rule Expression (**RRE**).

**Rust public surface (Class B, REB-8):** symbols and dialect ids use RQL names,
for example `BuiltinDialect::Rql`, dialect id `"rql"`, `compile_rql`,
`parse_rql`, `RqlProgram`, `CollectionClient::rql` / `explain_rql`,
`RQL_APP_CORE_PROFILE`, `RQL_PLAN_PROFILE`, console terminology **RQL**, and
source links to `RQL_SPEC.md`.

**Serialized / wire values reset with the product:**

| Rust symbol (public) | Serialized value |
|----------------------|------------------|
| `RQL_APP_CORE_PROFILE` | `rql-app-core-v1` |
| `RQL_PLAN_PROFILE` / `PLAN_PROFILE` | `rql-plan-v1` |
| `PLAN_ENCODING_PROFILE` | `rql-plan-encoding-v1` |
| `PLAN_HASH_DOMAIN` | `residiuum:rql-plan-v1:canonical-v1` |

Wire fixtures, diagnostics, accepted vectors, predicate profiles, and server
profiles use only Residiuum/RQL identities. No former dialect aliases are
offered.

## 8. Storage format and on-disk

| Item | Policy |
|------|--------|
| Frame magics | Reset to `RESIDFRM` / `RESIDEND` |
| Wire major/minor draft profile | Unchanged (`1.0-draft` still draft) |
| Example paths | `*.residiuum` |
| Migration jobs / store meta | Residiuum identity only |

Former test stores are intentionally unreadable; no migration is supplied.

## 9. Wire and cluster

| Item | Policy |
|------|--------|
| RPC profile strings | `residiuum-rpc-v1` |
| Cluster/node URNs | `urn:residiuum:…` |
| Raft snapshot sentinel | `__residiuum_snapshot_base__` |
| Client multi-seed URL scheme | Now `residiuum://` only (hard break) |

## 10. HeapKey, tokens, crypto domains

| Item | Policy |
|------|--------|
| Former `DINGODB-*` domain separators | **Hard reset to `RESIDIUUM-*`; old pre-release artifacts invalidated** |
| Process auth token env name | Renamed to `RESIDIUUM_TOKEN` (operators must re-export secrets) |
| Wire profiles for heap | `residiuum-heap-v1` and related Residiuum ids |

## 11. Aliases and removal policy

No temporary `dingo` crate or CLI aliases. Removal N/A.

## 12. Operator upgrade and rollback

**Upgrade**

1. Rebuild from tree after Phase 2.
2. Replace `dingo` binary with `residiuum` on PATH.
3. Update service unit env: `DINGO_TOKEN` → `RESIDIUUM_TOKEN`.
4. Update client URLs: `dingo://` → `residiuum://`.
5. Update Cargo dependencies / imports as in §3–4.
6. Delete and recreate pre-reset test stores.

**Rollback**

Rollback across the identity reset is unsupported. Restore a complete
pre-reset test environment if historical experiments must be inspected.

## 13. Website / domain

**Phase 4 (Feature WEB, WEB-0…WEB-7) — implemented in-repo 2026-07-31.**

| Former | New |
|--------|-----|
| Local dirs `web/dingodb.org`, `web/docs.dingodb.org` | `web/residiuumdb.org`, `web/docs.residiuumdb.org` |
| package/astro `dingodb.org` / `docs.dingodb.org` | `residiuumdb.org` / `docs.residiuumdb.org` |
| Product copy DingoDB / Residuum / ResiduumDB | **Residiuum** |
| Public languages DQL / DRE (site surface) | **RQL** / **RRE** |
| Docs routes `/guides/dql/`, `/concepts/dre/`, … | `/guides/rql/`, `/concepts/rre/`, … |
| `/getting-started/choose-dingodb/` | `/getting-started/choose-residiuum/` |
| Main `/docs/*` redirect target | `https://docs.residiuumdb.org/:splat` |

Canonical public hosts: `https://residiuumdb.org`, `https://docs.residiuumdb.org`.

**Hosting freeze:** `web/residiuumdb.org/.openai/hosting.json` `project_id`
`appgprj_6a6a4bd6baf08191949a0106278a04d8` **unchanged**.

**Protocol identity on web:** capability tables and operator documentation use
the Residiuum profile identifiers.

**Class D left:** `github.com/frogfishio/dingodb` remote links.

**Principal ops remaining:** DNS/CDN cutover for `dingodb.org` /
`docs.dingodb.org`; `www.residiuumdb.org` → apex 301; accept WEB cards to `done`.

Inventory: [doc/done/rebrand/WEB_REBRAND_INVENTORY.md](./WEB_REBRAND_INVENTORY.md).

## 14. Intentionally retained legacy identifiers

See [doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md](./REBRAND_PROTOCOL_IDENTITY_RESET.md).

Also Class D: git history, release tags, remote `github.com/frogfishio/dingodb`
(principal out of scope for this Feature).

## 15. Evidence (observed only)

### REB-2…REB-7 (mechanical rename + residual)

| Check | Result |
|-------|--------|
| `cargo check --workspace` | exit 0 (2026-07-31, post REB-7 residual fixes) |
| `cargo test -p residiuum-format --lib` | 58/58 (REB-2) |
| `cargo test -p residiuum-sdk --lib` | 64/64 (REB-2) |
| multiproc residual (DEF-041-N) | 6/6 earlier same day |
| Magics `DINGOFRM`/`DINGOEND` present | yes (Class C) |
| Fuzz package name | `residiuum-fuzz` (was `dingo-fuzz`) |

### REB-8 (RQL public surface)

| Check | Result |
|-------|--------|
| `cargo check --workspace` | exit 0 |
| `cargo test -p residiuum-sdk` | exit 0 |
| `cargo test -p residiuum-cli --test console` | exit 0 (1/1) |
| residual `Dql` / `compile_dql` / `DQL_APP_CORE_PROFILE` rg | empty |
| RQL serialized identities | reset to `rql-*` |

### REB-9 (protocol identity reset)

| Check | Result |
|-------|--------|
| `cargo test -p residiuum-format` | exit 0 |
| `cargo test -p residiuum-heap` | exit 0 |
| `cargo test -p residiuum-store` | exit 0 (default features) |
| `cargo test -p residiuum-cluster` | exit 0 |
| Former identity greps (magics, profiles, URNs, domains) | absent from active surfaces |
| `stage_def_010_012_013` with `legacy-raw-store` | exit 0 (5/5; DEF-013 residual fixed) |

### REB-10 (public identity residual)

| Check | Result |
|-------|--------|
| `cargo check -p residiuum-sdk -p residiuum-server -p residiuum-cli -p residiuum-store` | exit 0 |
| `app1_collection_create` | 4/4 |
| `cpr001_legacy_opt_in` | 3/3 |
| `hp007_heap_isolation` | 3/3 |
| `stage_def_054_config` | 7/7 |
| `residiuum-cli --test console` | 1/1 |
| public `DingoDeployment` / `DingoConfigFile` / `dingo://` in crates | none remaining |

### REB-12 (final verification)

| Check | Result |
|-------|--------|
| `cargo check --workspace` | exit 0 |
| `cargo test --workspace` | exit 0 (~1265 tests passed, 0 failed across package targets) |
| residual public identity in `crates/**/*.rs` (`DingoDeployment`, `DingoConfigFile`, `dingo://`) | none |
| protocol identity reset greps | still retained (REB-9) |

**Incidental product fix during REB-12:** DEF-013 catalog frontier — when a
**new** durable collection name is first observed, persist the durable-only
collections catalog immediately (`note_collection_for_subject` →
`refresh_collection_catalog`). Index-cache checkpoints remain rate-limited
(DEF-023). Unblocked `stage_def_010_012_013` under workspace test.

Website Phase 4 is implemented (Feature WEB / §13). Phase 5 website re-audit
after principal accept and DNS cutover remains open.

### Residual classification summary (REB-7…REB-12)

| Class | Disposition | Examples |
|-------|-------------|---------|
| Class A/B | Renamed | crates, packages, `Residiuum`, `ResidiuumDeployment`, CLI, `residiuum://`, `RESIDIUUM_*`, RQL symbols |
| Former Class C | **hard reset** | Residiuum/RQL/RRE profiles, magics, URNs, domains, MIME ids, media and store metadata |
| Class D | history only | git remote `github.com/frogfishio/dingodb`; website redirects |
| Forbidden intermediate | documented only | `Residuum` / `residuum-*` (never shipped) |
| Cosmetic residual fixed in REB-7 | example paths | `/var/lib/residiuum`, `alias/residiuum-test`, fuzz package |

Full reset contract: [doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md](./REBRAND_PROTOCOL_IDENTITY_RESET.md).
