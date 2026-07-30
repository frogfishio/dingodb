# dingo console v1 — implementation plan (mixed mode)

## Goal
Add an interactive text console to the existing `dingo` CLI.

- Invocation: `dingo console ./app.dingo`
- Mixed mode UX:
  - **Console/meta commands** (help, collections listing, session controls) are shell-like.
  - **Data operations** are **RQL-first**.
- Delete semantics: **no confirmation** for routine deletes.

## Non-goals (v1)
- Operator/maintenance operations (salvage/restore/migrate/scrub/serve).
- Remote sessions, multi-node sessions, auth flows.
- Multiline editing, command history persistence, tab completion.

## UX / command surface (draft)

### Meta / console commands (begin with `.`)
- `.help` — show help.
- `.exit` / `.quit` — leave console.
- `.collections` — list collections.
- `.use <collection>` — set active collection for subsequent RQL.
- `.status` — show store path + active collection.
- `.examples` — print short examples.

### Data operations (RQL-first)
Console accepts one-line RQL statements.

- `SELECT <key>` (or `GET <key>`) — read JSON/document for the active collection.
- `PUT <key> <json>` — write JSON for the active collection.
- `DELETE <key>` — delete key for active collection (no confirmation).
- `HISTORY <key>` — show history for key.

Additionally, support optional fully-qualified targets:
- `... FOR <collection>.<key>` or `... <collection>/<key>`.

> Note: final exact RQL syntax will be aligned to what the existing RQL parser/evaluator supports in this repo.

## Implementation steps

### 1) Add `console` subcommand to `crates/dingo-cli`
- File: `crates/dingo-cli/src/main.rs`
- Extend the `Command` enum:
  - `Console { store: PathBuf }`
- In `run()` match arm, dispatch to `cmd_console(store)`.

### 2) Implement interactive loop
- File: `crates/dingo-cli/src/console.rs` (new)
- Use a minimal line reader (prefer `rustyline` only if already in deps; otherwise use `std::io::stdin` + buffering for v1).
- Session state:
  - `store: PathBuf`
  - `active_collection: Option<String>`
  - `json_out_pretty: bool` (optional)

Loop:
1. Print prompt: `dingo> ` or `dingo[<collection>]> `.
2. Read a line.
3. If empty: continue.
4. If line starts with `.`, route to meta handlers.
5. Else: treat as RQL statement and evaluate.
6. Print results or errors.

### 3) Wire meta commands to existing CLI functionality
Re-use existing operations in `main.rs`:
- `.collections` → `cmd_list(store, None, json_out=false)` equivalent, but return strings not exit codes.
- `.use` updates `active_collection`.
- `.status` prints current state.
- `.help` prints static help string.

To avoid duplicating logic, refactor existing `cmd_list/cmd_get/...` into internal helpers that can be called both from one-shot CLI and from console (recommended).

### 4) RQL evaluation integration
- Find RQL entry points in the repo (search for `dql`, `Dql`, `query`, `parse` packages) and integrate them.
- Provide an evaluator that can:
  - parse a statement
  - execute it against the open store
  - return structured results for display

If RQL does not support the full v1 data operations:
- Extend RQL to include the missing primitives (minimal set):
  - SELECT/GET by key
  - PUT/INSERT by key with JSON payload
  - DELETE by key
  - HISTORY by key

> Implementation choice: if RQL already maps to the existing store APIs (`db.collection(&coll).put/get/delete/...`), keep v1 aligned with that.

### 5) Display formatting
- For SELECT/GET/HISTORY results:
  - pretty-print JSON (existing formatting helper or `serde_json` pretty).
- For PUT/DELETE outcomes:
  - print a concise confirmation plus ack/committed fields if available.

### 6) Tests
- Extend `crates/dingo-cli/tests/cli.rs` or add new integration test `tests/console.rs`.
- Test strategy:
  - Spawn `dingo console ./tmp_store` with stdin piping scripted lines.
  - Assert stdout contains expected prompts and outputs.

Cases:
1. `.collections` prints at least one known collection after a `.use`/`PUT`.
2. `.use users` followed by `PUT user-1 {"name":"a"}` then `GET user-1`.
3. `DELETE user-1` removes the key (subsequent GET errors or returns null).

### 7) Documentation
- Update `crates/dingo-cli/README.md` with a `dingo console` section + examples.
- Mention mixed mode and delete semantics.

## Acceptance criteria
- `cargo test` passes.
- `cargo build` passes.
- Manual smoke test:
  - `dingo console ./app.dingo`, `.help`, `.collections`, `.use`, and RQL data ops work.
- Routine deletes require no extra confirmation.

## Open questions (to resolve before coding)
1. What exact RQL syntax does the repo support for GET/PUT/DELETE/HISTORY?
2. Is the RQL engine embedded and callable from the CLI crate (or via an existing client module)?
3. Should RQL require explicit collection qualification or can it use `active_collection` by default?
