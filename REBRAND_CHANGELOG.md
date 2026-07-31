# ResiduumDB rebrand changelog (Phase 2)

Status: **implemented (REB-2…REB-6 labor complete, 2026-07-31)**  
Becomes normative after REB-7 quality evidence and principal accept.  
Normative plan: [REBRAND.md](REBRAND.md). Inventory: [doc/REBRAND_INVENTORY.md](doc/REBRAND_INVENTORY.md).  
Class C freeze: [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).

## 1. Reason and effective release

**Why:** The working name DingoDB collides with an existing product. Canonical
product identity is **ResiduumDB** / **Residuum** (docs Phase 1 already complete).

**What this cut does:** Implementation identity hard-break for crates, public
Rust API entry type, CLI, URI scheme, and process environment variables.

**What this cut does not do:** Rewrite wire magics, profile string IDs, crypto
domains, or website Phase 4 surfaces.

Effective release: **unreleased** until principal tags; apply when upgrading
from a pre-Phase-2 tree.

## 2. Public name table (hard break — no aliases)

| Former | New |
|--------|-----|
| Cargo packages `dingo-*` | `residuum-*` |
| Crate dirs `crates/dingo-*` | `crates/residuum-*` |
| Published `dingo-sda` (dir `crates/sda-core`) | `residuum-sda` |
| `dingo-sda-cli` / bin `dingo-sda` | `residuum-sda-cli` / bin `residuum-sda` |
| Type `Dingo` | `Residuum` |
| `Dingo::open` / `connect` / `open_deployment` / … | `Residuum::…` |
| CLI binary `dingo` | `residuum` |
| URI `dingo://host:port[/label]` | `residuum://host:port[/label]` |
| Env `DINGO_*` (token, fuzz, quality, crash matrix, …) | `RESIDUUM_*` |
| `parse_dingo_url` / `ParsedDingoUrl` | `parse_residuum_url` / `ParsedResiduumUrl` |

**Compatibility aliases:** none in this cut.

## 3. Rust dependency and import migration

**Cargo.toml (consumer):**

```toml
# before
dingo-sdk = { path = "path/to/crates/dingo-sdk" }
# or
dingo-sdk = { version = "0.2.0", ... }

# after
residuum-sdk = { path = "path/to/crates/residuum-sdk" }
# or
residuum-sdk = { version = "0.2.0", ... }
```

**Rust imports:**

```rust
// before
use dingo_sdk::{Dingo, Filter, json};

// after
use residuum_sdk::{Residuum, Filter, json};
```

Workspace members and `[workspace.dependencies]` use `residuum-*` paths.

## 4. API / type migration

| Before | After |
|--------|-------|
| `Dingo::open(path)` | `Residuum::open(path)` |
| `Dingo::connect(url)` | `Residuum::connect(url)` |
| `Dingo::open_deployment` / `create_deployment` | same methods on `Residuum` |
| `Dingo::connect_heap` | `Residuum::connect_heap` |

Module path `dingo_sdk::dingo` → `residuum_sdk::residuum` (usually use crate root re-export).

## 5. CLI and executables

| Before | After |
|--------|-------|
| `dingo serve` | `residuum serve` |
| `dingo serve-cluster` | `residuum serve-cluster` |
| `dingo doctor` / `config` / `migrate` / `scrub` / … | `residuum …` |
| `dingo-sda` | `residuum-sda` |
| `dingo-testrig` | `residuum-testrig` |
| `dingo-authority` | `residuum-authority` |
| `dingo-cluster-multiproc-child` | `residuum-cluster-multiproc-child` |
| `dingo-store-crash-child` | `residuum-store-crash-child` |

Build: `cargo build -p residuum-cli --bin residuum`.

## 6. URI, environment, configuration

