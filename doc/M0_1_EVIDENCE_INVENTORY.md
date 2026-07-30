# M0-1 — Whole-database evidence inventory

Status: **accept (local inventory)** — 2026-07-30

Scoreboard: M0-1 `accept` in [NEXT_BUILD_STATUS.md](NEXT_BUILD_STATUS.md) after M0-2 reconciliation.

Package: `M0-1` (P0-GATE)

Source revision inventoried: `1d75199428d2f386ff5b8c87a2bddf9a728d9ee9`

This document is evidence mapping. It does **not** upgrade capability claims.
`qualified=false` remains mandatory until CPR-005 and remaining partial gates close.

## 1. VFY identifiers (established)

| ID | Meaning | State after this inventory |
|---|---|---|
| `VFY-0` | claim/suite/profile/report registries | **missing** — no `spec/verification/` yet |
| `VFY-1` | preflight + evidence-producing runner | **missing** — disk/infra failures not classified in CI runner |
| `VFY-2` | map tests/proofs to claims/oracles | **partial** — Heap has `hp010-matrix-v1.json`; rest ad hoc |
| Profile | `dingo-heap-v1` (HP-010) | matrix present; **not** qualified |
| Profile | `dingo-rust-app-v1` / `dql-app-core-v1` | planned under APP; not verified product claims |

## 2. Verification surfaces run this session

| Surface | Command | Result | Notes |
|---|---|---|---|
| Architecture | `bash scripts/check_heap_architecture.sh` | **pass** | scripts lacked `+x`; fixed locally with `chmod +x scripts/*.sh` |
| Heap quick | `bash scripts/verify-heap.sh quick` | **pass** | full suite listed in script; all green |
| Kani check | `bash scripts/check_kani_heap.sh` | **was stale; fixed** | required `VERUS_PROOFS_CONNECTED=false` while code/docs say `true` |
| Verus | `bash scripts/check_verus_heap.sh` | **pass** | 8 verified, 0 errors via `tools/verus/verus` |
| Full workspace | `cargo test --workspace` | **not_run** | ~1.2 Gi free disk; prior ENOSPC on 7.2 Gi `target/` |
| TLC (TLA+) | model-check formal/heap | **not_run** | `tlc` not required for quick path |

### 2.1 verify-heap quick contents (all passed)

- `dingo-heap` lib + tests (incl. trybuild HeapCap)
- `dingo-format` lib
- `dingo-store` catalog unit + `hp004_catalog_rebuild`
- `dingo-authority` `hp005_accept`
- `dingo-sdk` `hp007_heap_isolation`
- `dingo-server` `hp008_heap_handshake`, `hp008_accept_loop`
- `dingo-store` `hp006_heap_migration`, `hp009_lifecycle`, `hp010_qualification` (24 tests)
- `qualification::qualified_claim_remains_false_until_hp010_complete`

## 3. HP-010 matrix snapshot

Artifact: [`spec/heap/qualification/hp010-matrix-v1.json`](../spec/heap/qualification/hp010-matrix-v1.json)

| Field | Value |
|---|---|
| `qualified` | **false** |
| Gate H0 | partial |
| Gate H1 | partial |
| Gate H2 | partial |
| Gate H3 | **accept** |
| Gate H4 | partial |
| Gate H5 | partial |
| Gate H6 | partial |
| Gate HC1 | not_applicable_single_node |
| Drills recorded accept | **22 / 22** |

Mandatory drills are `accept` in the matrix, but **gates** remain partial where open residuals are listed. Gate accept ≠ product qualified.

## 4. Map to HAR-0 … HAR-7

Legend: `accept` / `partial` / `missing` / `not_run` — about **product package readiness**, not “code exists somewhere.”

| Package | Inventory state | What exists | What is missing for accept |
|---|---|---|---|
| **HAR-0** Truth cleanup | **partial** | Matrix, runbook, complete-path review, Verus+Kani artifacts | Script/doc flag agreement (this inventory fixed one); scoreboard still claims “stale truth”; CI green not re-imported here |
| **HAR-1** Collection creation | **missing** | Op **106** `collection_create` is **reserved**, schemas null | Create/list/open parity, crash/idempotency, APP-1 |
| **HAR-2** Local Heap ceremony | **partial** | Authority genesis / `hp005_accept`, store lifecycle | Product CLI `heap create` journey per plan; phase crash recovery as package |
| **HAR-3** Application-key lifecycle | **partial** | Certificates, blacklist models, server handshake | Full issue/blacklist/grace/cycle product journey + docs |
| **HAR-4** Qualified remote posture | **partial** | HeapKey handshake tests; live TLS accept loop | HeapKey listener as **default** remote profile; legacy explicit only |
| **HAR-5** Heap operations | **partial** | Wipe, restore, key loss, retention, DR retain-id drills | Broader crash-matrix cells; PKCS#11/GCP/Azure live connectors |
| **HAR-6** Ordinary SDK/CLI journey | **missing** | `RemoteHeap` / isolation / find/history/indexes | `HeapClient` façade (APP-2…); no-legacy ordinary docs path |
| **HAR-7** Release evidence | **missing** | Partial H6 evidence | Full M1 critical journey + honest labels + CI evidence pack |

