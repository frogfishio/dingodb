# Residiuum Adaptive Write Optimiser specification

Status: **normative design v1.0-draft — developer-ready with implementation plan; execution requires admission**

Program: `AWO`

Product name: **Adaptive Write Optimiser**  
Primary component: **Intake Arbiter**

Normative companions:

- [Performance Qualification Harness](PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md)
- [Performance Qualification implementation plan](PERFORMANCE_QUALIFICATION_IMPLEMENTATION_PLAN.md)
- [Adaptive Write Optimiser implementation plan](ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md)
- [Crash and recovery contract](../../reference/operations/CRASH_AND_RECOVERY_CONTRACT.md)
- [Benchmark disclosure](../../reference/operations/BENCHMARK_DISCLOSURE.md)
- [Parallel ingest](../../reference/operations/PARALLEL_INGEST.md)
- [Testing strategy](../../reference/engineering/TESTING_STRATEGY.md)
- [Atomics specification](../atomics/ATOMICS_SPEC.md)

Execution authority remains [MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md).
This specification does not itself admit AWO work or alter the current critical
path.

---

## 1. Product statement

Residiuum MUST accept ordinary independent writes and automatically select the
least-cost safe physical execution plan supported by current load and measured
machine behaviour.

Under sparse load, a request MUST pass with minimal added latency. Under
compatible concurrent load, Residiuum SHOULD combine requests into efficient
physical batches, cook them on a persistent CPU pool, write them through an
ordered bounded pipeline, and return an independent result to every caller.

The client MUST NOT need to call `put_many`, understand segment layout, choose
a cooker count, or tune a batch size to obtain this behaviour.

The defining policy is:

```text
batch only when the conservative predicted outcome is better than releasing
the same admitted requests naturally, without violating any request deadline,
ordering rule, isolation boundary, durability promise, or memory bound
```

This property is called **adaptive write optimisation**. It is a physical
execution optimisation. It is not a transaction facility.

## 2. Motivation and measured premise

Current diagnostic evidence shows materially different service capacities for
the same logical 8 KiB writes:

```text
ordinary serial path                 approximately 10k ops/s in long peers
batched real-store path              approximately 120k ops/s on a measured bed
short parallel-cook laboratory path approximately 320k ops/s
```

These are diagnostic observations, not product SLOs. Their importance is the
shape, not the absolute values:

1. fixed per-write and serial costs are large;
2. compatible batching amortises those costs;
3. cooking is parallelisable;
4. clients and ingress layers already queue overload somewhere; and
5. an internal arbiter can convert unavoidable queueing into useful physical
   efficiency while preserving each operation's semantics.

At 320,000 operations per second with an 8 KiB payload, payload ingress alone
is approximately 2.62 GB/s (2.44 GiB/s). AWO therefore treats cooking and
storage as separate capacity stages and provisions cooking only until it is no
longer the limiting stage.

No number in this section may be advertised without a qualifying PQH evidence
bundle.

## 3. Scope and non-goals

AWO owns:

- admission into a bounded write queue;
- compatibility-lane selection;
- natural-versus-batched plan selection;
- deadline-bounded microbatch formation;
- persistent cooker-worker provisioning;
- ordered cooked-frame delivery;
- overlap of collection, cooking, installation, persistence, and publication;
- per-request completion distribution;
- backpressure before unbounded memory growth;
- measurement-driven adaptation; and
- evidence explaining every decision and outcome class.

AWO initially covers ordinary `put` and `delete` operations that the store can
encode as independent authoritative frames. Chunked values, mixed-operation
batches, Atomics groups, replicated acknowledgement, and multiple physical
devices enter only through explicitly qualified later profiles.

AWO does not:

- create a transaction, implicit atomic group, rollback group, or isolation
  level;
