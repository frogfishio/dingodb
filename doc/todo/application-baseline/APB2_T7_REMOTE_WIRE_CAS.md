# APB-2 T7 — Remote wire `if_version` / `if_absent` Key Atomic

Status: **labor done (in_review)** · 2026-08-02 · package **APB-2 active / not accept**  
Board: `e11fdb0c`  
Closes residual **R1** from [APB2_RESIDUAL_CHECKLIST.md](./APB2_RESIDUAL_CHECKLIST.md).

## Deliverable

| Surface | Change |
|---|---|
| `heap_dispatch` put | args `if_version` (hex) / `if_absent` → `put_collection_if` |
| `heap_dispatch` delete | args `if_version` / `if_present` → `delete_collection_if`; soft absent idempotent |
| CAS errors | `version_conflict` (+ expected/observed hex), `already_exists`, `not_found` |
| `RemoteHeap` | `put_json_if`, `delete_if`; maps structured `version_conflict` |
| `CollectionClient` remote | `create` / `replace` / `delete_with` use wire CAS |

## Tests

| Suite | Result |
|---|---|
| `apb2_facade_mutations` | **3/3** |
| `hp007_connect_heap` `apb1_heap_client_from_remote_full_parity_heap_admin` (includes `apb2_mutations`) | **1/1** |

## Residuals

- Concurrent multi-client lost-update matrix (R2)
- Crash/retry (R3)
- **No package accept**
