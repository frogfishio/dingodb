# APB-1 G6 — Dual-backend façade parity suite

Status: **scaffolded 2026-08-01** · package `APB-1` **active / not accept**  
Authority: [MUST_ADD.md](./MUST_ADD.md) §5 · [APB1_CLIENT_GAP_INVENTORY.md](./APB1_CLIENT_GAP_INVENTORY.md) G6

## Goal

One shared scenario pack runs against:

| Backend | Constructor | Host test |
|---|---|---|
| Embedded | `HeapClient::from(Heap)` | `cargo test -p residiuum-sdk --test apb1_heap_client_embedded` |
| Remote | `HeapClient::from(RemoteHeap)` | `cargo test -p residiuum-server --features dangerous-key-export --test hp007_connect_heap apb1_heap_client_from_remote` |

Shared code lives at:

`crates/residiuum-sdk/tests/common/apb1_facade_parity.rs`

(included via `#[path = …]` from both crates — not a public product API).

## Scenario matrix

| Id | Embedded | Remote (HP-007 vector cert) | Notes |
|---|---|---|---|
| `create_open_list` | **yes** (`run_full_facade_parity`) | **partial** (`scenario_list_and_open` only) | Remote vector cert is Read\|Write\|IndexAdmin — **no HeapAdmin create** |
| `put_get_delete` | yes | yes | Collection plane |
| `history_versions` | yes | yes | Collection plane |
| `index_lifecycle` | yes | yes | Needs IndexAdmin |

## Commands

```bash
# Embedded full pack (create + collection plane)
cargo test -p residiuum-sdk --test apb1_heap_client_embedded

# Remote collection plane (pre-provisioned collection)
cargo test -p residiuum-server --features dangerous-key-export \
  --test hp007_connect_heap apb1_heap_client_from_remote
```

## Exit criteria (package — not yet claimed)

- [x] Shared scenario module + both backends call it
- [ ] Remote create_open_list under a cert with HeapAdmin (or admin fixture)
- [ ] Named dual harness / CI job green for both commands
- [ ] Scoreboard APB-1 **accept** only after full matrix + residual honesty

## Explicit non-claims

- G6 scaffold ≠ APB-1 package accept.
- Unexplained semantic divergence between backends is a failure; known rights gaps
  (create) must stay documented in this matrix.
