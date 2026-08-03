# Adaptive Write Optimiser implementation plan

Status: **normative target v1.0-draft — implementation partially delivered; not current status authority**

> Delivery truth lives in [CURRENT_STATE.md](CURRENT_STATE.md). This document
> specifies the target architecture, not present delivery status.

Program: `AWO`  
Normative semantics: [ADAPTIVE_WRITE_OPTIMISER_SPEC.md](ADAPTIVE_WRITE_OPTIMISER_SPEC.md)  
Executable contracts: [spec/performance/awo/](../../../spec/performance/awo/)

This document closes implementation choices. Developers MUST NOT substitute an
async runtime, queue library, controller formula, ordering rule, or
acknowledgement interpretation without amending this plan and its golden vectors.

## 1. Entry conditions and delivery rule

AWO implementation may begin only when admitted by
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md). Package dependencies
remain enforced even if several packages are placed on the board together.

Entry evidence:

1. core-storage acknowledgement and recovery suites are green;
2. PQH can measure L3 cooking and L4 real-store paths with honest boundary
   events;
3. the AWO registries pass `scripts/verify-awo-contract.sh`; and
4. `AWO-0` is accepted before product-path mutation begins.

AWO-1/2 may qualify the single-store diagnostic lane. AWO-3 product integration
additionally requires the heap-qualified active-writer layout from `HEAP_SPEC`
§34: one heap per segment and `active/<heap-id>/<shard-id>`. A legacy
empty-envelope active segment may be used only by an explicitly non-qualified
diagnostic profile and can never support AWO-G7 or default-on acceptance.

No package may weaken format verification, heap qualification, writer locking,
conditional-write semantics, or durability to improve a number.

## 2. Current code truth

The implementation MUST begin from these facts:

- `residiuum-store` is synchronous and uses standard-library threads;
- the qualified host owns `Arc<Mutex<PhysicalStore>>`;
- each `HeapStore` capability façade shares that physical store;
- the server uses bounded connection worker threads, not Tokio;
- qualified mutations are currently executed synchronously while holding the
  physical-store mutex;
- the existing parallel cooker creates scoped OS threads per batch, clones
  bodies, publishes results through a shared mutex, installs serially, then
  writes;
- existing batch paths may apply in-memory visibility before the final tail
  write succeeds; and
- conditional writes (`WriteCondition`) depend on evaluation under ordered
  store ownership.

AWO therefore uses `std::thread`, `Mutex`, `Condvar`, `VecDeque`, atomics, and
bounded byte-credit accounting. Adding Tokio, Rayon, crossbeam, an actor
framework, or an alternative executor is out of scope for AWO-0…AWO-4.

## 3. V1 support matrix

| Operation class | V1 execution |
|---|---|
| Unconditional inline `put` | AWO eligible |
| Unconditional `delete` | AWO eligible after AWO-3 delete vectors pass |
| Conditional/CAS put or delete | Natural path through the same writer authority |
| Chunked/large-value put | Natural path until an explicit AWO chunk profile |
| `Memory` durability | Natural path |
| `Buffered` | Dedicated AWO lane |
| `Durable` | Dedicated AWO lane |
| Atomics group | Atomics-owned path; never decomposed by AWO |
| Cluster/Raft mutation | Cluster commit owns ordering; local apply MAY use AWO only after a separate profile |
| Maintenance/admin mutation | Fence, drain relevant lane, execute naturally |

“Natural path” does not mean bypassing coordination. When AWO owns a store, all
mutations enter its writer command stream; ineligible work is executed as a
single-operation natural command in ticket order.

## 4. Crate and module ownership

### 4.1 `residiuum-store`

Create:

