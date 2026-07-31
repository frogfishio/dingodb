# residuum-sda-cli

`residuum-sda-cli` ships the **`residuum-sda`** binary — the command-line interface for
ResiduumDB's **SDA+ENR1 hybrid** evaluator.

It evaluates programs over JSON input, validates source without executing it,
and emits canonical formatting for editor and CI workflows.

> **Naming:** package `residuum-sda-cli`, binary `residuum-sda`. Do **not** publish or
> install under the bare crates.io name `sda` (that identity must not be reused).

## When to use this package

| You want… | Use |
|-----------|-----|
| Shell / CI: evaluate, check, format SDA+ENR1 | **`residuum-sda`** (this binary) |
| Embed evaluation in a Rust program | [`residuum-sda`](https://crates.io/crates/residuum-sda) |
| ResiduumDB recovery examination | [`residuum-examine`](https://crates.io/crates/residuum-examine) or `dingo doctor` |

## Install

From crates.io (once published under the new name):

```sh
cargo install residuum-sda-cli
```

From a local checkout of the monorepo:

```sh
cargo install --path crates/sda-cli
```

## Commands

```sh
# Evaluate a program (expression or file) over JSON stdin / -i file
residuum-sda eval -e 'values(input)' < event.json
residuum-sda eval -f extract.sda -i event.json --compact

# Validate source without running it
residuum-sda check -f extract.sda

# Canonical format (check or write)
residuum-sda fmt -f extract.sda --check
residuum-sda fmt -f extract.sda --write

residuum-sda --version
residuum-sda --license
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
`sda-enr1-v0.1`) lives in [`residuum-sda`](https://crates.io/crates/residuum-sda);
this binary is the shell front-end (`eval`, `check`, `fmt`).

## Library

Embed evaluation in a Rust program with the `residuum-sda` crate, not by shelling
out to this binary.

```toml
[dependencies]
residuum-sda = "0.1"
```

```rust
let out = residuum_sda::run("input<\"x\">!", serde_json::json!({"x": 1}))?;
# Ok::<(), residuum_sda::SdaError>(())
```

## Documentation

- Spec: [SDA_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/SDA_SPEC.md)
- User docs: [doc/SDA/](https://github.com/frogfishio/dingodb/tree/main/doc/SDA)
- Library: [`residuum-sda`](https://crates.io/crates/residuum-sda)

## License

MIT.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb).
