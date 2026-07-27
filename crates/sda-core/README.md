# dingo-sda

`dingo-sda` is **DingoDB's hybrid pure evaluator** for Structured Data Algebra
(SDA) plus the additive **ENR1** enrichment kernel. It is a DingoDB monorepo
package — not a claim on the bare crates.io names `sda` / `sda-lib`.

It provides a small host-facing API for parsing, validating, formatting, and
evaluating programs over JSON. Evaluation is pure: no file or network IO lives
in this crate.

> **Package name:** publish and depend as **`dingo-sda`**. Source lives under
> `crates/sda-core`. Inside this package the Rust path is `dingo_sda`; workspace
> dependents that use the `sda-core` key import as `sda_core`.

## Why not `sda` / `sda-lib`?

Earlier publish under the bare names collided with (and must not reuse) those
crates.io package identities. This crate is also **not pure standalone SDA
only**: ENR1 (`Match` / `enrich` / `one?` / `one!` / `merge`, …) shares the same
compile path. Treat it as DingoDB's SDA+ENR1 hybrid surface.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Embed SDA+ENR1 evaluation in a Rust program | **`dingo-sda`** (this crate) |
| Shell CLI (`eval` / `check` / `fmt`) | [`dingo-sda-cli`](https://crates.io/crates/dingo-sda-cli) (`dingo-sda` binary) |
| DingoDB recovery examination units | [`dingo-examine`](https://crates.io/crates/dingo-examine) |

## Install

```toml
[dependencies]
dingo-sda = "0.1"
```

Or: `cargo add dingo-sda`

## Quick example

```rust
let output = dingo_sda::run(
    r#"input<"name">!"#,
    serde_json::json!({"name": "Ada"}),
)?;
assert_eq!(
    output,
    serde_json::json!({"$type": "ok", "$value": "Ada"})
);
# Ok::<(), dingo_sda::SdaError>(())
```

Bind host input under a name other than `input` with
`run_with_input_binding`.

## API surface

| Function / type | Role |
|-----------------|------|
| `Program::parse` | **Compile once** for repeated evaluation (preferred host path) |
| `Program::run_json` / `Program::eval` | Eval a parsed program (JSON bridge or SDA `Value`) |
| `run` | Convenience: parse + evaluate against JSON bound as `input` |
| `run_with_input_binding` | Convenience with a caller-chosen binding name |
| `check` | Parse and validate source without evaluating |
| `format_source` | Emit canonical SDA formatting |
| `from_json` / `to_json` | Bridge between JSON and SDA values |
| `CONFORMANCE_CORPUS_TAG` | Freeze identity for standalone semantics (`sda-standalone-v1.0`) |
| `ENR1_PROFILE_TAG` | Additive enrichment kernel (`sda-enr1-v0.1`: `Match`/`enrich`/`one?`/`one!`/`merge`) |

Hosts that apply one program to many documents should use `Program::parse` once
and `run_json` / `eval` per document — not `run` in a loop (re-parses every call).

### ENR1 enrichment (same compile path)

ENR1 (match bag + explicit cardinality) is implemented **inside this crate**, not
as a second parser. Spec: [`crates/enr-core/`](../enr-core/README.md). Example:

```rust
let prog = dingo_sda::Program::parse(
    r#"orders
       |> enrich {
            customer:
              one!(Match(l, customers,
                getPath(l, Seq["customer_id"]),
                getPath(r, Seq["id"])))
          }"#,
)?;
# let _ = prog;
# Ok::<(), dingo_sda::SdaError>(())
```

ENR2 (candidates, ranking, explain) is **not** implemented.

## Status

**Shipped.** Standalone behavior is frozen under
`dingo_sda::CONFORMANCE_CORPUS_TAG` (`sda-standalone-v1.0`). Semantic changes
require a new corpus tag. ENR1 is additive under `ENR1_PROFILE_TAG`.

DingoDB recovery examination (ExaminationUnit projection) lives in
[`dingo-examine`](https://crates.io/crates/dingo-examine), not here — this crate
stays pure.

## Conformance

```sh
cargo test -p dingo-sda
```

## Latency / phase breakdown (diagnostic)

```sh
cargo run -p dingo-sda --release --example sda_latency_breakdown
cargo test -p dingo-sda --test sda_bench_skeleton
```

## Documentation

- Spec: [SDA_SPEC.md](../../SDA_SPEC.md)
- User docs: [doc/SDA/](../../doc/SDA/)
- ENR: [crates/enr-core/](../enr-core/README.md)
- CLI package: [`dingo-sda-cli`](https://crates.io/crates/dingo-sda-cli)

## License

MIT.

Part of [DingoDB](https://github.com/frogfishio/dingodb).
