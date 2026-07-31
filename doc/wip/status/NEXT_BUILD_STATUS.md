# Next build status

Status: program scoreboard

Sources: [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md),
[NEXT_BUILD_PLAN.md](../../done/programs/NEXT_BUILD_PLAN.md),
[M0_1_EVIDENCE_INVENTORY.md](../../done/programs/M0_1_EVIDENCE_INVENTORY.md), and active package plans.

Updated: 2026-07-31 (post-rebrand and emergency-defect reconciliation)

This file records package qualification state and dependency truth. It does not
change normative semantics and it does not mirror live Kanban columns. Kanban
owns assignment, execution stage, review, and acceptance workflow.

## Allowed states

```text
not_started | ready | active | blocked | accept | deferred
```

Rules (from master plan):

- `ready` — every dependency and entry condition is satisfied.
- `active` — the package is admitted and an owner is producing required
  artifacts; Kanban may use more detailed workflow states.
- `blocked` — named unsatisfied dependency or defect.
- `accept` — every exit test and evidence item; never with an unresolved release-gate defect.
- Code existing in the repository does **not** by itself mean `accept`.
- Inventory may report precursor evidence in the Evidence column while State remains
  `not_started` / `ready` until the package workstream exits.

## Verification truth (do not drift)

| Claim | Value | Source |
|---|---|---|
| Heap profile | `residiuum-heap-v1` | `spec/heap/qualification/hp010-matrix-v1.json` |
| `qualified` | **false** | same |
| Gate H3 | accept | same |
| Gates H0,H1,H2,H4,H5,H6 | partial | same |
| Level-1 product language only | yes | HEAP_SPEC / claim_language |
| Last Heap quick surface | pass | `bash scripts/verify-heap.sh quick` @ `1d75199428d2` (see M0-1) |
| Verus pure_kernel | 8 verified | `scripts/check_verus_heap.sh` |
| Full workspace suite | pass | `cargo test --workspace` exit 0 on 2026-07-31 after REB-12 |

Inventory baseline: `1d75199428d2f386ff5b8c87a2bddf9a728d9ee9`.
Current verification includes the completed Residiuum rebrand through REB-12.

## Scoreboard

