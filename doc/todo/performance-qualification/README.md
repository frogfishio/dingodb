# Performance Qualification Harness

State: **ACTIVE — PQH-0 labor floor delivered (principal accept open)**

**Principal scoreboard (locked):** [TPS_ONLY.md](TPS_ONLY.md) — **TPS = acked puts/s only.** Do not answer the principal with component timings or meter reinterpretations.

**Disk hygiene (locked):** [OWN_DISK_FILL_CLEANUP.md](OWN_DISK_FILL_CLEANUP.md) — **always `rm -rf` peer work dirs** after each cell. Do not fill the volume with test stores.

Program: `PQH`

This program builds the controlled laboratory used to explain Residiuum
performance from the storage device up to the acknowledged database operation.
It is not a marketing benchmark and it is not the existing damage/scale
testrig.

| Document | Authority |
|---|---|
| [PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md) | Measurement semantics, experiment matrix, metrics, attribution mathematics, safety and acceptance |
| [PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md](PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md) | Packages, dependencies, artifacts, tests and delivery order |
| [ADAPTIVE_WRITE_OPTIMISER_SPEC.md](ADAPTIVE_WRITE_OPTIMISER_SPEC.md) | Post-PQH adaptive intake, cooking, write-pipeline, acknowledgement, control, proof and qualification contract |
| [ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md](ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md) | Exact std-thread architecture, Rust contracts, algorithms, defaults, files, packages, tests and acceptance commands |
| [AWO_LABOR_EXECUTION.md](AWO_LABOR_EXECUTION.md) | Developer start pack: entry honesty E1–E6, package DAG, board tasks, first-pull order (does not amend norms) |
| [AWO-0_T1_CONTRACT_RESIDUAL_CHECKLIST.md](AWO-0_T1_CONTRACT_RESIDUAL_CHECKLIST.md) | AWO-0 T1 evidence: E1–E6 stamp, contract inventory, plan §15 residual (not package accept) |
| [AWO_THREE_WAY_MEASURE_RUNBOOK.md](AWO_THREE_WAY_MEASURE_RUNBOOK.md) | Three-way measure T2/T3: fixed diagnostic matrix + correctness smoke gate (no throughput claims) |
| [AWO_THREE_WAY_T3_CORRECTNESS_SMOKE.md](AWO_THREE_WAY_T3_CORRECTNESS_SMOKE.md) | T3 evidence: three-mode unit + CLI driver-smoke green before numbers |
| [AWO_THREE_WAY_T4_DISKSAFE_MEASURE.md](AWO_THREE_WAY_T4_DISKSAFE_MEASURE.md) | T4 disk-safe first numbers (smoke slice); diagnostic residual; artifact paths |
| [AWO_THREE_WAY_T5_HONESTY.md](AWO_THREE_WAY_T5_HONESTY.md) | T5 honesty: claim table; ~30 GiB free host budget; no product ranking |
| [AWO_THREE_WAY_T6_INTERACTIVE.md](AWO_THREE_WAY_T6_INTERACTIVE.md) | T6 interactive re-run on Scratch; smoke OK; diagnostic exFAT residual |
| [AWO_THREE_WAY_T7_SPARSE_SATURATED.md](AWO_THREE_WAY_T7_SPARSE_SATURATED.md) | T7 v2: sparse/saturated **independent singles** (not harness batch_size=N); L-API vs L-AWO |
| [AWO_THREE_WAY_T8_SINGLES_RUN.md](AWO_THREE_WAY_T8_SINGLES_RUN.md) | T8 APFS smoke run: pin batch=1; all modes sync/op=1; collection residual |
| [AWO_THREE_WAY_T9_DECISIVE_FINDING.md](AWO_THREE_WAY_T9_DECISIVE_FINDING.md) | **Decisive (pre-connect):** harness OK; independent path was natural-only |
| [AWO_INDEPENDENT_COLLECTION_CONNECT.md](AWO_INDEPENDENT_COLLECTION_CONNECT.md) | Collection connect labor: queue+collector; concurrent file_sync amortize test |
| [AWO_THREE_WAY_T10_HARNESS_RERUN.md](AWO_THREE_WAY_T10_HARNESS_RERUN.md) | T10: PQH admit_put path + re-run; saturated sync/op=0.5 thr~2× |
| [AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md](AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md) | **T11 evidence freeze principal `done`:** saturated thr×2 + sparse 11–20% smoke penalty (card only; not package accept) |
| [AWO_QUALIFICATION_SERIES.md](AWO_QUALIFICATION_SERIES.md) | **AWO-Q series plan:** Q1 multi-thread admit → Q2 adaptive quality → Q3 sustained → Q4 sparse product bound |
| [PERF_BEARINGS_2026-08-03.md](PERF_BEARINGS_2026-08-03.md) | **Post-hang bearings:** where we are vs bigger truth (T11 + Q1/Q2 + PEER-SQL + PQH); not package accept |
| [AWO_10X_VS_2X_ACCOUNTING.md](AWO_10X_VS_2X_ACCOUNTING.md) | **Aim check:** 10K→120K bucket band vs T11 Durable k≈2 (~2×); missing ~8× is cross-band, not lost sync |
| [AWO_120K_NOT_DISK_OFF.md](AWO_120K_NOT_DISK_OFF.md) | **Clarify:** ~120k ≠ disk off; Buffered short/no mid-seal vs Discard ~330k vs Durable T11 |
| [PERF_HONEST_MAX_CHARTER.md](PERF_HONEST_MAX_CHARTER.md) | **Principal charter:** honest max; no vanity cheat; squeeze under named contracts |
| [ODOMETER_FIRST_COMPLETED_WRITES.md](ODOMETER_FIRST_COMPLETED_WRITES.md) | **Comm rule:** lead with acked puts/s (SQLite-comparable); ratios second |
| [TPS_ONLY.md](TPS_ONLY.md) | **Principal lock:** TPS is the only scoreboard; stop meter noise in answers |
| [OWN_DISK_FILL_CLEANUP.md](OWN_DISK_FILL_CLEANUP.md) | **Own it:** labor left test stores filling disk; mandatory rm after peer runs |
| [WATERMARK_MADE_TPS_WORSE.md](WATERMARK_MADE_TPS_WORSE.md) | **Answer:** watermark failed as TPS play (≈ grow, more disk); default unchanged |
| [WHAT_HAPPENED_PREALLOC.md](WHAT_HAPPENED_PREALLOC.md) | **Recap:** added opt-in prealloc; no TPS win; default still grow |
| [WHERE_IS_THE_BOTTLENECK.md](WHERE_IS_THE_BOTTLENECK.md) | **Answer:** TPS wall = append/growth of active segment |
| [PREALLOC_STILL_APPEND.md](PREALLOC_STILL_APPEND.md) | **Answer:** watermark ≠ EOF grow; still first-touches on put → TPS ≈ grow |
| [CLARIFY_SHIPPED_FOR_TPS.md](CLARIFY_SHIPPED_FOR_TPS.md) | **Clarify:** ½ GiB watermark WAS TPS-measured (~7.5k); “never shipped” was wrong |
| [SMART_MODE_X_MODE_A.md](SMART_MODE_X_MODE_A.md) | **Answer:** Adaptive X on Mode A bed = unknown (not measured) |
| [WHY_SMART_MODE_A_UNMEASURED.md](WHY_SMART_MODE_A_UNMEASURED.md) | **Answer:** unknown ≠ feature-blocked; campaign not run |
| [FIRM_NUMBERS_GOALS.md](FIRM_NUMBERS_GOALS.md) | **Goals:** what/to what end/compared to what; FN-1..3 sequence |
| [FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md) | **FN-2 labor:** Mode A four-cell odometer (APFS); Adaptive X≈2.5k loses to off≈12.5k |
| [SQLITE_10K_TO_30K.md](SQLITE_10K_TO_30K.md) | **Answer:** SQLite 10k→30k = Scratch exFAT → APFS `/var/tmp`, not SQLite getting faster |
| [FAST_DISK_CPU_WALL.md](FAST_DISK_CPU_WALL.md) | **Confirm:** Scratch 10k parity = SQLite disk ∩ our CPU; fast disk → our CPU wall (~12.5k) |
| [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md) | **Answer:** Static/Adaptive ~2.5k ≠ CPU hammer — QD=1 collection delay (~5× wall) |
| [FIRM_NUMBERS_MULTICORE.md](FIRM_NUMBERS_MULTICORE.md) | **FN multicore:** Mode A cook1/2/4 flat ~13k; Mode B cook4≈cook1 on APFS long peer |
| [WHAT_BATCH_1_MEANS.md](WHAT_BATCH_1_MEANS.md) | **Explainer:** batch=1 = one key per `put_many` (Mode A), not “one core” |
| [WHY_PUT_MANY_NOT_FASTER.md](WHY_PUT_MANY_NOT_FASTER.md) | **Answer:** Residiuum put_many(N) ≉ slower than N×put — ≈same; SQLite B is the big jump |
| [STATIC_IS_NOT_BATCHED_ON_FN2.md](STATIC_IS_NOT_BATCHED_ON_FN2.md) | **Answer:** FN-2 Static ≠ successful batch — “allowed to batch” + delay tax, still flush 1 |
| [WHY_CANT_WE_MICROBATCH.md](WHY_CANT_WE_MICROBATCH.md) | **Answer:** we can microbatch — not on Mode A QD=1 (no pile-up); T11 can |
| [WHAT_PARTNER_PUT_MEANS.md](WHAT_PARTNER_PUT_MEANS.md) | **Jargon:** “partner” = second key in the collector queue, not a person |
| [HOW_MANY_REQUESTS_FN2.md](HOW_MANY_REQUESTS_FN2.md) | **Answer:** FN-2 = 32 768 Mode A requests/cell; 131 072 across four cells |
| [FN2_NOT_ONE_SECOND.md](FN2_NOT_ONE_SECOND.md) | **Answer:** not a 1s timed test — fixed 256 MiB work; SQLite just finished in ~1.1s |
| [ZERO_IN_WAITING_WINDOW.md](ZERO_IN_WAITING_WINDOW.md) | **Confirm:** FN-2 Mode A wait window had zero other requests (QD=1 by construction) |
| [WHO_WAITS_FOR_ACK.md](WHO_WAITS_FOR_ACK.md) | **Answer:** peer-pump client loop waits; AWO would accept N+1 if client sent it |
| [EMBEDDED_SYNC_VS_SERVER_ASYNC.md](EMBEDDED_SYNC_VS_SERVER_ASYNC.md) | **Principal lock:** Mode A QD=1 = embedded sync feed; don’t judge AWO as multi-user DB from FN-2 |
| [FIRM_NUMBERS_CONCURRENT_FEED.md](FIRM_NUMBERS_CONCURRENT_FEED.md) | **FN concurrent:** c=8 Mode A — Adaptive≈off~13.6k (not ~2.5k); still ≪ SQLite~30k |
| [FIRM_NUMBERS_CONCURRENT_MULTICORE.md](FIRM_NUMBERS_CONCURRENT_MULTICORE.md) | **FN concurrent+cook:** cook4/8 no lift — still ~13–14k vs SQLite~27k |
| [SQLITE_PAGES_VS_RESIDIUUM_FRAMES.md](SQLITE_PAGES_VS_RESIDIUUM_FRAMES.md) | **Answer:** we do NOT write SQLite 4KiB pages — append hashed frames to segments |
| [FIRM_NUMBERS_DIAG_COALESCE.md](FIRM_NUMBERS_DIAG_COALESCE.md) | **Spike:** 64 KiB/250 ms coalesce ≈ Real (~10k); Discard ~129k — write *size* ≠ wall; `write_all` is |
| [UNDERSTAND_THE_NUMBERS.md](UNDERSTAND_THE_NUMBERS.md) | **Explainer:** one-page stitch of FN-2 → concurrent → coalesce/Discard |
| [HOW_WE_WRITE_CORRECTION.md](HOW_WE_WRITE_CORRECTION.md) | **Correction:** not write *size*; yes write_all cost / vs SQLite path gap |
| [WRITE_ALL_BISECT.md](WRITE_ALL_BISECT.md) | **Bisect:** Discard≈DevNull≈SeekOnly~120k; RealOverwrite~96k; Real~10k → **append/growth** |
| [PREALLOC_SPIKE.md](PREALLOC_SPIKE.md) | **Answer:** sparse `set_len` no; page-touch prealloc ~37k (~4× Real, >SQLite) |
| [NEXT_STEPS_WRITE_GROWTH.md](NEXT_STEPS_WRITE_GROWTH.md) | **Next:** honest prealloc → product-shaped alloc → SQLite gap → Scratch → design |
| [GEMINI_PREALLOC_PLATFORM_REVIEW.md](GEMINI_PREALLOC_PLATFORM_REVIEW.md) | **Review:** sparse trap ✓; `F_PREALLOCATE`≠37k on APFS (fcntl≈Real; touch still wins) |
| [PREALLOC_ZERO_SPIKE.md](PREALLOC_ZERO_SPIKE.md) | **Spike:** F_PREALLOCATE+bulk zero → ~51k pump (confirms zero/first-touch; fcntl alone still no) |
| [PREALLOC_WATERMARK_SPIKE.md](PREALLOC_WATERMARK_SPIKE.md) | **Spike:** 64 MiB ahead-of-write zero → ~32k pump, **best E2E wall** (~28.5k) |
| [PRODUCT_SEGMENT_WATERMARK.md](PRODUCT_SEGMENT_WATERMARK.md) | **Ship:** opt-in product `SegmentGrowthPolicy::Watermark` + peer `--segment-growth` |
| [FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md) | **Paired:** product watermark ≈ grow; prior diag ~32k = seal-fail cheat (fixed) |
| [NOT_SQUARE_ONE.md](NOT_SQUARE_ONE.md) | **Answer:** not square 1 — lost fake ~32k win; write-path map kept |
| [SEAL_WAS_BROKEN.md](SEAL_WAS_BROKEN.md) | **Answer:** “broken seal” = diag seal failed + peer ignored error |
| [FIFTY_TO_TEN.md](FIFTY_TO_TEN.md) | **Answer:** 50k→10k = first-touch paid in-timer vs offline |
| [FIFTY_TO_6_5K_PREALLOC.md](FIFTY_TO_6_5K_PREALLOC.md) | **Answer:** 50k→6.5k = that + E2E seal + noisy bed; not prealloc failing |
| [WHY_EXTEND_EACH_TIME.md](WHY_EXTEND_EACH_TIME.md) | **Answer:** default grow-on-append = log design + space trade |
| [GROW_ON_APPEND_BUYS.md](GROW_ON_APPEND_BUYS.md) | **Answer:** what grow-on-append buys besides thr (space/salvage) |
| [GROW_ON_APPEND_BUYS_RETRACT.md](GROW_ON_APPEND_BUYS_RETRACT.md) | **Correction:** those buys were soft; principal pushback |
| [WATERMARK_DRAWBACKS_REAL_VS_RELIGION.md](WATERMARK_DRAWBACKS_REAL_VS_RELIGION.md) | **Answer:** real watermark costs vs soft anti-reasons |
| [PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md](PRINCIPAL_STEER_PREALLOC_NOT_MORALITY.md) | **Steer:** prealloc ≠ morality; extend ahead via watcher; grow-on-append-as-virtue rejected |
| [PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md](PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md) | **Steer+ship:** capacity/chunk configurable; default **64 MiB** (not fixed ½ GiB) |
| [TRY_WM_64MIB.md](TRY_WM_64MIB.md) | **Try:** watermark@64 paired ≈ grow (~6.5–6.8k); SQLite ~29k; not default-on |
| [WHY_7K_VS_12K.md](WHY_7K_VS_12K.md) | **Answer:** ~7k vs ~12k = noisy/full disk vs quiet bed; not a regression |
| [MQ_RAW_FS_AWARENESS.md](MQ_RAW_FS_AWARENESS.md) | **Answer:** MQ-class raw/owned layout vs Residiuum files |
| [HOW_MANY_TPS_NOW.md](HOW_MANY_TPS_NOW.md) | **TPS now:** quiet ~12–14k · full-disk ~6.5–8k · SQLite ~25–30k |
| [artifacts/firm-numbers-product-wm-apfs/](artifacts/firm-numbers-product-wm-apfs/) | Product-flag paired JSON + summary |
| [artifacts/firm-numbers-prealloc-wm-apfs/](artifacts/firm-numbers-prealloc-wm-apfs/) | Watermark vs full-zero JSON |
| [artifacts/firm-numbers-prealloc-zero-apfs/](artifacts/firm-numbers-prealloc-zero-apfs/) | Bulk-zero vs fcntl vs touch JSON |
| [artifacts/firm-numbers-fpreallocate-apfs/](artifacts/firm-numbers-fpreallocate-apfs/) | F_PREALLOCATE vs set_len vs touch JSON |
| [artifacts/firm-numbers-prealloc-apfs/](artifacts/firm-numbers-prealloc-apfs/) | Prealloc spike JSON |
| [artifacts/firm-numbers-write-all-bisect-apfs/](artifacts/firm-numbers-write-all-bisect-apfs/) | Write-path bisect JSON ladder |
| [artifacts/firm-numbers-diag-coalesce-apfs/](artifacts/firm-numbers-diag-coalesce-apfs/) | Coalesce spike JSON (real/coalesce/discard) |
| [artifacts/firm-numbers-concurrent-apfs/](artifacts/firm-numbers-concurrent-apfs/) | Concurrent-feed JSON + summary |
| [artifacts/firm-numbers-concurrent-multicore-apfs/](artifacts/firm-numbers-concurrent-multicore-apfs/) | Concurrent × cook sweep JSON |
| [artifacts/firm-numbers-fn2-mode-a-apfs/](artifacts/firm-numbers-fn2-mode-a-apfs/) | FN-2 JSON + summary |
| [artifacts/firm-numbers-multicore-apfs/](artifacts/firm-numbers-multicore-apfs/) | Multicore campaign JSON + summary |
| [AWO_Q1_1_IMPLEMENTER_BRIEF.md](AWO_Q1_1_IMPLEMENTER_BRIEF.md) | Q1.1 brief (anchors) |
| [AWO_Q1_1_HARNESS.md](AWO_Q1_1_HARNESS.md) | **Q1.1 labor:** concurrent path wired + per-seq ledger; test green |
| [artifacts/awo-three-way-t10-apfs-smoke/](artifacts/awo-three-way-t10-apfs-smoke/) | T10 smoke numbers (SoT for T11 freeze) |
| [artifacts/awo-three-way-t7-apfs-smoke/](artifacts/awo-three-way-t7-apfs-smoke/) | T8 numeric summary + campaigns |
| [artifacts/awo-three-way-t4-disksafe/](artifacts/awo-three-way-t4-disksafe/) | T4 JSON evidence only (no store trees) |
| [artifacts/awo-three-way-t6-scratch-smoke/](artifacts/awo-three-way-t6-scratch-smoke/) | T6 Scratch smoke three-way |
| [../../../spec/performance/README.md](../../../spec/performance/README.md) | PQH-0 live registries |
| [../../../spec/performance/awo/README.md](../../../spec/performance/awo/README.md) | AWO executable contracts (`verify-awo-contract.sh`) |

