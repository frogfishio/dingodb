# Contributing to DingoDB

## Spec before behavior

Implementation work should map to a named section of a normative document
listed in [ARCHITECTURE.md](ARCHITECTURE.md). Stages **0–9** and product
follow-ons 1–4 are already landed (see [DELIVERY_PLAN.md](DELIVERY_PLAN.md)
and the root [README.md](README.md) status table).

If the change is not covered by an existing stage exit criterion, freeze label,
or a named MUST in a spec, open a short design note or amend the relevant spec
first.

## Engineering rules

1. **SDA stays pure.** No file, network, or ambient IO in `crates/sda-core`.
2. **Authority before acceleration.** Catalogs and indexes must be rebuildable
   from segments; salvage must not depend on them.
3. **Damage honesty.** Corrupt candidates are never labeled verified. Holes
   are explicit; later islands still scan.
4. **Conformance is the gate.** Prefer golden and destructive tests over API
   surface alone.
5. **Cluster does not own the bytes.** Node directories remain ordinary
   `dingo-store` salvage targets without cluster software.

## Workspace commands

```sh
# Full workspace
cargo test --workspace

# SDA library (Stage 1 freeze)
cargo test -p sda-lib

# SDA CLI
cargo test -p sda

# Wire format / frame codec
cargo test -p dingo-format

# Single-node store (+ Stage 6 / 9 suites)
cargo test -p dingo-store

# Collection SDK (+ remote / cluster routing tests)
cargo test -p dingo-sdk

# SDA examination
cargo test -p dingo-examine

# Operator CLI
cargo test -p dingo

# Cluster federation
cargo test -p dingo-cluster

# Quick SDA eval
cargo run -p sda --bin sda -- eval -e '1 + 2'
echo '{"name":"Ada"}' | cargo run -p sda --bin sda -- eval -e 'input<"name">!'

# CLI help
cargo run -p dingo --bin dingo -- --help
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

CI runs format check and tests on every PR.

### Nightly packaging

Destructive corpora and the performance skeleton also run on a schedule:

```sh
./scripts/nightly.sh
```

GitHub Actions: `.github/workflows/nightly.yml` (daily + `workflow_dispatch`).

Human demos: [scripts/demos/](scripts/demos/).

## Current product surface (apportionment)

Stages **0–9** are implemented (single-node through cluster federation and
filesystem tiering/archive). Follow-ons 1–4 landed: S3/GCS mirrors, multi-hop
`serve-cluster`, freeze labels, lifecycle/erasure scaffolds + benchmark
disclosure.

| Area | Crate | Notes |
|------|-------|--------|
| SDA | `sda-lib` / `sda` | Frozen `sda-standalone-v1.0` |
| Wire | `dingo-format` | `WIRE_PROFILE_LABEL` = `1.0-draft` |
| Store | `dingo-store` | Authority + tiers + media mirrors |
| SDK | `dingo-sdk` | `SDK_API_VERSION` = `1.0` |
| Examination | `dingo-examine` | Profile over salvage |
| CLI | `dingo` | doctor / salvage / serve / serve-cluster |
| Cluster | `dingo-cluster` | `CLUSTER_PROFILE_VERSION` = `v1` |

Immediate follow-on priorities (not stage gates): network Raft log shipping
over TCP (multi-hop client routing is already shipped), optional native cloud
object SDKs beyond the mirror seam, and erasure encode/decode codecs.

## Version and BUILD numbers

- `VERSION` — semantic version mirrored by `[workspace.package].version`
- `BUILD` — integer build stamp used by CLI `--version` output
- `crates/sda-cli/BUILD` and `crates/dingo-cli/BUILD` — keep in sync with root
  `BUILD` when CLI version tests require it

Freeze labels (product, not crate semver):

- `SDK_API_VERSION` (`dingo-sdk`) = `1.0`
- `CLUSTER_PROFILE_VERSION` (`dingo-cluster`) = `v1`
- `WIRE_PROFILE_LABEL` (`dingo-format`) = `1.0-draft`
- `CONFORMANCE_CORPUS_TAG` (`sda-lib`) = `sda-standalone-v1.0`

## License

Contributions are under the MIT License (see [LICENSE](LICENSE)).
