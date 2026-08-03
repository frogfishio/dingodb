# Benchmark Disclosure summary (PQH-9)

Campaign: `pqh9-synthetic-000000000000002a`

Profile: `residiuum-performance-qualification-v1`

Platform: `synthetic_harness` (product baseline allowed: false)

## Checklist

| Field | Status | Notes |
|---|---|---|
| profile | ok | residiuum-performance-qualification-v1 |
| platform | harness_only | synthetic_harness |
| durability_modes | ok | durable |
| layers | ok | L4 |
| repetitions | ok | 5 |
| processes | ok | 2 |
| correctness_interlock | ok | ack ledger + independent digest on harness cells |
| store_driver | real_store_multi_rep | driver=real_store (real_store needs --features store-driver) |
| absolute_throughput_claim | not_eligible | Product MB/s only when product_claim_eligible=true; no optimisations in PQH |
| primary_bottleneck | pending_or_mixed | none |
| optimization | none | follow-up cards are stubs with run IDs only |

## Runs

- valid: 6
- invalid: 0
- repetitions requested: 5
- processes requested: 2

## Multiproc finding (4 KiB / 8 KiB)

No 4 KiB/8 KiB multiproc cells present in this campaign slice; finding deferred.

## Ranked bottlenecks

1. **mixed_or_unknown** (low) runs=6
   - falsify: isolate_one_variable, close_stage_residual

## Follow-up optimization cards (stubs only)

None.

## Warnings

- No optimization applied in PQH-9; follow-up cards are stubs only
- platform=synthetic_harness allows_product_baseline=false
- attribution: no synthesized L1/L2/L3, stage, residual, queue, CPU, or OS inputs; missing evidence → mixed_or_unknown
- Synthetic/harness platform: do not publish absolute MB/s as product qualification
- all ranked bottlenecks are mixed_or_unknown (expected without full ladder/stage/OS evidence)
- WITHDRAWN: any prior smoke-mode primary bottleneck claim (including io_queue_underdriven from PQH-11 smoke multi-rep) is not qualification evidence
- run_class=smoke: functional harness only; no product bottleneck verdicts
- process slots ran sequential in-process (spawn_workers=false); multiproc OS claim not made
- observer overhead cell=L4-durable-s16384-c1-o8-43-pin-b1-c4-o8 mean=0.014566 median=0.014566 pairs=2 stop="smoke_op_cap" within_budget=true
- bottleneck attach refused: run_class=smoke cannot support registered verdicts
- WITHDRAWN finding: io_queue_underdriven (and any other smoke-derived primary bottleneck)
- NON-PRODUCT / smoke: not qualification evidence; no absolute MB/s claims
- real_store multi-rep run_class=smoke valid_runs=6 product_claim_eligible=false surface=real_store_uncontrolled workers_spawned=false worker_pids=[]
- WITHDRAWN: smoke-mode primary bottlenecks (incl. io_queue_underdriven)
- no primary bottleneck attached (smoke or missing sustained window/floors)
- NO optimisations applied; follow-up cards are stubs only
- driver=real_store surface=real_store_uncontrolled product_claim_eligible=false
- Product baseline claim not eligible on this campaign (need real_store + controlled platform + --controlled)
- WITHDRAWN: any prior smoke-mode primary bottleneck claim (including io_queue_underdriven from PQH-11 smoke multi-rep) is not qualification evidence
- run_class=smoke: functional harness only; no product bottleneck verdicts
- process slots ran sequential in-process (spawn_workers=false); multiproc OS claim not made
- observer overhead cell=L4-durable-s16384-c1-o8-43-pin-b1-c4-o8 mean=0.014566 median=0.014566 pairs=2 stop="smoke_op_cap" within_budget=true
- bottleneck attach refused: run_class=smoke cannot support registered verdicts
- WITHDRAWN finding: io_queue_underdriven (and any other smoke-derived primary bottleneck)
- run_class=smoke
- no primary bottleneck (smoke withdrawn, or qualification floors/window unmet)

This disclosure does **not** authorize product marketing numbers without a controlled-runner accept of PQH-9.
