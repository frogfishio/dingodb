# residiuum-sda-cli

`residiuum-sda-cli` ships the **`residiuum-sda`** binary — the command-line interface for
Residiuum's **SDA+ENR1 hybrid** evaluator.

It evaluates programs over JSON input, validates source without executing it,
and emits canonical formatting for editor and CI workflows.

> **Naming:** package `residiuum-sda-cli`, binary `residiuum-sda`. Do **not** publish or
> install under the bare crates.io name `sda` (that identity must not be reused).

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / CI: evaluate, check, format SDA+ENR1 | **`residiuum-sda`** (this binary) |
| Embed evaluation in a Rust program | [`residiuum-sda`](https://crates.io/crates/residiuum-sda) |
| Residiuum recovery examination | [`residiuum-examine`](https://crates.io/crates/residiuum-examine) or `residiuum doctor` |

## Install

From crates.io (once published under the new name):

```sh
cargo install residiuum-sda-cli
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/sda-cli
```

## Commands

```sh
# Evaluate a program (expression or file) over JSON stdin / -i file
residiuum-sda eval -e 'values(input)' < event.json
residiuum-sda eval -f extract.sda -i event.json --compact

# Validate source without running it
residiuum-sda check -f extract.sda

# Canonical format (check or write)
residiuum-sda fmt -f extract.sda --check
residiuum-sda fmt -f extract.sda --write

residiuum-sda --version
residiuum-sda --license
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
`sda-enr1-v0.1`) lives in [`residiuum-sda`](https://crates.io/crates/residiuum-sda);
this binary is the shell front-end (`eval`, `check`, `fmt`).

## Library

Embed evaluation in a Rust program with the `residiuum-sda` crate, not by shelling
out to this binary.

```toml
[dependencies]
residiuum-sda = "0.1"
```

```rust
let out = residiuum_sda::run("input<\"x\">!", serde_json::json!({"x": 1}))?;
# Ok::<(), residiuum_sda::SdaError>(())
```

## Documentation

- Spec: [SDA_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/SDA_SPEC.md)
- User docs: [doc/SDA/](https://github.com/frogfishio/dingodb/tree/main/doc/SDA)
- Library: [`residiuum-sda`](https://crates.io/crates/residiuum-sda)

## License

MIT.

Part of [Residiuum](https://github.com/frogfishio/dingodb).
