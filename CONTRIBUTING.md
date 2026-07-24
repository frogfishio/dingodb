# Contributing to DingoDB

## Spec before behavior

DingoDB is specified ahead of large parts of the code. Implementation work
should map to a stage in [DELIVERY_PLAN.md](DELIVERY_PLAN.md) and a section of
a normative document listed in [ARCHITECTURE.md](ARCHITECTURE.md).

If the change is not covered by a stage exit criterion or a named MUST in a
spec, open a short design note or amend the relevant spec first.

## Engineering rules

1. **SDA stays pure.** No file, network, or ambient IO in `crates/sda-core`.
2. **Authority before acceleration.** Catalogs and indexes must be rebuildable
   from segments; salvage must not depend on them.
3. **Damage honesty.** Corrupt candidates are never labeled verified. Holes
   are explicit; later islands still scan.
4. **Conformance is the gate.** Prefer golden and destructive tests over API
   surface alone.
5. **No cluster-before-salvage.** Stage 8 waits on Stage 2–4 gates.

## Workspace commands

```sh
# Full workspace
cargo test --workspace

# Stage 1: SDA library
cargo test -p sda-lib

# Stage 1: SDA CLI
cargo test -p sda

# Stage 2: wire format / frame codec
cargo test -p dingo-format

# Stage 3: single-node store
cargo test -p dingo-store

# Stage 4: embedded collection SDK
cargo test -p dingo-sdk

# Quick SDA eval
cargo run -p sda --bin sda -- eval -e '1 + 2'
echo '{"name":"Ada"}' | cargo run -p sda --bin sda -- eval -e 'input<"name">!'
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

## Suggested work streams (apportionment)

| Stream | Owner focus | Stages | Depends on |
|--------|-------------|--------|------------|
| **A — SDA** | Conformance corpus, §14 suite, determinism | 1 | Stage 0 |
| **B — Survival format** | Frames, segments, salvage scanner, holes | 2 | Stage 0; parallel after 1a |
| **C — Store** | Append journal, durability modes, open path | 3 | Stage 2a+ |
| **D — DX / SDK** | `Dingo.open`, collections, filters, errors | 4 | Stage 3 core put/get |
| **E — Examination** | ExaminationUnit + SDA profile | 5 | Stage 1 + 2/3 salvage |
| **F — Operator** | Indexes, CLI doctor/salvage, server | 6–7 | Stage 4–5 |

Immediate priority: **Stage 4c–4d** (filter builder, fuller error taxonomy).
Stage **4a–4b** (`dingo-sdk`: open, JSON/bytes put/get/delete, scan) and
Stage 3a–3c / 2a–2d are done. Optional: Stage 1 full §14 freeze and
deterministic CBOR envelope validation in parallel.

## Version and BUILD numbers

- `VERSION` — semantic version mirrored by `[workspace.package].version`
- `BUILD` — integer build stamp used by the `sda` CLI `--version` output
- `crates/sda-cli/BUILD` — must stay in sync with root `BUILD` for CLI tests

## License

Contributions are under the MIT License (see [LICENSE](LICENSE)).
