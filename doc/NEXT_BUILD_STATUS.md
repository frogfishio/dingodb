# Next build status

Status: program scoreboard

Sources: [MASTER_DELIVERY_PLAN.md](../MASTER_DELIVERY_PLAN.md),
[NEXT_BUILD_PLAN.md](../NEXT_BUILD_PLAN.md),
[M0_1_EVIDENCE_INVENTORY.md](M0_1_EVIDENCE_INVENTORY.md), and active package plans.

Updated: 2026-07-30 (principal course-correct: APP/CORE track active; HAR/M2 board backlog)

This file records delivery state. It does not change normative semantics.

## Allowed states

```text
not_started | ready | active | blocked | accept | deferred
```

Rules (from master plan):

- `ready` — every dependency and entry condition is satisfied.
- `active` — an owner is producing required artifacts.
- `blocked` — named unsatisfied dependency or defect.
- `accept` — every exit test and evidence item; never with an unresolved release-gate defect.
- Code existing in the repository does **not** by itself mean `accept`.
- Inventory may report precursor evidence in the Evidence column while State remains
  `not_started` / `ready` until the package workstream exits.

## Verification truth (do not drift)

| Claim | Value | Source |
|---|---|---|
| Heap profile | `dingo-heap-v1` | `spec/heap/qualification/hp010-matrix-v1.json` |
| `qualified` | **false** | same |
| Gate H3 | accept | same |
| Gates H0,H1,H2,H4,H5,H6 | partial | same |
| Level-1 product language only | yes | HEAP_SPEC / claim_language |
| Last Heap quick surface | pass | `bash scripts/verify-heap.sh quick` @ `1d75199428d2` (see M0-1) |
| Verus pure_kernel | 8 verified | `scripts/check_verus_heap.sh` |
| Full workspace suite | not_run (disk) | M0-DISC-003 |

Inventory revision: `1d75199428d2f386ff5b8c87a2bddf9a728d9ee9`.
Working tree may be ahead (M0-1/M0-2 doc and script fixes).

## Scoreboard

