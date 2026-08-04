# Next build status

Status: program scoreboard

Sources: [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md),
[NEXT_BUILD_PLAN.md](../../done/programs/NEXT_BUILD_PLAN.md),
[M0_1_EVIDENCE_INVENTORY.md](../../done/programs/M0_1_EVIDENCE_INVENTORY.md), and active package plans.

Updated: 2026-08-04 (**P0 segment-ID collision** — inventory + immutable
no-replace publish + refuse open on duplicates; see
`doc/todo/performance-qualification/P0_SEGMENT_ID_COLLISION.md`. Pause unrelated
perf work until P0 accepted. CSE-3 2k+2l remain accepted; campaign ~21–23K SoT
unchanged.) Ops: `CSE3_COMPACTSHADOW_OPS_NOTES.md`. **ETQ-2 paused.** **AWO paused.**

**How to read program order:** open
[MASTER_DELIVERY_PLAN.md §0 Reader map](../../../MASTER_DELIVERY_PLAN.md)
first (stages vs packages, ID glossary, boundaries). This file is only the
**package state table** — not a second roadmap.

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
| CSQ-0 | accept | 2026-07-31 | — | [specification](../../todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md); [implementation plan](../../todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md); DEF-098…DEF-104 accepted | CSQ-0 registries materialised under `spec/verification/core-storage/`; `scripts/verify-core-storage-registry.sh` green; Rust `csq0_registry` tests agree | core-storage contract |
| CSQ-1 | accept | 2026-07-31 | CSQ-0 | `crates/residiuum-store-model/`; `tools/core-storage-reference-reader/`; `scripts/verify-csq-oracle-firewall.sh` green | independent model + reference-reader; residual depth optional | storage oracles |
| CSQ-2 | accept | 2026-07-31 | CSQ-0 | `scripts/verify-csq-boundary-instrumentation.sh`; `scripts/verify-csq-crash-campaign.sh`; failpoint/boundary labor | hit-proof failpoints + crash controller; residual campaign depth optional | failure injection |
| CSQ-3 | accept | 2026-07-31 | CSQ-1, CSQ-2 | `scripts/verify-csq-format-corpus.sh`; FMT-001…005; frozen microframes | format corpus green; residual multi-fault depth optional | format qualification |
| CSQ-4 | accept | 2026-07-31 | CSQ-1, CSQ-2 | `scripts/verify-csq-state-machine.sh`; `crates/residiuum-store-model` publication/history; DEF-099/100 | transition + false harnesses; residual shrinker depth optional | transition qualification |
| CSQ-5 | accept | 2026-07-31 | CSQ-2, CSQ-4 | `scripts/verify-csq-crash-campaign.sh`; persistence matrix + FS-image inventory | reopen/ENOSPC/lock cells; residual portable-image depth optional | persistence qualification |
| CSQ-6 | accept | 2026-07-31 | CSQ-3…CSQ-5 | `tests/csq6_chunk_large_value.rs` (9); `scripts/verify-csq-chunk-large-value.sh`; DEF-098 + DEF-103 linked; suite `CSQ-SUITE-CHK` → `active_csq6`; **board `in_review`** | principal accept of CSQ-3…5 still open; deeper damage/conflict campaign residual (CSQ-7) | chunk qualification |
| CSQ-7 | accept | 2026-07-31 | CSQ-3…CSQ-5 | `tests/csq7_damage_salvage.rs` (8); `scripts/verify-csq-damage-salvage.sh`; DEF-011 + stage salvage linked; suites DMG/REC → `active_csq7`; **board `in_review`** | principal accept CSQ-3…6 still open; encryption-unavailable semantic hole residual | survival qualification |
| CSQ-8 | accept | 2026-07-31 | CSQ-4, CSQ-5, CSQ-7 | `tests/csq8_derived_maintenance.rs` (9); `scripts/verify-csq-derived-maintenance.sh`; DEF-102/050/051/052/024 linked; suites DER/MNT/BAK/MIG → `active_csq8`; **board `in_review`** | principal accept CSQ-4/5/7 still open; tier reclaim interruption depth residual | maintenance qualification |
| CSQ-9 | accept | 2026-07-31 | CSQ-2, CSQ-4, CSQ-8 | `tests/csq9_concurrency_resources.rs` (9); `scripts/verify-csq-concurrency-resources.sh`; DEF-101/096/020 linked; suites CON/RES → `active_csq9`; **board `in_review`** | Loom/Shuttle kernels + full multi-process soak residual; principal accept CSQ-2/4/8 still open | boundedness qualification |
| CSQ-10 | accept | 2026-07-31 | CSQ-3, CSQ-4, CSQ-6…CSQ-9 | `tests/csq10_mutation_fuzz.rs` (7); `scripts/verify-csq-mutation-fuzz.sh`; P0 mutants 5/5 killed; fuzz-smoke property bar owned; suite `CSQ-SUITE-MUT` → `active_csq10`; **board `in_review`** | cargo-fuzz scheduled smoke optional; Miri/sanitizer CI residual; 95% broader mutant surface residual | suite sensitivity |
| CSQ-11 | accept | 2026-07-31 | CSQ-5…CSQ-10 | `tests/csq11_compat_scale_soak.rs` (7); `scripts/verify-csq-compat-scale-soak.sh`; suite `CSQ-SUITE-COMPAT` → `active_csq11`; platforms-v1 + wire matrix floors; DEF-104 linked in journey; **board `in_review`** | principal accept CSQ-5…10 still open; multi-version released-writer fixtures residual; full multi-platform CI + 24h/72h soak residual | release campaign |
| CSQ-12 | accept | 2026-08-01 | CSQ-0…CSQ-11 | `bash scripts/residiuum-verify-core-storage.sh` (+ `--require-a2-pass`) exit 0; `target/csq-evidence/a2-evaluation.json` **a2_pass=true** missing=0 cells=111; `residiuum-core-storage-report-v1.json` result=pass; independent verify ok; CSQ-0…11 scoreboard accept; principal note: accept when A2 verifies | A3 residuals only (platform matrix, 72h soak, full-mutation %); not A2 blockers | core-storage qualification |
| PQH-0 | active | 2026-07-31 | CSQ-12 | `spec/performance/*` registries+schemas+fixtures; `scripts/verify-performance-registry.sh`; `crates/residiuum-perf` registry load/tests; **board `in_review`** | runner/workloads residual PQH-1…; CSQ-12 scoreboard accept for program entry honesty | performance measurement contract |
| PQH-1 | active | 2026-07-31 | PQH-0 | `crates/residiuum-perf/src/runner/*` path guard, marker, budgets, preflight, platform adapters, fingerprint, cancel→`invalid_partial_cancelled`; `cargo test -p residiuum-perf --lib` 32/32; **board `in_review`** | L0+ measurement residual PQH-2…; principal accept PQH-0/1 | safe controlled runner |
| PQH-2 | active | 2026-07-31 | PQH-0 | `crates/residiuum-perf/src/workload/*` generator/distributions/digest oracle/scheduler; fixed-size + threshold probes; `cargo test -p residiuum-perf --lib` 53/53; **board `in_review`** | L0 calibration residual PQH-4; metrics residual PQH-3; principal accept | workload oracle |
| PQH-3 | active | 2026-07-31 | PQH-0 | `crates/residiuum-perf/src/metrics/*` histogram/clock/counters/probes/result writers; `cargo test -p residiuum-perf --lib` 80/80; **board `in_review`** | L0/L1 envelope residual PQH-4; principal accept PQH-0…3 | measurement integrity |
| PQH-4 | active | 2026-07-31 | PQH-1, PQH-2, PQH-3 | `crates/residiuum-perf/src/envelope/*` L0 cal + L1 fake/file adapters; no raw devices; honest direct/cold; `cargo test -p residiuum-perf --lib` 98/98; **board `in_review`** | L2 shadow residual PQH-5; principal accept | filesystem/device ceiling |
| PQH-5 | active | 2026-07-31 | PQH-4 | `crates/residiuum-perf/src/shadow/*` PhysicalWritePlan+replay+opaque L2 shadow; equivalence; store seam residual `pending_store_boundary_emitter`; `cargo test -p residiuum-perf --lib` 107/107; **board `in_review`** | store-native plan emission residual; L3 residual PQH-6 | shaped-I/O ceiling |
| PQH-6 | active | 2026-07-31 | PQH-5 | `crates/residiuum-perf/src/pipeline/*` L3 null/memory sink, selectable stages, timeline+residual; no FS in L3; `cargo test -p residiuum-perf --lib` 121/121; **board `in_review`** | L4/L5/L6 residual PQH-7; principal accept | stage attribution |
| PQH-7 | active | 2026-07-31 | PQH-5, PQH-6 | `crates/residiuum-perf/src/matrix/*` L4/L5/L6 cells, ack ledger, durability mutant reject, seeded matrix; `cargo test -p residiuum-perf --lib` 133/133; **board `in_review`** | real store driver residual; PQH-8 analyzer | complete-path measurement |
| PQH-8 | active | 2026-07-31 | PQH-7 | `crates/residiuum-perf/src/analyze/*` matched-run validator, retention/efficiency/amp/scaling, residual, bootstrap CI, registered verdicts, §12 false-narrative suite (10/10), falsification experiments, MD+JSON reports; `cargo test -p residiuum-perf --lib` 158/158; **board `in_review`** | PQH-9 campaign residual; principal accept PQH-0…8 | bottleneck verdicts |
| PQH-9 | active | 2026-07-31 | PQH-0…PQH-8 | `crates/residiuum-perf/src/campaign/*` macOS/Linux/synthetic plans, ≥5 reps × ≥2 processes, reports, multiproc 4K/8K finding, ranked bottlenecks, hashed evidence bundle + disclosure, opt-card stubs only; `cargo test -p residiuum-perf --lib` 164/164; **board `in_review`** | controlled-runner product baseline accept; principal accept PQH-0…9 | performance qualification |
| PQH-10 | active | 2026-07-31 | PQH-0…PQH-9 | `store_driver/*` synthetic+real (feature `store-driver`/legacy-raw-store), receipt-stream PhysicalWritePlan emitter, `residiuum-perf` CLI (preflight/run/analyze/verify/driver-smoke); synthetic NON-PRODUCT; `cargo test -p residiuum-perf --lib` 165/165; `--features store-driver` 166/166; **board `in_review`** | principal accept | operational PQH completion |
| PQH-11 | active | 2026-07-31 | PQH-10 | **Still not qualification-accept.** Prior accepted: tput formula + floor gating. **This slice:** instrument `Store::put_many_parallel` (append/tail/publish probe + encoded_frame_len); harness always `put_many` (identical product path probe-on/off — no sequential-put bypass); multi-shard concurrent preparers + outstanding-bounded batches. Evidence: store DEF-096 probe test + store-driver lib green. **No 120s qual.** Board `in_review` | controlled host `--class qualification` 120s+512MiB + sustained window; principal accept | put_many_parallel probe + product path |
| FAS-0 | accept | 2026-08-01 | CSQ-12 | Full §12 catalogue (35) + §5 assumptions (8); ownership map; 10 schemas; negative-fixture self-tests; `formal/registry/FAS0_CLOSED`; `bash scripts/check-formal-registry.sh` **exit 0** (`structural_ok`+`closed`); [FAS0_GATE.md](./FAS0_GATE.md); CSQ-12 accept same day | residual: expand schemas/linter depth; no theorem `machine_proved` claims yet | formal claim governance |
| FAS-1 | accept | 2026-08-01 | FAS-0 | Lock closed: Verus `0.2026.07.27.31579f0`, Kani `0.67.0`, Lean `v4.32.2`, TLC `tla2tools-1.7.4` (jar sha256 pinned); smokes: pure_kernel 8 verified, kani-smoke, `lake build` fas1_smoke, TLC FAS1Smoke; `setup-formal-tools.sh --locked` + `check-formal-toolchain.sh` **exit 0**; report `target/formal-assurance/fas1-toolchain-report.json` | TLAPS deferred (not accept-required); CI job wiring residual; archive sha for Lean/Kani installers residual | reproducible proof toolchain |
| FAS-2 | accept | 2026-08-01 | FAS-0, FAS-1 | Lean kernel `formal/lean/Residiuum/{Identity,Observation,State,WellFormed,Operations,Observe,Vectors,Foundation}.lean`; `init_well_formed`; observation separation + forbidden-collapse; 19 ops in `operations-v1.json`; `bash scripts/check-formal-foundation.sh` **exit 0**; report `target/formal-assurance/fas2-foundation-report.json` | residual: strengthen WF proofs beyond empty-map rfl; full put/get preservation; feature ops still stub Step | mathematical semantics |
| FAS-3 | accept | 2026-08-01 | FAS-2 | Entrypoint census + type-map; vertical slice **FAS-BRIDGE-AUTHORITY-BINDING-001** (Lean `Refinement.lean` + Verus `pure_kernel` + `decide`/`pure_proofs`); negative rename/demo controls; `bash scripts/check-formal-refinement.sh` **exit 0**; report `target/formal-assurance/fas3-refinement-report.json` | residual: store put/get full forward simulation; more CON bridges; Kani not re-run in gate (flag only) | implementation connection |
| FAS-4 | accept | 2026-08-01 | FAS-3, CSQ-12 | All 8 `FAS-CON-*` Lean theorems in `Residiuum.Consistency`; connections + live negatives; CSQ A2 links; FS assumption named; `bash scripts/check-formal-consistency.sh` **exit 0**; report `target/formal-assurance/fas4-consistency-report.json`; profile **MVP** (`mvp_abstract_plus_csq_links`, not full `physically_qualified`) | residual: full physical profile; store put/get refinement; stronger durable-ack under FS ledger | formal consistency |
| FAS-5 | deferred | 2026-08-01 | FAS-3, FAS-4, Heap contract freeze | FAS-4 MVP accept; Heap Verus/Kani 8 + TLA sketches; **principal: more FAS later** (not active product lane) | unified security theorem/refinement bundle residual | formal security |
| FAS-6 | not_started | — | FAS-3…FAS-5, ATM-1 | Atomics formal contract drafted | Atomic safety/preservation proofs absent | formal Atomic safety |
| FAS-7 | not_started | — | FAS-6, Atomic recovery freeze | — | isolation/liveness proofs absent | formal isolation |
| FAS-8 | deferred | — | cluster protocol freeze, FAS-3…FAS-5 | cluster spec exists | consensus/refinement proofs absent | formal cluster |
| FAS-9 | not_started | — | FAS-1…FAS-3, one accepted theorem family | — | public proof bundle/CLI absent | reproducible proof product |
| APB-0 | accept | 2026-08-01 | CSQ-12 (accept), APP-0/APP-1 evidence | [spec/app/baseline-v1/](../../../spec/app/baseline-v1/) **frozen**; `bash scripts/verify-app-baseline-contract.sh --require-frozen` exit 0; fixtures under baseline-v1/fixtures/; APP-0 error_mapping total; [APB_QUERY_ATOMICS_SEQUENCE.md](../../todo/application-baseline/APB_QUERY_ATOMICS_SEQUENCE.md) | residual: product APB-1…12 implementation; compile fixtures expand with packages | application contract |
| APB-1 | active | 2026-08-01 | APB-0, HAR-1 | inventory + **G1–G6:** sealed backends; full dual pack (embedded + remote HeapAdmin create mint); shared `apb1_facade_parity`; UUIDv4 create ids; [APB1_DUAL_BACKEND_SUITE.md](../../todo/application-baseline/APB1_DUAL_BACKEND_SUITE.md); [APB1_CLIENT_GAP_INVENTORY.md](../../todo/application-baseline/APB1_CLIENT_GAP_INVENTORY.md) v1.6 | HAR-1 **active** (evidence reconcile; not accept); RecoveryClient reserved; optional CI harness; **no package accept** | backend-neutral client |
| APB-2 | active | 2026-08-02 | APB-1 | T5 store CAS + T7 remote wire CAS + **T8 concurrent lost-update** (`apb2_concurrent_cas` **3/3**, store concurrent **2/2**); T6 checklist [APB2_RESIDUAL_CHECKLIST.md](../../todo/application-baseline/APB2_RESIDUAL_CHECKLIST.md); [APB2_T8_CONCURRENT_CAS.md](../../todo/application-baseline/APB2_T8_CONCURRENT_CAS.md) | R3 crash/retry; multi-process remote concurrent residual; **no package accept** | safe single-key mutation |
| APB-3 | not_started | — | APB-1, HAR-1 | — | lifecycle/capability APIs absent | collection lifecycle |
| APB-4 | not_started | — | APB-2 | — | document-path operations absent | atomic document mutation |
| APB-5 | not_started | — | APB-2, APB-4 | — | bounded bulk contract absent | bulk mutation |
| APB-6 | active | 2026-08-02 | APB-1, APB-3 | **T1–T3**: scaffold + embedded pin + **retention enforce** (`max_hold`/`max_pinned_documents`) + `PinCapability` remote residual honesty + multipage under-pin re-check/accounting; tests `apb6_read_view_scaffold` **3/3**, `apb6_view_retention` **4/4**; APB-7 T5 view-bound gate; inventory [APB6_READ_VIEW_GAP_INVENTORY.md](../../todo/application-baseline/APB6_READ_VIEW_GAP_INVENTORY.md) | export under pin; HAR-4 remote product pin; reclamation fence; **no package accept / no snapshot claim** | read consistency |
| APB-7 | active | 2026-08-02 | APB-1, APB-6, APP-4, APP-5 | **T0–T11 + T6**: dual-pack + **op 118 wire active** (APP-7 T6); remote dual pack `apb7_query_from_remote_collection_plane` **1/1** via product wire; accept checklist [APB7_DUAL_BACKEND_SUITE.md](../../todo/application-baseline/APB7_DUAL_BACKEND_SUITE.md); inventory [APB7_QUERY_RUNTIME_GAP_INVENTORY.md](../../todo/application-baseline/APB7_QUERY_RUNTIME_GAP_INVENTORY.md); **no package accept** | principal accept gate; residual multipage SI / range index | query baseline |
| APB-8 | not_started | — | APB-7 | — | bounded aggregate baseline absent | aggregates |
| APB-9 | not_started | — | APB-2, APB-6 | — | resumable change feed absent | watches |
| APB-10 | not_started | — | APB-3, APB-5, APB-6 | — | resumable import/export absent | data movement |
| APB-11 | not_started | — | APB-1…APB-10 | — | public application test kit absent | consumer verification |
| APB-12 | not_started | — | APB-0…APB-11, HAR-4 | — | baseline A2 bundle absent | application qualification |
| HAR-0 | ready | 2026-07-30 | — | matrix; Verus/Kani flags aligned (M0-DISC-001 fixed); architecture OK; M0 complete | residual: confirm CI kani-heap job; HAR-0 plan checklist; **board stage backlog** (principal: APP/CORE first) | truth cleanup residual |
| HAR-1 | active | 2026-08-01 | HAR-0, APP-0 | **Reconcile:** op **106** `collection_create` is **active** in `operations-v1.json` + `rpc-v1/collection_create.*` + fixtures; embedded `Heap::create_collection` / `create_collection_idempotent`; server dispatch 106; `RemoteHeap::create_collection`; façade dual create (APB-1 G6b HeapAdmin mint); [HAR1_COLLECTION_CREATE_EVIDENCE.md](../../todo/heap-application-ready/HAR1_COLLECTION_CREATE_EVIDENCE.md) | crash/failpoint/journey residual; product bootstrap cert still no HeapAdmin; **no package accept** | collection creation |
| HAR-2 | not_started | — | HAR-1 | precursor: `hp005_accept`, authority genesis | CLI ceremony package not accept | local Heap ceremony |
| HAR-3 | not_started | — | HAR-2 | precursor: certs, handshake | full key lifecycle journey open | application-key lifecycle |
| HAR-4 | active | 2026-08-02 | HAR-3 | **T0–T4**: product default HeapKey + CLI + config auth path + **tutorial journey** (`connect_heap` primary in server/cli/sdk READMEs; token appendix); [HAR4_T4_CONNECT_HEAP_JOURNEY.md](../../todo/heap-application-ready/HAR4_T4_CONNECT_HEAP_JOURNEY.md); [HAR4_QUERY_REMOTE_GAP_INVENTORY.md](../../todo/heap-application-ready/HAR4_QUERY_REMOTE_GAP_INVENTORY.md); op 118 active (APP-7) | package accept residual (principal); full ceremony tutorial HAR-2/3/6 | qualified remote path |
| HAR-5 | not_started | — | HAR-4 | precursor: wipe/restore/key-loss/DR drills (hp009/hp010) | broader crash cells; non-AWS KMS live | Heap operations |
| HAR-6 | not_started | — | HAR-5, APB-12 | precursor: RemoteHeap CRUD/find/history/indexes | no qualified application-baseline journey | SDK/CLI journey |
| HAR-7 | not_started | — | HAR-6 | partial H6 evidence only | M1 critical journey + honest labels | P1 release gate |
| APP-0 | active | 2026-07-30 | — | plan: [CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](../../todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) §14; [spec/app/v1/](../../../spec/app/v1/) + residuals; wire staged schemas/fixtures; `residiuum_sdk::app_v1`; `verify-app0-contract.sh` + `app0_contract_lock` (verify PASS; contract_lock 6/6); **board `in_review`** (labor handoff) | owner sign-off still open (APP0-R3; principal → `done`); plan_hash/mac placeholders (APP0-R1/R2) | application contract |
| APP-1 | active | 2026-07-30 | — | op **106 active** + schemas; `create_collection_idempotent` (UUIDv4 ids); server dispatch 106; `RemoteHeap::create_collection`; `HeapClient` create both backends (APB-1); tests app1 4/4 + dispatch 1/1 + dual pack | crash-matrix/failpoint residual; bootstrap cert lacks HeapAdmin; HAR-1 package exit still open | qualified collection create |
| APP-2 | not_started | — | APP-1 | SDK precursor types | façade not product | backend-neutral Rust API |
| APP-3 | active | 2026-08-02 | APP-2, HAR-4 | façade put/get/delete + history + indexes (APB-1) + APB-2 mutations dual-pack; OCC version alignment; embedded Key Atomic CAS (APB-2 T5) | APP-2 scoreboard lag; remote wire if_version; durability options remote; crash matrices; **no package accept** | typed data/history/index |
| APP-4 | accept | 2026-08-01 | APP-0 freeze | `residiuum_sdk::predicate` + `plan_v1` (`rql-plan-encoding-v1`); `spec/app/v1/plan_vectors_v1.json` hashes locked; `cargo test -p residiuum-sdk --test app4_predicate_plan` **4/4**; builder↔fixture plan hash parity; predicate totality model (absent≠value); name-binding fail-closed | full RQL source is APP-5; scan/index oracle parity at execution (APP-6/APB-7); **no product query claim** until APB-7/HAR | canonical predicates/plans |
| APP-5 | accept | 2026-08-01 | APP-4 | CORE §14 exit: `residiuum_sdk::rql_app_core` `compile_app_core` → `RqlPlanV1` + explain/budget run metadata; profile **`rql-app-core-v1`** (not full RQL v1); §9 surface (multi-where, project/order/nulls, coverage/consistency, predicates, budget `{documents,bytes,result_bytes}`); non-Core reject (`enrich`/`within`/`at rank`/access + `after`→APP-6) with `rql_feature_unavailable`; corpus `spec/app/v1/rql_app_core_corpus_v1.json`; `cargo test -p residiuum-sdk --lib rql_app_core` **13/13**; `--test app5_rql_app_core` **3/3**; plan_vectors `source_rql` hash lock; bounded fuzz panic-free | `after`/continuation product = APP-6; no query execution/product claim until APB-7 (+ APP-3/HAR-4 path); host must merge `CompiledAppCore.budget` with `QueryRunOptions` | RQL Application Core |
| APP-6 | active | 2026-08-02 | APP-3, APP-5, HAR-4 | **T1–T3**: cursor + page executor + multipage field-order; APB-7 T10 product ring binds params; `app6_page_executor` **4/4**; `app6_field_order_multipage` **3/3** | HAR-4 remote op 118; Heap-confined cursor secrets residual; **no product query / APB-7 package accept** | query execution |
| APP-7 | active | 2026-08-02 | APP-6, HAR-4 | **T6**: op **118** `rql_query` **active** in heap+baseline ops; server dispatch + `RemoteHeap::rql_query` + façade remote `rql` wire path; dual pack remote green; gates `har4_query_remote_gate` **7/7** | package accept residual; plan-only args residual | remote query |
| APP-8 | not_started | — | APP-1…APP-7 | — | release evidence pack | application journey |
| DEL-0 | not_started | — | HAR-3 (drafting may start after) | — | drafting only until M1; no live surface | Evidence registries |
| TEL-0 | not_started | — | HAR-3 (drafting may start after) | — | drafting only until M1 | Telemetry registries |
| DST-000 | not_started | — | HAR-3 (drafting may start after) | — | not M2 engine gate | Studio scaffolding |
| RRE-0 | ready | 2026-08-01 | C0 accept (pure prep) | plan §14 / §0.8: pure semantic oracle + adversarial corpus permitted after C0; **product activation still requires M3 packages** | no ruleset activation; diagnostic only | semantic oracle |
| RRE-1 | not_started | — | RRE-0 | — | — | source language |
| RRE-2 | not_started | — | RRE-1 | — | — | canonical invariant core |
| RRE-3 | not_started | — | RRE-2 | — | encoding amendment required | verified artifact |
| RRE-4 | not_started | — | RRE-3 | — | — | document-local enforcement |
| RRE-5 | not_started | — | RRE-4, ATM path | — | — | operational lifecycle |
| RRE-6 | not_started | — | RRE-5, REL | — | — | P2 release gate |
| ATM-0 | not_started | — | HAR-2 freeze identity | plan §14 / §0.8: pure oracle/profile after HAR-2; principal wants early risk discovery | no LocalHeap Atomic product API until M4 packages accept | semantic oracle |
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
then the PQH measurement lane and FAS foundation alongside APB. Do not
admit APP-2…APP-8 or HAR-1…HAR-7 as active product packages before `CSQ-12`
accepts. Existing precursor code and Kanban review cards remain valid evidence;
they do not override this package interlock.

