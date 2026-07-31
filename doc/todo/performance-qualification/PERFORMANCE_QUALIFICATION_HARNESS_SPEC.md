# Residiuum Performance Qualification Harness specification

Status: **normative design v1.0-draft — developer-ready after CSQ-12**

Program: `PQH`

Profile:

```text
residiuum-performance-qualification-v1
```

Normative companions:

- [Core Storage Qualification](../core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md)
- [Crash and recovery contract](../../reference/operations/CRASH_AND_RECOVERY_CONTRACT.md)
- [Benchmark disclosure](../../reference/operations/BENCHMARK_DISCLOSURE.md)
- [Performance strategies](../../reference/operations/PERFORMANCE_STRATEGIES.md)
- [Telemetry specification](../telemetry/TELEMETRY_SPEC.md)
- [Testing strategy](../../reference/engineering/TESTING_STRATEGY.md)

Implementation order is defined by
[PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md](PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md).

## 1. Purpose

The harness answers one question:

> For a fixed logical workload on a fixed machine, where does each unit of
> throughput disappear and where does each unit of latency arise between the
> payload generator and Residiuum acknowledgement?

It MUST distinguish:

1. physical-device and filesystem limits;
2. the cost of Residiuum-shaped I/O;
3. CPU transformation cost;
4. queueing and synchronization cost;
5. indexing and lifecycle cost;
6. durability cost;
7. background-work interference; and
8. complete database-path performance.

It MUST produce a reproducible causal report, not merely an ops/s number.

The first motivating observation is a buffered 4-process campaign near
`280 MB/s` on an M4-class machine while aggregate process CPU remained below
the available CPU capacity. That observation is a hypothesis generator only.
The harness MUST be able to prove whether the unused capacity comes from I/O
submission, serialization, locks, queues, durability barriers, lifecycle work,
memory pressure, or another measured cause.

## 2. Non-goals

V1 does not:

- change storage semantics to make a benchmark pass;
- choose a tuning value before measuring it;
- treat multi-store throughput as product sharding;
- claim raw block-device access;
- write directly to a mounted block device;
- replace correctness, crash, damage, or recovery qualification;
- use production telemetry as the high-resolution profiler;
- compare Residiuum with another database;
- publish an SLO automatically; or
- declare a bottleneck from CPU percentage alone.

`residiuum-testrig` remains the scale/damage/survival rig. PQH MAY reuse its
payload generation and store drivers, but its experiment engine, clocks,
metrics and result schema are separate.

## 3. Measurement law

Every conclusion MUST come from a matched pair or matched ladder:

- same host;
- same dedicated filesystem and work root;
- same payload bytes and key sequence;
- same logical byte count;
- same duration/steady-state rule;
- same concurrency and queue policy where applicable;
- same acknowledgement/durability boundary;
- same warm/cold state;
- same instrumentation mode; and
- randomized or counterbalanced run order.

Changing two independent variables in one comparison invalidates causal
attribution.

Every result is one of:

```text
valid
invalid_environment
invalid_correctness
invalid_instrumentation
inconclusive
```

Only `valid` results enter an attribution report. An invalid run is retained.

## 4. Safety boundary

The harness MUST operate only inside an explicitly supplied, dedicated work
directory. It MUST:

- reject `/`, a home directory, repository root, mount root, or non-empty
  unowned directory;
- create and verify a run marker containing the run ID;
- resolve paths before deletion;
- delete only paths carrying the matching marker;
- preflight free bytes and inodes;
- reserve a configurable free-space floor;
- stop before crossing that floor;
- use ordinary files on the selected filesystem;
- never open a raw block device for writing;
- record filesystem, mount and storage-device identity; and
- cleanly report `invalid_environment` on ENOSPC, thermal intervention,
  sleep, power-mode change, or observer failure.

The default profile MUST be safe on a developer workstation. Large and soak
profiles require an explicit byte/time budget.

## 5. Experiment ladder

Each layer consumes the same deterministic operation stream. Layer output
includes logical operations, logical payload bytes, requested physical bytes,
completed physical bytes, elapsed time, latency distribution and resource
samples.

