# sda-lib

`sda-lib` is the Rust library for **Structured Data Algebra (SDA)** — a pure
examination and transformation language used by DingoDB and usable on its own
over JSON.

It provides a small host-facing API for parsing, validating, formatting, and
evaluating standalone SDA programs. Evaluation is pure: no file or network IO
lives in this crate.

> **Package name:** published on crates.io as **`sda-lib`**. The source lives
> under `crates/sda-core` in the monorepo; dependents import the crate as
> `sda_lib`.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Embed SDA evaluation in a Rust program | **`sda-lib`** (this crate) |
| Shell CLI (`eval` / `check` / `fmt`) | [`sda`](https://crates.io/crates/sda) |
| DingoDB recovery examination units | [`dingo-examine`](https://crates.io/crates/dingo-examine) |

## Install

```toml
[dependencies]
sda-lib = "0.1"
```

Or: `cargo add sda-lib`

## Quick example

```rust
let output = sda_lib::run(
    r#"input<"name">!"#,
    serde_json::json!({"name": "Ada"}),
)?;
assert_eq!(
    output,
    serde_json::json!({"$type": "ok", "$value": "Ada"})
);
# Ok::<(), sda_lib::SdaError>(())
```

Bind host input under a name other than `input` with
`run_with_input_binding`.

## API surface

| Function | Role |
|----------|------|
| `run` | Evaluate an SDA program against JSON bound as `input` |
| `run_with_input_binding` | Evaluate against a caller-chosen binding name |
| `check` | Parse and validate source without evaluating |
| `format_source` | Emit canonical SDA formatting |
| `from_json` / `to_json` | Bridge between JSON and SDA values |
| `CONFORMANCE_CORPUS_TAG` | Freeze identity for standalone semantics (`sda-standalone-v1.0`) |

## Status

**Shipped.** Standalone behavior is frozen under
`sda_lib::CONFORMANCE_CORPUS_TAG` (`sda-standalone-v1.0`). Semantic changes
require a new corpus tag.

DingoDB recovery examination (ExaminationUnit projection) lives in
[`dingo-examine`](https://crates.io/crates/dingo-examine), not here — this crate
stays pure.

## Conformance

```sh
cargo test -p sda-lib
```

Golden vectors and the automated suite lock §14 MUST behavior under the
conformance corpus tag.

## Documentation

- Spec: [SDA_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/SDA_SPEC.md)
- User docs: [doc/SDA/](https://github.com/frogfishio/dingodb/tree/main/doc/SDA)
- DingoDB examination profile: [SDA_PROFILE.md](https://github.com/frogfishio/dingodb/blob/main/SDA_PROFILE.md)
- CLI package: [`sda`](https://crates.io/crates/sda)

## License

MIT.

Part of [DingoDB](https://github.com/frogfishio/dingodb).