- acknowledge an operation before its requested durability boundary;
- weaken BLAKE3, CRC, frame verification, heap binding, or recovery rules;
- combine security authorisation decisions;
- require applications to submit explicit batches;
- promise that throughput rises without bound as pressure rises;
- hide overload behind unbounded RAM;
- reorder two mutations in the same declared ordering domain;
- make a performance decision from CPU percentage alone;
- claim global mathematical optimality over an unknowable future workload; or
- introduce multi-device striping in the first implementation.

Explicit `put_many` remains a useful caller hint and bulk API, but it MUST not
be the only route to the efficient physical path.

## 4. Terminology

| Term | Meaning |
|---|---|
| **request** | One independently authorised logical mutation and its completion handle |
| **admitted** | Residiuum accepted responsibility for a terminal result; cancellation no longer cancels the mutation |
| **lane** | Requests that may share scheduling and physical work without violating isolation, ordering, layout, or durability |
| **natural plan** | Release through the ordinary non-coalesced path without intentional collection delay |
| **batch plan** | Cook and/or persist independent requests using shared physical work |
| **collection delay** | Deliberate bounded time used to admit more compatible requests |
| **cooker** | Persistent worker producing an immutable encoded frame result |
| **writer lane** | Sole ordered installer/persister for one active append authority |
| **publication** | Making an already-persisted mutation visible through authoritative projections/indexes |
| **completion distributor** | Resolves each request's waiter with its own receipt or error |
| **fence** | Non-transactional wait until preceding admitted work reaches a named boundary |
| **pressure** | Queue depth/bytes, arrival rate, oldest age, or predicted drain time—not CPU percentage alone |

## 5. Semantic boundary: batching is not Atomics

A physical batch contains independent operations. For requests `r1, ..., rn`:

- each request has its own identity and authorisation result;
- each request has its own event identity and frame;
- each request has its own durability requirement;
- each request receives its own receipt or error;
- one logical validation failure does not roll back other requests;
- no caller may infer all-or-nothing visibility from shared physical I/O; and
- Atomics semantics apply only to an explicitly invoked Atomics construct.

Required observational refinement:

```text
ProjectLogical(BatchExecute(r1, ..., rn))
  = ProjectLogical(AllowedIndependentExecution(r1, ..., rn))
```

apart from completion timing and reordering already permitted across independent
ordering domains. For encoded frames `E(ri)`:

```text
EncodeBatch(r1, ..., rn) = E(r1) || ... || E(rn)
```

Physical coalescing MUST NOT invent an enclosing authoritative record required
to interpret otherwise independent frames.

## 6. Request state machine

Every request carries at least:

```text
Request {
  operation_id?       // client idempotency identity where available
  heap_id
  store_id
  subject
  operation_kind      // put | delete initially
  body
  durability
  ordering_domain
  admitted_at
  completion_deadline
  authorization_fact // already verified; never a shared batch decision
}
```

Normative state machine:

```text
Received
  ├─ reject-before-admission ─────────────────────────────► Rejected
  └─ admit ─► Queued ─► Cooking ─► Ready ─► Persisting
                                      ├─ failure ─────────► FailedOrUncertain
                                      └─ persisted ─► Published ─► Acknowledged
```

Rules:

1. Every admitted request reaches exactly one terminal internal result:
   `Acknowledged`, `Failed`, or `UncertainPendingRecovery`.
2. A disconnected/cancelled waiter after admission does not erase the request.
3. An operation rejected before admission may be safely retried as new work.
4. An uncertain operation MUST be resolved through its idempotency/event
   identity; it MUST NOT be reported absent because its acknowledgement was lost.
5. Recovery and acknowledgement state transitions are monotonic.

## 7. Compatibility lanes

A lane key is at least:

```text
LaneKey = (
  heap_id,
  store_id,
  writer_shard,
  durability_class,
  ordering_domain,
  frame/layout_profile,
  atomics_class
)
```

`atomics_class = independent` for ordinary AWO work. Explicit Atomics groups
use their own execution contract and MUST NOT be silently dissolved.

