# DingoDB fuzz targets (DEF-091 / DEF-091-F)

Untrusted parser surfaces for continuous / scheduled fuzzing. This package is
**not** a workspace member; build it with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Continuous policy

| Layer | When | Command |
|-------|------|---------|
| Property / hostile unit tests | Every PR (`quality.sh`) | `DINGO_FUZZ_SKIP_CARGO_FUZZ=1 ./scripts/fuzz-smoke.sh` |
| cargo-fuzz smoke | Nightly + `scripts/nightly.sh` | `./scripts/fuzz-smoke.sh` (30s/target in CI) |
| Deep / OSS-Fuzz | Residual | Longer budgets; land crashes under `fuzz/corpus/<target>/` |

## Targets

| Binary | Surface | Package |
|--------|---------|---------|
| `decode_frame` | frame decode/verify | residuum-format |
| `cbor_envelope` | deterministic CBOR envelope | residuum-format |
| `scan_forward` / `scan_reverse` | salvage scanners | residuum-format |
| `heap_ownership` | subject/ownership/heap descriptors | residuum-format |
| `sda_parse` | SDA program lex/parse | residuum-sda |
| `rpc_frame` | length-prefixed RPC framing | residuum-client |
| `chunk_manifest` | chunked-value manifest decode | residuum-store |
| `item_envelope` | item event envelope CBOR | residuum-store |
| `backup_manifest` | backup control JSON | residuum-store |
| `cursor_token` | continuation-token MAC decode | residuum-store |

## Local run

```bash
cargo install cargo-fuzz
# cargo-fuzz currently wants a nightly toolchain for the fuzz profile
./scripts/fuzz-smoke.sh
# or one target:
cargo +nightly fuzz run decode_frame -- -max_total_time=30
```

## Corpus policy

Crashes must be minimized and landed under `fuzz/corpus/<target>/` (or as a
regression unit test next to the decoder) before the finding is closed. Seed
files under `fuzz/corpus/` are optional starters for coverage.

Ownership: store/format/client/SDA maintainers for their decoder crates; CI
ownership is the `fuzz_smoke` nightly job + `scripts/fuzz-smoke.sh`.