```text
crates/residiuum-store/src/adaptive_write/
  mod.rs
  types.rs          request, lane, ticket, completion and public status types
  policy.rs         validated policy and machine-derived defaults
  credits.rs        byte/entry credit ledger
  queue.rs          bounded Mutex<VecDeque<T>> + Condvar queues
  estimator.rs      workload buckets and uncertainty envelopes
  selector.rs       exact natural/batch candidate arithmetic
  controller.rs     hysteresis, worker activation and model invalidation
  cooker.rs         persistent parked cooker workers
  ordered_ready.rs  ticket-indexed bounded ready ring
  coordinator.rs    admission, batching, reservation and writer command loop
  persist.rs        staged install, tail I/O, publish and failure poisoning
  telemetry.rs      bounded snapshots/counters; no bodies/subjects
  model.rs          pure executable transition/selector model used by vectors
```

Modify:

```text
crates/residiuum-store/src/lib.rs
crates/residiuum-store/src/error.rs
crates/residiuum-store/src/store.rs
crates/residiuum-format/src/segment.rs         retained-tail checkpoint/restore primitive
crates/residiuum-store/src/heap/host.rs
crates/residiuum-store/src/heap/heap_store.rs
crates/residiuum-store/src/boundary_probe.rs
```

`residiuum-format::ActiveSegment` owns any retained-tail checkpoint/rollback
primitive because writer sequence, frame count, base offset, and retained bytes
must change together.

### 4.2 `residiuum-server`

Modify only after AWO-3:

```text
crates/residiuum-server/src/config.rs
crates/residiuum-server/src/serve.rs
crates/residiuum-server/src/heap_dispatch.rs
crates/residiuum-server/src/metrics.rs
```

The server passes validated policy to `StoreHost`; it does not implement a
second batching queue. Heap authorisation and SubjectV2 validation occur before
AWO admission.

### 4.3 `residiuum-perf`

Create after AWO-3:

```text
crates/residiuum-perf/src/awo/
  mod.rs
  workload.rs
  trace.rs
  compare.rs
  report.rs
```

PQH consumes store-native AWO events. It MUST NOT reconstruct decisions from
receipts.

### 4.4 Specification and formal artifacts

```text
spec/performance/awo/
formal/awo/tla/AdaptiveWrite.tla
formal/awo/tla/AdaptiveWrite.cfg
formal/awo/verus/model.rs
verification/awo/golden/
scripts/verify-awo-contract.sh
scripts/verify-awo.sh
```

## 5. Exact public and internal Rust contracts

Names below are normative for V1 unless compilation requires a documented
collision. Visibility may be narrowed but semantics may not change.

```rust
pub const AWO_PROFILE: &str = "residiuum-adaptive-write-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveWriteMode {
    Disabled,
    Static,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct AdaptiveWritePolicy {
    pub mode: AdaptiveWriteMode,
    pub queue_byte_limit: usize,
    pub queue_entry_limit: usize,
    pub maximum_batch_bytes: usize,
    pub maximum_batch_entries: usize,
    pub maximum_collection_delay: Duration,
    pub default_completion_deadline: Duration,
    pub minimum_active_cookers: usize,
    pub maximum_cookers: usize,
    pub pipeline_depth_limit: usize,
    pub decision_margin_ppm: u32,
    pub estimator_min_samples: u32,
    pub estimator_stale_after: Duration,
    pub controller_interval: Duration,
    pub scale_up_dwell: Duration,
    pub scale_down_dwell: Duration,
}

pub struct AdaptiveWriteRuntime { /* owns threads and shared state */ }

#[derive(Clone)]
pub struct AdaptiveWriteHandle { /* enqueue/control handle only */ }

pub struct WriteCompletion {
    /* one-shot receiver; wait(self) consumes the handle */
}

pub enum AdmissionResult {
    Admitted(WriteCompletion),
    Rejected(AdaptiveWriteError),
}

pub enum AdaptiveWriteError {
    QueueFull { retry_after: Duration },
    AdmissionDeadlineExceeded,
    Draining,
    WriterPoisoned { recovery_required: bool },
    Store(StoreError),
}
```

Internal request:

```rust
struct WriteRequest {
    ticket: LaneTicket,
    operation_id: Option<[u8; 16]>,
    heap_id: [u8; 16],
    subject: Arc<[u8]>,
    body: Option<Arc<[u8]>>,
    kind: EventKind,
    durability: DurabilityMode,
    condition: WriteCondition,
    admitted_at: Instant,
    completion_deadline: Instant,
    reserved_bytes: usize,
    completion: OneShotSender<Result<WriteReceipt, AdaptiveWriteError>>,
}
```

The operation body is owned once as `Arc<[u8]>`; cookers borrow it. AWO MUST
not make the current serial `value.to_vec()` preparation clone.

Required `StoreHost` changes:

```rust
pub fn create_with_adaptive_write(
    path: impl AsRef<Path>,
    policy: AdaptiveWritePolicy,
) -> Result<Self, StoreError>;

pub fn open_with_adaptive_write(
    path: impl AsRef<Path>,
    policy: AdaptiveWritePolicy,
) -> Result<Self, StoreError>;

pub fn adaptive_write_status(&self) -> Option<AdaptiveWriteStatus>;
pub fn drain_writes(&self, deadline: Instant) -> Result<(), AdaptiveWriteError>;
```

Existing `create/open` retain direct/natural semantics until AWO-7 changes the
accepted default. `HeapStore` routes eligible and ineligible writes through the
handle when present. Reads retain the shared physical-store lock.

V1 qualified RPC does not add a caller deadline field. The server assigns
`now + policy.default_completion_deadline` after full request validation.
Adding a caller-selected deadline requires a separately versioned RPC contract;
developers MUST NOT smuggle it into `args` because qualified envelopes reject
unknown fields.

## 6. Single mutation authority

Starting AWO acquires a process-local `AdaptiveWriteLease` inside `Store`. While
the lease exists:

- public direct mutation methods return `StoreError::AdaptiveWriterActive`;
- only coordinator calls carrying the unforgeable private lease token may
  mutate;
- reads, scans, history and diagnostics remain available;
- maintenance obtains a fence and coordinator-owned natural command; and
- dropping runtime without a successful drain marks the store for normal reopen
  recovery before another in-process writer is created.

This prevents a direct `Store::put` from changing segment identity or sequence
between AWO reservation and installation.

## 7. Exact lane and ordering definition

V1 lane key:

```rust
struct LaneKey {
    heap_id: [u8; 16],
    writer_shard: u32,
    durability: DurabilityMode, // Buffered or Durable only
    layout: LayoutClass,        // InlinePut or Delete
}
```

`store_id` is implicit in one runtime. Atomics and cluster operations never
enter this lane. `ordering_domain` is the physical writer shard; same-subject
order follows because subject hashing always chooses the same shard.

The physical active-writer key is `(heap_id, writer_shard)`. Switching logical
lanes never appends heap B to heap A's active segment. If the qualified
heap-specific active writer is unavailable, AWO product admission fails closed;
it does not fall back to the legacy empty-envelope writer.

Every admitted command receives a monotonically increasing `u64` ticket per
lane. Wraparound is fatal/poisoning and is tested through an injected counter.

Conditional commands share the shard command stream but execute naturally.
They cannot be overtaken by later unconditional work on the same shard.

Across non-empty lanes, the coordinator selects the lane whose head request has
the earliest absolute completion deadline. Ties use earliest `admitted_at`, then
lexicographic `LaneKey`. It serves at most one selected batch before choosing
again. This is the V1 fairness algorithm; a local hash-map iteration order is
not an acceptable scheduler.

## 8. Admission algorithm

Admission order is exact:

1. capability liveness and rights check;
2. SubjectV2 heap/object validation;
3. operation-id/content-identity validation where required;
4. payload and layout admission limits;
5. calculate credit:

   ```text
   credit = subject_len + body_len + MAX_FRAME_OVERHEAD
          + size_of(request metadata) + size_of(completion slot)
   ```

6. reserve one entry and `credit` bytes atomically under the credit mutex;
7. assign lane ticket;
8. enqueue;
9. return `Admitted`.

