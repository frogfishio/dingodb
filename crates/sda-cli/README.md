# sda

`sda` is the command-line interface for Structured Data Algebra.

It evaluates SDA programs over JSON input, validates source without executing
it, and emits canonical SDA formatting for editor and CI workflows.

## Status

**Shipped** with Stage 1. Library freeze tag `sda-standalone-v1.0` lives in
`sda-lib`; this binary is the shell front-end (`eval`, `check`, `fmt`).

## Install from this workspace

```sh
cargo install --path crates/sda-cli
```

## Commands

```sh
sda eval -e 'values(input)' < event.json
sda eval -f extract.sda -i event.json --compact
sda check -f extract.sda
sda fmt -f extract.sda --check
sda fmt -f extract.sda --write
sda --version
sda --license
```

## Exit behavior

- successful evaluation prints JSON to stdout
- validation and formatting failures exit nonzero with a readable error
- `check` prints `ok` on success
- `fmt --check` exits nonzero when source is not canonical

## Documentation

- Spec: [SDA_SPEC.md](../../SDA_SPEC.md)
- User docs: [doc/SDA/](../../doc/SDA/)
- Library: [crates/sda-core](../sda-core) (`sda-lib`)
- Delivery: Stage 1 **done** in [DELIVERY_PLAN.md](../../DELIVERY_PLAN.md)

## Library

Embed SDA in a Rust program with the `sda-lib` crate (`crates/sda-core`), not
by shelling out to this binary. For DingoDB recovery examination, use
[`dingo-examine`](../dingo-examine) (or `dingo doctor`).