Profile: `residiuum-performance-qualification-v1`

Entry dependency: `CSQ-12 = accept`.

Execution position: the first post-C0 measurement lane. It may run alongside
M1 feature work, but no performance optimization or new quantitative product
claim may be selected from intuition once `PQH-0` begins. Optimization must
follow a reproduced PQH finding.

**PQH-0 evidence:** `bash scripts/verify-performance-registry.sh` +
`cargo test -p residiuum-perf --lib`.

**Next:** PQH-1 safe runner (after PQH-0 accept).

The Adaptive Write Optimiser is a specified post-PQH implementation candidate.
Its presence here does not admit it ahead of the master delivery plan.

**AWO labor (2026-08-02):** Full labor plan in `AWO_LABOR_EXECUTION.md`. Kanban
Feature **AWO — Adaptive Write Optimiser** pre-staged (AWO-0 T1–T3 + AWO-1…7).
**AWO-0 T1–T3 labor floor complete** + **AWO-1 deepen:**
persist-before-publish on single-shard, parallel-cook, and multi-shard `put_many`
(all-or-nothing publish; checkpoint restore on clean fail; poison on short write).
`awo_persist_before_publish` 4/4. Residuals: full AdaptiveWriteLease, full crash matrix.
Master-plan AWO admission residual (E1).