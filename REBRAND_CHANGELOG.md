# Residiuum rebrand changelog (Phase 2)

Status: **Phase 2 implementation complete through REB-12 (2026-07-31);
unreleased pending a release tag**  
Normative plan: [REBRAND.md](REBRAND.md). Inventory: [doc/REBRAND_INVENTORY.md](doc/REBRAND_INVENTORY.md).  
Class C freeze: [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).

**Product name:** Residiuum only. **`Residuum` / `ResiduumDB` / `residuum-*`**
are incorrect unreleased intermediate spellings with no compatibility status.
**`residiuumdb`** is domain-only (`residiuumdb.org`, `docs.residiuumdb.org`).
Websites (`web/`) remain Phase 4.

## 1. Reason and effective release

**Why:** The working name DingoDB collides with an existing product. The
canonical product identity is **Residiuum**.

**Pre-release spelling correction:** An internal rebrand pass briefly used
`Residuum` / `residuum-*`. Those names were never released and have no
compatibility status. The canonical spelling contains the second `i`:
`Residiuum` / `residiuum-*`.

**What this cut does:** Implementation identity hard-break for crates, public
Rust API entry type, CLI, URI scheme, and process environment variables.

**What this cut does not do:** Rewrite wire magics, profile string IDs, crypto
domains, or website Phase 4 surfaces.

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

**Serialized / wire values remain frozen Class C (do not rename):**

| Rust symbol (public) | Frozen serialized value |
|----------------------|-------------------------|
| `RQL_APP_CORE_PROFILE` | `dql-app-core-v1` |
| `RQL_PLAN_PROFILE` / `PLAN_PROFILE` | `dql-plan-v1` |
| `PLAN_ENCODING_PROFILE` | `dql-plan-encoding-v1` |
| `PLAN_HASH_DOMAIN` | `dingo:dql-plan-v1:canonical-v1` |

Also retained: wire fixtures `dql_query.*`, error diagnostic
`dql_feature_unavailable`, accepted vectors under `spec/app/v1/` and
`spec/heap/`, and predicate/server profiles such as `dingo-predicate-v1`.

Obsolete public dialect aliases `"dql"` / `"dingo-ql"` are **not** offered.

## 8. Storage format and on-disk

| Item | Policy |
|------|--------|
| Frame magics `DINGOFRM` / `DINGOEND` | **Unchanged** (Class C) |
| Wire major/minor draft profile | Unchanged (`1.0-draft` still draft) |
| Example paths `*.dingo` | Still valid demos; not required to rename media |
| Migration jobs / store meta | Still readable by renamed binaries |

No on-disk rewrite is required solely for the package rename.

## 9. Wire and cluster

| Item | Policy |
|------|--------|
| RPC profile strings `dingo-rpc-v1` | **Unchanged** |
| Cluster/node URNs `urn:dingo:…` | **Unchanged** |
| Raft snapshot sentinel `__dingo_snapshot_base__` | **Unchanged** |
| Client multi-seed URL scheme | Now `residiuum://` only (hard break) |

## 10. HeapKey, tokens, crypto domains

| Item | Policy |
|------|--------|
| Domain separators containing historical `DINGODB` / frozen domains | **Unchanged** |
| Process auth token env name | Renamed to `RESIDIUUM_TOKEN` (operators must re-export secrets) |
| Wire profiles for heap | String ids remain `dingo-heap-v1` etc. |

## 11. Aliases and removal policy

No temporary `dingo` crate or CLI aliases. Removal N/A.

## 12. Operator upgrade and rollback

**Upgrade**

1. Rebuild from tree after Phase 2.
2. Replace `dingo` binary with `residiuum` on PATH.
3. Update service unit env: `DINGO_TOKEN` → `RESIDIUUM_TOKEN`.
4. Update client URLs: `dingo://` → `residiuum://`.
5. Update Cargo dependencies / imports as in §3–4.
6. Do **not** rewrite store segment bytes for branding.

**Rollback**

1. Redeploy pre-Phase-2 binaries and restore old env/URL spellings.
2. Stores remain readable (Class C unchanged).
3. Do not mix half-migrated client URL schemes against servers that only
   advertise one scheme.

## 13. Website / domain

Deferred to **Phase 4**. Local dirs `web/dingodb.org` and marketing copy still
contain “Dingo” strings. Canonical public hosts remain `residiuumdb.org` /
`docs.residiuumdb.org` per REBRAND.md.

## 14. Intentionally retained legacy identifiers

See [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).

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
| profile string bleed `rql-app-core-v1` / `rql-plan-v1` | none (restored to `dql-*`) |

### REB-9 (Class C audit)

| Check | Result |
|-------|--------|
| `cargo test -p residiuum-format` | exit 0 |
| `cargo test -p residiuum-heap` | exit 0 |
| `cargo test -p residiuum-store` | exit 0 (default features) |
| `cargo test -p residiuum-cluster` | exit 0 |
| Class C greps (magics, profiles, URNs, domains) | retained |
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
| Class C freeze greps | still retained (REB-9) |

**Incidental product fix during REB-12:** DEF-013 catalog frontier — when a
**new** durable collection name is first observed, persist the durable-only
collections catalog immediately (`note_collection_for_subject` →
`refresh_collection_catalog`). Index-cache checkpoints remain rate-limited
(DEF-023). Unblocked `stage_def_010_012_013` under workspace test.

Website Phase 4 and Phase 5 final audit remain out of scope.

### Residual classification summary (REB-7…REB-12)

| Class | Disposition | Examples |
|-------|-------------|---------|
| Class A/B | Renamed | crates, packages, `Residiuum`, `ResidiuumDeployment`, CLI, `residiuum://`, `RESIDIUUM_*`, RQL symbols |
| Class C | **retain_legacy** | profiles `dingo-*-v1` / frozen `dql-*-v1` wire values, magics, URNs, crypto domains, `application/dingo.*`, `.dingo` media, `dingo-store-*` |
| Class D | history / Phase 4 | `web/dingodb.org`, marketing strings, git remote `dingodb`, historical `DRE-*` work-package ids |
| Forbidden intermediate | documented only | `Residuum` / `residuum-*` (never shipped) |
| Cosmetic residual fixed in REB-7 | example paths | `/var/lib/residiuum`, `alias/residiuum-test`, fuzz package |

Full freeze list: [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).