Steps 6–8 occur under one coordinator-admission lock. Failure before step 8
returns credit. Once step 8 succeeds, caller cancellation only detaches waiting.

`MAX_FRAME_OVERHEAD` is computed from format constants and maximum admitted
envelope length, not a guessed number. Queue accounting uses checked arithmetic;
overflow rejects before admission.

## 9. Persist-before-publish implementation

AWO-1 introduces these internal store concepts:

```rust
struct BatchReservation {
    lease_id: u64,
    segment_id: [u8; 16],
    segment_generation: u64,
    first_writer_sequence: u64,
    start_offset: u64,
    frame_count_before: u64,
    retained_len_before: usize,
    requests: Vec<ReservedMutation>,
}

struct CookedMutation {
    ticket: LaneTicket,
    encoded_frame: Vec<u8>,
    publication: PreparedPublication,
}
```

Reservation procedure:

1. coordinator selects a batch whose conservative encoded upper bound fits the
   active segment;
2. lock store and process any required prior rotation;
3. validate operation identities and natural/conditional barriers in ticket
   order;
4. assign segment, item/event identity, writer sequence and timestamp;
5. pre-reserve publication/index capacity;
6. capture exact active-segment checkpoint;
7. release lock and cook immutable frames.

Only one unresolved reservation exists per writer shard in V1. Pipeline overlap
comes from writing batch A while CPU workers cook already-reserved batch B; no
third reservation may pass B.

Persistence procedure:

1. reacquire store with lease token;
2. verify segment id, generation, sequence, offset and checkpoint unchanged;
3. install pre-encoded frames in ticket order;
4. perform one tail write and required durability barrier;
5. on complete success, apply prepared publications in ticket order using
   pre-reserved capacity;
6. construct individual receipts;
7. release store lock;
8. resolve completions outside the lock;
9. release queue credit.

If install fails before I/O, restore the exact segment checkpoint. If physical
I/O is short, errors, or its durability barrier fails:

- publish nothing from the affected reservation;
- fail/mark uncertain every affected request;
- poison the adaptive writer;
- refuse further mutation;
- require ordinary close/reopen recovery before resuming.

Continuing to append behind an uncertain physical tail is forbidden.

`PreparedPublication` includes primary-index visibility, history/locator facts,
collection notes, and secondary-index stale transitions. `HeapStore` MUST NOT
acknowledge and then reacquire the store to mark indexes stale: all correctness-
relevant publication consequences occur before completion distribution.

`ActiveSegment` gains an internal checkpoint/restore operation covering
`bytes`, `base_offset`, `writer_sequence`, `frame_count`, and sealed state. It
may restore only before physical I/O; using it after partial I/O is forbidden.

## 10. Persistent cooker design

At runtime start, create `maximum_cookers` named worker threads once. Workers
above `active_cookers` park on a `Condvar`; autoscaling changes the active permit
count rather than creating/destroying threads.

Cook queue is bounded by pipeline credits. Each worker:

1. waits for an active permit and task;
2. borrows subject/body from `Arc`;
3. encodes deterministic envelope and complete frame into a pooled `Vec<u8>`;
4. returns `CookedMutation` into the ordered-ready ring;
5. records aggregate timing; and
6. never touches `Store`, indexes, file handles, completions, or telemetry sinks.

The ready ring is a `BTreeMap<LaneTicket, CookOutcome>` plus byte credits in
AWO-2; replacement by a slot ring requires a separate measured patch. The
coordinator removes only the next expected ticket.

One mutex acquisition per completed frame is acceptable in AWO-2 correctness
floor; AWO-4 may switch to per-worker completion chunks only after PQH evidence.

## 11. Exact controller

### 11.1 Workload buckets

Bucket key:

```text
(lane layout, durability, payload-size power-of-two bucket,
 batch-entry candidate, active cookers)
```

Payload bucket is `0` for delete/empty; otherwise
`2^floor(log2(payload_bytes))`, capped at the admitted maximum.

### 11.2 Estimator

