# Release artifacts (DEF-003)

Status: living policy for workspace release content  
Companion: [DEFECTS.md](../DEFECTS.md) DEF-003, [CONTRIBUTING.md](../CONTRIBUTING.md)

This document defines what ships in a ResiduumDB **workspace release** (the git
tree and CI package gates). Independent crates.io publication of every member
is a later concern (path deps are versioned so packaging is possible; they are
not yet the primary distribution channel).

## Workspace members (crate packages)

Every entry under `crates/` is a release crate. Package contents are controlled
by each crate’s `include` list in `Cargo.toml`.

| Directory | Package name | Binary | Notes |
|-----------|--------------|--------|-------|
| `crates/sda-core` | `residuum-sda` | — | SDA+ENR1 hybrid pure evaluator (not bare `sda`/`sda-lib`) |
| `crates/sda-cli` | `residuum-sda-cli` | `residuum-sda` | Hybrid evaluator CLI |
| `crates/residuum-format` | `residuum-format` | — | Wire format / salvage scan |
| `crates/residuum-store` | `residuum-store` | — | Includes `crash_matrix.v1.json` |
| `crates/residuum-client` | `residuum-client` | — | Wire framing / client protocol |
| `crates/residuum-sdk` | `residuum-sdk` | — | Collection API + dialects (incl. official `dql`) |
| `crates/residuum-server` | `residuum-server` | — | Network serve runtime |
| `crates/residuum-examine` | `residuum-examine` | — | Examination host |
| `crates/residuum-cli` | `residuum-cli` | `dingo` | Operator CLI |
| `crates/residuum-cluster` | `residuum-cluster` | — | In-process federation |
| `crates/residuum-testrig` | `residuum-testrig` | `residuum-testrig` | Stress rig (`publish = false`; workspace release only) |

**Package rules**

1. `include` must list every file required to build and test the crate.
2. Manifests must not reference paths that are absent from the package (no
   dangling `include` globs; `cargo package --list` must succeed).
3. Crate READMEs may link to monorepo specs (`../../FORMAT_SPEC.md`, etc.).
   Those links are valid in the **workspace release**, not inside a standalone
   crates.io tarball. Do not claim a crate is independently documented until
   its package includes those docs or the links are rewritten.

## Repository (non-crate) release artifacts

These ship with the workspace release and are **not** embedded in individual
crate tarballs:

### Normative specifications

- `OVERVIEW.md`, `FORMAT_SPEC.md`, `DX_SPEC.md`, `CLUSTER_SPEC.md`
- `SDA_SPEC.md`, `SDA_PROFILE.md`, `ARCHITECTURE.md`
- `doc/SDA/*` (grammar, doctrine, user-facing SDA notes)

### Product / operator documentation

- Root `README.md`, `CONTRIBUTING.md`, `LICENSE`, `LICENSE-MIT`,
  `LICENSE-MPL-2.0`, `LICENSE-AGPL-3.0`, `doc/LICENSING.md`, `VERSION`
- `doc/CAPABILITY_MATRIX.md`, `doc/CRASH_CONSISTENCY.md`
- `doc/RUNBOOK_RETENTION.md`, `doc/BENCHMARK_DISCLOSURE.md`
- `DEFECTS.md`, `DELIVERY_PLAN.md` (execution plan; not customer marketing)

### Human demos

Living scripts under `scripts/demos/`:

| Script | Role |
|--------|------|
| `02_punch_a_hole.sh` | Corrupt segment; doctor/salvage still speak |
| `03_salvage_survives.sh` | Wipe catalogs; salvage recovers live keys |
| `07_tier_move.sh` | Tier/archive + retention runbook |
| `08_kill_a_node.sh` | Multi-hop serve-cluster kill-node survivor |
| `README.md` | Index and run instructions |

Demos are release artifacts: they must remain runnable from a clean checkout
with a built `dingo` binary (or `DINGO_BIN`). They are not crates.io content.

### Tooling

- `.github/workflows/ci.yml`, `.github/workflows/nightly.yml`
- `scripts/nightly.sh`, `scripts/release_content.sh`
- `.cargo/config.toml` (workspace aliases)

## Explicitly not release artifacts

| Path | Why |
|------|-----|
| `target/` | Build output |
| `.gremlin/`, `.tinker/` | Local agent/session state |
| `JURISDICTION_PROPOSAL.md`, `TRANSACTIONS.md` | Design proposals; not product claims |
| Uncommitted local edits | Block release CI (`git status --short` empty) |

## Gates (how we enforce this)

```sh
# Local or CI (DEF-003)
./scripts/release_content.sh
```

The script:

1. Requires a clean git work tree (`git status --short` empty).
2. Runs `cargo package --list` for every workspace package and checks required
   entries (`Cargo.toml`, `README.md`, sources).
3. Assembles a temporary workspace from those package file lists (the same
   content that would appear in each crate tarball) and runs
   `cargo build --workspace --all-targets`, so unlisted files cannot be load-
   bearing.

**Why not `cargo package` verify against crates.io?** Internal path
dependencies are versioned so packaging *can* rewrite them later, but members
are not yet the primary distribution channel. Full crates.io verify would
resolve foreign packages. **Do not republish under bare `sda` / `sda-lib`** —
those names were misused for this hybrid surface; new publishes use
`residuum-sda` / `residuum-sda-cli` only. Mistaken hybrid `0.1.0` under the old
names was yanked on crates.io (2026-07-28); pre-existing pure-SDA `1.0.x`
under those names is out of scope for this monorepo. The workspace release
gate validates tarball *content completeness* via `--list` + rebuild, which
is the monorepo equivalent of “builds from the packaged tree.”

Main CI runs the script on every PR/push. Nightly continues to run heavier
corpora via `scripts/nightly.sh`.
