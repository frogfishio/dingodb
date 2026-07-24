# sda

`sda` is the command-line interface for Structured Data Algebra.

It evaluates SDA programs over JSON input, validates source without executing
it, and emits canonical SDA formatting for editor and CI workflows.

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
```

## Exit behavior

- successful evaluation prints JSON to stdout
- validation and formatting failures exit nonzero with a readable error
- `check` prints `ok` on success
- `fmt --check` exits nonzero when source is not canonical

## Documentation

- Spec: [SDA_SPEC.md](../../SDA_SPEC.md)
- User docs: [doc/SDA/](../../doc/SDA/)
- Library: [crates/sda-core](../sda-core)

## Library

Embed SDA in a Rust program with the `sda-lib` crate (`crates/sda-core`), not
by shelling out to this binary.
