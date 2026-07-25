# dingo (CLI)

Stage 7 operator and everyday CLI for DingoDB.

## Commands

```text
dingo put ./app.dingo users/user-42 --json '{"name":"Alice"}'
dingo get ./app.dingo users/user-42
dingo delete ./app.dingo users/user-42
dingo list ./app.dingo
dingo list ./app.dingo users
dingo put-bytes ./app.dingo artifacts/build-19 ./build.bin
dingo history ./app.dingo users/user-42
dingo doctor ./app.dingo
dingo salvage ./damaged.dingo --output ./recovered.dingo
dingo serve ./app.dingo --bind 127.0.0.1:7434
dingo serve ./app.dingo --bind 127.0.0.1:7434 --token SECRET
```

## Guarantees

- `doctor` is **read-only** (`Store::open_inspect`) — no repairs, compact, or catalog writes.
- `salvage` never mutates the **source**; it materialises live subjects into a new store path.
- `--json-out` emits stable machine-readable output (distinct from put `--json` body).
- Nonzero exit status when an operation fails its guarantee.

## Remote

```text
dingo serve ./app.dingo --bind 127.0.0.1:7434
# Optional shared token (or env DINGO_TOKEN):
dingo serve ./app.dingo --token SECRET
```

SDK:

```rust
use dingo_sdk::{ConnectOptions, Dingo};

let mut db = Dingo::connect("dingo://127.0.0.1:7434/app")?;
let mut db = Dingo::connect_with(
    "dingo://127.0.0.1:7434/app",
    ConnectOptions::new().auth_token("SECRET"),
)?;
```

Normative: DX_SPEC §§4.2, §§13–14; DELIVERY_PLAN Stage 7 (+ 7e/7f).