For each measured nanosecond sample `x`, maintain integer EWMA mean `m` and
absolute deviation `d`:

```text
m' = m + (x - m) / 8
d' = d + (abs(x - m') - d) / 8
lower = max(0, m - 3d)
upper = m + 3d
```

The first sample sets `m=x,d=0`. Arithmetic uses checked/saturating `u128`
intermediates and returns `u64`. An estimate is usable only after 32 samples and
before 30 seconds of monotonic-clock staleness. These defaults are closed in
`policy-v1.json`; changing them changes the AWO profile revision.

### 11.3 Candidate plans

Candidate entries are:

```text
1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, and current queued count
```

clipped by queue availability, maximum entries, maximum bytes, segment room,
and earliest deadline. Candidate `1` is the natural plan.

### 11.4 Objective

For requests present at decision time:

```text
J(plan) = predicted_mean_completion_ns + predicted_tail_completion_ns
```

Resource limits are hard constraints, not mixed-unit objective terms.

Natural completion uses the lower uncertainty bound when comparing against a
batch. Batch completion uses the upper uncertainty bound, including collection,
cook, install, write, barrier and expected lifecycle share.

For a candidate of `n` entries, natural and batch objectives cover the same
first `n` queued requests; residual requests are reconsidered after that service
decision. Batch wins only when:

```text
J_batch_upper * 1_000_000
  < J_natural_lower * (1_000_000 - decision_margin_ppm)
```

Default `decision_margin_ppm = 100_000` (10%). Overflow means “no batch”. Ties
choose natural. Among winning batches choose lowest `J`; ties choose fewer bytes,
then fewer entries.

If batch completion exceeds the earliest deadline while natural tail completion
does not, choose natural with `natural_deadline`. If both exceed it after
admission, collection delay becomes zero and the plan with the smaller upper
tail is selected as deadline mitigation (`batch_deadline_mitigation` or
`natural_deadline_mitigation`). Admission MUST reject before enqueue when all
known eligible plans already exceed the deadline. A later service-rate collapse
is the only ordinary route to post-admission mitigation.

No estimate, stale estimate, overflow, or incompatible member produces a
speculative batch; the selector emits the specific closed reason.

### 11.5 Collection delay

If two or more requests are already queued, evaluate immediately without adding
collection delay. With exactly one request, deliberate collection is permitted
only when the arrival estimator is warm and predicts a winning candidate within
the maximum collection window. Predicted collection delay is charged at its
upper bound. Otherwise release naturally.

Safety cap is 250 microseconds by default. This cap is not the batch-size tuning
mechanism and does not imply every request waits 250 microseconds.

### 11.6 Autoscaling and hysteresis

Controller evaluates every 100 ms.

Scale up one active cooker when, for five consecutive intervals:

- cook queue bytes increased;
- cooker utilisation exceeded 80%;
- writer-ready queue was not increasing;
- prior worker addition showed positive marginal throughput; and
- host available parallelism and policy maximum permit it.

Scale down one when, for twenty consecutive intervals:

- cooker utilisation is below 30%, or writer-ready bytes remain above 75% of
  their limit; and
- no deadline miss is attributed to cooking.

Minimum dwell: 500 ms after scale-up; 2 s after scale-down. Writer saturation
blocks scale-up. All percentages and intervals live in `policy-v1.json`.

## 12. Default policy and configuration

Closed safe defaults:

| Field | Default |
|---|---:|
| mode before AWO-7 | `disabled` |
| queue entries | 8,192 |
| queue bytes | 64 MiB |
| batch entries | 1,024 |
| batch bytes | 16 MiB |
| collection cap | 250 µs |
| completion deadline | 30 s |
| minimum active cookers | 1 |
| maximum cookers | `min(max(available_parallelism-1,1),16)` |
| pipeline depth | 2 |
| decision margin | 10% |
| estimator warm floor | 32 samples |
| estimator stale | 30 s |

