# APB-2 T8 — Concurrent lost-update CAS matrix

Status: **labor done (in_review)** · 2026-08-02 · package **APB-2 active / not accept**  
Board: `2a28fea4`  
Closes residual **R2 partial** from [APB2_RESIDUAL_CHECKLIST.md](./APB2_RESIDUAL_CHECKLIST.md).

## Deliverable

Multi-thread contention over store Key Atomic (shared exclusive writer path):

| Test | Result |
|---|---|
| `store::concurrent_put_if_one_wins` | **1/1** |
| `store::concurrent_create_absent_one_wins` | **1/1** |
| `apb2_concurrent_cas` replace / create / delete | **3/3** |

`HeapCollection` is `Clone` (shared `Arc<HeapStore>`) for thread handles.

## Semantics proven

Under concurrent threads racing the same `LiveEventId` / `Absent` condition on one store:

- **exactly one** mutation succeeds;
- losers receive `VersionConflict` or `AlreadyExists`.

## Residuals

| Residual | Notes |
|---|---|
| Multi-process / multi-client remote concurrent | not this card |
| Crash / retry matrix (R3) | open |
| **Package accept** | forbidden |

## Commands

```bash
cargo test -p residiuum-store --lib concurrent_
cargo test -p residiuum-sdk --test apb2_concurrent_cas
```
