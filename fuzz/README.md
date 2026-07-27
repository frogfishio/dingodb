# DingoDB fuzz targets (DEF-091)

Untrusted parser surfaces for continuous / scheduled fuzzing. This package is
**not** a workspace member; build it with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Targets

| Binary | Surface |
|--------|---------|
| `decode_frame` | `decode_frame` / `verify_frame_at` |
| `cbor_envelope` | `validate_deterministic_cbor_envelope` |
| `scan_forward` | forward salvage scan |
| `scan_reverse` | reverse salvage scan |

## Local run

```bash
cargo install cargo-fuzz
# cargo-fuzz currently wants a nightly toolchain for the fuzz profile
cargo +nightly fuzz run decode_frame -- -max_total_time=30
cargo +nightly fuzz run cbor_envelope -- -max_total_time=30
cargo +nightly fuzz run scan_forward -- -max_total_time=30
cargo +nightly fuzz run scan_reverse -- -max_total_time=30
```

## CI / nightly

Property tests in `crates/dingo-format/tests/stage_def_091_properties.rs` run on
every PR (`cargo test`). Nightly runs a short smoke pass over these targets when
`cargo-fuzz` is available (see `.github/workflows/nightly.yml`).

## Corpus policy

Crashes must be minimized and landed under `fuzz/corpus/<target>/` (or as a
regression unit test in `dingo-format`) before the finding is closed.
