# dingo-sda-cli

`dingo-sda-cli` ships the **`dingo-sda`** binary — the command-line interface for
ResiduumDB's **SDA+ENR1 hybrid** evaluator.

It evaluates programs over JSON input, validates source without executing it,
and emits canonical formatting for editor and CI workflows.

> **Naming:** package `dingo-sda-cli`, binary `dingo-sda`. Do **not** publish or
> install under the bare crates.io name `sda` (that identity must not be reused).

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / CI: evaluate, check, format SDA+ENR1 | **`dingo-sda`** (this binary) |
| Embed evaluation in a Rust program | [`dingo-sda`](https://crates.io/crates/dingo-sda) |
| ResiduumDB recovery examination | [`dingo-examine`](https://crates.io/crates/dingo-examine) or `dingo doctor` |

## Install

From crates.io (once published under the new name):

```sh
cargo install dingo-sda-cli
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/sda-cli
```

## Commands

```sh
# Evaluate a program (expression or file) over JSON stdin / -i file
dingo-sda eval -e 'values(input)' < event.json
dingo-sda eval -f extract.sda -i event.json --compact

# Validate source without running it
dingo-sda check -f extract.sda

# Canonical format (check or write)
dingo-sda fmt -f extract.sda --check
dingo-sda fmt -f extract.sda --write

dingo-sda --version
dingo-sda --license
```

## Exit behavior

| Situation | Behavior |
|-----------|----------|
| Successful `eval` | JSON on stdout |
| Successful `check` | prints `ok` |
| Validation / format failures | nonzero exit, readable error |
| `fmt --check` and source not canonical | nonzero exit |

## Status

**Shipped.** Library freeze tag `sda-standalone-v1.0` (plus additive ENR1
`sda-enr1-v0.1`) lives in [`dingo-sda`](https://crates.io/crates/dingo-sda);
this binary is the shell front-end (`eval`, `check`, `fmt`).

## Library

Embed evaluation in a Rust program with the `dingo-sda` crate, not by shelling
out to this binary.

```toml
[dependencies]
dingo-sda = "0.1"
```

```rust
let out = dingo_sda::run("input<\"x\">!", serde_json::json!({"x": 1}))?;
# Ok::<(), dingo_sda::SdaError>(())
```

## Documentation

- Spec: [SDA_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/SDA_SPEC.md)
- User docs: [doc/SDA/](https://github.com/frogfishio/dingodb/tree/main/doc/SDA)
- Library: [`dingo-sda`](https://crates.io/crates/dingo-sda)

## License

MIT.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb).
