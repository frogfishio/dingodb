# sda

`sda` is the command-line interface for Structured Data Algebra.

It evaluates SDA programs over JSON input, validates source without executing it, and emits canonical SDA formatting for editor and CI workflows.

SDA is a deterministic language for structured-data reduction. The CLI is intended for shell use, CI checks, ETL glue, fixture replay, and jq-like JSON reshaping where exact semantics matter.

## Install

```sh
cargo install sda
```

## Commands

```sh
sda eval -e 'values(input)' < event.json
sda eval -f extract.sda -i event.json --compact
sda check -f extract.sda
sda fmt -f extract.sda --check
sda fmt -f extract.sda --write
```

## Install From Source

```sh
cargo install --path crates/sda-cli
```

## What SDA Is For

- deterministic transformation over structured data
- exact numeric semantics
- explicit success and failure values
- stable formatting and validation in automation

## Exit Behavior

- successful evaluation prints JSON to stdout
- validation and formatting failures exit nonzero with a readable error
- `check` prints `ok` on success
- `fmt --check` exits nonzero when source is not canonical

## Documentation

- Repository: https://github.com/frogfishio/axiom
- docs.rs package page: https://docs.rs/sda
- User manual: https://github.com/frogfishio/axiom/blob/main/SDA/USER_MANUAL.md
- Cheat sheet: https://github.com/frogfishio/axiom/blob/main/SDA/CHEATSHEET.md
- jq guide: https://github.com/frogfishio/axiom/blob/main/SDA/FOR_JQ_USERS.md
- Formal specification: https://github.com/frogfishio/axiom/blob/main/SDA/SDA_SPEC.md

## Library

If you want to embed SDA in a Rust program rather than shell out to the CLI, use the `sda-lib` crate.