V1 requirements:

- no physical authoritative segment crosses heap identity;
- authorisation is completed before lane admission;
- request bodies and receipts cannot cross heap/client lanes;
- same-subject mutations preserve admitted order;
- incompatible durability classes use separate batches unless the entire batch
  is safely and explicitly over-delivered at the strongest level;
- a segment-generation/rotation change closes or safely retargets a batch;
- chunked and inline layouts do not share a batch before joint qualification;
- low-volume lanes cannot starve behind a saturated lane.

## 8. Acknowledgement and visibility law

AWO inherits the crash contract exactly:

| Mode | Required boundary before successful completion |
|---|---|
| `Memory` | Process-memory publication; outside the initial persisted AWO lane |
| `Buffered` | Complete bytes handed to the qualified OS page-cache/device-queue boundary; no power-loss promise |
| `Durable` | Authoritative bytes and required metadata crossed the qualified stable-storage boundary |

For every request `r`:

```text
Ack(r) => BoundaryReached(r, RequestedDurability(r))
VisibleAsAcknowledged(r) => PersistSucceeded(r)
```

Required order:

```text
authorise → validate → assign identity/order → cook
          → persist → publish → complete waiter
```

The implementation MUST NOT publish an unpersisted locator and then return an
I/O error while leaving it ordinarily visible. Refactoring the current batched
path to meet this rule is an AWO prerequisite, not an optional optimisation.

A shared `write_all` or durability barrier may release many independent
waiters. It does not couple their logical success semantics. Mixed durability
may initially use separate queues. A later strongest-mode batch is legal only
when `ActualDurability(r) >= RequestedDurability(r)` for every member.

## 9. Ordering law

Each admitted request receives a monotonic lane ticket. In the same ordering
domain:

```text
ticket(r1) < ticket(r2)
  => frame_position(r1) < frame_position(r2)
```

Publication cannot reverse that order. Cookers may finish out of order. The
ordered-ready structure retains later frames until the next ticket is ready or
has reached a terminal pre-persistence failure. A failed request leaves no
fabricated frame; its absent frame and terminal result remain explicit.

## 10. Queueing and decision mathematics

### 10.1 Service estimates

For workload class `x`, maintain conservative estimates:

```text
mu_natural(x)          natural service rate
T_cook(n, w, x)        cooking time for n requests on w workers
T_install(n, x)        ordered installation/publication-preparation time
T_write(bytes, d, x)   physical write time at durability d
T_barrier(d, x)        required durability-barrier time
T_rotate(x)            expected rotation/lifecycle interference
```

Measurements are partitioned by variables that materially change the curve:
payload distribution, durability, layout, shard, batch bytes, workers,
device/filesystem class, and background interference. Unknown values remain
unknown; they are never replaced by zero.

### 10.2 Natural completion cost

For `q` compatible requests already queued, with natural service times `s_i`:

```text
C_natural(i) = now + sum(j=1..i, s_j)
```

For equal service time `s`:

```text
mean_natural(q) = s(q + 1) / 2
tail_natural(q) = sq
```

At 10,000 writes/s, 1,000 already-queued equal writes take about 100 ms to
drain and complete in about 50 ms on average.

### 10.3 Batch completion cost

For candidate batch size `n`:

```text
C_batch(n, w) =
    collection_delay
  + T_cook(n, w)
  + T_install(n)
  + T_write(encoded_bytes, durability)
  + T_barrier(durability)
  + expected_rotation_interference
```

In the first non-overlapped implementation, terms are additive. In steady
pipelined operation, the service interval approaches the slowest bounded stage:

```text
T_pipeline_interval ~= max(T_cook, T_install_write, T_lifecycle_share)
```

The controller MUST use the model matching the running implementation. It MUST
NOT claim pipelined overlap from an additive implementation.

### 10.4 Selection rule

For candidate plan `p`:

```text
J(p) =
    predicted_mean_completion_ns(p)
  + predicted_tail_completion_ns(p)
```

