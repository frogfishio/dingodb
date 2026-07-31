# REBRAND Phase 2 inventory (REB-1)

Date: 2026-07-31  
Feature: REB (`da4cc5c7-0952-4fa7-9378-4b38b2089080`)  
Normative: [REBRAND.md](../REBRAND.md) §9 classes, §10 map  
Principal decisions: Class B hard break; Class C keep; repo rename out of scope

Baseline (REB-0): `cargo check --workspace` exit 0 @ commit `e090c05` (main).

## 1. Paths with the former `dingo` name

| Kind | Path | Class | Disposition |
|------|------|-------|-------------|
| dir | `crates/dingo-authority` | B | → `crates/residiuum-authority` (REB-2) |
| dir | `crates/dingo-cli` | B | → `crates/residiuum-cli` |
| dir | `crates/dingo-client` | B | → `crates/residiuum-client` |
| dir | `crates/dingo-cluster` | B | → `crates/residiuum-cluster` |
| dir | `crates/dingo-examine` | B | → `crates/residiuum-examine` |
| dir | `crates/dingo-format` | B | → `crates/residiuum-format` |
| dir | `crates/dingo-heap` | B | → `crates/residiuum-heap` |
| dir | `crates/dingo-sdk` | B | → `crates/residiuum-sdk` |
| dir | `crates/dingo-sdk/src/dingo` | B | → `crates/residiuum-sdk/src/residiuum` |
| dir | `crates/dingo-server` | B | → `crates/residiuum-server` |
| dir | `crates/dingo-store` | B | → `crates/residiuum-store` |
| dir | `crates/dingo-testrig` | B | → `crates/residiuum-testrig` |
| dir | `web/dingodb.org` | B* | Website Phase 4 preferred; path rename deferred unless forced |
| dir | `web/docs.dingodb.org` | B* | Website Phase 4 preferred; deferred |
| file | `web/docs.dingodb.org/.../choose-dingodb.md` | B* | Route rename Phase 4; deferred with web dirs |

\*Feature scope: website dirs deferred (Phase 4). Content path strings updated only if forced by crate renames.

## 2. Cargo packages and binaries (Class B → rename)

| Package (`name =`) | Dir | Intended package | Notable bins |
|--------------------|-----|------------------|--------------|
| `dingo-format` | `crates/dingo-format` | `residiuum-format` | lib |
| `dingo-heap` | `crates/dingo-heap` | `residiuum-heap` | lib |
| `dingo-authority` | `crates/dingo-authority` | `residiuum-authority` | `dingo-authority` → `residiuum-authority` |
| `dingo-store` | `crates/dingo-store` | `residiuum-store` | `dingo-store-crash-child` → `residiuum-store-crash-child` |
| `dingo-client` | `crates/dingo-client` | `residiuum-client` | lib |
| `dingo-sdk` | `crates/dingo-sdk` | `residiuum-sdk` | lib |
| `dingo-server` | `crates/dingo-server` | `residiuum-server` | lib |
| `dingo-examine` | `crates/dingo-examine` | `residiuum-examine` | lib |
| `dingo-cli` | `crates/dingo-cli` | `residiuum-cli` | **`dingo` → `residiuum`** |
| `dingo-cluster` | `crates/dingo-cluster` | `residiuum-cluster` | `dingo-cluster-multiproc-child` → `residiuum-…` |
| `dingo-testrig` | `crates/dingo-testrig` | `residiuum-testrig` | `dingo-testrig` → `residiuum-testrig` |
| `dingo-sda` | `crates/sda-core` | `residiuum-sda` | lib (dir name may stay `sda-core`) |
| `dingo-sda-cli` | `crates/sda-cli` | `residiuum-sda-cli` | **`dingo-sda` → `residiuum-sda`** |

Rust import roots after rename: `dingo_format` → `residiuum_format`, etc.
(REB-2 compile loop).

Also: `verification/heap-verus` package name `dingo-heap-verus` if present →
`residiuum-heap-verus`.

## 3. Public API surface (Class B — REB-3 / REB-8 / REB-10)