Validation rejects zero limits, queue smaller than one maximum batch, pipeline
depth outside `1..=4`, minimum cookers above maximum, maximum cookers above 64,
collection cap above 10 ms, and deadline below collection cap.

Server config adds optional `store.adaptive_write` with the same fields. Mode,
queue limits, maximum batch, maximum cookers, and pipeline depth are
restart-required. Decision margin and collection cap may reload dynamically
only after AWO-7 atomic-policy-snapshot tests.

Environment/CLI names use `RESIDIUUM_AWO_*` / `--awo-*`. No secret exists in
this configuration.

## 13. Error and wire mapping

Add store errors:

```text
AdaptiveWriterActive
AdaptiveWriterPoisoned
AdaptiveQueueFull
AdaptiveDraining
AdaptiveAdmissionDeadline
```

Qualified wire mapping:

| Internal | Wire code | Retryable |
|---|---|---|
| queue full | `write_overloaded` | true |
| admission deadline | `write_deadline` | true |
| draining | `server_draining` | true |
| poisoned/recovery required | `heap_unavailable` | true |
| validation/capability | existing fail-closed code | false |
| uncertain I/O | `write_outcome_uncertain` | true only with same operation id |

`operation_id` is mandatory for qualified mutating RPCs before AWO-3 remote
enablement. Error payloads never reveal another heap, queue, subject, or batch.

Exact `HeapStore` routing:

```text
put/delete + Unconditional + AWO-eligible layout
  → submit eligible request → block on WriteCompletion

conditional, chunked, Memory, Atomics, maintenance
  → submit NaturalCommand to the same coordinator → block on completion
```

No `HeapStore` method locks the physical store directly for mutation while an
adaptive lease is active.

## 14. Telemetry and evidence types

Store-native event kinds:

```text
AdmissionAccepted, AdmissionRejected, DecisionNatural, DecisionBatch,
BatchClosed, CookStart, CookComplete, ReadyBlocked, PersistStart,
PersistComplete, PublishComplete, CompletionSent, CreditReleased,
WorkerActivated, WorkerParked, ControllerFallback, WriterPoisoned
```

Production holds exact aggregate counters and bounded histograms. Per-request
events are sampled only in controlled PQH mode and chained into the existing
boundary evidence digest. Bodies, subjects, credentials and raw operation ids
are forbidden.

PQH artifacts record profile, policy hash, model hash, candidate set, selected
plan/reason, predicted bounds, observed stages, queue state, active cookers,
durability, correctness result, and reopen result.

## 15. Package execution details

### AWO-0 — Contracts and model

Deliver:

- all files under `spec/performance/awo/`;
- pure model types and selector arithmetic in `adaptive_write/model.rs`;
- registry verifier and golden-vector runner;
- TLA+ state names/transitions skeleton;
- no store write-path change.

Tests/exit:

```bash
bash scripts/verify-awo-contract.sh
cargo test -p residiuum-store --features legacy-raw-store adaptive_write::model
```

Golden accepted and rejected vectors all pass; code and JSON closed sets agree.

### AWO-1 — Persist-before-publish

Deliver reservation/checkpoint/staged publication, direct batch-path correction,
writer poisoning, reopen recovery, and failpoints.

Required failpoints:

```text
awo.reserve.after
awo.cook.before
awo.cook.after
awo.install.frame.before
awo.install.frame.after
awo.persist.before
awo.persist.after_write
awo.persist.after_sync
awo.publish.before
awo.publish.after
awo.complete.before
```

Tests:

```text
awo_persist_before_publish.rs
awo_partial_write_recovery.rs
awo_publication_failure.rs
awo_direct_writer_lease.rs
```

### AWO-2 — Persistent cooker

Deliver bounded queues/credits, parked persistent workers, buffer pool, ordered
ready structure, and copy/thread-count evidence.

Exit requires zero thread creation after runtime warm-up, exact credit return on
every path, deterministic frame equivalence, and no store access from cookers.

### AWO-3 — Static arbiter