| Package | State | last_verified | blocked_by | Evidence | Open defects | Capability impact |
|---|---|---|---|---|---|---|
| M0-1 | accept | 2026-07-30 | — | [M0_1_EVIDENCE_INVENTORY.md](M0_1_EVIDENCE_INVENTORY.md); [VERIFICATION_STATUS.md](VERIFICATION_STATUS.md); `verify-heap.sh quick` pass; matrix map HAR/APP | full workspace not_run; CPR-005 remains product gate not M0-1 gate | program truth inventory |
| M0-2 | accept | 2026-07-30 | — | this file reconciled to M0-1 §4–§7; DEL/TEL/DST/VFY rows; last_verified/blocked_by | none | scoreboard honesty |
| M0-3 | accept | 2026-07-30 | — | [scripts/verify-delivery-status.sh](../scripts/verify-delivery-status.sh); [scripts/quality.sh](../scripts/quality.sh); `.github/workflows/ci.yml` job `quality` step **Delivery scoreboard (M0-3)** | none | CI program-status gate |
| VFY-0 | not_started | — | — | — | missing `spec/verification/` registries | claim registry |
| VFY-1 | not_started | — | VFY-0 | — | no preflight/infra-classified runner | evidence runner |
| VFY-2 | not_started | — | VFY-0 | Heap matrix is ad-hoc VFY-2 partial only | no whole-DB claim map | oracle mapping |
| HAR-0 | ready | 2026-07-30 | — | matrix; Verus/Kani flags aligned (M0-DISC-001 fixed); architecture OK; M0 complete | residual: confirm CI kani-heap job; HAR-0 plan checklist; **board stage backlog** (principal: APP/CORE first) | truth cleanup residual |
| HAR-1 | not_started | — | HAR-0, APP-0 | op **106** `collection_create` **reserved**, schemas null | product create missing | collection creation |
| HAR-2 | not_started | — | HAR-1 | precursor: `hp005_accept`, authority genesis | CLI ceremony package not accept | local Heap ceremony |
| HAR-3 | not_started | — | HAR-2 | precursor: certs, handshake | full key lifecycle journey open | application-key lifecycle |
| HAR-4 | not_started | — | HAR-3 | precursor: HeapKey handshake, TLS accept loop | HeapKey not proven default remote posture | qualified remote path |
| HAR-5 | not_started | — | HAR-4 | precursor: wipe/restore/key-loss/DR drills (hp009/hp010) | broader crash cells; non-AWS KMS live | Heap operations |
| HAR-6 | not_started | — | HAR-5, APP-8 | precursor: RemoteHeap CRUD/find/history/indexes | no ordinary `HeapClient` journey accept | SDK/CLI journey |
| HAR-7 | not_started | — | HAR-6 | partial H6 evidence only | M1 critical journey + honest labels | P1 release gate |
| APP-0 | active | 2026-07-30 | — | plan: [CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) §14; [spec/app/v1/](../spec/app/v1/) + residuals; wire staged schemas/fixtures; `dingo_sdk::app_v1`; `verify-app0-contract.sh` + `app0_contract_lock` (verify PASS; contract_lock 6/6); **board `in_review`** (labor handoff) | owner sign-off still open (APP0-R3; principal → `done`); plan_hash/mac placeholders (APP0-R1/R2) | application contract |
| APP-1 | active | 2026-07-30 | — | embedded: `Heap::create_collection` / `list_collections`; durable `(H,O)` admin_op_dedup (APP1-R1); tests `app1_collection_create` (4 pass) | op 106 still reserved; no server dispatch/remote (APP1-R2); HeapClient façade (APP1-R3/APP-2) | qualified collection create |
| APP-2 | not_started | — | APP-1 | SDK precursor types | façade not product | backend-neutral Rust API |
| APP-3 | not_started | — | APP-2, HAR-4 | CRUD/history/index precursor | parity suite not package-accept | typed data/history/index |
| APP-4 | not_started | — | APP-0 | filter/SDA/dialect precursors | no single `dingo-predicate-v1` accept | canonical predicates/plans |
| APP-5 | not_started | — | APP-4 | — | dql-app-core-v1 compiler not accept | DQL Application Core |
| APP-6 | not_started | — | APP-3, APP-5, HAR-4 | — | authenticated cursor not product | query execution |
| APP-7 | not_started | — | APP-6, HAR-4 | op 118 `dql_query` reserved | remote query parity missing | remote query |
| APP-8 | not_started | — | APP-1…APP-7 | — | release evidence pack | application journey |
| DEL-0 | not_started | — | HAR-3 (drafting may start after) | — | drafting only until M1; no live surface | Evidence registries |
| TEL-0 | not_started | — | HAR-3 (drafting may start after) | — | drafting only until M1 | Telemetry registries |
| DST-000 | not_started | — | HAR-3 (drafting may start after) | — | not M2 engine gate | Studio scaffolding |
| DRE-0 | not_started | — | M1 exit | — | — | semantic oracle |
| DRE-1 | not_started | — | DRE-0 | — | — | source language |
| DRE-2 | not_started | — | DRE-1 | — | — | canonical invariant core |
| DRE-3 | not_started | — | DRE-2 | — | encoding amendment required | verified artifact |
| DRE-4 | not_started | — | DRE-3 | — | — | document-local enforcement |
| DRE-5 | not_started | — | DRE-4, ATM path | — | — | operational lifecycle |
| DRE-6 | not_started | — | DRE-5, REL | — | — | P2 release gate |
| ATM-0 | not_started | — | HAR-2 freeze identity | — | — | semantic oracle |
| ATM-1 | not_started | — | ATM-0 | — | — | canonical plans |
| ATM-2 | not_started | — | ATM-1 | — | — | prepare/member evidence |
| ATM-3 | not_started | — | ATM-2 | — | — | durable decision |
| ATM-4 | not_started | — | ATM-3 | — | — | recovery/convergence |
| ATM-5 | not_started | — | ATM-4 | — | — | LocalHeap Atomic API |
| REL-0 | not_started | — | ATM-3 path | — | — | reference metadata |
| REL-1 | not_started | — | REL-0 | — | — | parent-exists/restrict |
| REL-2 | not_started | — | REL-1 | — | — | uniqueness |
| REL-3 | not_started | — | REL-2 | — | — | activation/validation |
| REL-4 | not_started | — | REL-3 | — | — | P3 release gate |
| DDA-0 | not_started | — | DRE predicate freeze | — | profile amendment required | rank oracle |
| DDA-1 | not_started | — | DDA-0 | — | — | natural direct rank |
| DDA-2 | not_started | — | DDA-1 | — | — | filtered direct rank |
| DDA-3 | not_started | — | DDA-2 | — | — | ordered admission seam |
| DDA-4 | not_started | — | DDA-3 | — | cursor profile required | P4 public surface |
| DDA-5 | deferred | — | cluster profile | — | cluster profile unavailable | distributed rank |
| DDA-6 | deferred | — | P4 accept | — | P4 not accepted | adaptive optimization |
| DOW-0 | not_started | — | DDA order-domain freeze | — | — | mathematical oracle |
| DOW-1 | not_started | — | DOW-0 | — | — | immutable order blocks |
| DOW-2 | not_started | — | DOW-1 | — | — | compressed exact indexes |
| DOW-3 | not_started | — | DOW-2 | — | — | P5 immutable path |
| DOW-4 | not_started | — | DOW-3 | — | — | mutable order path |
| DOW-5 | deferred | — | cluster profile | — | cluster profile unavailable | distributed order |

