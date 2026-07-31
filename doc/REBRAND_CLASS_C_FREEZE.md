# Class C freeze list (REB-5)

Date: 2026-07-31  
Feature: REB (`da4cc5c7-0952-4fa7-9378-4b38b2089080`)  
Principal policy: **retain legacy** for this Feature (no dual-read / hard break)  
Companion: [REBRAND.md](../REBRAND.md) §9.3, [REBRAND_INVENTORY.md](REBRAND_INVENTORY.md)

These identifiers are **protocol, on-disk, or cryptographic facts**. Phase 2
Class A/B renames must not overwrite them. A future Feature may introduce
versioned dual-read; that is out of REB scope.

## 1. Frame magics (on-disk / salvage)

| Identifier | Role | Disposition |
|------------|------|-------------|
| `DINGOFRM` | Start-of-frame magic (8 ASCII bytes) | **retain_legacy** |
| `DINGOEND` | End-of-frame magic (8 ASCII bytes) | **retain_legacy** |

Evidence: `crates/residiuum-format/src/frame.rs` (`START_MAGIC` / `END_MAGIC`).

## 2. Profile / policy string IDs (`dingo-*-v1` family)

Examples still live in tree (non-exhaustive; ~360 matches of `dingo-…-v1`):

| Identifier (examples) | Role | Disposition |
|-----------------------|------|-------------|
| `dingo-heap-v1` | Heap qualification / authority profile | **retain_legacy** |
| `dingo-cursor-v1` | Authenticated continuation profile | **retain_legacy** |
| `dingo-rpc-v1` / `RPC_WIRE_LABEL` draft strings | Network RPC profile | **retain_legacy** |
| `dingo-predicate-v1` | Shared predicate profile label | **retain_legacy** |
| `dingo-rust-app-v1` | APP-0 façade profile label | **retain_legacy** |
| `dingo-query-plan-v1` | Legacy filter plan profile | **retain_legacy** |
| `dingo-config-v1`, `dingo-log-v1`, `dingo-metrics-v1`, `dingo-health-v1`, … | Server process profiles | **retain_legacy** |
| `dingo-migrate-v1`, `dingo-backup-v1`, `dingo-scrub-v1`, … | Control-document profiles | **retain_legacy** |
| `dingo-supported-versions-v1`, `dingo-security-audit-package-v1`, `dingo-crash-recovery-v1` | Policy document ids | **retain_legacy** |
| `dingo-wire-major1-freeze-v1` | Wire freeze policy id | **retain_legacy** |
| Plan hash domains containing historical `dingo:` labels where already frozen | Crypto domain sep | **retain_legacy** (do not rewrite frozen vectors) |

**Rule:** Package/crate names use `residiuum-*`; **profile constant string values**
keep `dingo-…` until a versioned migration Feature lands.

## 3. Identity URIs

| Identifier | Role | Disposition |
|------------|------|-------------|
| `urn:dingo:cluster:{id}` | Cluster TLS / peer identity | **retain_legacy** |
| `urn:dingo:node:{id}` | Node TLS / peer identity | **retain_legacy** |

## 4. On-disk / store conventions

| Identifier | Role | Disposition |
|------------|------|-------------|
| `*.dingo` path / open path examples | Historical / demo store path suffix | **retain_legacy** (examples may still use `.dingo`) |
| Store meta / segment layouts keyed by wire major 1 | On-disk | Bound to wire Class C above |

## 5. Internal control subjects

| Identifier | Role | Disposition |
|------------|------|-------------|
| `__dingo_snapshot_base__` | Raft snapshot base subject sentinel | **retain_legacy** (persisted log identity) |

## 6. Cryptographic / domain separators

| Identifier | Role | Disposition |
|------------|------|-------------|
| Literals containing `DINGODB` in domain-separation strings (specs/code) | Crypto domain | **retain_legacy** |
| Any BLAKE3/MAC domain already used in production fixtures | Integrity domains | **retain_legacy** |

Changing a domain separator creates a new cryptographic domain and may
invalidate keys, tokens, or proofs. Not in REB Phase 2.

## 7. Explicitly **not** Class C (already hard-broken in REB-2/3)

| Former | New | Notes |
|--------|-----|--------|
| Crates `dingo-*` | `residiuum-*` | Package identity |
| Type `Dingo` | `Residiuum` | Public API |
| URI `dingo://` | `residiuum://` | Client URL scheme |
| Env `DINGO_*` | `RESIDIUUM_*` | Process config |
| CLI bin `dingo` | `residiuum` | Operator command |

## 8. Verification (REB-5 exit)

After REB-2/3, confirm:

```bash
# Magics still present
rg -n 'DINGOFRM|DINGOEND' crates/residiuum-format/src/frame.rs
# Profiles still string-valued dingo-*
rg -n 'dingo-cursor-v1|dingo-rust-app-v1' crates/residiuum-sdk/src
# No accidental magic rewrite
rg -n 'RESIDIUUMFRM|RESIDIUUMEND' crates || true  # should be empty
```

Accidental Class C rewrites found during audit → **revert immediately** and
file under REB-7 residual.

## 9. Future work (out of this Feature)

- Dual-read wire major or profile aliases if product requires non-`dingo` labels.
- Website Phase 4: `web/dingodb.org` copy still says “Dingo” (marketing, not wire).
- Repo remote `github.com/frogfishio/dingodb` rename (principal: out of scope).
