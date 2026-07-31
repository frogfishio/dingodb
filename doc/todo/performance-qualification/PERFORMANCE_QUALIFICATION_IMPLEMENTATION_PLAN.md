# Performance Qualification Harness implementation plan

Status: **ACTIVE — PQH-0 registries labor floor 2026-07-31; blocked on CSQ-12 accept for program entry honesty**

Program: `PQH`

Normative semantics:
[PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md).

## 1. Delivery rule

PQH is the first post-Core Storage Qualification measurement lane. It may run
alongside M1, but it does not authorize speculative optimization.

No package may:

- change durability or verification semantics;
- use a database-produced value as its own correctness oracle;
- hard-code one developer machine;
- publish a product performance claim;
- make CI depend on an absolute workstation throughput; or
- merge the testrig damage/survival result schema with the PQH result schema.

## 2. Proposed code shape

```text
crates/
  residiuum-perf/
    src/
      experiment.rs
      workload.rs
      physical_plan.rs
      probes.rs
      histogram.rs
      correctness.rs
      result.rs
      attribution.rs
      platform/
        macos.rs
        linux.rs
        portable.rs
  residiuum-perf-cli/
    src/main.rs
spec/
  performance/
    pqh-matrix-v1.json
    pqh-metrics-v1.json
    pqh-verdicts-v1.json
    schemas/
    fixtures/
scripts/
  verify-performance-registry.sh
  performance-smoke.sh
```

`residiuum-perf` is an unpublished development crate. Its runner MUST invoke
the same store code used by product builds. The optional stage-probe feature is
compile-time named and absent from ordinary release artifacts.

## 3. Dependency graph

```text
CSQ-12
  ↓
PQH-0  Contract registries
  ├── PQH-1  Safe runner + platform fingerprint
  ├── PQH-2  Deterministic workload engine
  └── PQH-3  Metrics/histogram/result kernel
          ↓
PQH-4  L0/L1 device envelope
          ↓
PQH-5  Shared PhysicalWritePlan + L2 shadow writer
          ├── PQH-6  L3 CPU pipeline + stage probes
          └── PQH-7  L4/L5/L6 database matrix
                       ↓
PQH-8  Attribution analyzer + synthetic false narratives
                       ↓
PQH-9  Repetition campaign + accepted evidence bundle
```

## 4. PQH-0 — Contract registries

Depends: `CSQ-12`

Deliver:

- matrix, metric, stage, layer, validity and verdict registries;
- JSON Schemas for manifests/results/comparisons;
- accepted and rejected fixtures;
- stable profile identifiers;
- registry verification script; and
- compile-time Rust representations generated from or checked against the
  registries.

Exit:

- every layer, metric, verdict and omission reason is closed;
- unknown identifiers fail closed;
- schemas reject missing units and false zero-as-unavailable values; and
- CI verifies spec/code agreement.

## 5. PQH-1 — Safe runner and environment fingerprint

Depends: `PQH-0`

Deliver:

- dedicated-root marker protocol;
- destructive-target rejection;
- byte/time/free-space budgets;
- preflight and environment validity classifier;
- macOS, Linux and portable adapters;
- debug-build rejection for qualification;
- run directory/artifact writer; and
- signal-safe cancellation that retains an invalid partial report.

Tests:

- root/home/repository/mount-root rejection;
- symlink/path replacement;
- wrong or stale marker;
- non-empty foreign directory;
- ENOSPC/free-space floor;
- cancellation and crash residue;
- unavailable metric representation; and
- environment hash stability.

Exit:

- the runner cannot delete or fill an unowned path;
- every interruption leaves a classifiable artifact; and
- platform fingerprints satisfy the specification.

## 6. PQH-2 — Deterministic workload engine

Depends: `PQH-0`

Deliver:

- counter-based deterministic generator;
- fixed-size series and threshold probes;
- canonical distributions;
- operation stream for insert, rewrite and history;
- compressibility profiles;
- batch/concurrency/outstanding-operation scheduler;
- workload manifest and digest oracle; and
- stream replay without storing payloads.

Tests:

- same seed/config produces the same operation digest across platforms;
- partitioning across producers does not change the logical stream;
- every size/distribution boundary is hit;
- generated keys and payloads never enter results;
- generator throughput is measured by L0; and
- large values stream without workload-size memory growth.