Deliver StoreHost/HeapStore integration, fixed conservative batching, natural
ineligible commands, independent completions, fences, config disabled by
default, server error mapping, and put/delete integration tests.

### AWO-4 — Overlap

Deliver depth-2 reservation/persistence pipeline, batch-boundary seal safety,
bounded shutdown, actual stage-overlap evidence, and no third unresolved
reservation.

### AWO-5 — Adaptive controller

Deliver estimator, selector, autoscaling, cold-start/model hash, invalidation,
dynamic allowed policy fields, deterministic oracle traces, and falsification
tests against unstable/lying estimators.

### AWO-6 — Qualification/formal connection

Deliver full crash matrix, loom/model campaigns, PQH controlled runs, Verus/TLA+
artifacts, FAS registry entries, and signed/hash-bound evidence bundle.

### AWO-7 — Productisation

Deliver accepted default posture, operator inspection/drain/reset commands,
SDK documentation, upgrade/rollback test, support matrix, and benchmark
disclosure. Principal alone accepts default-on.

## 16. Test inventory

Create:

```text
crates/residiuum-store/tests/awo_contract.rs
crates/residiuum-store/tests/awo_persist_before_publish.rs
crates/residiuum-store/tests/awo_partial_write_recovery.rs
crates/residiuum-store/tests/awo_credit_bounds.rs
crates/residiuum-store/tests/awo_ordering.rs
crates/residiuum-store/tests/awo_lane_isolation.rs
crates/residiuum-store/tests/awo_cancellation.rs
crates/residiuum-store/tests/awo_shutdown.rs
crates/residiuum-store/tests/awo_static_equivalence.rs
crates/residiuum-store/tests/awo_adaptive_oracle.rs
crates/residiuum-store/tests/awo_controller_stability.rs
crates/residiuum-store/tests/awo_crash_matrix.rs
crates/residiuum-server/tests/awo_heap_rpc.rs
crates/residiuum-perf/tests/awo_qualification.rs
```

Every concurrency test has a bounded deterministic version. Wall-clock sleeps
are forbidden in controller correctness tests; inject `AwoClock` and advance it.

## 17. Formal model variables and transitions

TLA+ variables:

```text
reqState, reqLane, reqTicket, reqDurability, laneNextAdmit,
laneNextInstall, queueBytes, queueEntries, cookOwner, ready,
reservation, persisted, published, acked, failed, uncertain,
activeCookers, controllerMode, writerHealth
```

Transitions:

```text
Receive, Reject, Admit, FormNatural, FormBatch, Reserve, StartCook,
FinishCook, CookFail, Install, PersistOk, PersistFail, Publish,
Complete, CancelWaiter, ReleaseCredit, ActivateCooker, ParkCooker,
BeginDrain, FinishDrain, Crash, Recover
```

Model-check bounds include two heaps, two lanes, three tickets/lane, two
cookers, queue capacity two, success/cook-fail/partial-write, cancellation, and
crash. Required invariants use the exact names in the normative spec.

Verus models the pure request/lane/credit/ACK kernel. I/O timing and predictor
accuracy are assumptions, never theorem conclusions.

## 18. Acceptance commands

Package verification command:

```bash
bash scripts/verify-awo.sh
```

It must run, in order:

```text
verify-awo-contract.sh
AWO model/unit tests
AWO store integration tests
existing CSQ acknowledgement/recovery subset
heap isolation tests
server qualified mutation tests
formal source/proof checks available for the package
PQH AWO smoke (not qualification claim)
```

Controlled qualification is a separate explicit command under
`residiuum-perf --class qualification`; smoke never marks AWO-G8 green.

## 19. Developer handoff rule

A developer accepts one AWO package at a time. Each card includes:

- package id and dependency evidence;
- exact files above;
- normative registry/profile version;
- tests and failpoints required;
- acceptance command output;
- explicit residuals; and
- no self-acceptance into `done`.

If implementation discovers that a normative type, transition, error, formula,
or default cannot be implemented safely, work stops at a spec amendment. It is
not replaced by a local invention.