| Before | After |
|--------|-------|
| `dingo://127.0.0.1:7434/app` | `residuum://127.0.0.1:7434/app` |
| `DINGO_TOKEN` | `RESIDUUM_TOKEN` (`DEFAULT_TOKEN_ENV`) |
| `DINGO_FUZZ_*`, `DINGO_QUALITY_*`, `DINGO_CRASH_MATRIX_FULL`, … | `RESIDUUM_*` equivalents |
| `DINGO_MP_*` (multiproc child harness) | `RESIDUUM_MP_*` |
| `DINGO_CRASH_*` (crash-matrix child) | `RESIDUUM_CRASH_*` |
| `DINGO_S3_ROOT` / `DINGO_GS_ROOT` / `DINGO_AWS_*` / `DINGO_KMS_*` | `RESIDUUM_*` equivalents |
| `env:DINGO_TOKEN` secret refs | `env:RESIDUUM_TOKEN` |

Config files that referenced old env names must be updated.

## 7. RQL / RRE naming

Docs already use RQL/RRE. Implementation package renames do not rename the
language profiles. Predicate/plan profiles retain `dingo-predicate-v1` /
related Class C strings until a separate migration.

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
| Client multi-seed URL scheme | Now `residuum://` only (hard break) |

## 10. HeapKey, tokens, crypto domains

| Item | Policy |
|------|--------|
| Domain separators containing historical `DINGODB` / frozen domains | **Unchanged** |
| Process auth token env name | Renamed to `RESIDUUM_TOKEN` (operators must re-export secrets) |
| Wire profiles for heap | String ids remain `dingo-heap-v1` etc. |

## 11. Aliases and removal policy

No temporary `dingo` crate or CLI aliases. Removal N/A.

## 12. Operator upgrade and rollback

**Upgrade**

1. Rebuild from tree after Phase 2.
2. Replace `dingo` binary with `residuum` on PATH.
3. Update service unit env: `DINGO_TOKEN` → `RESIDUUM_TOKEN`.
4. Update client URLs: `dingo://` → `residuum://`.
5. Update Cargo dependencies / imports as in §3–4.
6. Do **not** rewrite store segment bytes for branding.

**Rollback**

1. Redeploy pre-Phase-2 binaries and restore old env/URL spellings.
2. Stores remain readable (Class C unchanged).
3. Do not mix half-migrated client URL schemes against servers that only
   advertise one scheme.

## 13. Website / domain

Deferred to **Phase 4**. Local dirs `web/dingodb.org` and marketing copy still
contain “Dingo” strings. Canonical public hosts remain `residuumdb.org` /
`docs.residuumdb.org` per REBRAND.md.

## 14. Intentionally retained legacy identifiers

See [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).

Also Class D: git history, release tags, remote `github.com/frogfishio/dingodb`
(principal out of scope for this Feature).

## 15. Evidence (REB-6/7)

| Check | Result |
|-------|--------|
| `cargo check --workspace` | exit 0 (2026-07-31, post REB-7 residual fixes) |
| `cargo test -p residuum-format --lib` | 58/58 (REB-2) |
| `cargo test -p residuum-sdk --lib` | 64/64 (REB-2) |
| multiproc residual (DEF-041-N) | 6/6 earlier same day |
| Magics `DINGOFRM`/`DINGOEND` present | yes (Class C) |
| Fuzz package name | `residuum-fuzz` (was `dingo-fuzz`) |

### Residual classification summary (REB-7)

| Class | Disposition | Examples |
|-------|-------------|---------|
| Class A/B | Renamed | crates, packages, `Residuum`, CLI, `residuum://`, `RESIDUUM_*` |
| Class C | **retain_legacy** | profiles `dingo-*-v1`, magics, URNs, crypto domains, `application/dingo.heap-*`, `.dingo` media |
| Class D | history / Phase 4 | `web/dingodb.org`, marketing strings, git remote `dingodb` |
| Cosmetic residual fixed in REB-7 | example paths | `/var/lib/residuum`, `alias/residuum-test`, fuzz package |

Full freeze list: [doc/REBRAND_CLASS_C_FREEZE.md](doc/REBRAND_CLASS_C_FREEZE.md).