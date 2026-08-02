# APB-2 T5 — Store Key Atomic CAS for `if_version`

Status: **labor done (in_review)** · 2026-08-02 · package **APB-2 active / not accept**  
Board: `d08e4633`  
Authority: [MUST_ADD.md](./MUST_ADD.md) §6 · APB-2 T2/T3 façade OCC

## Deliverable

Single-key **version test + mutation** under the store exclusive writer path:

| Surface | API |
|---|---|
| Store | `WriteCondition::{Unconditional,Absent,LiveEventId,Present}` |
| Store | `put_subject_bytes_if` / `delete_subject_bytes_if` |
| HeapStore | `put_collection_if` / `delete_collection_if` (under `Mutex`) |
| HeapCollection | `create_if_absent` / `replace_if_version` / `delete_if` |
| `CollectionClient` (embedded) | `create` / `replace` / `delete_with` use store CAS |

Errors: `StoreError::VersionConflict` / `KeyExists` → SDK `Error::VersionConflict` / `already_exists`.

## Tests

| Suite | Result |
|---|---|
| `residiuum-store` `key_atomic_cas_put_and_delete` | **1/1** |
| `residiuum-sdk` `apb2_facade_mutations` | **3/3** |

## Residuals (honest)

| Residual | Notes |
|---|---|
| **Remote concurrent CAS** | Wire path productized in T7; multi-client stress residual (R2) |
| **Crash / multiproc matrix** | Named residual for package exit (MUST_ADD §6 exit matrices) |
| **Package accept** | Forbidden — residual checklist [APB2_RESIDUAL_CHECKLIST.md](./APB2_RESIDUAL_CHECKLIST.md) + principal |

## Non-claims

- No APB-2 package accept.
- No claim that remote multi-client lost-update is closed.
- No wire schema change required for embedded CAS.