### L0 — generator and clock calibration

Measure without storage:

- deterministic key/payload generation;
- memory copy into the requested buffers;
- histogram recording;
- monotonic clock reads;
- probe-disabled versus probe-enabled overhead; and
- result serialization after the timed interval.

L0 establishes the observer floor. Result serialization, console output and
artifact hashing MUST occur outside the timed interval.

### L1 — filesystem/device envelope

Use disposable ordinary files on the target filesystem to measure:

- sequential append;
- positioned random write where supported;
- sequential read;
- positioned random read;
- buffered writes;
- direct/non-cached I/O where safely and portably supported;
- data-only and full-file synchronization;
- directory synchronization where the qualified durability contract requires
  it;
- configurable block size;
- configurable outstanding I/O/worker count; and
- finite and sustained runs.

L1 is called the device envelope, not “raw disk speed.” Page cache, filesystem,
encryption and the operating system remain part of the observed path and MUST
be disclosed.

### L2 — Residiuum-shaped shadow writer

Write the exact byte sizes, file rotation pattern, alignment, append order,
metadata cadence, batching, segment thresholds and synchronization schedule
that the selected Residiuum configuration requests, but use generated opaque
bytes and perform no:

- document parsing;
- frame encoding;
- checksumming or hashing;
- chunk-manifest computation;
- index mutation; or
- database recovery/publication logic.

The shadow writer MUST share a closed `PhysicalWritePlan` type with the real
store or consume a trace emitted from that type. It MUST NOT independently
reimplement assumed I/O behavior.

L2 answers: “What is the ceiling imposed by Residiuum’s physical I/O shape?”

### L3 — CPU transformation pipeline

Execute key validation, frame encoding, integrity calculation, chunking,
manifest construction and optional index preparation against a bounded
memory/null sink. Each substage is independently selectable.

No filesystem operation may occur inside the measured L3 interval.

L3 answers: “How much work can the CPU pipeline sustain if storage never
waits?”

### L4 — storage pipeline without derived features

Run the authoritative store with the minimum qualified primary publication
path. Secondary indexes, optional derived sidecars, maintenance and telemetry
are disabled only when the selected profile permits them to be disabled.

Correctness and acknowledgement semantics remain identical to the declared
profile.

### L5 — additive feature ladder

Enable exactly one feature family at a time, then the declared realistic set:

1. primary index/publication;
2. each secondary index independently;
3. all selected secondary indexes;
4. inline values;
5. chunked values;
6. seal/rotation lifecycle;
7. derived checkpoint/sidecar work;
8. integrity verification;
9. encryption when available;
10. bounded production telemetry; and
11. chaos/scrub/background maintenance as a separately named interference
    profile.

An option that changes correctness or durability MUST be labeled as a
different product profile, not a faster configuration of the same profile.

### L6 — complete database path

Measure:

```text
operation generation
→ admission
→ queue
→ validation
→ encoding/integrity
→ chunk/manifest preparation
→ index preparation
→ I/O submission
→ I/O completion
→ publication
→ durability barrier
→ acknowledgement
```

Applicable surfaces:

- direct `residiuum-store`;
- embedded public API after APB exists;
- server/RPC/remote SDK after those paths qualify; and
- cluster/replicated path only under a future profile.

Results from different surfaces MUST NOT be merged.

## 6. Canonical workload matrix

### 6.1 Fixed payload sizes

The mandatory fixed-size series is:

```text
256 B
1 KiB
4 KiB
8 KiB
16 KiB
64 KiB
256 KiB
1 MiB
4 MiB
16 MiB
```

The series deliberately crosses inline/chunk boundaries and RPC-frame
boundaries. A value rejected by the selected admission profile is recorded as
`not_applicable`, never silently omitted.

Boundary probes MUST also test:

```text
threshold - 1
threshold
threshold + 1
```

for every active inline, chunk, frame, batch and admission threshold.

### 6.2 Distributions

Mandatory distributions:

- `tiny`: 90% 256 B, 9% 4 KiB, 1% 64 KiB;
- `document`: log-normal, median 4 KiB, p95 64 KiB, capped at 1 MiB;
- `mixed-large`: 70% 4 KiB, 20% 64 KiB, 9% 1 MiB, 1% 16 MiB;
- `rewrite-heavy`: stable key set with geometric rewrite frequency;
- `append-history`: unique keys plus repeated generations;
- `chunk-boundary`: values concentrated around the chunk threshold; and
- one user-supplied manifest containing sizes only, never production payloads.

All generated bytes are deterministic from a recorded seed and MUST be
incompressible or compressible according to an explicit payload profile.

### 6.3 Control dimensions

Mandatory values:

| Dimension | Values |
|---|---|
| producer concurrency | 1, 2, 4, 8, 16; bounded by host |
| outstanding operations | 1, 2, 4, 8, 16, 32, 64 |
| batch size | 1, 8, 64, 512, 4096 |
| writer shards | 1, 2, 4, 8 where supported |
| durability | memory, buffered, durable; replicated later |
| database state | empty, steady populated, rewrite-heavy, fragmented |
| cache state | warm; cold only where the platform can establish it honestly |
| lifecycle | below rotation, crossing rotation, sustained rotations |
| background work | absent, seal/checkpoint, scrub, chaos, telemetry |

The runner MUST prune impossible or redundant cells using a committed matrix
registry and explain every omission.

#### 6.3.1 Canonical sweep construction

The mandatory matrix is structured, not the full Cartesian product:

1. **Size sweep:** every fixed size at concurrency 1, outstanding 1, batch 1
   for each durability mode.
2. **Submission sweep:** payloads 1 KiB, 4 KiB, 16 KiB, 256 KiB and 4 MiB
   across every concurrency and outstanding-operation value, batch 1, for
   buffered and durable modes.
3. **Batch sweep:** payloads 1 KiB, 4 KiB, 16 KiB and 256 KiB across every
   batch size at concurrency 1 and 4.
4. **Shard sweep:** payloads 4 KiB, 8 KiB, 64 KiB and 1 MiB across every
   supported shard count at matched aggregate concurrency.
5. **Distribution sweep:** every canonical distribution at concurrency 1, 4
   and the highest non-oversubscribed host value.
6. **State/lifecycle sweep:** 4 KiB, 64 KiB and 1 MiB against every database
   state and lifecycle profile.
7. **Boundary sweep:** every threshold ±1 at concurrency 1 and 4.
8. **Interference sweep:** the 4 KiB, 8 KiB and mixed-large profiles against
   each background-work profile.

The registry MAY add cells selected by pairwise covering-array generation.
It MUST NOT silently expand to an infeasible Cartesian product or cherry-pick
only favorable cells.

### 6.4 Run classes

| Class | Purpose | Minimum measured interval |
|---|---|---:|
| smoke | functional harness verification | 3 s and 64 MiB |
| diagnostic | local bottleneck search | 30 s and 2 GiB where safe |
| qualification | repeatable accepted evidence | 120 s and enough bytes to leave initial burst behavior |
| soak | thermal, cache, lifecycle and long-tail behavior | explicit, normally ≥1 h |

Both time and byte conditions apply unless the safety floor would be crossed.
Qualification MUST demonstrate a steady-state interval; otherwise the result is
`inconclusive`.

## 7. Timing and probes

The authoritative clock is monotonic. Wall time is metadata only.

Every sampled operation receives:

```text
t_generated
t_admitted
t_enqueued
t_dequeued
t_validated
t_encoded
t_chunk_ready
t_index_ready
t_io_submitted
t_io_completed
t_published
t_sync_completed
t_acknowledged
```

Stages that do not apply are explicitly absent. Timestamps MUST NOT be forged
from surrounding stages.

Routine runs use deterministic sampling, default 1 in 1024 operations, plus
fixed-bucket aggregate counters for every operation. Full per-operation timing
is a diagnostic mode and is never the default throughput result.

Every experiment has:

1. probes disabled;
2. aggregate probes enabled;
3. sampled stage probes enabled.

If median throughput loss from aggregate probes exceeds 1%, or sampled probes
exceed 3% at the default sample rate, the instrumentation is
`invalid_instrumentation` until corrected or the overhead is explicitly
subtracted with confidence bounds.

Probe code MUST:

- allocate no per-operation map;
- write no per-operation file or network message;
- use fixed identifiers and bounded memory;
- avoid a global mutex;
- never inspect payload contents; and
- remain outside production builds unless the fixed cheap aggregate is part
  of the Telemetry contract.

## 8. Required measurements

### 8.1 Operation and stage measurements

- attempted, admitted, acknowledged and failed operations;
- logical payload and key bytes;
- end-to-end latency p50/p90/p95/p99/p99.9/max;
- per-stage service and queue residence distributions;
- active and maximum queue depth;
- batch fill and dispatch size;
- lock acquisition wait and hold distributions;
- active writers and outstanding I/O;
- rotations, seals, checkpoints and sync counts/durations;
- logical generations and chunk counts; and
- errors by closed error code.

### 8.2 Process and host measurements

- process and system CPU time;
- normalized CPU utilization and core count;
- context switches;
- resident and peak memory;
- allocations/allocated bytes when allocator instrumentation is enabled;
- bytes copied at registered copy boundaries;
- filesystem bytes requested/completed;
- physical/device bytes where the platform exposes them;
- read/write operations and average request size;
- device utilization, queue depth and wait time where available;
- page faults and page-cache/dirty-page signals where available;
- filesystem free bytes/inodes before, during and after;
- thermal/power state where available; and
- competing host I/O/CPU indicators.

Unsupported metrics are recorded as unavailable with a reason. Zero is not a
substitute for unavailable.

### 8.3 Platform adapters

V1 MUST provide:

- macOS adapter for Apple Silicon developer qualification;
- Linux adapter for CI/server qualification; and
- a portable fallback containing process clocks, memory, filesystem and
  application probes.

Platform commands and kernel counters are adapters behind a stable schema.
Their raw outputs are retained as hashed attachments.

## 9. Attribution mathematics

For experiment layer \(i\):

\[
T_i = \frac{B_{\mathrm{logical,ack}}}{t_{\mathrm{steady}}}
\qquad
O_i = \frac{N_{\mathrm{ack}}}{t_{\mathrm{steady}}}
\]

Throughput retention between matched layers:

\[
R_i = \frac{T_i}{T_{i-1}}
\qquad
L_i = 1-R_i
\]

End-to-end efficiency against the Residiuum-shaped I/O ceiling is:

\[
E_{\mathrm{shape}} = \frac{T_{\mathrm{L6}}}{T_{\mathrm{L2}}}
\]

Write amplification is reported in both requested and observed forms:

\[
WA_{\mathrm{requested}} =
\frac{B_{\mathrm{filesystem\ requested}}}{B_{\mathrm{logical,ack}}}
\]

\[
WA_{\mathrm{device}} =
\frac{B_{\mathrm{device\ written}}}{B_{\mathrm{logical,ack}}}
\]

The second is unavailable when the platform cannot observe it.

Concurrency scaling and efficiency are:

\[
S(n)=\frac{T(n)}{T(1)}
\qquad
P(n)=\frac{S(n)}{n}
\]

For sampled operation \(j\), latency accounting is:

\[
\ell_j =
q_j + v_j + e_j + c_j + x_j + io_j + p_j + d_j + \epsilon_j
\]

where the terms are queue, validation, encoding, chunking, indexing, I/O,
publication, durability and uninstrumented residual. The report MUST show
\(\epsilon\). If absolute median residual exceeds 5% of median end-to-end
latency, the attribution is incomplete and no stage-level “primary bottleneck”
verdict may be issued.

For each reported median or throughput, qualification uses repeated runs and
reports median, minimum, maximum, median absolute deviation and a bootstrapped
95% confidence interval. A single run cannot support a tuning decision.

## 10. Experimental protocol

Each cell follows:

1. preflight;
2. record immutable run manifest;
3. prepare or recreate the work root;
4. establish declared cache/database state;
5. calibration and warm-up;
6. wait for stable window;
7. measure without console output;
8. drain outstanding work according to the acknowledgement profile;
9. verify every acknowledged value/count using an independent digest;
10. collect post-run filesystem and resource state;
11. serialize and hash artifacts;
12. classify validity; and
13. restore the configured free-space state.

Run order is seeded and counterbalanced to reduce thermal/cache/order bias.
Qualification requires at least five valid repetitions per cell across at
least two fresh process starts. Cross-machine aggregation is forbidden; each
machine produces its own report.

Steady state requires all of:

- no monotonic throughput trend larger than 10% across the accepted window;
- no initial allocation/preallocation interval in the accepted window;
- at least one applicable segment lifecycle event for sustained profiles;
- no free-space-floor breach;
- no thermal or power invalidation; and
- queue and latency samples covering the entire accepted window.

## 11. Bottleneck verdicts

The analyzer may emit only registered verdicts:

| Verdict | Required evidence |
|---|---|
| `generator_bound` | L0 consumes the dominant capacity; changing storage layers does not explain loss |
| `cpu_transform_bound` | L3 saturates available CPU and predicts L4/L6 ceiling |
| `io_bandwidth_bound` | device bandwidth/utilization near L1/L2 envelope with outstanding work present |
| `io_queue_underdriven` | L2/L6 below envelope, device not busy, low outstanding I/O, throughput rises with queue depth |
| `serialized_writer` | throughput plateaus, available CPU/device capacity remains, and lock/single-stage residence dominates |
| `durability_barrier_bound` | sync duration/cadence explains loss and matched buffered run removes it |
| `lifecycle_bound` | seal/checkpoint/rotation windows explain tails or sustained loss |
| `index_bound` | additive index layer and index stage time explain matched loss |
| `memory_pressure_bound` | faults/reclaim/RSS/dirty throttling align with loss and controlled memory change reproduces it |
| `telemetry_bound` | telemetry-on/off matched pair exceeds its declared budget |
| `mixed_or_unknown` | evidence cannot isolate one cause |

A verdict MUST cite:

- compared run IDs;
- effect size and confidence interval;
- stage accounting closure;
- correctness validation;
- contradicted alternative explanations; and
- the smallest follow-up experiment that could falsify it.

The analyzer MUST NOT output tuning advice for `mixed_or_unknown`.

## 12. Tuning experiment contract

The harness identifies problems; a tuner changes one registered parameter and
reruns the affected cells.

Candidate parameters include:

- segment and rotation thresholds;
- chunk threshold and chunk size;
- batch size and maximum batch delay;
- producer/consumer queue capacity;
- outstanding I/O depth;
- writer shard count;
- sync/group-commit cadence;
- index publication strategy;
- checkpoint/seal scheduling;
- worker count and affinity where supported; and
- bounded cache allocation.

For candidate \(c\), acceptance requires:

\[
\Delta T > 0
\]

with a 95% confidence interval excluding zero, while:

- correctness remains green;
- declared durability is unchanged;
- p99/p99.9 do not exceed the profile budget;
- memory and write amplification remain within declared bounds;
- no workload class regresses by more than the registered tolerance; and
- the improvement reproduces after reverting and reapplying the change.

Automatic application of tuning recommendations is outside V1. V1 emits a
ranked, evidence-linked recommendation file.

## 13. Artifact contract

Every run writes:

```text
run/
  manifest.json
  result.json
  timeseries.ndjson.zst
  histograms.json
  correctness.json
  environment.json
  attachments/
  hashes.json
```

Canonical result profile:

```text
residiuum-performance-result-v1
```

Required manifest fields:

```text
profile
run_id
source_revision
dirty_tree_hash
binary_hashes
configuration_hash
workload_id
seed
layer
surface
durability
payload_profile
payload_size
dataset_state
logical_target_bytes
time_budget
concurrency
outstanding_limit
batch_size
writer_shards
instrumentation_mode
platform_adapter
start/end
```

