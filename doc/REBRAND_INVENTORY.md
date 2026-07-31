# REBRAND Phase 2 inventory (REB-1)

Date: 2026-07-31  
Feature: REB (`da4cc5c7-0952-4fa7-9378-4b38b2089080`)  
Normative: [REBRAND.md](../REBRAND.md) §9 classes, §10 map  
Principal decisions: Class B hard break; Class C keep; repo rename out of scope

Baseline (REB-0): `cargo check --workspace` exit 0 @ commit `e090c05` (main).

## 1. Paths with `residuum` in the name

| Kind | Path | Class | Disposition |
|------|------|-------|-------------|
| dir | `crates/residuum-authority` | B | → `crates/residuum-authority` (REB-2) |
| dir | `crates/residuum-cli` | B | → `crates/residuum-cli` |
| dir | `crates/residuum-client` | B | → `crates/residuum-client` |
| dir | `crates/residuum-cluster` | B | → `crates/residuum-cluster` |
| dir | `crates/residuum-examine` | B | → `crates/residuum-examine` |
| dir | `crates/residuum-format` | B | → `crates/residuum-format` |
| dir | `crates/residuum-heap` | B | → `crates/residuum-heap` |
| dir | `crates/residuum-sdk` | B | → `crates/residuum-sdk` |
| dir | `crates/residuum-sdk/src/residuum` | B | → `crates/residuum-sdk/src/residuum` |
| dir | `crates/residuum-server` | B | → `crates/residuum-server` |
| dir | `crates/residuum-store` | B | → `crates/residuum-store` |
| dir | `crates/residuum-testrig` | B | → `crates/residuum-testrig` |
| dir | `web/dingodb.org` | B* | Website Phase 4 preferred; path rename deferred unless forced |
| dir | `web/docs.dingodb.org` | B* | Website Phase 4 preferred; deferred |
| file | `web/docs.dingodb.org/.../choose-dingodb.md` | B* | Route rename Phase 4; deferred with web dirs |

\*Feature scope: website dirs deferred (Phase 4). Content path strings updated only if forced by crate renames.

## 2. Cargo packages and binaries (Class B → rename)

| Package (`name =`) | Dir | Intended package | Notable bins |
|--------------------|-----|------------------|--------------|
| `residuum-format` | `crates/residuum-format` | `residuum-format` | lib |
| `residuum-heap` | `crates/residuum-heap` | `residuum-heap` | lib |
| `residuum-authority` | `crates/residuum-authority` | `residuum-authority` | `residuum-authority` → `residuum-authority` |
| `residuum-store` | `crates/residuum-store` | `residuum-store` | `residuum-store-crash-child` → `residuum-store-crash-child` |
| `residuum-client` | `crates/residuum-client` | `residuum-client` | lib |
| `residuum-sdk` | `crates/residuum-sdk` | `residuum-sdk` | lib |
| `residuum-server` | `crates/residuum-server` | `residuum-server` | lib |
| `residuum-examine` | `crates/residuum-examine` | `residuum-examine` | lib |
| `residuum-cli` | `crates/residuum-cli` | `residuum-cli` | **`residuum` → `residuum`** |
| `residuum-cluster` | `crates/residuum-cluster` | `residuum-cluster` | `residuum-cluster-multiproc-child` → `residuum-…` |
| `residuum-testrig` | `crates/residuum-testrig` | `residuum-testrig` | `residuum-testrig` → `residuum-testrig` |
| `residuum-sda` | `crates/sda-core` | `residuum-sda` | lib (dir name may stay `sda-core`) |
| `residuum-sda-cli` | `crates/sda-cli` | `residuum-sda-cli` | **`residuum-sda` → `residuum-sda`** |

Rust import roots after rename: `residuum_format` → `residuum_format`, etc. (REB-2 compile loop).

Also: `verification/heap-verus` package name `residuum-heap-verus` if present → `residuum-heap-verus`.

## 3. Public API surface (Class B — REB-3)

| Current | Intended | Notes |
|---------|----------|--------|
| `Residuum` type / `Residuum::open` | `Residuum` / `Residuum::open` | Hard break |
| `residuum://` URI | `residuum://` | Hard break |
| `DINGO_*` env vars | `RESIDUUM_*` | Hard break; inventory exact keys in REB-3 |
| Feature flags naming product `residuum` | `residuum` where Class B | Not Class C profiles |

## 4. Class C — keep legacy (REB-5 freeze; do not bulk-replace)

| Identifier | Role | Disposition |
|------------|------|-------------|
| `DINGOFRM` / `DINGOEND` | Frame magics | **retain_legacy** |
| `dingo-*-v1` wire/persist profiles (e.g. `residuum-heap-v1`, `dingo-cursor-v1`, `dingo-rpc-v1`, `dingo-config-v1`, …) | Protocol/profile strings | **retain_legacy** unless proven purely cosmetic packaging labels in REB-5 audit |
| Crypto domain separators containing `DINGODB` / `dingo:` in hash domains that bind keys/proofs | Crypto | **retain_legacy** |
| `urn:dingo:cluster:` / `urn:dingo:node:` | Identity URIs in TLS | **retain_legacy** or dual-read later (not this Feature hard break without REB-5 explicit exception) |
| Golden vectors / fixtures encoding above | Evidence | **retain_legacy** |
| `.dingo` store path conventions if any | On-disk | **retain_legacy** |

**Rule:** REB-2/3 must not rewrite Class C. If a string is both a package path reference and a profile id, change only the path/package form, not the profile constant value.

## 5. Class D — immutable history

| Item | Disposition |
|------|-------------|
| Released git tags / historical commit messages | leave |
| `github.com/frogfishio/dingodb` remote URL | out of scope (principal) |
| Published crates.io coordinates until re-publish | document in changelog |
| Old evidence ledgers / accepted audit artifacts | leave |

## 6. Scripts / CI (Class B path updates — REB-4)

Update after package renames: `scripts/*`, `.github/workflows/*`, demos that `cargo -p dingo-*` or `target/debug/residuum`.

## 7. Content hit scale (pre-REB-2)

Approx (excl `target` / `node_modules`): ~495 files / ~4k matches for `residuum`. Not all rows; bulk is REB-2/3/4 grep-fix.

## 8. Task mapping

| Task | Consumes this inventory |
|------|-------------------------|
| REB-2 | §1 dirs (crates), §2 packages |
| REB-3 | §3 API/CLI/env/URI |
| REB-4 | §6 scripts/CI |
| REB-5 | §4 Class C freeze confirmation |
| REB-6 | changelog from all sections |
| REB-7 | residual search vs approved Class C/D |

## 9. REB-1 exit

Inventory complete for implementer handoff. No renames performed in REB-1 labor beyond this document.