Exit:

- the independent expected digest and byte/operation counts are stable; and
- L0 can prove when the generator itself would contaminate a result.

## 7. PQH-3 — Metrics, probes and result kernel

Depends: `PQH-0`

Deliver:

- fixed-bucket HDR-compatible latency histograms;
- monotonic stage clock;
- deterministic sampling;
- fixed stage and counter registries;
- bounded per-thread aggregation with merge after timing;
- process/host sampler interface;
- result, histogram and timeseries writers;
- artifact hashing; and
- probe-off/aggregate/sampled modes.

Tests:

- monotonicity and reordered timestamp rejection;
- percentile/golden-vector correctness;
- counter saturation/overflow;
- thread aggregation without lost counts;
- unavailable versus zero;
- no timed-path artifact I/O;
- bounded memory;
- disabled probes compile to no calls in ordinary builds; and
- measured probe overhead fixtures.

Exit:

- synthetic distributions match independent golden calculations; and
- the runner can invalidate instrumentation that exceeds its budget.

## 8. PQH-4 — L0/L1 calibration and device envelope

Depends: `PQH-1`, `PQH-2`, `PQH-3`

Deliver:

- L0 generator/copy/clock calibration;
- safe disposable-file sequential and positioned I/O;
- buffered and supported direct/non-cached modes;
- configurable block size, workers and outstanding depth;
- sync mode/cadence controls;
- cold-state honesty classifier;
- finite/sustained-window detector; and
- L1 report.

Tests use a fake I/O adapter to inject:

- bandwidth ceiling;
- queue-depth-dependent scaling;
- fixed and variable sync delays;
- short writes;
- EINTR/EIO/ENOSPC;
- partial completion; and
- dirty-throttle-like periodic pauses.

Exit:

- L1 maps the throughput/latency curve across block size and queue depth;
- unsupported direct/cold modes are honest; and
- no raw device is opened.

## 9. PQH-5 — PhysicalWritePlan and L2 shadow writer

Depends: `PQH-4`

Deliver:

- closed `PhysicalWritePlan` emitted by the authoritative store boundary;
- trace schema containing sizes/order/destination class/sync boundary but no
  payload or identity;
- replay validator;
- opaque-byte shadow executor;
- equivalence counters between planned real-store I/O and shadow I/O; and
- segment rotation/chunk/metadata/batch/sync profiles.

The store and shadow writer MUST consume the same plan semantics. If the store
cannot expose this seam without changing outcomes, PQH-5 stops and records an
architecture finding rather than duplicating the logic.

Tests:

- plan golden vectors;
- trace redaction;
- plan/replay ordering;
- partial and failed I/O;
- segment threshold boundaries;
- chunk threshold boundaries;
- sync cadence;
- planned/requested byte accounting; and
- real/shadow syscall-shape equivalence on a small deterministic stream.

Exit:

- L2 is a defensible ceiling for the selected Residiuum physical shape.

## 10. PQH-6 — L3 CPU pipeline and stage probes

Depends: `PQH-5`

Deliver:

- bounded null/memory sink;
- independently selectable validation, encoding, integrity, chunking,
  manifest and index-preparation stages;
- store stage-probe integration;
- queue, lock, copy, allocation and lifecycle measurements;
- sampled timeline consistency checker; and
- residual accounting.

Tests:

- output digest equals the real pipeline for the same stream;
- no filesystem operation occurs in L3;
- injected CPU work is attributed correctly;
- injected lock and queue delays are distinguished;
- omitted/reordered stages invalidate the run; and
- stage residual calculation matches independent vectors.

Exit:

- L3 establishes a CPU ceiling by payload size and producer count; and
- sampled stage accounting closes within the normative bound on controlled
  fixtures.

## 11. PQH-7 — L4/L5/L6 matrix runner

Depends: `PQH-5`, `PQH-6`

Deliver:

- minimal authoritative store driver;
- additive feature profiles;
- complete database-path driver;
- durability and acknowledgement ledger;
- independent post-run digest/reopen check;
- background-interference profiles;
- seeded/counterbalanced matrix scheduler;
- steady-state detector; and
- matched comparison selector.

Tests:

- fixed sizes and all threshold ±1 probes;
- canonical distributions;
- all durability modes;
- concurrency, outstanding, batch and shard ladders;
- empty/populated/rewrite/fragmented states;
- seal/checkpoint crossings;
- chunked large values;
- errors retained in attempted/admitted/acknowledged counts;
- durability weakened by a mutant is rejected; and
- acknowledged data mismatch invalidates the run.

Exit:

- the full matrix can execute from one manifest without manual intervention;
- database and shadow layers are matchable; and
- correctness remains a hard interlock.

## 12. PQH-8 — Attribution analyzer

Depends: `PQH-7`

Deliver:

- matched-run validator;
- throughput retention, efficiency, amplification and scaling calculations;
- latency accounting/residual calculation;
- repetition statistics and bootstrap confidence intervals;
- registered bottleneck classifier;
- alternative-explanation checklist;
- ranked falsification experiment generator;
- candidate-tuning comparison; and
- human Markdown plus canonical JSON reports.

Required false-narrative fixtures:

- idle aggregate CPU hiding one saturated core;
- page-cache burst mistaken for sustained device throughput;
- per-operation sync hidden by aggregate bandwidth;
- failed operations excluded from denominator;
- higher throughput caused by weaker durability;
- multi-store capacity called single-store scaling;
- queue starvation called disk saturation;
- lifecycle spikes hidden by means;
- observer overhead called database cost; and
- two changed variables called causal.

Exit:

- every injected defect in specification §17 is detected;
- the analyzer chooses `mixed_or_unknown` when evidence is insufficient; and
- mutation testing kills false causal conclusions.

## 13. PQH-9 — Qualification campaign

Depends: `PQH-0`…`PQH-8`

Deliver:

- macOS Apple Silicon campaign;
- Linux controlled-runner campaign;
- at least five repetitions across two fresh processes per accepted cell;
- fixed-size, distribution, concurrency and durability reports;
- current 4 KiB/8 KiB multi-process finding;
- ranked measured bottlenecks;
- raw hashed evidence bundle;
- Benchmark Disclosure summary; and
- documented follow-up optimization cards containing reproduced run IDs.

No optimization is part of PQH-9. The campaign selects subsequent work.

Exit:

- an independent reviewer reproduces one attribution;
- variability and observer budgets pass;
- each bottleneck verdict names falsifying evidence;
- no result overstates the measured surface; and
- the scoreboard may mark `PQH-9 = accept`.

## 14. CLI contract

Proposed surface:

```text
residiuum-perf preflight --work <dedicated-dir>
residiuum-perf calibrate --work <dir> --profile diagnostic
residiuum-perf run --matrix spec/performance/pqh-matrix-v1.json --work <dir>
residiuum-perf replay --manifest <manifest.json> --work <dir>
residiuum-perf compare <run-a> <run-b>
residiuum-perf analyze <campaign-dir>
residiuum-perf verify <run-or-campaign>
```

All commands support canonical JSON output. Progress output is sparse and
outside timed intervals. Raw high-volume measurements go to bounded artifacts,
not stdout, ordinary log files, production telemetry or the Evidence Ledger.

## 15. CI and scheduled execution

| Lane | Packages/features | Frequency |
|---|---|---|
| PR | registries, schemas, analyzer vectors, fake I/O, smoke | every change |
| nightly | smoke matrix, correctness, instrumentation overhead | nightly |
| controlled macOS | diagnostic/qualification matrix | scheduled/manual |
| controlled Linux | diagnostic/qualification matrix | scheduled/manual |
| soak | selected sustained/lifecycle/interference profiles | explicit |

Only controlled runners maintain performance baselines. General CI validates
the harness but does not gate absolute throughput.

## 16. Definition of done

PQH is complete when it can take the observed statement:

> Four processes produced approximately 280 MB/s while the machine appeared
> underutilized.

and replace it with a reproducible statement of this form:

```text
L1 device envelope:                 ... MB/s
L2 Residiuum-shaped I/O ceiling:    ... MB/s
L3 CPU transformation ceiling:      ... MB/s
L6 acknowledged database path:      ... MB/s
requested/device write amp:         ... / ...
queue and stage accounting:         ...%
primary verdict:                    <registered verdict>
effect size and 95% CI:             ...
falsified alternatives:             [...]
next smallest experiment:           ...
```

Anything less is a benchmark collection, not the required performance
qualification harness.