Required result fields include all metrics in §8, validity, exclusions,
warnings, artifact hashes and correctness outcome.

The comparison report contains matched run IDs, formulas, confidence
intervals, attribution closure, verdicts and ranked follow-up experiments.

Artifacts MUST contain no document payload, key, credential, Heap identity,
raw query, path outside the dedicated work root, or production data.

## 14. Correctness interlock

No throughput sample counts an operation unless it reaches the declared
acknowledgement boundary.

After each database-layer run:

- acknowledged operation count matches the driver ledger;
- deterministic aggregate digest matches independently computed expectation;
- reopened state is checked when durability claims require it;
- error count is included in the denominator and report;
- partial/incomplete outcomes remain explicit; and
- no background operation remains unaccounted.

Any mismatch makes the run `invalid_correctness` and opens a defect. A faster
incorrect run is never a performance result.

## 15. Reproducibility and environmental validity

The environment report MUST disclose:

- hardware model, CPU topology, RAM;
- OS/kernel version;
- filesystem, mount options and volume encryption;
- storage device/model when observable;
- free/total bytes and inodes;
- power source/mode and thermal state when observable;
- compiler/profile/build features;
- process limits;
- database format/profile;
- active background features;
- observer availability;
- workload and run order seed; and
- all deviations from the canonical profile.

Qualification comparisons require:

- median coefficient of variation within 5%, or explicit `inconclusive`;
- no run with environmental invalidation;
- identical semantic configuration;
- no debug build;
- no mixed source revision;
- no silently unavailable required metric; and
- raw artifacts retained.

## 16. Continuous integration

PR CI runs:

- schema and registry validation;
- synthetic analyzer tests;
- smoke L0/L2/L3/L6 on temporary storage;
- instrumentation-overhead absurdity bounds; and
- correctness interlock tests.

PR CI MUST NOT enforce a universal MB/s threshold.

Scheduled controlled runners execute diagnostic/qualification matrices.
Regression gates compare a runner only with its own accepted baseline and
require:

- matched environment class;
- median throughput regression greater than 15% with confidence;
- or p99 regression greater than 25%;
- or memory/write-amplification regression greater than its declared budget.

A regression alert is not automatically a correctness failure. It blocks new
performance claims and requires triage.

## 17. Harness self-tests

The harness is not accepted until it detects:

1. an injected fixed queue delay;
2. an injected CPU burn;
3. an injected lock serialization point;
4. a forced per-operation sync;
5. an injected lifecycle pause;
6. a deliberately under-driven I/O queue;
7. dropped/failed operations hidden by a false throughput counter;
8. a timestamp omitted or reordered;
9. counter overflow and histogram saturation;
10. observer overhead above budget;
11. ENOSPC and free-space-floor intervention;
12. thermal/power/environment invalidation where simulatable;
13. nondeterministic payload/operation generation;
14. mismatched run pairing;
15. stale binary/source identity;
16. unavailable platform metrics represented falsely as zero; and
17. a recommendation that improves throughput by weakening durability.

Mutation tests MUST prove the analyzer rejects these false narratives.

## 18. Acceptance

`residiuum-performance-qualification-v1` may be claimed only when:

- `PQH-0` through `PQH-9` are accepted;
- all ladder layers execute on macOS and Linux or declare a justified
  unsupported submode;
- the fixed-size and distribution matrices execute;
- stage accounting closes within the specified residual bound;
- observer overhead is within budget;
- correctness interlocks pass;
- synthetic bottlenecks are correctly classified;
- repeated-run variability meets the contract;
- the current 4 KiB/8 KiB multi-process observation has an evidence-backed
  verdict;
- reports comply with Benchmark Disclosure; and
- an independent reviewer can reproduce one complete attribution from the
  stored manifest and raw artifacts.

The first accepted output is not “Residiuum is fast.” It is:

> On this disclosed platform and profile, Residiuum retains X% of its
> I/O-shaped ceiling; Y is the largest measured loss; this experiment
> falsifies the principal alternatives.