Resource pressure remains a hard bound rather than a mixed-unit objective.
Hard constraints precede optimisation at admission:

```text
for every r in p:
  predicted_completion(r) <= completion_deadline(r)

predicted_memory(p) <= available_queue_credit
ordering_preserved(p)
lane_compatible(p)
durability_preserved(p)
```

If no known plan can meet the deadline, reject before admission. If service
conditions degrade after admission so every plan is predicted late, AWO adds no
further collection delay and selects the plan with the smallest conservative
tail. It records explicit deadline mitigation; it does not discard admitted
work or pretend the deadline remains achievable.

Batching may be selected only when:

```text
UpperBound(J(batch)) + decision_margin < LowerBound(J(natural))
```

If confidence bounds are unavailable, overlapping, or stale, release naturally.
This is the **no-known-regression rule**. The optimiser claims preference under
declared evidence and constraints, not knowledge of the future or global
optimality.

### 10.5 Candidate sizes and forced release

Evaluate a bounded candidate set, normally geometrically spaced plus the
currently queued amount:

```text
N = {1, 2, 4, 8, 16, 32, 64, 128, ..., q}
```

Candidates are clipped by bytes, count, deadline, segment, and memory. Byte
limits are mandatory because record count alone is unsafe for heterogeneous
values.

The first request does not automatically start a fixed delay. Collection is
permitted only when predicted arrivals and amortisation justify it. Flush is
forced when any of the following is true:

```text
candidate benefit is no longer positive
OR oldest safe deadline reached
OR maximum collection window reached
OR maximum batch bytes/count reached
OR segment capacity boundary reached
OR memory/backpressure threshold reached
OR explicit fence/shutdown reached
OR controller confidence lost
```

If backlog already exists, form a batch immediately; do not wait again merely
to hit a preferred size.

## 11. Capacity and autoscaling mathematics

```text
mu_system = min(mu_intake, mu_cook, mu_install, mu_write, mu_lifecycle)
```

Grow cookers only while cooking is the measured limiting stage:

```text
cook_queue is increasing
AND mu_cook < min(mu_install, mu_write)
AND another worker improved recent marginal capacity
AND CPU/memory budget permits
```

Stop growing or shrink when the writer queue grows, cooker output blocks,
marginal gain falls below worker cost, host pressure crosses a bound, or load
falls. Target sufficient cooking headroom, not maximum CPU consumption:

```text
mu_cook >= mu_downstream * (1 + headroom)
```

Worker changes require minimum dwell time and hysteresis. Oscillation fails
qualification even if mean throughput is high. V1 uses persistent workers;
creating operating-system threads per batch is forbidden on the qualified path.

## 12. Target pipeline

```text
ordinary put/delete calls
          |
          v
  bounded Intake Arbiter
          |
          v
 adaptive batch builder  <──── measured model + deadlines
          |
          v
 persistent cooker pool
          |
          v
 ordered bounded ready ring
          |
          v
 append/write lane ───────► durability barrier when required
          |
          v
 publish persisted locators
          |
          v
 distribute independent receipts
```

At steady load:

```text
collect batch C | cook batch B | install/write batch A
```

Requirements:

- every queue is bounded by bytes and entries;
- ownership transfer avoids unnecessary body clones;
- encoded buffers are pooled/reused where safe;
- cooked results are immutable;
- backpressure propagates toward admission;
- the writer lane remains sole allocator of authoritative physical order;
- completion callbacks never hold writer-critical locks;
- telemetry cannot block the pipeline;
- shutdown has explicit drain and abort policies.

Static microbatching may land before overlap but reports itself as
`awo_static`, never `awo_adaptive_pipeline`.

## 13. Memory and backpressure

AWO accounts for admitted source bytes, preparation metadata, encoded frames,
ordered-ready bytes, active write tails, completion handles, and attributable
idempotency state. Credits are reserved before admission:

```text
reserved_queue_bytes <= configured_queue_byte_limit
```

Without credit, the caller waits only within its admission deadline or receives
an explicit retryable pre-admission overload result. Residiuum MUST NOT accept
unlimited work and call later memory exhaustion backpressure. Once admitted,
work cannot be silently discarded.

Large values use byte-weighted fairness and may receive a dedicated lane; one
large request cannot permanently block small-request progress.

## 14. Failure, cancellation, recovery, and shutdown

Before persistence, a validation/cook failure affects only that request. The
ordered-ready ring records a terminal hole and advances. No authoritative bytes
or visible locator are created.

On partial/failed physical write:

- no affected request is acknowledged successful;
- no affected locator is ordinarily published before persistence succeeds;
- complete self-verifying frames that reached storage remain recoverable;
- outcome is failed or uncertain under the crash contract, never fabricated as
  absent; and
- retry with the same operation identity resolves/deduplicates.

Recovery produces the same verified authoritative prefix and logical projection
allowed by existing contracts. Queues and cooked-but-unwritten buffers are not
authority.

Cancellation before admission removes the request. After admission it detaches
the waiter but does not revoke the mutation. Resolution uses operation identity.

Graceful shutdown stops admission, drains admitted lanes within an operator
deadline, resolves completions, and persists required metadata. Forced shutdown
uses ordinary crash recovery and has no weaker special path.

## 15. API and operational contract

The ordinary API remains conceptually:

```rust
let receipt = db.put(subject, body, durability).await?;
```

A synchronous SDK facade may block on the same completion primitive; it must
not bypass AWO merely because the caller used a blocking API.

Required policy controls:

```text
AdaptiveWritePolicy {
  enabled
  queue_byte_limit
  queue_entry_limit
  maximum_collection_delay
  default_completion_deadline
  minimum_cookers
  maximum_cookers
  pipeline_depth_limit
  decision_margin
}
```

Names are illustrative; semantics are normative. Safe defaults derive from
machine discovery and PQH evidence. No configuration file is required;
process parameters, environment, or API configuration may override bounds.

Required administrative operations:

- inspect policy and learned model;
- enter conservative/static/disabled mode;
- drain admitted work;
- issue a durability fence;
- reset learned performance state without touching authority;
- export evidence behind recent decisions.

Disabling AWO changes execution only, never format or logical semantics.

## 16. Cold start and learning

On first start or material environment change:

1. load only a compatible integrity-checked prior model;
2. otherwise begin with natural/small-batch execution;
3. collect passive measurements;
4. explore only within latency/resource budgets;
5. promote candidates after repeated evidence;
6. invalidate estimates after material device, filesystem, build, durability,
   encryption, or workload-class change.

Model identity binds product/build, wire/profile, CPU class,
filesystem/device, durability, payload/layout, and relevant features. Learned
state is derived and disposable; corruption falls back conservatively and does
not fail database open. Production exploration has a strict regret budget.

## 17. Observability

AWO emits bounded telemetry through the Residiuum telemetry channel, not the
Evidence Ledger unless an operator records qualification or administration.

Required aggregates by lane/workload class:

- admitted/rejected/completed/failed/uncertain counts;
- queue depth/bytes and oldest age;
- arrival/completion rates;
- natural versus batch decisions;
- batch record/byte and collection-delay distributions;
- per-stage queue/service latency;
- cooker count and marginal gain;
- ready-ring stalls/reorder depth;
- write/barrier/rotation timing;
- prediction error;
- decisions refused for uncertainty/deadline/memory/incompatibility;
- backpressure/rejections;
- per-mode acknowledgement latency;
- controller modes, transitions, and oscillation indicators.

Bodies, subjects, credentials, and heap secrets never enter telemetry. Every
claim discloses disabled/static/adaptive mode and additive/overlapped pipeline.

## 18. Formal assurance obligations

### 18.1 Safety/refinement layer

