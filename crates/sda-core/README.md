# sda-lib

`sda-lib` is the Rust library for Structured Data Algebra.

It provides a small host-facing API for parsing, validating, formatting, and evaluating standalone SDA programs over JSON values.

The library is intended for host applications that want SDA semantics without shelling out to the CLI. It keeps evaluation pure and leaves file IO, process control, and orchestration to the caller.

## Install

```toml
[dependencies]
sda-lib = "1"
serde_json = "1"
```

## Example

```rust
let output = sda_lib::run("input<\"name\">!", serde_json::json!({"name": "Ada"}))?;
assert_eq!(output, serde_json::json!({"$type": "ok", "$value": "Ada"}));
# Ok::<(), sda_lib::SdaError>(())
```

If you want to bind host input under a name other than `input`, use `run_with_input_binding`.

## API Surface

- `run`: evaluate an SDA program against JSON bound as `input`
- `run_with_input_binding`: evaluate against a caller-chosen binding name
- `check`: parse and validate source without evaluating it
- `format_source`: emit canonical SDA formatting
- `from_json` / `to_json`: bridge between JSON and SDA values

## Documentation

- Repository: https://github.com/frogfishio/axiom
- docs.rs crate docs: https://docs.rs/sda-lib
- Formal specification: https://github.com/frogfishio/axiom/blob/main/SDA/SDA_SPEC.md
- User manual: https://github.com/frogfishio/axiom/blob/main/SDA/USER_MANUAL.md
- jq guide: https://github.com/frogfishio/axiom/blob/main/SDA/FOR_JQ_USERS.md

## CLI

If you want the Unix-facing tool instead of the embedded library, install the `sda` crate.