## Ready queue (honest)

### Kanban-first labor rule (2026-08-02)

**Principal tracking = Kanban only** (todo / doing / in_review / done).  
Scoreboard markdown is agent/package evidence — **not** the human dashboard.

Labor **must not** deliver product features ad-hoc. Sequence:

1. `work_board_get` — read Features + tasks + `project_version` (**what’s next / what landed**)
2. If no matching **Feature**, `kanban_feature_upsert` first
3. If no matching **task**, `kanban_task_upsert` **before code**
4. Stage `todo` → `doing` → `in_review` (labor never self-`done`; principal gates `done`)
5. On `in_review`, put short **evidence in the card objective** so the board records what was done
6. **Pre-stage** open backlog as `todo` (no next-pull invented only in docs)
7. Prefer host board tools; do not invent boards under `.koderra/`

**Pull order (Application Core product path — board SoT; 2026-08-02 complete pre-stage):**

| Priority | Board card | Stage | Note |
|---:|---|---|---|
| 1 | **APP-7 / T6** op 118 `48b8f01b` | `in_review` | wire active + dispatch + façade; dual pack remote green; **not** package accept |
| — | **HAR-4 T2** default + legacy flag `98f6e855` | `in_review` | product default qualified; legacy opt-in; no package accept |
| — | **HAR-4 T3** config auth path `0ee0812c` | `in_review` | config keys + co-host refuse + report labels |
| — | **HAR-4 T4** connect_heap tutorials `b4eda326` | `in_review` | public READMEs lead with product path; token appendix; no package accept |
| — | **HAR-4 dep** query remote `7872d5fa` | `in_review` | inventory + gate locks; not package accept |
| — | **APB-6 T3** view residual `be072203` | `in_review` | retention + PinCapability + multipage accounting |
| — | **APB-7 T11** multipage oracle matrix `f6633005` | `in_review` | 6/6 multipage vs list_keys+get; residual product dual-remote |
| — | **APB-7 T10** product cursor secrets `b11912fe` | `in_review` | product ring + parameter_hash; tests 4/4 |
| — | **APB-7 T9** coverage grade `99e32b76` | `in_review` | CoverageIncomplete fail-closed; evidence fields; tests 4/4 |
| — | **APB-7 T8** deadline+cancel `5bd3fe3b` | `in_review` | deadline + CancelToken; tests 4/4 |
| — | **APP-6 T3** field-order multipage `c7cd3cca` | `in_review` | last_sort_tuple resume; tests 3/3 |
| — | **APB-7 T5** ReadView-bound `6c7601a5` | `in_review` | labor done; not SI / no package accept |
| — | **APB-7 T7** dual-pack + accept checklist `9e19bd5f` | `in_review` | suite + checklist; collection-plane dual green; **not** package accept |
| — | **RQL PATH T0** map `32d30efe` | `in_review` | [PATH_TO_FULL_RQL.md](../../todo/rql/PATH_TO_FULL_RQL.md); post-0.2.2 query de-risk |
| — | **RQL PATH T1** Core baseline `0a5c700a` | `in_review` | Phase 1 labor: expanded corpus + execute oracle + residual checklist; **not** package accept |
| — | **RQL PATH T2** expressiveness corpus `b4ebdaf9` | `in_review` | sql+ emit/refuse scaffold + Core gotchas; joins pending Phase 3 |
| — | **RQL-v1 full language** `89a80e77` | `backlog` | **NOT** APB-7; promote only after Phase 1 accept |
| — | **BASELINE-PLAN T1** journey plan `e6d68a31` | `in_review` | post-query path to APB-12 / baseline-v1; [BASELINE_V1_JOURNEY_PLAN.md](../../todo/application-baseline/BASELINE_V1_JOURNEY_PLAN.md); **not** code pull |
| — | **APB-2 T5** store Key Atomic CAS `d08e4633` | `in_review` | embedded CAS labor; no package accept |
| — | **APB-2 T6** residual checklist `1b8a52b7` | `in_review` | honesty map; **never** self-mark package accept |
| — | **APB-2 T7** remote wire CAS `e11fdb0c` | `in_review` | heap if_version/if_absent + façade remote; dual pack green |
| — | **APB-2 T8** concurrent CAS `2a28fea4` | `in_review` | multi-thread lost-update one-wins; residual multi-process |
| — | **APB-2 T9** crash/retry matrix `5ffd205b` | `todo` | **pre-staged** R3; pull this next for APB-2 exit labor |
| — | **APB-2 T10** multi-process remote concurrent `32ff87be` | `todo` | **pre-staged** after T8 multi-process residual |