The formal model MUST establish for bounded instances:

1. `Ack(r) -> BoundaryReached(r, requested_durability(r))`.
2. Ordinarily visible authoritative projection implies successful persistence.
3. Required ticket order implies physical/publication order.
4. No request cooks, persists, publishes, or completes through an incompatible
   heap lane.
5. Exactly one terminal internal outcome exists per admitted request.
6. Reserved memory and occupancy remain within bounds.
7. Projecting optimised execution yields an allowed independent execution.
8. Partial persistence cannot create a successful ACK; recovery exposes only
   complete verified authoritative frames.

Safety is independent of prediction. A wrong prediction may hurt performance
but cannot weaken semantics.

### 18.2 Decision layer

For a finite candidate set and supplied measurement intervals, prove that the
selector:

- rejects every hard-constraint violation;
- batches only when its conservative objective beats natural release by the
  decision margin;
- adds no collection delay past the latest safe start for the earliest deadline,
  or enters explicit best-effort deadline mitigation when all post-admission
  plans are already late; and
- falls back naturally when evidence is missing/stale.

This proves policy conformance, not that a physical machine obeys a prediction.
Suggested split:

- Verus: Rust states, lane typing, tickets, credits, completions, ACK guards;
- TLA+/PlusCal or equivalent: concurrent pipeline, failure, cancellation,
  shutdown, fairness, bounded queues;
- model/property tests: arithmetic, uncertainty, hysteresis, workload traces.

No public formal claim exists until artifacts connect to shipped code through
the Formal Assurance Spine.

## 19. Required testing

### 19.1 Semantic equivalence

For identical deterministic streams compare AWO disabled, static, and adaptive:

- same accepted/rejected operations;
- same per-domain history/order and current logical projection;
- same frame validity/recovery and heap isolation;
- equivalent receipts except permitted timing/over-delivered durability.

Inject deterministic identities/time for byte comparison where possible.

### 19.2 Concurrency and crash testing

- exhaustive bounded state exploration;
- loom-style schedules for admission, cooker completion, ordered readiness,
  publication, cancellation, shutdown, and waiter delivery;
- races on same/different subjects, fairness, worker resizing, credit
  conservation;
- crash/fail after every transition and at every short-write boundary;
- crash after write before publication, during barrier/receipt distribution,
  rotation, full queues, and shutdown.

Qualification proves no false ACK, no visible unpersisted locator, recoverable
complete frames, explicit uncertainty, and idempotent resolution.

### 19.3 Controller oracle tests

Use deterministic curves for sparse traffic, bursts, load below natural
capacity, load between natural/batch capacity, overload above all capacity,
ramps, bimodal payloads, latency-sensitive lanes, device slowdown, seals, CPU
contention, and adversarial noise.

Compare with natural release and the best eligible static candidate known in
hindsight. Record regret, mean/p99, deadlines, throughput, oscillation, and
memory. Declared bounds replace any impossible universal zero-regret claim.

### 19.4 PQH qualification

- sparse latency stays inside the accepted regression budget;
- concurrent ordinary `put` approaches equivalent explicit `put_many` without
  client batching;
- where `mu_natural < arrival_rate < mu_batch`, natural queueing is unstable
  while AWO remains bounded and drains;
- persistent cookers equal/beat per-batch thread creation;
- overlapped mode proves simultaneous stage occupancy;
- CPU growth stops when storage limits;
- overload backpressure bounds memory;
- Buffered and Durable results remain separate;
- every run finishes correctness/reopen verification.

Absolute ops/s is not a CI gate. CI gates semantics, invariants, deterministic
controller traces, and catastrophic relative regressions. Controlled beds
supply performance evidence.

## 20. Acceptance gates