## Principal labor track (board SoT)

**Active labor:** **APP-1** — embedded collection create first cut.
**Handoff:** **APP-0** is `in_review` (labor done; principal owns accept).

Board stages (principal 2026-07-30; handoff rule: labor → `in_review`, never
`done` — principal moves `done` after accept):

| Stage | Cards |
|---|---|
| done | M0-1, M0-2, M0-3 |
| in_review | **APP-0** (contract freeze handoff; APP0-R3 sign-off) |
| doing | **APP-1** (embedded create first cut) |
| backlog | HAR-0…HAR-7, APP-2…APP-8, DEL-0, TEL-0, DST-000 |

Do **not** pull HAR/M2 cards into `todo` while APP/CORE is the principal track.
Scoreboard may still list HAR-0 as `ready` (dependency honesty); that is not a
labor order override.

## Ready queue (honest)

Labor order for this principal (not “everything that is ready”):

1. **APP-0** — freeze application contract (CORE plan §14) — **in_review** (labor handoff).
2. **APP-1** (≡ product HAR-1 capability) — **doing** (embedded first cut; residuals open).
3. **APP-4** — may parallel APP-1 after APP-0 (predicates/plan).
4. **HAR-0** residual — ready on scoreboard but **board backlog** until principal re-queues it.
5. **DEL-0 / TEL-0 / DST-000** — after HAR-3 only; board backlog.

Do **not** mark any HAR or APP package `accept` from precursor tests alone.

## M0-2 exit checklist

- [x] Observed HAR/APP state from M0-1 reflected (evidence + defects, not false accept)
- [x] DEL / TEL / DST / VFY rows present
- [x] `last_verified` and `blocked_by` columns present
- [x] No completed work left as `not_started` (M0-1 → accept)
- [x] No partial precursor work marked `accept`
- [x] Ready packages have named dependencies
- [x] `scripts/verify-delivery-status.sh` exists and passes against this file

## M0-3 exit checklist

- [x] `scripts/verify-delivery-status.sh` exists (allowed states, unique IDs, deps, evidence, stage order, plan links, matrix honesty)
- [x] Script invoked from `scripts/quality.sh` (local mirror of CI bar)
- [x] Script invoked from `.github/workflows/ci.yml` `quality` job (step **Delivery scoreboard (M0-3)**)
- [x] Local `bash scripts/verify-delivery-status.sh` passes against this scoreboard
- [x] M0 exit companion: HAR-1 not falsely ready — blocked by named predecessors **HAR-0**, **APP-0**

## Next package after M0

| Order | Package | Note |
|---:|---|---|
| 1 | **APP-0** | CORE plan — contract and fixture lock (**doing**) |
| 2 | APP-1 ≡ HAR-1 capability | collection_create 106 after APP-0 |
| 3 | APP-4 | predicates/plan; may parallel APP-1 after APP-0 |
| — | HAR-0 residual | board **backlog** until principal re-queues |