| Package | State | last_verified | blocked_by | Evidence | Open defects | Capability impact |
|---|---|---|---|---|---|---|
| M0-1 | accept | 2026-07-31 | — | [M0_1_EVIDENCE_INVENTORY.md](../../done/programs/M0_1_EVIDENCE_INVENTORY.md); [VERIFICATION_STATUS.md](./VERIFICATION_STATUS.md); `verify-heap.sh quick` pass; REB-12 full workspace pass | CPR-005 remains a product gate, not an M0-1 gate | program truth inventory |
| M0-2 | accept | 2026-07-30 | — | this file reconciled to M0-1 §4–§7; DEL/TEL/DST/VFY rows; last_verified/blocked_by | none | scoreboard honesty |
| M0-3 | accept | 2026-07-30 | — | [scripts/verify-delivery-status.sh](../../../scripts/verify-delivery-status.sh); [scripts/quality.sh](../../../scripts/quality.sh); `.github/workflows/ci.yml` job `quality` step **Delivery scoreboard (M0-3)** | none | CI program-status gate |
| VFY-0 | not_started | — | — | — | missing `spec/verification/` registries | claim registry |
| VFY-1 | not_started | — | VFY-0 | — | no preflight/infra-classified runner | evidence runner |
| VFY-2 | not_started | — | VFY-0 | Heap matrix is ad-hoc VFY-2 partial only | no whole-DB claim map | oracle mapping |
| CSQ-0 | active | 2026-07-31 | — | [specification](../../todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md); [implementation plan](../../todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md); DEF-098…DEF-104 accepted | CSQ-0 registries materialised under `spec/verification/core-storage/`; `scripts/verify-core-storage-registry.sh` green; Rust `csq0_registry` tests agree | core-storage contract |
| CSQ-1 | active | 2026-07-31 | CSQ-0 | — | independent model (`residiuum-store-model`) + reference-reader tool; firewall script green | storage oracles |
| CSQ-2 | active | 2026-07-31 | CSQ-0 | — | hit-proof failpoints; boundary↔source CI; composed-failure schedule; crash controller; FS-image inventory (campaign CSQ-5) | failure injection |
| CSQ-3 | active | 2026-07-31 | CSQ-1, CSQ-2 | — | frozen Residiuum microframes; bit/byte/trunc/insert/delete + holes + pairwise multi-fault; FMT-001…005 tests green | format qualification |
| CSQ-4 | active | 2026-07-31 | CSQ-1, CSQ-2 | — | publication kernel + transition coverage; HIST/scan/gen/shrinker; false harnesses; DEF-099/100 linked | transition qualification |
| CSQ-5 | active | 2026-07-31 | CSQ-2, CSQ-4 | — | matrix + composed pairs; reopen/continuation; ENOSPC/perm; writer-lock; portable FS image; outcome validator; no silent skips | persistence qualification |
| CSQ-6 | not_started | — | CSQ-3…CSQ-5 | DEF-098 regression authority | chunk/large-value suite absent | chunk qualification |
| CSQ-7 | not_started | — | CSQ-3…CSQ-5 | — | damage/recovery differential absent | survival qualification |
| CSQ-8 | not_started | — | CSQ-4, CSQ-5, CSQ-7 | — | maintenance/backup/migration matrix absent | maintenance qualification |
| CSQ-9 | not_started | — | CSQ-2, CSQ-4, CSQ-8 | — | concurrency/resource suite absent | boundedness qualification |
| CSQ-10 | not_started | — | CSQ-3, CSQ-4, CSQ-6…CSQ-9 | — | mutation/fuzz thresholds absent | suite sensitivity |
| CSQ-11 | not_started | — | CSQ-5…CSQ-10 | — | compatibility/scale/soak absent | release campaign |
| CSQ-12 | not_started | — | CSQ-0…CSQ-11 | — | verified A2 bundle absent | core-storage qualification |
| PQH-0 | not_started | — | CSQ-12 | [specification](../../todo/performance-qualification/PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md); [implementation plan](../../todo/performance-qualification/PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md) | registries absent | performance measurement contract |
| PQH-1 | not_started | — | PQH-0 | — | safe runner/platform fingerprint absent | safe controlled runner |
| PQH-2 | not_started | — | PQH-0 | — | canonical deterministic workloads absent | workload oracle |
| PQH-3 | not_started | — | PQH-0 | — | metric/result kernel absent | measurement integrity |
| PQH-4 | not_started | — | PQH-1, PQH-2, PQH-3 | — | device-envelope ladder absent | filesystem/device ceiling |
| PQH-5 | not_started | — | PQH-4 | — | shared PhysicalWritePlan/shadow writer absent | shaped-I/O ceiling |
| PQH-6 | not_started | — | PQH-5 | — | CPU pipeline/stage probes absent | stage attribution |
| PQH-7 | not_started | — | PQH-5, PQH-6 | — | additive/full database matrix absent | complete-path measurement |
| PQH-8 | not_started | — | PQH-7 | — | causal analyzer/false-narrative suite absent | bottleneck verdicts |
| PQH-9 | not_started | — | PQH-0…PQH-8 | — | controlled qualification campaign absent | performance qualification |
| APB-0 | not_started | — | CSQ-12, APP-0, APP-1 | [MUST_ADD.md](../../todo/application-baseline/MUST_ADD.md) | complete baseline contract not frozen | application contract |
| APB-1 | not_started | — | APB-0, HAR-1 | — | unified client absent | backend-neutral client |
| APB-2 | not_started | — | APB-1 | — | conditional/add/upsert APIs absent | safe single-key mutation |
| APB-3 | not_started | — | APB-1, HAR-1 | — | lifecycle/capability APIs absent | collection lifecycle |
| APB-4 | not_started | — | APB-2 | — | document-path operations absent | atomic document mutation |
| APB-5 | not_started | — | APB-2, APB-4 | — | bounded bulk contract absent | bulk mutation |
| APB-6 | not_started | — | APB-1, APB-3 | — | stable read views absent | read consistency |
| APB-7 | not_started | — | APB-1, APB-6, APP-4, APP-5 | — | RQL application runtime absent | query baseline |
| APB-8 | not_started | — | APB-7 | — | bounded aggregate baseline absent | aggregates |
| APB-9 | not_started | — | APB-2, APB-6 | — | resumable change feed absent | watches |
| APB-10 | not_started | — | APB-3, APB-5, APB-6 | — | resumable import/export absent | data movement |
| APB-11 | not_started | — | APB-1…APB-10 | — | public application test kit absent | consumer verification |
| APB-12 | not_started | — | APB-0…APB-11, HAR-4 | — | baseline A2 bundle absent | application qualification |
| HAR-0 | ready | 2026-07-30 | — | matrix; Verus/Kani flags aligned (M0-DISC-001 fixed); architecture OK; M0 complete | residual: confirm CI kani-heap job; HAR-0 plan checklist; **board stage backlog** (principal: APP/CORE first) | truth cleanup residual |
| HAR-1 | not_started | — | HAR-0, APP-0 | op **106** `collection_create` **reserved**, schemas null | product create missing | collection creation |
| HAR-2 | not_started | — | HAR-1 | precursor: `hp005_accept`, authority genesis | CLI ceremony package not accept | local Heap ceremony |
| HAR-3 | not_started | — | HAR-2 | precursor: certs, handshake | full key lifecycle journey open | application-key lifecycle |
| HAR-4 | not_started | — | HAR-3 | precursor: HeapKey handshake, TLS accept loop | HeapKey not proven default remote posture | qualified remote path |
| HAR-5 | not_started | — | HAR-4 | precursor: wipe/restore/key-loss/DR drills (hp009/hp010) | broader crash cells; non-AWS KMS live | Heap operations |
| HAR-6 | not_started | — | HAR-5, APB-12 | precursor: RemoteHeap CRUD/find/history/indexes | no qualified application-baseline journey | SDK/CLI journey |
| HAR-7 | not_started | — | HAR-6 | partial H6 evidence only | M1 critical journey + honest labels | P1 release gate |
| APP-0 | active | 2026-07-30 | — | plan: [CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](../../todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) §14; [spec/app/v1/](../../../spec/app/v1/) + residuals; wire staged schemas/fixtures; `residiuum_sdk::app_v1`; `verify-app0-contract.sh` + `app0_contract_lock` (verify PASS; contract_lock 6/6); **board `in_review`** (labor handoff) | owner sign-off still open (APP0-R3; principal → `done`); plan_hash/mac placeholders (APP0-R1/R2) | application contract |
| APP-1 | active | 2026-07-30 | — | op **106 active** + schemas; `create_collection_idempotent`; server dispatch 106 (HeapAdmin); `RemoteHeap::create_collection`; tests app1_collection_create 4/4 + app1_collection_create_dispatch | crash-matrix cells optional; HeapClient façade (APP1-R3/APP-2); bootstrap cert lacks HeapAdmin (TLS create needs admin cert) | qualified collection create |
| APP-2 | not_started | — | APP-1 | SDK precursor types | façade not product | backend-neutral Rust API |
| APP-3 | not_started | — | APP-2, HAR-4 | CRUD/history/index precursor | parity suite not package-accept | typed data/history/index |
| APP-4 | blocked | 2026-07-31 | CSQ-12, APP-0 | precursor implemented: `residiuum_sdk::predicate` + `plan_v1` (`rql-plan-encoding-v1`); plan vectors locked; `app4_predicate_plan` passes; live review stage is Kanban-owned | package admission waits for core-storage qualification; full RQL source parser is APP-5 | canonical predicates/plans |
| APP-5 | not_started | — | APP-4 | — | rql-app-core-v1 compiler not accept | RQL Application Core |
| APP-6 | not_started | — | APP-3, APP-5, HAR-4 | — | authenticated cursor not product | query execution |
| APP-7 | not_started | — | APP-6, HAR-4 | op 118 `rql_query` reserved | remote query parity missing | remote query |
| APP-8 | not_started | — | APP-1…APP-7 | — | release evidence pack | application journey |
| DEL-0 | not_started | — | HAR-3 (drafting may start after) | — | drafting only until M1; no live surface | Evidence registries |
| TEL-0 | not_started | — | HAR-3 (drafting may start after) | — | drafting only until M1 | Telemetry registries |
| DST-000 | not_started | — | HAR-3 (drafting may start after) | — | not M2 engine gate | Studio scaffolding |
| RRE-0 | not_started | — | M1 exit | — | — | semantic oracle |
| RRE-1 | not_started | — | RRE-0 | — | — | source language |
| RRE-2 | not_started | — | RRE-1 | — | — | canonical invariant core |
| RRE-3 | not_started | — | RRE-2 | — | encoding amendment required | verified artifact |
| RRE-4 | not_started | — | RRE-3 | — | — | document-local enforcement |
| RRE-5 | not_started | — | RRE-4, ATM path | — | — | operational lifecycle |
| RRE-6 | not_started | — | RRE-5, REL | — | — | P2 release gate |
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
| DDA-0 | not_started | — | RRE predicate freeze | — | profile amendment required | rank oracle |
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

