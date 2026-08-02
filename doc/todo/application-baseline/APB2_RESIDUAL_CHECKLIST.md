# APB-2 residual checklist (T6) — no false package accept

Status: **labor done (in_review)** · 2026-08-02 · package **APB-2 active / not accept**  
Board: `1b8a52b7` (T6) · prior CAS labor `d08e4633` (T5)  
Feature: APB-2 mutations `08f99c7b`  
Authority: [MUST_ADD.md](./MUST_ADD.md) §6 · [APB2_T5_KEY_ATOMIC_CAS.md](./APB2_T5_KEY_ATOMIC_CAS.md) ·
[APB1_DUAL_BACKEND_SUITE.md](./APB1_DUAL_BACKEND_SUITE.md) · scoreboard

**This document does not authorize `APB-2 = accept`.** Principal gates package accept.

---

## 1. MUST_ADD §6 deliverables — labor map

| Deliverable | Labor status | Evidence |
|---|---|---|
| `create` | **labor green (T5/T7)** | Embedded store CAS + remote wire `if_absent` |
| `replace` + `if_version` | **labor green (T5/T7)** | Embedded store CAS + remote wire `if_version` |
| `delete_with` + `if_version` / `if_present` | **labor green (T5/T7)** | Embedded store CAS + remote wire conditions |
| `add` + key profile | **partial green** | `KeyProfile::RandomV1`; not store-atomic create-on-mint collision path |
| `upsert` | **partial green** | Embedded create-if-absent then put; remote residual |
| OCC version = establishing `event_id` | **green (labor)** | Receipts + history alignment (T2/T3) |
| Dual-backend façade parity | **green (labor)** | `apb2_mutations` dual pack (embedded + remote; wire CAS after T7) |
| **Package accept** | **forbidden** | Concurrent/crash exit matrices open (below) |

---

## 2. Labor complete (T0–T5 board cards)

| Card | Stage | What landed |
|---|---|---|
| T1 create/upsert/list_keys | done / prior | Façade surface first cut |
| T2 replace + delete_with | done | OCC tokens, `VersionConflict` / `NotFound` |
| T3 add + receipt.version | done | RandomV1; version=event_id |
| T4 dual-pack `apb2_mutations` | done | Shared scenarios dual host |
| T5 store Key Atomic CAS | **in_review** | `WriteCondition` + embedded façade path |
| T6 residual checklist | **in_review** | Honesty map; **no accept** |
| T7 remote wire CAS | **in_review** | heap + façade remote CAS |
| **T8 concurrent matrix** | **in_review** | multi-thread lost-update; [APB2_T8](./APB2_T8_CONCURRENT_CAS.md) |

---

## 3. Residual matrix (blocks package accept)

| ID | Residual | Blocks MUST_ADD exit? | Notes / next labor |
|---|---|---|---|
| R1 | Remote wire `if_version` / `if_absent` | **closed (labor T7)** | Dual pack + heap dispatch CAS; multi-client stress still R2 |
| R2 | Concurrent lost-update (embedded multi-thread) | **closed (labor T8)** | [APB2_T8_CONCURRENT_CAS.md](./APB2_T8_CONCURRENT_CAS.md); multi-process remote still open |
| R3 | **Crash / retry / damage matrices** | **yes** | MUST_ADD §6 exit language; not labored |
| R4 | **Local/remote parity under concurrent multi-client CAS** | **yes** | Dual pack serial green; multi-client remote residual |
| R5 | **Sortable / non-Random key profiles** | soft | Only RandomV1 productized |
| R6 | **Remote durability options on put/delete** | soft | Server default; façade notes residual |
| R7 | **Idempotent operation_id + conditional put coupling** | soft | DEF-010 exists; not bound to `if_version` semantics on wire |
| R8 | **Coverage-incomplete create** | soft | Incomplete coverage cannot prove absence (MUST_ADD rule) — not matrixed for façade create |

Exit language (MUST_ADD §6): *concurrent lost-update, crash, retry, damage, and local/remote parity matrices pass.*  
**None of those matrices are package-green.** Labor T5 closes only the **embedded store atomicity** gap under a single exclusive writer.

---

## 4. Evidence index (what *is* proven)

| Artifact | Proves |
|---|---|
| `cargo test -p residiuum-store --lib key_atomic_cas` | Store `WriteCondition` put/delete CAS serial |
| `cargo test -p residiuum-sdk --test apb2_facade_mutations` | Embedded façade create/replace/delete_with/add/upsert/list_keys |
| Dual pack `apb2_mutations` (APB-1 suite) | Embedded + remote façade serial parity |
| `apb2_concurrent_cas` + store concurrent_ | Multi-thread lost-update (T8) |
| [APB2_T5_KEY_ATOMIC_CAS.md](./APB2_T5_KEY_ATOMIC_CAS.md) / T7 / T8 docs | API + wire + concurrent maps |
| Scoreboard APB-2 row | **active / not accept** honesty |

---

## 5. Explicit non-claims

1. **No `APB-2 = accept`** from T1–T6 labor alone.
2. Dual-pack green ≠ package exit.
3. Embedded store CAS ≠ remote multi-client Key Atomic.
4. Façade OCC tests ≠ crash/recovery matrices.
5. Do **not** claim “safe single-key mutations product complete” until R1–R4 close under principal gate.

---

## 6. Recommended next labor (after this checklist)

| Priority | Board card (pre-staged) | Outcome |
|---:|---|---|
| 1 | ~~T7 remote wire~~ | **done (in_review)** |
| 2 | ~~T8 concurrent embedded~~ | **done (in_review)** |
| 3 | **T9** crash/retry `5ffd205b` | Closes R3 partial — **pull this card** |
| 4 | **T10** multi-process remote concurrent `32ff87be` | Multi-client residual |
| — | Principal package accept only when exit matrices + scoreboard gate | Accept |

**GOV:** Do not invent T11+ at package start. Pre-stage then pull.

---

## 7. Scoreboard instruction

| Field | Value |
|---|---|
| State | **active** (not accept) |
| Evidence | T1–T5 labor + this checklist |
| Blocked_by | R3 crash/retry; multi-process concurrent residual; principal gate |
| Forbidden | Self-marking accept from residual checklist alone |