# sda

`sda` is the command-line interface for **Structured Data Algebra (SDA)**.

It evaluates SDA programs over JSON input, validates source without executing
it, and emits canonical SDA formatting for editor and CI workflows.

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / CI: evaluate, check, format SDA | **`sda`** (this binary) |
| Embed SDA in a Rust program | [`sda-lib`](https://crates.io/crates/sda-lib) |
| DingoDB recovery examination | [`dingo-examine`](https://crates.io/crates/dingo-examine) or `dingo doctor` |

## Install

From crates.io:

```sh
cargo install sda
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/sda-cli
```

## Commands

```sh
# Evaluate a program (expression or file) over JSON stdin / -i file
sda eval -e 'values(input)' < event.json
sda eval -f extract.sda -i event.json --compact

# Validate source without running it
sda check -f extract.sda

# Canonical format (check or write)
sda fmt -f extract.sda --check
sda fmt -f extract.sda --write

sda --version
sda --license
```

## Exit behavior

| Situation | Behavior |
|-----------|----------|
| Successful `eval` | JSON on stdout |
| Successful `check` | prints `ok` |
| Validation / format failures | nonzero exit, readable error |
| `fmt --check` and source not canonical | nonzero exit |

## Status

**Shipped.** Library freeze tag `sda-standalone-v1.0` lives in
[`sda-lib`](https://crates.io/crates/sda-lib); this binary is the shell
front-end (`eval`, `check`, `fmt`).

## Library

Embed SDA in a Rust program with the `sda-lib` crate, not by shelling out to
this binary.

```toml
[dependencies]
sda-lib = "0.1"
```

```rust
let out = sda_lib::run("input<\"x\">!", serde_json::json!({"x": 1}))?;
# Ok::<(), sda_lib::SdaError>(())
```

## Documentation

- Spec: [SDA_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/SDA_SPEC.md)
- User docs: [doc/SDA/](https://github.com/frogfishio/dingodb/tree/main/doc/SDA)
- Library: [`sda-lib`](https://crates.io/crates/sda-lib)

## License

MIT.

Part of [DingoDB](https://github.com/frogfishio/dingodb).