## 5. Map to APP packages

| Package | Inventory state | Notes |
|---|---|---|
| APP-0 | missing | Contract fixtures not frozen as delivery package |
| APP-1 | missing | Implements HAR-1; op 106 reserved |
| APP-2…APP-8 | missing | Façade/query/cursor not product-accepted; substantial precursor code in SDK |

## 6. Discrepancies (named)

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| **M0-DISC-001** | P0-GATE | `scripts/check_kani_heap.sh` required `VERUS_PROOFS_CONNECTED=false` while `verification/heap-verus` and docs set **true** | **Fixed** this revision: kani script now requires `true` (matches `check_verus_heap.sh`) |
| **M0-DISC-002** | ops | `scripts/*.sh` not executable (`Permission denied` on `./script`) | Local `chmod +x scripts/*.sh`; ensure git file mode on commit |
| **M0-DISC-003** | infra | Full workspace suite previously **infrastructure_failure** (ENOSPC); ~1.2 Gi free at this inventory | Do not treat as product fail; VFY-1 preflight required; free disk before full suite |
| **M0-DISC-004** | honesty | Scoreboard `HAR-0` open defect text “stale Heap qualification truth/check” still accurate until HAR-0 exit | HAR-0 next after M0-1/M0-2 |
| **M0-DISC-005** | claim | Matrix drills all `accept` while gates mostly `partial` and `qualified=false` | Correct — do not flip qualified; keep Level-1 language |
| **M0-DISC-006** | H6 | CPR-005 signed external security review **not on file** | Blocks `qualified=true`; not closable by in-tree self-review |

## 7. Genuinely missing work (program-critical)

1. **VFY-0** registries under `spec/verification/`
2. **HAR-1 / APP-1** — activate collection create (106)
3. **HAR-4** — default HeapKey remote posture
4. **APP-2…APP-7** — application API + RQL Application Core + authenticated cursors
5. **CPR-005** external review for H6
6. **VFY-1** evidence runner with disk preflight and infrastructure classification
7. Broader crash-matrix / multi-process / Windows / coverage lanes (see `VERIFICATION_STATUS.md`)

## 8. What may be claimed today

Allowed (Level 1 / self-assessed):

> Named Heap namespaces with substantial isolation testing; machine-checked pure-kernel lemmas (Kani harnesses + Verus pure_kernel when tools present); **not** a qualified isolation product claim.

Forbidden:

- `qualified=true` / “strong isolation qualified”
- “complete RQL v1” (only planned `dql-app-core-v1`)
- Inferring HAR package `accept` from partial code presence

## 9. Exit criteria for M0-1

| Criterion | This inventory |
|---|---|
| Inventory tests/proofs/fuzzers/CI lanes | Done for Heap primary surfaces; whole-DB counts remain in `VERIFICATION_STATUS.md` |
| Run quick Heap surface | **Done** — pass at `1d75199…` |
| Inspect matrix | **Done** — `qualified=false`, H3 accept, others partial |
| Map HAR-0…HAR-7 | **Done** — table §4 |
| Mark accept/partial/missing | **Done** |
| Link evidence + revision | **Done** |
| Raise discrepancies | **Done** — §6 |
| Full suite / full Kani install | **not_run** (infra / optional tool) |

M0-1 is **ready to close** once this report is accepted and M0-2 reconciles the scoreboard against §4. HAR-0 still owns residual check/doc cleanup beyond M0-DISC-001.

## 10. Recommended next labor

1. Close **M0-1** on the board (done/review) after human glance.
2. **M0-2** — update `doc/NEXT_BUILD_STATUS.md` states from this table (do not mark HAR-1 accept).
3. **M0-3** — `scripts/verify-delivery-status.sh` + CI/quality wire-up (**accept** on scoreboard 2026-07-30).
4. **HAR-0** residual — ensure CI kani-heap job agrees with fixed script.
5. Then **APP-0** + **APP-1/HAR-1** on the critical path.
