# APB-1 G6 — Dual-backend façade parity suite

Status: **matrix green 2026-08-01** · package `APB-1` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §5 · [APB1_CLIENT_GAP_INVENTORY.md](./APB1_CLIENT_GAP_INVENTORY.md) G6

## Goal

One shared scenario pack runs against:

| Backend | Constructor | Host test |
|---|---|---|
| Embedded | `HeapClient::from(Heap)` | `cargo test -p residiuum-sdk --test apb1_heap_client_embedded` |
| Remote (vector cert) | `HeapClient::from(RemoteHeap)` | `… hp007_connect_heap apb1_heap_client_from_remote_open_put_get_delete` |
| Remote (HeapAdmin mint) | same | `… hp007_connect_heap apb1_heap_client_from_remote_full_parity_heap_admin` |

Shared code lives at:

`crates/residiuum-sdk/tests/common/apb1_facade_parity.rs`

(included via `#[path = …]` from both crates — not a public product API).

## Scenario matrix

| Id | Embedded | Remote vector cert (rights 13) | Remote HeapAdmin mint (0x100d) | Notes |
|---|---|---|---|---|
| `create_open_list` | **yes** | list/open only | **yes** (`run_full_facade_parity`) | Product vectors stay at 13; test mints admin COSE from same seeds |
| `put_get_delete` | yes | yes | yes | Collection plane |
| `history_versions` | yes | yes | yes | Collection plane |
| `index_lifecycle` | yes | yes | yes | Needs IndexAdmin |

## Commands

```bash
# Embedded full pack (create + collection plane)
cargo test -p residiuum-sdk --test apb1_heap_client_embedded

# Remote: collection plane (vector cert) + full pack (test-local HeapAdmin)
cargo test -p residiuum-server --features dangerous-key-export \
  --test hp007_connect_heap apb1_heap_client_from_remote
```

## Related product fix (same labor)

- Store create mints **UUIDv4** collection object ids (`create_collection_idempotent`).
- Façade wire parse prefers UUIDv4, falls back to nonzero unchecked (list/open/create).

## Exit criteria (package — not yet claimed)

- [x] Shared scenario module + both backends call it
- [x] Remote create_open_list under HeapAdmin test mint
- [ ] Named dual harness / CI job green for both commands (optional residual)
- [ ] Scoreboard APB-1 **accept** only with full residual honesty (HAR-1, G5, etc.)

## Explicit non-claims

- Dual matrix green ≠ APB-1 package accept (IndexManager product claims, HAR-1, Recovery still open).
- Product bootstrap vectors remain rights_mask **13** (no silent expand of public vectors).