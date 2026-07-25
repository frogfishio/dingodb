# sda-lib

`sda-lib` is the Rust library for Structured Data Algebra (SDA), the pure
examination and transformation language used by DingoDB.

It provides a small host-facing API for parsing, validating, formatting, and
evaluating standalone SDA programs over JSON values. Evaluation is pure: no
file or network IO lives in this crate.

## Install (workspace)

```toml
[dependencies]
sda-lib = { path = "crates/sda-core" }
# or via the workspace key (crate path becomes sda_core):
# sda-core = { workspace = true }
```

## Example

```rust
let output = sda_lib::run("input<\"name\">!", serde_json::json!({"name": "Ada"}))?;
assert_eq!(output, serde_json::json!({"$type": "ok", "$value": "Ada"}));
# Ok::<(), sda_lib::SdaError>(())
```

If you want to bind host input under a name other than `input`, use
`run_with_input_binding`.

## API surface

- `run` — evaluate an SDA program against JSON bound as `input`
- `run_with_input_binding` — evaluate against a caller-chosen binding name
- `check` — parse and validate source without evaluating it
- `format_source` — emit canonical SDA formatting
- `from_json` / `to_json` — bridge between JSON and SDA values

## Conformance freeze

Standalone behavior is locked under the corpus tag
`sda_lib::CONFORMANCE_CORPUS_TAG` (`sda-standalone-v1.0`):

- Automated suite: `tests/sda_conformance.rs` (`section_14_1_minimal_suite`,
  `section_14_must_lock`, and related §6–§13 modules)
- Golden vectors: `tests/sda/section14_must.json` (tag in `tests/sda/VERSION`)

Semantic changes require a new corpus tag.

## Documentation

- Spec: [SDA_SPEC.md](../../SDA_SPEC.md)
- DingoDB examination profile: [SDA_PROFILE.md](../../SDA_PROFILE.md)
- User-facing SDA docs: [doc/SDA/](../../doc/SDA/)
- Delivery stage: Stage 1 in [DELIVERY_PLAN.md](../../DELIVERY_PLAN.md)

## CLI

For shell use, see the `sda` package in `crates/sda-cli`.
