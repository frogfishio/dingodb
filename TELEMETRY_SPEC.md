# ResiduumDB Telemetry v1 specification

Status: normative design v1.0-draft; implementation not yet qualified

Profiles:

```text
dingo-telemetry-v1
dingo-telemetry-ratatouille-v1
dingo-health-v1
```

Normative companions:
[DATABASE_DOCTRINE.md](DATABASE_DOCTRINE.md),
[HEAP_SPEC.md](HEAP_SPEC.md),
[EVIDENCE_LEDGER_SPEC.md](EVIDENCE_LEDGER_SPEC.md),
[FORMAT_SPEC.md](FORMAT_SPEC.md), and
[CLUSTER_SPEC.md](CLUSTER_SPEC.md).

Ratatouille integration target:
[`ratatouille` 0.1.0](https://docs.rs/ratatouille/0.1.0/ratatouille/).
Changing the pinned version requires relay, output-vector, overflow, and
performance requalification.

## 1. Decision

Ratatouille is ResiduumDB's exclusive operational telemetry export channel.

ResiduumDB:

- collects cheap measurements at named instrumentation points;
- aggregates routine activity in bounded process memory;
- emits periodic cumulative snapshots;
- emits bounded exceptional and slow-operation exemplars;
- exports only through a bounded Ratatouille relay;
- writes no telemetry files;
- emits no request-path telemetry to stdout or stderr; and
- never lets telemetry success, failure, pressure, or absence change a
  database outcome.

Ratatouille describes itself as a best-effort telemetry firehose with bounded
topic state. Its logger filters topics, assigns per-topic sequences, formats
text or NDJSON, and forwards to a sink. Its bounded TCP and HTTP relays and
explicit flush model are the only qualified ResiduumDB sink class. Direct network
sinks and `StdoutSink` are not qualified for ResiduumDB's hot path.

The product statement is:

> Measure everything useful. Aggregate the ordinary. Sample the exceptional.
> Drop telemetry before slowing the database.

## 2. Requirement language

MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 3. Boundary with the Evidence Ledger

Telemetry and evidence have different truth contracts:

| Property | Telemetry | Residuum Evidence Ledger |
|---|---|---|
| Purpose | performance and operational understanding | proof of accepted security/administrative facts |
| Volume | high | selective |
| Delivery | best effort | durable according to obligation |
| Backpressure | drop | fail closed for required evidence |
| Retention | external collector policy | explicit ResiduumDB evidence policy |
| Ordering | Ratatouille topic sequence and sample sequence | signed ledger sequence |
| Integrity | transport/collector concern | canonical signatures and checkpoints |
| Database effect | none | atomically coupled where required |

An event does not become evidence because a collector retained it. An evidence
record does not become telemetry until an after-commit adapter emits a redacted
notification.

The dependency direction is:

```text
database operation
    ├── update bounded in-memory telemetry state
    ├── commit authoritative state/evidence
    └── optionally offer redacted event to Ratatouille
```

Telemetry MUST NOT be called to complete, acknowledge, roll back, or recover a
database operation.

## 4. What telemetry is

Telemetry consists of four signal classes:

| Class | Meaning | Normal emission |
|---|---|---|
| counter | monotonically increasing total since process start | periodic cumulative snapshot |
| gauge | bounded point-in-time state | periodic snapshot and material transition |
| histogram | cumulative fixed-bucket distribution | periodic cumulative snapshot |
| exemplar | one bounded diagnostic example | sampled/token-bucketed event |

Telemetry is not:

- an access log;
- a document-change stream;
- query history;
- an application analytics store;
- a security audit trail;
- a profiler that records arbitrary stack traces;
- an unbounded label system; or
- a place to put payloads because they are inconvenient elsewhere.

### 4.1 Runtime scope

V1 covers:

- the ResiduumDB single-node server;
- each ResiduumDB cluster-node process;
- the embedded engine when its host explicitly supplies a Ratatouille relay;
  and
- maintenance processes such as scrub, salvage, backup, and migration when
  explicitly configured with the same deployment telemetry identity.

The Rust client SDK does not open an autonomous telemetry connection or emit
application information by default. Remote request behaviour is observed at
the server boundary. A future client profile requires a separate consent,
privacy, identity, and cardinality specification.

An embedded engine with no configured relay keeps only the bounded in-memory
state needed for local health and incurs no background network activity.

## 5. Collection architecture

### 5.1 Hot path

The ordinary hot path may perform only:

- relaxed atomic counter increments;
- fixed-bucket histogram increments;
- bounded gauge updates;
- one monotonic-clock read when latency is already required; and
- a non-blocking offer for an eligible exemplar.

It MUST NOT:

- allocate a map entry based on request data;
- serialize JSON for every successful operation;
- lock a global telemetry mutex;
- perform DNS, socket, filesystem, or collector I/O;
- format a human message;
- inspect a document body for telemetry; or
- wait for queue capacity.

Instrumentation overhead is part of the performance contract, not “free
debugging.”

### 5.2 Aggregator

One process-wide aggregator owns fixed registries and periodically creates
immutable snapshots. It reads counters without stopping workers.

Counters and histograms are cumulative since `boot_id`, so a collector can
recover after dropped snapshots. Gauges describe the instant sampled.

Every snapshot contains:

```text
profile
schema_version
boot_id
sample_sequence
counter_epoch
process_monotonic_ns
observed_unix_ns?
snapshot_interval_ms
partial
unavailable_sources
```

`observed_unix_ns` is optional time evidence. `process_monotonic_ns` is used
for rates and elapsed durations within one process incarnation.

### 5.3 Exporter

The exporter passes already bounded messages to a Ratatouille `Logger`
configured with:

- `Format::Ndjson`;
- fixed ResiduumDB topic names only;
- a bounded TCP or HTTP relay;
- a source identity defined in §8;
- a queue-overflow policy that preserves the most recent telemetry; and
- explicit bounded flush.

Only the exporter worker owns and calls the mutable Ratatouille `Logger`.
Request/storage workers never call `Logger::log` or a Ratatouille sink. An
eligible transition or exemplar is copied into a preallocated bounded sharded
ring; the exporter drains those rings, encodes the closed message, and offers
it to Ratatouille. A contended/full ring drops immediately and increments its
fixed suppression counter.

No ResiduumDB qualified profile uses `StdoutSink`, a direct synchronous
`TcpSink`/`HttpSink`, a file adapter, syslog, journald, or a second exporter.
Collectors may translate Ratatouille output into any downstream system.

### 5.4 Health APIs

`health_live`, `health_ready`, and authenticated health detail are control
status APIs, not telemetry transports. They remain available under
`dingo-health-v1`.

The legacy `metrics` RPC is not part of the Ratatouille-only qualified profile.
During migration it MAY expose the same in-memory snapshot for tests and local
compatibility, but production configuration MUST disable it. No new consumer
may depend on it.

## 6. Fixed topic registry

Topic names are permanent, ASCII, and supplied only by code. Request data can
never construct a topic.

| Topic | Content |
|---|---|
| `dingo:lifecycle` | bounded process/node start, ready, drain, stop, fatal state transitions |
| `dingo:runtime` | process/runtime/resource snapshots |
| `dingo:transport` | connection, TLS, byte, and network admission snapshots |
| `dingo:rpc` | aggregate operation outcomes and latency histograms |
| `dingo:rpc:exemplar` | sampled errors and slow operations |
| `dingo:admission` | rate, concurrency, replay, and resource-control snapshots |
| `dingo:storage` | append, read, sync, segment, byte, and amplification snapshots |
| `dingo:query` | RQL/SDA work, coverage, cursor, scan, and result snapshots |
| `dingo:index` | index state, lag, build, maintenance, and cache snapshots |
| `dingo:integrity` | scrub, corruption, holes, partial material, salvage, and repair snapshots |
| `dingo:atomic` | Atomic/RRE/relationship aggregate outcomes |
| `dingo:lifecycle:data` | backup, restore, compaction, retention, tier, purge aggregates |
| `dingo:cluster` | partition, leader, quorum, replica, repair, and rebalance snapshots |
| `dingo:evidence` | redacted Evidence Ledger health and after-commit notifications |
| `dingo:telemetry` | Ratatouille/logger/relay queue, filter, drop, and flush self-observation |

No per-Heap, per-collection, per-key, per-client, per-query, per-partition, or
per-error-code topic is permitted. Subclassification is a bounded field inside
the topic schema.

Unknown topics are filtered. A future topic requires a schema, cardinality
review, privacy review, overhead budget, compatibility classification, and
qualification vectors.

## 7. Ratatouille message contract

### 7.1 Outer record

Ratatouille owns the canonical outer NDJSON record:

```json
{
  "ts": "2026-03-11T12:34:56.789Z",
  "seq": 42,
  "topic": "dingo:rpc",
  "src": {
    "app": "dingodb",
    "where": "deployment-ref",
    "instance": "boot-id"
  },
  "meta": null,
  "args": ["<ResiduumDB message JSON>"],
  "env": null
}
```

ResiduumDB does not fork or replace that format. The ResiduumDB message passed to
`Logger::log(topic, message)` is one minified JSON object encoded as UTF-8.
The pinned Rust profile emits exactly one string in `args`; `meta` and `env`
are null/absent according to Ratatouille's pinned output vector. A collector
parses the outer NDJSON record, requires exactly one string argument, then
parses `args[0]` according to this specification.

### 7.2 Common ResiduumDB message

Every ResiduumDB message is:

```json
{
  "v": 1,
  "profile": "dingo-telemetry-v1",
  "kind": "snapshot",
  "boot": "base32-128",
  "sample": 42,
  "epoch": 1,
  "mono_ns": 123456789,
  "unix_ns": 1780000000000000000,
  "interval_ms": 10000,
  "partial": false,
  "unavailable": [],
  "data": {}
}
```

Rules:

- exactly the listed common keys are present;
- `profile` is exactly `dingo-telemetry-v1`;
- `kind` is `snapshot`, `transition`, or `exemplar`;
- `boot` is a random process-incarnation ID, not a host identity;
- `sample` is monotonically increasing per ResiduumDB topic and process;
- `epoch` changes only when cumulative counters are deliberately reset;
- `unix_ns` is nullable;
- `interval_ms` is null for transitions/exemplars;
- `unavailable` contains only closed source codes;
- `data` is the closed topic/kind schema;
- integers outside their schema range reject local emission;
- non-finite numbers are forbidden;
- the encoded ResiduumDB message is at most 64 KiB; and
- user strings are never inserted into `data`.

Ratatouille's topic sequence and ResiduumDB's sample sequence are independent.
Both are exported so logger reconfiguration and collector gaps are visible.

### 7.3 Snapshots

Snapshots contain cumulative counters/histograms and current gauges. A dropped
snapshot does not lose the next cumulative total.

Collectors calculate a rate only between two samples with equal:

```text
source deployment reference
source instance boot_id
schema version
counter epoch
```

Counter decrease without `boot_id` or `counter_epoch` change is invalid
telemetry and raises a collector-side diagnostic.

### 7.4 Transitions

Transitions are emitted only for low-rate state changes whose timing matters,
including:

- process ready/draining/stopped;
- readiness gained/lost;
- store opened/degraded/unavailable;
- index state transitions;
- partition leadership/quorum transitions;
- scrub/backup/repair job start and finish;
- Evidence Ledger admitted/blocked; and
- telemetry relay connected/disconnected.

Repeated identical state does not emit another transition. Flapping uses a
token bucket and is also visible through cumulative transition counters.

### 7.5 Exemplars

Exemplars explain aggregate outliers. They are not complete records.

An exemplar may contain only:

```text
operation_code
outcome_code
public_error_code
latency_us
request_bytes_bucket
result_bytes_bucket
work_units_bucket
guarantee_requested
guarantee_achieved
committed?
partition_class?
heap_ref?
correlation_ref?
```

`correlation_ref` is a random or keyed digest usable to join approved ResiduumDB
telemetry events. It is never a bearer token, raw operation ID, Evidence ID,
document key, or client-supplied trace string.

Every exemplar states `sampled=true`. Its existence never implies that
unsampled events did not occur.

## 8. Source identity

Ratatouille `SourceIdentity` is:

```text
app      = "dingodb"
where    = deployment_ref
instance = boot_id
```

`deployment_ref` is:

```text
base32(
  keyed_BLAKE3(
    telemetry_identity_key,
    "DINGODB-TELEMETRY-DEPLOYMENT-V1" || DeploymentId
  )[0..16]
)
```

The telemetry identity key is not an authorization key and is independently
rotatable. Rotation changes references and emits a lifecycle transition
without exposing either old or new key material.

Source identity MUST NOT include:

- hostname;
- IP address;
- filesystem path;
- raw `DeploymentId`, `HeapId`, node ID, or cluster ID;
- cloud account/project ID;
- customer or tenant name; or
- process command line.

Collectors add deployment inventory metadata outside ResiduumDB when required.

## 9. Cardinality doctrine

### 9.1 Fixed dimensions

The following dimensions are closed registries:

```text
topic
operation_code
outcome_code
public_error_code
durability_code
index_state
job_kind
damage_kind
atomic_outcome
admission_reason
cluster_role
coverage_state
```

Unknown values map to one `other` code and increment
`telemetry_unknown_code_total`.

### 9.2 Forbidden default dimensions

Default telemetry never labels by:

- raw or hashed document key;
- collection name or ID;
- Heap name or raw ID;
- certificate, holder, user, owner, or scope ID;
- source IP;
- query text, normalized query, or query-plan hash;
- operation/request ID;
- segment, frame, event, backup, or evidence ID;
- filesystem path;
- arbitrary error text; or
- user-controlled media type.

Hashing an unbounded value does not make its cardinality bounded.

### 9.3 Optional Heap detail

The default profile emits only deployment/node aggregates.

An operator MAY configure a local allowlist of at most 64 Heap IDs for
temporary Heap-detail telemetry. Each is represented as:

```text
heap_ref = base32(
  keyed_BLAKE3(
    telemetry_identity_key,
    "DINGODB-TELEMETRY-HEAP-V1" || HeapId
  )[0..16]
)
```

All non-allowlisted Heaps merge into `heap_ref = "other"`. The allowlist is
loaded through the protected local control plane, is never request-modifiable,
expires after at most 24 hours unless renewed, and is itself Evidence Ledger
material.

Collection-level telemetry is not supported in v1.

### 9.4 Optional topology detail

Cluster operation sometimes requires identifying one unhealthy node or
partition. V1 supports a protected local allowlist of at most:

```text
64 node identities
128 partition identities
```

References use keyed BLAKE3 with domain separators
`DINGODB-TELEMETRY-NODE-V1` and
`DINGODB-TELEMETRY-PARTITION-V1`. Non-allowlisted identities merge into
`other`. The allowlist has the same 24-hour expiry, Evidence Ledger recording,
and non-request-modifiability rules as Heap detail.

Automatic “top N” identity selection is forbidden because it makes disclosure
and series identity depend on workload. Aggregate backlog and health remain
available without topology detail.

## 10. Standard buckets

### 10.1 Latency

All latency histograms use cumulative microsecond buckets:

```text
10
25
50
100
250
500
1_000
2_500
5_000
10_000
25_000
50_000
100_000
250_000
500_000
1_000_000
2_500_000
5_000_000
10_000_000
+Inf
```

The hot path does not calculate percentiles. Collectors derive percentiles
from buckets.

### 10.2 Bytes

Request, result, payload, frame, and batch-size histograms use:

```text
0
64
256
1_024
4_096
16_384
65_536
262_144
1_048_576
4_194_304
16_777_216
67_108_864
+Inf
```

### 10.3 Work and count

Documents, frames, members, scanned units, and retry counts use:

```text
0
1
2
4
8
16
32
64
128
256
512
1_024
4_096
16_384
65_536
+Inf
```

Changing buckets requires a profile revision. A collector MUST NOT merge
histograms with different bucket profiles.

## 11. Collection-point registry

### 11.1 Process and runtime

Collected by the process sampler, never by request workers:

| Signal | Type | Source |
|---|---|---|
| `process_uptime_ms` | gauge | monotonic clock since boot |
| `process_rss_bytes` | gauge | qualified OS process sampler |
| `process_virtual_bytes` | gauge | qualified OS process sampler |
| `process_cpu_user_ns_total` | counter | qualified OS process sampler |
| `process_cpu_system_ns_total` | counter | qualified OS process sampler |
| `process_threads` | gauge | runtime/OS sampler |
| `process_open_fds` | gauge | OS sampler where supported |
| `process_memory_pressure_total` | counter | allocator/OS pressure callback |
| `process_panics_total` | counter | bounded panic hook; no panic message |
| `process_build_info` | transition once | compile-time version/profile IDs |

Unsupported platform values are listed in `unavailable_sources`; they are not
reported as zero.

### 11.2 Transport and TLS

Collected at accept, handshake completion, framed read/write, and connection
close:

| Signal | Type |
|---|---|
| `connections_accepted_total` | counter |
| `connections_rejected_total{reason}` | counter |
| `connections_active` | gauge |
| `connections_peak` | gauge |
| `connection_lifetime_us` | histogram |
| `transport_bytes_read_total` | counter |
| `transport_bytes_written_total` | counter |
| `frames_read_total` / `frames_written_total` | counters |
| `frame_decode_error_total{class}` | counter |
| `tls_handshake_total{outcome}` | counter |
| `tls_handshake_latency_us` | histogram |
| `tls_protocol_total{version}` | counter |
| `connection_close_total{class}` | counter |

No peer address, certificate fingerprint, SNI, or TLS exporter enters
telemetry.

### 11.3 Authentication and admission

Collected at the pre-auth limiter, HeapKey validation outcome, replay gate,
work admission, and resource-budget gate:

| Signal | Type |
|---|---|
| `auth_attempt_total{outcome}` | counter |
| `auth_failure_total{public_class}` | counter |
| `auth_lockout_total` | counter |
| `admission_total{decision,reason}` | counter |
| `admission_inflight{class}` | gauge |
| `admission_queue_us{class}` | histogram |
| `replay_total{fresh,retry,reject}` | counter |
| `resource_limit_total{class}` | counter |
| `expensive_inflight` | gauge |
| `expensive_rejected_total` | counter |

Internal cryptographic failure causes belong in the Evidence Ledger or
protected diagnostics, not ordinary telemetry.

### 11.4 RPC and SDK operation boundary

Collected once at operation admission and once at completion:

| Signal | Type |
|---|---|
| `rpc_started_total{operation_code}` | counter |
| `rpc_completed_total{operation_code,outcome}` | counter |
| `rpc_inflight{operation_class}` | gauge |
| `rpc_latency_us{operation_code,outcome}` | histogram |
| `rpc_request_bytes{operation_class}` | histogram |
| `rpc_result_bytes{operation_class}` | histogram |
| `rpc_error_total{operation_class,public_error_class}` | counter |
| `rpc_cancel_total{class}` | counter |
| `rpc_deadline_total{stage}` | counter |
| `rpc_retry_observed_total{class}` | counter |

The operation registry numeric ID is the label source. Unknown names map to
`other`; strings from legacy dispatch never become labels.

No event is emitted for each successful RPC. Slow/error exemplars follow §19.

### 11.5 Durability and guarantee boundary

Collected when acknowledgement mode is selected and when the receipt is
closed:

| Signal | Type |
|---|---|
| `guarantee_requested_total{mode}` | counter |
| `guarantee_achieved_total{mode}` | counter |
| `guarantee_miss_total{requested,achieved}` | counter |
| `commit_outcome_total{outcome}` | counter |
| `ack_latency_us{mode}` | histogram |
| `unknown_commit_total` | counter |

A telemetry counter never substitutes for receipt or Atomic evidence.

### 11.6 Storage write path

Collected at encode, append admission, physical append completion, sync,
segment seal, and receipt closure:

| Signal | Type |
|---|---|
| `write_items_total{kind,outcome}` | counter |
| `write_logical_bytes_total` | counter |
| `write_encoded_bytes_total` | counter |
| `write_physical_bytes_total` | counter |
| `write_batch_items` | histogram |
| `write_batch_bytes` | histogram |
| `encode_latency_us` | histogram |
| `compression_latency_us{algorithm_class}` | histogram |
| `compression_input_bytes_total` / `compression_output_bytes_total` | counters |
| `encryption_latency_us{operation}` | histogram |
| `encryption_bytes_total{operation}` | counter |
| `append_latency_us` | histogram |
| `sync_latency_us` | histogram |
| `writer_wait_latency_us{class}` | histogram |
| `receipt_latency_us{mode}` | histogram |
| `short_write_total{surface}` | counter |
| `segment_open_total` / `segment_sealed_total` | counters |
| `active_segment_bytes` / `sealed_segment_bytes` | gauges |
| `active_segments` / `sealed_segments` | gauges |
| `media_capacity_bytes{tier_class}` | gauge |
| `media_available_bytes{tier_class}` | gauge |
| `media_reserved_bytes{tier_class}` | gauge |
| `storage_queue_depth{class}` | gauge |
| `storage_read_only` | gauge 0/1 |
| `media_io_error_total{class}` | counter |
| `write_amplification_milli` | derived snapshot gauge |

```text
write_amplification_milli =
  floor(1000 × physical_bytes / max(logical_bytes, 1))
```

The sampler computes amplification from cumulative counters; writers do not
perform division.

### 11.7 Storage read and cache path

Collected at index lookup, frame read/verification, chunk assembly, and result
closure:

| Signal | Type |
|---|---|
| `read_total{kind,outcome}` | counter |
| `read_logical_bytes_total` | counter |
| `read_physical_bytes_total` | counter |
| `read_latency_us{kind}` | histogram |
| `frames_examined` | histogram |
| `chunks_examined` | histogram |
| `cache_total{cache_kind,hit_miss}` | counter |
| `cache_entries{cache_kind}` | gauge |
| `cache_bytes{cache_kind}` | gauge |
| `partial_result_total{cause}` | counter |
| `read_amplification_milli` | derived snapshot gauge |

Cache kinds are a frozen list. User-created index names are not cache labels.

### 11.8 Query, RQL, SDA, cursors, and ordering

Collected at parse, plan close, iterator open, work-budget consumption,
coverage close, cursor encode/decode, and result close:

| Signal | Type |
|---|---|
| `query_total{language,outcome}` | counter |
| `query_parse_latency_us{language}` | histogram |
| `query_plan_latency_us{language}` | histogram |
| `query_execute_latency_us{plan_class}` | histogram |
| `query_scanned_units` | histogram |
| `query_matched_units` | histogram |
| `query_returned_units` | histogram |
| `query_work_units` | histogram |
| `query_read_bytes` / `query_result_bytes` | histograms |
| `query_coverage_total{coverage}` | counter |
| `query_budget_exhausted_total{budget}` | counter |
| `cursor_total{operation,outcome}` | counter |
| `cursor_position_work` | histogram |
| `sort_total{strategy,outcome}` | counter |
| `sort_spill_total` / `sort_spill_bytes_total` | counters |
| `direct_access_total{outcome}` | counter |
| `order_wavelet_total{outcome}` | counter |

No query text, predicate value, selected field name, plan hash, or cursor bytes
are collected.

### 11.9 Indexes

Collected at definition transitions, maintenance enqueue/apply, lookup, rebuild
checkpoint, and state publication:

| Signal | Type |
|---|---|
| `index_count{state}` | gauge |
| `index_transition_total{from,to}` | counter |
| `index_lookup_total{outcome}` | counter |
| `index_maintenance_total{outcome}` | counter |
| `index_maintenance_latency_us` | histogram |
| `index_lag_events` | histogram |
| `index_lag_bytes` | histogram |
| `index_build_total{outcome}` | counter |
| `index_build_latency_us` | histogram |
| `index_build_units_total` | counter |
| `index_stale_total{cause}` | counter |

Index identity and collection identity are forbidden dimensions.

### 11.10 RRE, relationships, and Atomics

Collected after closed-plan validation, conflict checks, decision publication,
and recovery resolution:

| Signal | Type |
|---|---|
| `dre_evaluation_total{outcome}` | counter |
| `dre_evaluation_latency_us` | histogram |
| `dre_violation_total{rule_class}` | counter |
| `relationship_check_total{kind,outcome}` | counter |
| `atomic_total{scope,outcome}` | counter |
| `atomic_plan_members` | histogram |
| `atomic_read_set_units` / `atomic_predicate_units` | histograms |
| `atomic_prepare_latency_us` | histogram |
| `atomic_commit_latency_us` | histogram |
| `atomic_conflict_total{class}` | counter |
| `atomic_recovery_total{outcome}` | counter |
| `atomic_material_total{coverage}` | counter |

Rule IDs, collection IDs, keys, predicate values, and Atomic IDs do not enter
telemetry.

### 11.11 Damage, scrub, salvage, and repair

Collected only from verification results, never inferred from an ordinary
application error:

| Signal | Type |
|---|---|
| `integrity_frames_examined_total` | counter |
| `integrity_bytes_examined_total` | counter |
| `integrity_failure_total{kind}` | counter |
| `holes_discovered_total{kind}` | counter |
| `partial_payload_total{cause}` | counter |
| `scrub_total{outcome}` | counter |
| `scrub_duration_us` | histogram |
| `scrub_age_seconds` | gauge |
| `salvage_total{outcome}` | counter |
| `salvage_frames_recovered_total` | counter |
| `salvage_bytes_recovered_total` | counter |
| `repair_total{outcome}` | counter |
| `repair_units_total{result}` | counter |
| `quarantine_units` / `quarantine_bytes` | gauges |

Damage location, raw bytes, subject IDs, paths, and frame IDs are excluded.
Exact forensic facts belong in examination output and, where required, the
Evidence Ledger.

### 11.12 Lifecycle, tiering, backup, and purge

Collected at durable job state transitions and job completion:

| Signal | Type |
|---|---|
| `job_total{job_kind,outcome}` | counter |
| `job_active{job_kind}` | gauge |
| `job_duration_us{job_kind}` | histogram |
| `job_units_total{job_kind,result}` | counter |
| `job_bytes_total{job_kind,result}` | counter |
| `backup_age_seconds` | gauge |
| `retention_due_units` | gauge |
| `hold_block_total{operation}` | counter |
| `tier_bytes{tier_class}` | gauge |
| `compaction_reclaimed_bytes_total` | counter |
| `purge_coverage_total{coverage}` | counter |
| `key_provider_total{operation,outcome}` | counter |
| `key_provider_latency_us{operation}` | histogram |

Provider names, bucket names, object paths, key IDs, backup IDs, and purge IDs
are not telemetry dimensions.

### 11.13 Evidence Ledger

Collected from ledger state transitions and after a ledger operation has
resolved:

| Signal | Type |
|---|---|
| `evidence_operation_total{kind,outcome}` | counter |
| `evidence_commit_latency_us{obligation}` | histogram |
| `evidence_head_sequence` | gauge, aggregate or allowlisted Heap detail |
| `evidence_checkpoint_age_seconds` | gauge |
| `evidence_reserve_bytes` | gauge |
| `evidence_blocked` | gauge 0/1 |
| `evidence_gap_total` / `evidence_fork_total` | counters |
| `evidence_anchor_total{profile,outcome}` | counter |
| `evidence_bounded_drop_total` | counter |

Evidence IDs, signer IDs, actor identities, certificate fingerprints, assertion
contents, and exact ledger records never enter telemetry.

### 11.14 Cluster and replication

Collected at consensus role/quorum transition, proposal completion, replication
apply, repair, placement, and rebalance checkpoints:

| Signal | Type |
|---|---|
| `cluster_role{role}` | gauge |
| `leader_transition_total{outcome}` | counter |
| `quorum_available` | gauge 0/1 |
| `partition_count{state}` | gauge |
| `replica_count{state}` | gauge |
| `proposal_total{outcome}` | counter |
| `proposal_latency_us` | histogram |
| `apply_latency_us` | histogram |
| `replication_lag_entries` / `replication_lag_bytes` | histograms |
| `snapshot_total{operation,outcome}` | counter |
| `peer_transport_total{outcome}` | counter |
| `repair_queue_units` / `rebalance_queue_units` | gauges |
| `placement_transition_total{outcome}` | counter |
| `coverage_total{state}` | counter |

Raw node, peer, partition, placement, term, and endpoint identities are
excluded from the default profile. Bounded role/state classes remain.
Allowlisted `node_ref` and `partition_ref` fields follow §9.4.

### 11.15 Telemetry self-observation

Collected from Ratatouille `EmitResult`, logger stats, relay stats, snapshot
construction, and flush:

| Signal | Type |
|---|---|
| `telemetry_offer_total{result}` | counter |
| `telemetry_filtered_total{topic}` | counter |
| `telemetry_enqueued_total{topic}` | counter |
| `telemetry_dropped_total{topic,reason}` | counter |
| `telemetry_oversize_total{topic}` | counter |
| `telemetry_encode_failure_total{topic}` | counter |
| `telemetry_queue_depth` / `telemetry_queue_bytes` | gauges |
| `telemetry_relay_connected` | gauge 0/1 |
| `telemetry_send_total{outcome}` | counter |
| `telemetry_sent_bytes_total` | counter |
| `telemetry_reconnect_total` | counter |
| `telemetry_flush_total{outcome}` | counter |
| `telemetry_snapshot_build_us{topic}` | histogram |
| `telemetry_snapshot_skipped_total{topic,reason}` | counter |

These counters always exist in memory. They are emitted through
`dingo:telemetry` when the channel works and appear in authenticated health
detail when it does not. They never fall back to console or files.

## 12. Snapshot composition

Default snapshot interval is 10 seconds. Configurable range is 1–300 seconds.

Each topic snapshot contains only its §11 registry fields. Zero-valued counters
MAY be omitted. A gauge that is supported MUST be present even when zero.
Unsupported signals are named in the common `unavailable_sources` array using
closed source codes.

If a complete topic snapshot would exceed 64 KiB:

1. optional Heap-detail series are removed;
2. zero counters are removed;
3. the snapshot sets `partial=true`;
4. omitted registry groups are listed by closed group code; and
5. `telemetry_snapshot_skipped_total{reason=oversize}` increments.

The snapshot is never split into dynamically named topics.

## 13. Exemplars, slow operations, and error sampling

### 13.1 Slow threshold

An operation is slow when:

```text
latency >= max(
  configured_absolute_threshold(operation_class),
  rolling_fixed_bucket_threshold
)
```

V1 default absolute thresholds:

| Class | Threshold |
|---|---:|
| point read/write | 10 ms |
| bounded query | 100 ms |
| Atomic | 250 ms |
| administrative job request | 1 s |
| network/TLS handshake | 500 ms |

The rolling threshold may only select a predeclared histogram bucket and is
computed by the aggregator, not the hot path.

### 13.2 Token buckets

Exemplars use fixed process-wide buckets:

| Exemplar | Capacity | Refill |
|---|---:|---:|
| slow operation | 20 | 1/s |
| public operation error | 50 | 5/s |
| transport/TLS error | 20 | 1/s |
| integrity transition | 20 | 1/s |
| cluster transition | 20 | 1/s |

Exhaustion drops exemplars and increments
`telemetry_exemplar_suppressed_total{class}`. Aggregate counters continue.

The request cannot request sampling, change thresholds, or supply exemplar
fields.

## 14. Privacy, redaction, and secrets

Telemetry schemas are allowlists, not post-hoc redaction.

The following are forbidden anywhere in a telemetry message:

- document or chunk content;
- raw keys, subjects, collection names, Heap names, query text, RQL, or SDA;
- HeapKey, certificates, holder proofs, tokens, passwords, cookies, or TLS
  exporters;
- private/public key bytes and key IDs;
- raw request/operation/event/evidence IDs;
- filesystem paths, URLs with credentials, cloud object names, or environment
  variables;
- source addresses and hostnames;
- arbitrary error messages, panic messages, or backtraces;
- application owner/scope identities; and
- user-provided labels or metadata.

Public error codes and closed enum values are safe only after registry
validation. Unknown strings map to `other`; they are never truncated and
forwarded.

Telemetry encoding tests maintain a secret corpus and prove none of its
sentinels appear in emitted bytes.

## 15. Heap isolation and declassification

Telemetry is part of the observable system surface under `HEAP_SPEC.md`.

Default deployment aggregates may reveal only fields in the named isolation
profile's declassification registry. Heap-detail telemetry:

- requires the local allowlist in §9.3;
- uses only `heap_ref`, never identity or name;
- never combines payload-derived values from two Heaps into one exemplar;
- may aggregate numeric resource/operation counts across Heaps only where the
  profile permits deployment aggregates; and
- is unavailable through a HeapKey network channel.

The public health probes expose only `live`, `ready`, protocol version, and
build ID as already permitted. Ratatouille output is an operator-plane export,
not a client-plane query response.

## 16. Delivery, overflow, and failure semantics

### 16.1 Queue

Qualified defaults:

```text
relay queue entries      8192
maximum queue entries    65536
relay queue bytes        32 MiB
maximum queue bytes      256 MiB
maximum ResiduumDB message    65536 bytes
overflow                 preserve newest / discard oldest
snapshot interval        10 seconds
shutdown flush deadline  250 ms
maximum flush deadline   2 seconds
```

If the installed Ratatouille version cannot provide bounded non-blocking offer
and preserve-newest overflow semantics, that version is not qualified. If its
native relay bound counts only entries, the ResiduumDB adapter additionally tracks
encoded queued bytes and refuses/evicts offers before the byte bound is
exceeded.

### 16.2 Collector unavailable

When the relay is disconnected or its queue is full:

- database workers do not wait;
- telemetry messages are dropped according to policy;
- cumulative counters continue in memory;
- reconnect is attempted by the relay/export worker;
- no file, stdout, stderr, syslog, alternate network, or Evidence Ledger
  fallback occurs; and
- health detail reports the degraded telemetry channel without making the
  database unready.

Telemetry unavailability alone MUST NOT change `health_ready`.

### 16.2.1 Transport security

Ratatouille provides delivery mechanics, not ResiduumDB authorization. A
qualified endpoint is either:

- a loopback/local sidecar endpoint protected by host isolation; or
- an authenticated and encrypted channel whose transport profile is frozen
  separately.

Plain remote TCP/HTTP across an untrusted network is not qualified. Collector
credentials, if required, are resolved outside telemetry messages and never
enter Ratatouille source identity or payloads.

### 16.3 Initialization

Failure to initialize Ratatouille does not prevent the database from serving.
The process:

- retains in-memory self counters;
- exposes `telemetry_available=false` through authenticated health detail; and
- may print one bounded developer-console warning during bootstrap.

It does not repeatedly print failures.

### 16.4 Shutdown

Shutdown asks the relay to flush until the configured deadline. Expiry drops
the remainder, increments the in-memory final counter, and continues shutdown.
Telemetry never extends the shutdown deadline beyond its bound.

## 17. Configuration

Telemetry configuration is local operator configuration:

```text
telemetry.profile
telemetry.enabled
telemetry.transport              tcp_relay | http_relay
telemetry.endpoint_secret_ref
telemetry.topic_filter
telemetry.snapshot_interval_ms
telemetry.queue_entries
telemetry.queue_bytes
telemetry.overflow_policy
telemetry.flush_deadline_ms
telemetry.heap_detail_allowlist
telemetry.heap_detail_expires_at
telemetry.slow_threshold_profile
telemetry.exemplar_profile
telemetry.identity_key_secret_ref
```

Rules:

- endpoint credentials are secret references, never inline telemetry;
- topic filters are validated against the fixed registry;
- production defaults enable snapshots, transitions, self-observation, and
  bounded exemplars;
- request-success event emission has no configuration switch because it is
  forbidden;
- no file/stdout sink value exists;
- heap-detail expiry cannot exceed 24 hours;
- queue and timing bounds obey §16; and
- configuration changes are Residuum Evidence Ledger events.

Ratatouille itself enables no topics by default, so ResiduumDB MUST install an
explicit validated filter rather than depend on library defaults.

## 18. Developer console

Developer console output is not telemetry. It is a deliberately tiny
interactive surface.

Permitted normal messages:

```text
ResiduumDB <version> starting
Listening on <local bind description>
Ready
Draining
Stopped
```

One bounded bootstrap warning and one fatal startup error are permitted.

The console MUST NOT emit:

- per-request lines;
- structured telemetry;
- metrics snapshots;
- retry/reconnect chatter;
- collector failures after bootstrap;
- payloads, credentials, paths containing secrets, or certificates; or
- a fallback copy of Ratatouille messages.

Library/embedded use defaults to no console output.

## 19. Health and alerts

Authenticated health detail includes:

```text
telemetry_enabled
telemetry_available
telemetry_relay_connected
telemetry_queue_utilization_bucket
telemetry_dropped_since_boot
telemetry_last_success_age_bucket
telemetry_profile
```

It does not expose endpoints, queue contents, source references, or exact
collector errors.

Required downstream alert conditions are defined semantically:

- database not ready;
- guarantee miss;
- sustained public error-rate increase;
- p99 latency bucket regression;
- admission/resource-limit saturation;
- storage free/reserve pressure;
- sync latency regression;
- index stale/lag increase;
- damage/hole/partial-material discovery;
- scrub or backup age violation;
- Evidence Ledger blocked, forked, or reserve-low;
- quorum/leadership instability;
- repair/rebalance backlog growth; and
- telemetry disconnected or dropping.

ResiduumDB emits the signals. Dashboard and alert products are collector-side
packages and do not add an exporter to ResiduumDB.

## 20. Rust boundary

The implementation separates collection, aggregation, encoding, and delivery:

```rust
pub trait TelemetryCounters: Send + Sync {
    fn rpc_started(&self, operation: OperationCode);
    fn rpc_completed(&self, observation: RpcObservation);
    fn storage_write(&self, observation: StorageWriteObservation);
}

pub trait TelemetrySnapshotSource: Send + Sync {
    fn snapshot(&self, now: SampleTime) -> TopicSnapshotSet;
}

pub trait TelemetryEncoder {
    fn encode(&self, snapshot: &ClosedTelemetryMessage)
        -> Result<BoundedMessage, TelemetryEncodeError>;
}

pub trait TelemetryEmitter: Send + Sync {
    fn offer(&self, topic: TelemetryTopic, message: BoundedMessage)
        -> TelemetryOfferResult;
}
```

`TelemetryTopic`, operation codes, error codes, dimensions, and messages are
closed enums/typed structures. No producer accepts a generic map, arbitrary
topic string, log level, format string, or user-provided labels.

The Ratatouille adapter is the only production `TelemetryEmitter`.
Tests use a bounded memory emitter implementing identical offer/drop
semantics. There is no production file or console emitter.

## 21. Migration from current implementation

The current `dingo-server::slog` and default `StderrSink` are legacy
DEF-060 implementation, not the target architecture.

Migration:

1. add the fixed telemetry registry and typed observations;
2. adapt the current `MetricsRegistry` into the first snapshot source;
3. add Ratatouille behind a single production adapter;
4. emit cumulative RPC/admission snapshots through fixed topics;
5. add self-observation from `EmitResult` and relay stats;
6. replace `rpc.complete` with aggregate counters and bounded exemplars;
7. remove default `Logger::stderr()` and per-request NDJSON;
8. reduce console output to §18;
9. disable the `metrics` RPC in the qualified production profile;
10. progressively connect storage, query, index, integrity, lifecycle,
    Atomics, Evidence Ledger, and cluster sources; and
11. delete or test-only gate the old `dingo-log-v1` surface after compatibility
    expiry.

No migration stage may introduce a file sink or synchronous network delivery.

## 22. Qualification

A profile cannot claim `dingo-telemetry-v1` until all applicable tests pass:

1. exact topic/message schema golden vectors;
2. fixed-registry and unknown-to-`other` tests;
3. secret/payload/query/key/path sentinel corpus;
4. no stdout/stderr/file writes during sustained request traffic;
5. collector disconnected from startup through shutdown;
6. queue-full preserve-newest/drop accounting;
7. relay reconnect and cumulative snapshot recovery;
8. Ratatouille filtered/emitted/dropped outcome accounting;
9. maximum-size and oversize snapshot behaviour;
10. cardinality attack through operations, errors, Heaps, collections, and
    request fields;
11. exemplar token-bucket suppression;
12. clock rollback and missing wall-clock handling;
13. counter wrap/saturation and process restart;
14. Heap isolation/declassification tests for every topic field;
15. telemetry disabled with identical database outcomes;
16. telemetry overload with identical database outcomes;
17. Evidence Ledger blocked/unavailable with honest telemetry but no coupling;
18. bounded shutdown flush;
19. all supported OS sampler unavailable/error paths;
20. load tests at supported maximum RPS;
21. benchmark comparison with telemetry disabled, collector healthy, collector
    disconnected, and queue continuously full; and
22. fuzzing of ResiduumDB message encoder, configuration, filters, and collector
    fixture parser.

Performance qualification requires:

```text
steady-state throughput regression <= 2%
p99 database latency regression    <= 2%
telemetry memory                    bounded by declared queue + registries
hot-path allocations               zero for routine counter/histogram updates
```

If a workload cannot meet the gate, the affected instrumentation moves to
aggregation/sampling; the database gate is not relaxed.

## 23. Work packages

| ID | Deliverable | Exit evidence |
|---|---|---|
| TEL-0 | machine-readable topic, field, enum, bucket, and config registries | golden vectors + schema validation |
| TEL-1 | closed Rust telemetry types and fixed registries | compile-fail + property tests |
| TEL-2 | Ratatouille adapter and bounded relay profiles | disconnect/overflow/flush tests |
| TEL-3 | current RPC/admission metrics migration | parity + no per-RPC output |
| TEL-4 | process, transport, storage read/write sources | load and unit tests |
| TEL-5 | query, cursor, index, cache, RRE, Atomic sources | work/amplification fixtures |
| TEL-6 | integrity, lifecycle, Evidence Ledger sources | chaos/scrub/backup fixtures |
| TEL-7 | cluster/replication sources | partition and transition tests |
| TEL-8 | health integration, console reduction, legacy removal | production-profile tests |
| TEL-9 | collector fixtures, dashboards, and alert rules | end-to-end operator drill |
| TEL-10 | qualification and overhead disclosure | §22 matrix + benchmark report |

TEL-0 through TEL-4, TEL-6 Evidence Ledger health, TEL-8, and TEL-10 are
required for qualified single-node server telemetry. TEL-5 is required as each
named subsystem becomes qualified. TEL-7 is required only for a cluster claim.

## 24. Completion definition

Telemetry is development-complete for a qualified profile only when:

- Ratatouille is the sole production telemetry emitter;
- every enabled field has one named collection point and stable type;
- routine success activity is aggregated, not emitted per request;
- topic, label, Heap-detail, queue, message, and exemplar cardinality is
  bounded;
- telemetry contains no payload, credential, query, identity, or arbitrary
  error text;
- disconnected/full telemetry cannot affect database correctness or latency
  beyond the qualification budget;
- self-drops and unavailable sources are observable honestly;
- Evidence Ledger and telemetry semantics remain independent;
- console output conforms to §18;
- the legacy production stderr logger and metrics scrape are disabled; and
- the §22 qualification and overhead report is published.