**Process honesty (2026-08-02):** T1–T3 were often **JIT-created at package start** (scoreboard compass + create-on-pull). That is half-winging. Fix: **pre-stage** remaining sequence as `todo` before code turns; pull only existing cards.

**Process honesty (later same day, APB-2):** After pre-staged **T5/T6** pulls, **T7/T8** were again **create-on-pull** from residual checklist markdown (not pre-staged `todo`). Course-correct: pre-stage **T9 crash/retry (R3)** `5ffd205b` + **T10 multi-process remote concurrent** as `todo`; next keep-going pulls existing cards only.

Program order (packages; Kanban cards bind labor under them):

1. **CSQ-12 = accept** (2026-08-01): A2 independently verifies (`residiuum-verify-core-storage.sh`,
   `a2_pass=true`, missing=0). A3 residuals remain (platform / 72h soak / full mutation %) —
   not A2 blockers.
2. **FAS-0…FAS-4 = accept** (2026-08-01, MVP foundation closed). Principal steer: **past FAS stage** —
   do not pull FAS-5… as the active product lane; more FAS later when re-opened. FAS-6…FAS-8 still travel
   with Atomics/cluster when those packages admit formal work.
3. **Query spine** (principal §0.8): **APP-4/APP-5 = accept**; **APP-6 active** (T1/T2 in_review);
   **APB-6 active** (T1 scaffold + **T2** embedded segment pin; no accept / no snapshot claim);
   **APB-7 active** (**T0–T5 + T7–T11** in_review; **T6** op 118 still `todo`/blocked — no product query / no package accept);
   **APB-1 active** (G1–G6 dual matrix); **HAR-4 active** (inventory only); next pull **APP-7/T6** when HAR-4 admits, or **HAR-4 T2** default HeapKey (board SoT).
   Non-query APB may lag (APB-2 T5/T6 already `todo` on board).