| Gate | Requirement |
|---|---|
| `AWO-G1` | Request/lane/state registries and schemas frozen |
| `AWO-G2` | Persist-before-publish prerequisite proven and crash-tested |
| `AWO-G3` | Persistent cooker and bounded credits qualified |
| `AWO-G4` | Static coalescing equivalent to natural execution |
| `AWO-G5` | Ordered overlap equivalent and seal-safe |
| `AWO-G6` | Adaptive selector passes oracle/stability tests |
| `AWO-G7` | Heap, durability, cancellation, partial-write matrix green |
| `AWO-G8` | Controlled PQH shows ordinary puts converge toward explicit-batch capacity |
| `AWO-G9` | Sparse latency, overload memory, fairness budgets accepted |
| `AWO-G10` | Proof artifacts connected through Formal Assurance Spine |
| `AWO-G11` | Disabled/static fallback and inspection documented |
| `AWO-G12` | Principal accepts default-on posture |

Performance-gate failure leaves AWO disabled/static. Safety-gate failure blocks
shipment of the affected path.

## 21. Delivery packages

### AWO-0 — Contracts and executable model

Request states, lane keys, controller inputs, candidate plans, schemas,
deterministic simulator, safety predicates, and golden vectors. No product-path
change.

### AWO-1 — Persistence/publication prerequisite

Prepare/cook, persist successfully, then publish. Add failure handling,
uncertain outcome resolution, and crash evidence. Required even if later AWO is
deferred.

### AWO-2 — Persistent cooker and bounded ownership

Replace per-batch thread creation, per-result shared mutex publication, and
avoidable copies with persistent bounded workers and explicit buffer ownership.

### AWO-3 — Static Intake Arbiter

Coalesce independent ordinary requests by conservative fixed count, byte, and
deadline limits; return independent receipts. Learning remains off.

### AWO-4 — Ordered overlapping pipeline

Bounded ready rings/double buffers overlap collection, cooking, and writing.
Make segment rotation safe; prove backpressure and shutdown.

### AWO-5 — Adaptive mathematical controller

Measured curves, uncertainty bounds, candidate selection, deadlines, worker
scaling, hysteresis, cold start, model invalidation, and decision evidence.

### AWO-6 — Qualification and formal connection

Run semantic/crash/controller/PQH matrices, connect proofs, and publish evidence.

### AWO-7 — Productisation

Ordinary async/blocking APIs, operational controls, telemetry, fallbacks,
upgrade behaviour, and accepted defaults. Default-on requires principal accept.

```text
PQH accepted evidence
  → AWO-0 → AWO-1 → AWO-2 → AWO-3 → AWO-4 → AWO-5 → AWO-6 → AWO-7
```

AWO-0 modelling may overlap AWO-1 analysis. AWO-5 cannot mask incomplete
AWO-1 through AWO-4 semantics.

## 22. Future extension: multiple devices

V1 must not prevent future device-specific writer lanes. Prefer placing whole
independently verifiable segments rather than striping every frame:

```text
arbiter ─┬─ device A: segments 1, 5, 9
         ├─ device B: segments 2, 6, 10
         ├─ device C: segments 3, 7, 11
         └─ device D: segments 4, 8, 12
```

This preserves examinability and turns device loss into an explicit coverage
hole rather than damage to every striped frame. Placement, replication, loss,
and cross-device Atomics require separate Medusa/cluster proofs.

## 23. Product claim boundary

Permitted after qualification:

> Residiuum automatically combines compatible concurrent writes when doing so
> is conservatively predicted to improve completion cost, while preserving each
> write's independent ordering, isolation, durability, and acknowledgement
> semantics.

Permitted informal description:

> Within its adaptive operating envelope, Residiuum becomes more efficient as
> compatible write pressure increases.

Forbidden: infinite scaling, implicit transactions, atomic batch membership,
zero added latency for every request, global controller optimality, or absolute
throughput without full evidence disclosure.

```text
sparse/latency region → adaptive efficiency region → bounded saturation region
```

Beyond saturation, AWO applies bounded backpressure. It never converts overload
into unbounded memory growth or weaker durability.