| Former | Implemented | Notes |
|---------|-------------|--------|
| `Dingo` type / `Dingo::open` | `Residiuum` / `Residiuum::open` | Hard break (REB-3) |
| `DingoDeployment` | `ResidiuumDeployment` | Hard break (REB-10) |
| `DingoConfigFile` | `ResidiuumConfigFile` | Hard break (REB-10) |
| `dingo://` URI | `residiuum://` | Hard break (REB-3) |
| `DINGO_*` env vars | `RESIDIUUM_*` | Hard break (REB-3); message strings fixed REB-10 |
| Feature flags / Cargo keywords `dingodb` | `residiuum` | Class B packaging (REB-10 keywords) |
| DQL public symbols (`Dql`, `compile_dql`, …) | RQL symbols (`Rql`, `compile_rql`, …) | REB-8; **serialized** profile values stay `dql-*` (Class C) |

## 4. Class C — keep legacy (REB-5 freeze; do not bulk-replace)

| Identifier | Role | Disposition |
|------------|------|-------------|
| `DINGOFRM` / `DINGOEND` | Frame magics | **retain_legacy** |
| `dingo-*-v1` wire/persist profiles (e.g. `dingo-heap-v1`, `dingo-cursor-v1`, `dingo-rpc-v1`, `dingo-config-v1`, …) | Protocol/profile strings | **retain_legacy** unless proven purely cosmetic packaging labels in REB-5 audit |
| Crypto domain separators containing `DINGODB` / `dingo:` in hash domains that bind keys/proofs | Crypto | **retain_legacy** |
| `urn:dingo:cluster:` / `urn:dingo:node:` | Identity URIs in TLS | **retain_legacy** or dual-read later (not this Feature hard break without REB-5 explicit exception) |
| Golden vectors / fixtures encoding above | Evidence | **retain_legacy** |
| `.dingo` store path conventions if any | On-disk | **retain_legacy** |
| Frozen DQL wire profiles (`dql-app-core-v1`, `dql-plan-v1`, `dql-plan-encoding-v1`, `dingo:dql-plan-v1:canonical-v1`, `dql_query.*`, `dql_feature_unavailable`) | App-core / plan wire | **retain_legacy** (paired with RQL Rust symbols after REB-8) |
| `dingo-store-*` / `dingo-store-9` store meta | On-disk store family | **retain_legacy** (readers still recognize family) |

**Rule:** Class A/B renames must not rewrite Class C. If a string is both a
package path reference and a profile id, change only the path/package form, not
the profile constant value. REB-8 exemplifies the correct pattern:
`RQL_APP_CORE_PROFILE = "dql-app-core-v1"`.

## 5. Class D — immutable history

| Item | Disposition |
|------|-------------|
| Released git tags / historical commit messages | leave |
| `github.com/frogfishio/dingodb` remote URL | out of scope (principal) |
| Published crates.io coordinates until re-publish | document in changelog |
| Old evidence ledgers / accepted audit artifacts | leave |

## 6. Scripts / CI (Class B path updates — REB-4)

Update after package renames: `scripts/*`, `.github/workflows/*`, demos that
use `cargo -p dingo-*` or `target/debug/dingo`.

## 7. Content hit scale (pre-REB-2)

Approx (excl `target` / `node_modules`): ~495 files / ~4k matches for `dingo`.
Not all rows; bulk is REB-2/3/4 grep-fix.

## 8. Task mapping

| Task | Consumes this inventory | Status (2026-07-31) |
|------|-------------------------|---------------------|
| REB-2 | §1 dirs (crates), §2 packages | **done** |
| REB-3 | §3 API/CLI/env/URI | **done** |
| REB-4 | §6 scripts/CI | **done** |
| REB-5 | §4 Class C freeze confirmation | **done** |
| REB-6 | changelog from all sections | **done** |
| REB-7 | residual search vs approved Class C/D | **done** |
| REB-8 | RQL public symbols + frozen DQL wire values | **done** |
| REB-9 | Class C re-audit after mechanical renames | **done** |
| REB-10 | public identity residual (types, CLI, keywords) | **done** |
| REB-11 | doc/changelog reconcile with reality | **done** |
| REB-12 | `cargo check/test --workspace` + final evidence | **done** |

## 9. REB-1 exit (historical)

Inventory complete for implementer handoff. No renames performed in REB-1 labor
beyond this document.

## 10. Post–REB-10 inventory note (REB-11)

Implemented Class B identity is Residiuum-named throughout crates (see §3).
Website directories under `web/` remain deferred (Phase 4). Wrong intermediate
spellings `Residuum` / `residuum-*` are documentation-only forbidden forms, not
compatibility aliases. Observed test evidence lives in
[REBRAND_CHANGELOG.md](../REBRAND_CHANGELOG.md) §15.
