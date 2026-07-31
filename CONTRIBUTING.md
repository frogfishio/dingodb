# Contributing to Residiuum

## Spec before behavior

Implementation work should map to a named section of a normative document
listed in [ARCHITECTURE.md](ARCHITECTURE.md). Stages **0–9** and product
follow-ons 1–4 are already landed (see [DELIVERY_PLAN.md](DELIVERY_PLAN.md)
and the root [README.md](README.md) status table).

If the change is not covered by an existing stage exit criterion, freeze label,
or a named MUST in a spec, open a short design note or amend the relevant spec
first.

Work selection and package order are governed by
[MASTER_DELIVERY_PLAN.md](MASTER_DELIVERY_PLAN.md). Do not select the next
feature from `DELIVERY_PLAN.md`, `DEFECTS.md`, or an individual subsystem plan
without checking the master starting queue and stage gate first.

All new claims and stable capabilities require the assurance chain in
[TESTING_STRATEGY.md](TESTING_STRATEGY.md). Example-based tests alone do not
qualify persistence, isolation, concurrency, or recovery claims.

## Engineering rules

1. **SDA stays pure.** No file, network, or ambient IO in `crates/sda-core`.
2. **Authority before acceleration.** Catalogs and indexes must be rebuildable
   from segments; salvage must not depend on them.
3. **Damage honesty.** Corrupt candidates are never labeled verified. Holes
   are explicit; later islands still scan.
4. **Conformance is the gate.** Prefer golden and destructive tests over API
   surface alone.
5. **Cluster does not own the bytes.** Node directories remain ordinary
   `residiuum-store` salvage targets without cluster software.

## Workspace commands

```sh
# Full workspace
cargo test --workspace

# SDA library (Stage 1 freeze)
cargo test -p residiuum-sda

# SDA CLI
cargo test -p residiuum-sda-cli

# Wire format / frame codec
cargo test -p residiuum-format

# Single-node store (+ Stage 6 / 9 suites)
cargo test -p residiuum-store

# Collection SDK (+ remote / cluster routing tests)
cargo test -p residiuum-sdk

# SDA examination
cargo test -p residiuum-examine

# Operator CLI
cargo test -p dingo

# Cluster federation
cargo test -p residiuum-cluster

# Quick SDA eval
cargo run -p residiuum-sda-cli --bin residiuum-sda -- eval -e '1 + 2'
echo '{"name":"Ada"}' | cargo run -p residiuum-sda-cli --bin residiuum-sda -- eval -e 'input<"name">!'

# CLI help
cargo run -p dingo --bin residiuum -- --help
```

Aliases (see `.cargo/config.toml`):

```sh
cargo test-all
cargo sda-test
cargo sda-run -- eval -e '1 + 2'
```

## Formatting and lint

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings   # when clippy is installed
```

CI runs format check, tests, and the DEF-003 release-content gate on every PR.

### Release content (DEF-003)

Before a release slice lands, the work tree must be clean and every workspace
crate must package completely:

```sh
./scripts/release_content.sh
```

What counts as a release artifact (crates, specs, demos) is defined in
[doc/RELEASE_ARTIFACTS.md](doc/RELEASE_ARTIFACTS.md). Local dry-runs on a dirty
tree: `RESIDIUUM_RELEASE_ALLOW_DIRTY=1 ./scripts/release_content.sh`.

### Nightly packaging

Destructive corpora and the performance skeleton also run on a schedule:

```sh
./scripts/nightly.sh
```

GitHub Actions: `.github/workflows/nightly.yml` (daily + `workflow_dispatch`).

Human demos: [scripts/demos/](scripts/demos/) (workspace release artifacts).

## Current product surface (apportionment)

Stages **0–9** are **implemented in-tree** (not production-qualified). Follow-ons
1–4: S3/GCS **filesystem mirrors**, experimental multi-hop `serve-cluster`
routing, freeze labels, lifecycle/erasure scaffolds + benchmark disclosure.

| Area | Crate | Notes |
|------|-------|--------|
| SDA+ENR1 | `residiuum-sda` / `residiuum-sda-cli` | Conformance-locked `sda-standalone-v1.0` + ENR1 profile |
| Wire | `residiuum-format` | `WIRE_PROFILE_LABEL` = `1.0-draft` (freeze: `doc/WIRE_MAJOR1_FREEZE.md`) |
| Store | `residiuum-store` | Authority + tiers + media mirrors (early-access) |
| SDK | `residiuum-sdk` | `SDK_API_VERSION` = `1.0` |
| Examination | `residiuum-examine` | Profile over salvage |
| CLI | `residiuum` | doctor / salvage / development `serve` / experimental `serve-cluster` |
| Cluster | `residiuum-cluster` | `CLUSTER_PROFILE_VERSION` = `v1` (**in-process**) |

Immediate package selection, priority, and release order:
[MASTER_DELIVERY_PLAN.md](MASTER_DELIVERY_PLAN.md). Production-readiness defects
remain in [DEFECTS.md](DEFECTS.md); product maturity rules remain in
[doc/PRIME_TIME_PLAN.md](doc/PRIME_TIME_PLAN.md). Network Raft qualification,
native cloud SDKs, and erasure codecs remain later programs.

Capability matrix: [doc/CAPABILITY_MATRIX.md](doc/CAPABILITY_MATRIX.md).

## Version and BUILD numbers

- `VERSION` / crate semver (`0.2.0`) — packaging only; **not** a maturity claim
- `BUILD` — integer build stamp used by CLI `--version` output
- `crates/sda-cli/BUILD` and `crates/residiuum-cli/BUILD` — keep in sync with root
  `BUILD` when CLI version tests require it

Freeze labels (product API/profile labels, **not** crate semver):

- `SDK_API_VERSION` (`residiuum-sdk`) = `1.0` — collection API surface
- `CLUSTER_PROFILE_VERSION` (`residiuum-cluster`) = `v1` — in-process cluster only
- `WIRE_PROFILE_LABEL` (`residiuum-format`) = `1.0-draft` — draft wire bytes;
  freeze checklist `doc/WIRE_MAJOR1_FREEZE.md` (DEF-053; not frozen)
- `CONFORMANCE_CORPUS_TAG` (`residiuum-sda`) = `sda-standalone-v1.0`

## License

Residiuum is multi-licensed (MIT / MPL-2.0 / AGPL-3.0-or-later by crate). See
[LICENSE](LICENSE) and [doc/LICENSING.md](doc/LICENSING.md).

**Inbound = outbound:** by contributing, you license your contribution under
the same license(s) that apply to the files you modify (and any SPDX declared
on the containing crate). Do not add material you cannot offer under those
terms.