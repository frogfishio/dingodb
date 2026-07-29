# Heap complete-path review (HP-010 / Gate H6)

**Status:** living review for single-node `dingo-heap-v1` qualification  
**Date:** 2026-07-29  
**Claim impact:** does **not** authorize `qualified=true`  
**Normative anchors:** HEAP_SPEC §3.4 (complete-path enforcement), §27 Gate H6,
§39–§40 (HP-010)

## 1. Purpose

Gate H6 requires a complete-path review that finds **no unscoped access surface**
for the named isolation profile. This document records the review method, the
surfaces examined, residual findings, and the residual work that still blocks
an honest qualified claim.

## 2. Review method

1. Enumerate public entry points that can observe or mutate durable data
   (SDK, server, store façades, CLI, recovery).
2. For each entry point, ask: is the bound `HeapId` / `HeapCap` mandatory on
   every data-bearing path?
3. Classify:
   - **In-scope heap path** — confined by `HeapCap` / admit / decide
   - **Explicit legacy / pre-qualification path** — documented; not part of the
     qualified profile claim
   - **Finding** — unexpected or under-documented unscoped surface
4. Cross-check against HP-010 drills (`query_escape`, operational confinement,
   single-owner admit, isolation profile registry).

## 3. TCB sketch (single-node reference)

| Layer | In TCB for logical isolation | Notes |
|-------|------------------------------|-------|
| `dingo-heap` pure decide / caps | Yes | Unforgeable `HeapCap`, trybuild |
| `dingo-format` admit / SubjectV2 | Yes | Wrong-heap frames rejected |
| `dingo-store` heap façades / lifecycle | Yes | Catalog + purge/restore |
| `dingo-authority` (local ceremony) | Yes (ceremony only) | Not linked by server |
| `dingo-server` qualified HeapKey path | Yes (when enabled) | TLS exporter + session |
| Legacy `Dingo::collection` / raw `Store` | **Out of qualified profile** | Flat-store compatibility |
| Cluster routing / Raft | **Out of single-node profile** | HC1 / HP-011+ |

## 4. Surfaces reviewed

### 4.1 In-scope (heap-bound) — Accept evidence present

| Surface | Confinement | Evidence |
|---------|-------------|----------|
| `decide` / `authority_admission_ok` / mint / refresh | Generation, blacklist, serving, binding | `h6_decide_obligations`, unit tests |
| `require_admit` / single-owner | Known owner only | `single_owner_admit` |
| `HeapStore` / `HeapCollection` put/get/delete | SubjectV2 + bound `HeapCap` + rights | `subject_v2_put_get_isolated_across_heaps`, `heap_store_rejects_foreign_subject_v2` |
| `Dingo::connect_heap` / `RemoteHeap` | TLS + HeapKey credential; no token/RBAC | `connect_heap_welcome_and_process_ops`, `connect_heap_wrong_name_rejects` |
| Query observation | Bound heap + allowlists | `query_escape_faulty_planner_confined` |
| Indexes / streams catalogs | Heap-scoped paths | `derived_path_indexes_streams_scoped` |
| Metrics / logs / export / health / support | Profile declassification registry | operational + metadata-hardened drills |
| Lifecycle suspend/retire/purge | Terminates caps; incomplete purge stays retired | lifecycle + key-loss + retention drills |
| Payload restore / DR retain-ID | No access from payload-only; ceremony for retain-ID | `restore_drills_*` |
| Qualified TLS accept-loop | HeapKey session; no token/RBAC | hp008 Accept |

### 4.2 Explicit legacy / pre-qualification (not claimed under H6)

| Surface | Why out of qualified claim | Residual gate |
|---------|---------------------------|---------------|
| `Dingo::open` + `collection(name)` flat store | Deployment-global collection names; no `HeapCap` | H0 / H1 |
| `Dingo::store()` raw `Store` access | Bypasses heap façade | H1 / HP-003 |
| Default feature `legacy-raw-store` | Public raw store still default for Stages 3–9 | HP-003 residual |
| Token-auth serve path (`qualified_heap_key=false`) | Pre-HeapKey remote profile | H2 / HP-008 residual |
| Cluster `open_cluster` / multi-node | Not single-node profile | HC1 |

These paths **must not** be advertised as providing Gate H6 isolation. Product
language remains Level 1 until they are either removed from default profiles or
re-bound through heap APIs.

### 4.3 Findings (open)

| ID | Finding | Severity | Blocks |
|----|---------|----------|--------|
| CPR-001 | Flat SDK is feature-gated (`legacy-flat-sdk`, **still default on** for Stages 3–9); heap-only via `--no-default-features`; labelled non-qualified | High until default flips | H0, H1, H6 |
| CPR-002 | Reserved heap ops (beyond 1–3) not activated with §32.4 fixtures | Medium | H2 |
| CPR-003 | Live filesystem multi-tier media wipe / HSM adapters incomplete | Medium (ops) | H4 residual / HP-009 |
| CPR-004 | Formal models are connected sketches + executable obligations, not Verus-checked proofs | High for H6 | H6 |
| CPR-005 | No independent external security review receipt on file | High for H6 | H6 |
| CPR-006 | Resource/physical isolation profiles declared, not qualified | Low (out of reference) | H3 profile extension |

No **unknown** bypass through the reviewed heap-bound derived paths was found in
this pass; known escapes are the **documented legacy/default surfaces** above.

## 5. Conclusion

| Question | Answer |
|----------|--------|
| Unscoped surface on **qualified heap path**? | No known bypass in CI Accept drills |
| Unscoped surface on **default product path**? | **Yes** — `legacy-flat-sdk` still default-on (CPR-001 residual); heap-only profile exists |
| Ready for Gate H6 claim? | **No** — CPR-001 residual (default), CPR-004, CPR-005 remain |
| Matrix `qualified` | **Must stay `qualified=false`** |

## 6. Exit criteria for re-review

Re-run this review and mark findings closed only when:

1. Default single-node profile requires heap-bound APIs (or legacy is opt-in and
   clearly non-qualified).
2. Connected Verus/Kani (or equivalent) proofs cover pure decide + isolation Inv.
3. External review report is filed with open findings dispositioned.
4. HP-010 matrix H0–H5 are `accept` honestly and H6 prerequisites in §27 are met.

## 7. Related artifacts

- Matrix: `spec/heap/qualification/hp010-matrix-v1.json`
- External review brief: `doc/HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md`
- Operator runbook: `doc/RUNBOOK_HEAP_QUALIFICATION.md`
- Verify: `./scripts/verify-heap.sh`