## Execution ownership

Kanban is the source of truth for live stages, owners, handoffs, review, and
acceptance actions. This document deliberately does not reproduce its columns.
The package state above records qualification and dependency truth only.

The emergency DEF-098…DEF-104 family is accepted. The engine order is now CSQ,
then the PQH measurement lane alongside APB. Do not
admit APP-2…APP-8 or HAR-1…HAR-7 as active product packages before `CSQ-12`
accepts. Existing precursor code and Kanban review cards remain valid evidence;
they do not override this package interlock.

## Ready queue (honest)

Program order (Kanban determines the individual active cards):

1. Execute **CSQ-0…CSQ-12**, beginning with the ready CSQ-0 contract
   registries.
2. Begin **PQH-0…PQH-9** as the first post-C0 measurement lane. It may run
   alongside M1 but must precede speculative tuning or new performance claims.
3. Execute **APB-0…APB-12**, absorbing APP-2…APP-8 work according to
   [MUST_ADD.md](../../todo/application-baseline/MUST_ADD.md).
4. **HAR-0…HAR-7** interleave only where their APB dependencies permit.

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

## Next engine package

| Order | Package | Note |
|---:|---|---|
| 1 | DEF-098…DEF-104 acceptance | Close or explicitly retain every documented residual in Kanban |
| 2 | **CSQ-0** | Freeze the core-storage qualification registries |
| 3 | CSQ-1 / CSQ-2 | Independent oracles and failure-boundary instrumentation |