4. **HAR-0…HAR-3** identity/keys in parallel as deps require; full HAR-4…7 still for M1 exit.
5. **Pure risk prep:** RRE-0 oracle; ATM-0 after HAR-2 — **no** M3/M4 product claims from prep.
6. **PQH principal accept** remains measurement hygiene (labor largely `in_review`).
7. Finish remaining **APB/HAR** for M1 exit → **M2**.

Do **not** mark any HAR or APP package `accept` from precursor tests alone.
A2 claim language is admitted only with the CSQ-12 evidence bundle above; A3 is not claimed.

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
- [x] HAR-1 T1: scoreboard “106 reserved” corrected → **active** with residual honesty ([HAR1_COLLECTION_CREATE_EVIDENCE.md](../../todo/heap-application-ready/HAR1_COLLECTION_CREATE_EVIDENCE.md)); **not accept**

## Next engine package

| Order | Package | Note |
|---:|---|---|
| 1 | DEF-098…DEF-104 | Accepted; permanent regression authorities |
| 2 | **CSQ-12 / A2** | Scoreboard **accept** 2026-08-01; A3 residuals deferred |
| 3 | **FAS-0…FAS-4** | Scoreboard **accept** MVP 2026-08-01; foundation closed |
| 4 | **APB-0** | **accept** 2026-08-01 — baseline-v1 frozen |
| 5 | **APP-4 → APP-5 → APB-7** | APP-4/5 **accept**; APP-6/APB-6/APB-7 **active** (inventories + scaffolds); still **no** product query accept |
| 6 | **HAR-0…HAR-3** | Identity/keys; enables ATM-0 prep after HAR-2 |
| 7 | **RRE-0 / ATM-0** pure | Risk oracles/corpora only — **not** M3/M4 product exit |
| 8 | Remaining HAR/APB → M1 exit → **M2** | Complete baseline journey |
| hygiene | **PQH principal accept** | Measurement lane; labor largely `in_review` |
| later | **FAS-5…** | **deferred** — does not block query path |