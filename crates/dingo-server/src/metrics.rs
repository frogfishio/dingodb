//! Process metrics and health probes (DEF-061).
//!
//! Exports a versioned metrics snapshot (`dingo-metrics-v1`) with **bounded
//! label cardinality** and process health reports (`dingo-health-v1`):
//!
//! - **Liveness** (`health_live`): process accept path is up.
//! - **Readiness** (`health_ready`): node can provide advertised guarantees
//!   (fails closed while draining, when the store is not usable, or when
//!   cluster replication was advertised but Raft is unavailable).
//! - **Detail** (`health`): authenticated rollup for operators.
//! - **Metrics** (`metrics`): authenticated counter/histogram scrape.
//!
//! No request/response payloads or credentials appear in metric labels.

use crate::admission::AdmissionStats;
use crate::runtime::ServerStats;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Metrics export profile label (capability matrices / scrape payload).
pub const METRICS_PROFILE: &str = "dingo-metrics-v1";

/// Health report profile label.
pub const HEALTH_PROFILE: &str = "dingo-health-v1";

/// Maximum distinct `op` label values retained (known ops + overflow bucket).
pub const MAX_OP_LABELS: usize = 32;

/// Fixed latency histogram upper bounds in milliseconds (inclusive).
/// Final bucket is `+Inf` (everything above the last finite bound).
pub const LATENCY_BUCKET_MS: &[u64] = &[1, 2, 5, 10, 25, 50, 100, 250, 500, 1_000, 2_500, 5_000];

/// Known application ops that get their own counter series (bounded set).
const KNOWN_OPS: &[&str] = &[
    "ping",
    "put",
    "put_bytes",
    "get",
    "get_bytes",
    "delete",
    "scan_json",
    "find",
    "history",
    "list_keys",
    "list_collections",
    "store_info",
    "directory",
    "index_create",
    "index_drop",
    "index_rebuild",
    "index_list",
    "get_payload",
    "admin_stats",
    "health_live",
    "health_ready",
    "health",
    "metrics",
    "raft_request_vote",
    "raft_append_entries",
    "raft_install_snapshot",
    "raft_read_index",
    "salvage_export",
    "tier_move",
    "purge",
    "force_reconfig",
    "other",
];

fn op_index(op: &str) -> usize {
    for (i, name) in KNOWN_OPS.iter().enumerate() {
        if *name == op {
            return i;
        }
    }
    // Overflow bucket.
    KNOWN_OPS.len() - 1
}

/// One series of counters for a single op label.
#[derive(Debug, Default)]
struct OpCounters {
    total: AtomicU64,
    ok: AtomicU64,
    err: AtomicU64,
    /// Latency histogram counts per bucket (including +Inf).
    latency_buckets: [AtomicU64; LATENCY_BUCKET_COUNT],
    latency_sum_ms: AtomicU64,
    latency_count: AtomicU64,
}

const LATENCY_BUCKET_COUNT: usize = 13; // len(LATENCY_BUCKET_MS) + 1 (+Inf)

impl OpCounters {
    fn observe(&self, ok: bool, latency: Duration) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if ok {
            self.ok.fetch_add(1, Ordering::Relaxed);
        } else {
            self.err.fetch_add(1, Ordering::Relaxed);
        }
        let ms = latency.as_millis() as u64;
        self.latency_sum_ms.fetch_add(ms, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
        let mut idx = LATENCY_BUCKET_COUNT - 1;
        for (i, bound) in LATENCY_BUCKET_MS.iter().enumerate() {
            if ms <= *bound {
                idx = i;
                break;
            }
        }
        self.latency_buckets[idx].fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self, op: &str) -> OpMetricSnapshot {
        let mut buckets = Vec::with_capacity(LATENCY_BUCKET_COUNT);
        let mut cumulative = 0u64;
        for (i, c) in self.latency_buckets.iter().enumerate() {
            cumulative += c.load(Ordering::Relaxed);
            let le = if i < LATENCY_BUCKET_MS.len() {
                format!("{}", LATENCY_BUCKET_MS[i])
            } else {
                "+Inf".into()
            };
            buckets.push(HistogramBucket {
                le,
                count: cumulative,
            });
        }
        OpMetricSnapshot {
            op: op.to_string(),
            total: self.total.load(Ordering::Relaxed),
            ok: self.ok.load(Ordering::Relaxed),
            err: self.err.load(Ordering::Relaxed),
            latency_ms_sum: self.latency_sum_ms.load(Ordering::Relaxed),
            latency_ms_count: self.latency_count.load(Ordering::Relaxed),
            latency_ms_buckets: buckets,
        }
    }
}

/// Cumulative histogram bucket for JSON export.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HistogramBucket {
    /// Inclusive upper bound in milliseconds, or `+Inf`.
    pub le: String,
    /// Cumulative observations ≤ `le`.
    pub count: u64,
}

/// Per-op metric series (bounded labels).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OpMetricSnapshot {
    /// Operation label (`other` for unknown names).
    pub op: String,
    /// Total completions.
    pub total: u64,
    /// Successful completions (`ok=true`).
    pub ok: u64,
    /// Failed completions.
    pub err: u64,
    /// Sum of observed latencies in milliseconds.
    pub latency_ms_sum: u64,
    /// Number of latency samples.
    pub latency_ms_count: u64,
    /// Cumulative latency histogram.
    pub latency_ms_buckets: Vec<HistogramBucket>,
}

/// Durability / commit outcome counters (bounded; no free-form labels).
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct GuaranteeMetrics {
    /// Writes that returned `committed=true`.
    pub committed_true: u64,
    /// Writes that returned `committed=false`.
    pub committed_false: u64,
    /// Completions where requested guarantee ≠ achieved (or durability error).
    pub guarantee_miss: u64,
    /// Requested durable acknowledgements observed.
    pub requested_durable: u64,
    /// Achieved durable acknowledgements observed.
    pub achieved_durable: u64,
    /// Achieved buffered acknowledgements.
    pub achieved_buffered: u64,
    /// Achieved memory acknowledgements.
    pub achieved_memory: u64,
}

/// Connection / admission event counters mirrored into the scrape.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct EdgeMetrics {
    /// Connections accepted into a worker.
    pub connections_accepted: u64,
    /// Connections rejected (slot full, drain, or churn).
    pub connections_rejected: u64,
    /// Admission rate-limit rejects.
    pub rate_rejected: u64,
    /// Auth lockout rejects.
    pub auth_lockouts: u64,
    /// Auth failures recorded.
    pub auth_failures: u64,
    /// Expensive-op concurrency rejects.
    pub expensive_rejected: u64,
    /// Resource-limit / overload responses on the wire.
    pub resource_limit_responses: u64,
}

/// Full metrics scrape payload (`metrics` RPC `value`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MetricsSnapshot {
    /// Schema / profile tag.
    pub profile: &'static str,
    /// Unix epoch milliseconds when the snapshot was taken.
    pub ts_ms: u64,
    /// Process uptime in milliseconds.
    pub uptime_ms: u64,
    /// Per-op series (only ops with `total > 0`, plus always-empty omitted).
    pub ops: Vec<OpMetricSnapshot>,
    /// Durability / commit outcomes.
    pub guarantees: GuaranteeMetrics,
    /// Edge / admission events.
    pub edge: EdgeMetrics,
    /// Server connection runtime stats when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerStatsWire>,
    /// Admission controller stats when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admission: Option<AdmissionStatsWire>,
}

/// JSON-friendly server stats.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ServerStatsWire {
    /// Live connection workers.
    pub active_connections: usize,
    /// Peak concurrent connections.
    pub peak_connections: usize,
    /// Connections refused.
    pub rejected_connections: u64,
    /// Connections admitted.
    pub accepted_connections: u64,
    /// Mutations started.
    pub mutations_started: u64,
    /// Mutations finished.
    pub mutations_finished: u64,
    /// Drain active.
    pub draining: bool,
    /// Configured max connections.
    pub max_connections: usize,
}

impl From<ServerStats> for ServerStatsWire {
    fn from(s: ServerStats) -> Self {
        Self {
            active_connections: s.active_connections,
            peak_connections: s.peak_connections,
            rejected_connections: s.rejected_connections,
            accepted_connections: s.accepted_connections,
            mutations_started: s.mutations_started,
            mutations_finished: s.mutations_finished,
            draining: s.draining,
            max_connections: s.max_connections,
        }
    }
}

/// JSON-friendly admission stats.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AdmissionStatsWire {
    /// RPCs admitted under rate limits.
    pub admitted_rpcs: u64,
    /// Rate rejects.
    pub rate_rejected: u64,
    /// Auth lockouts.
    pub auth_lockouts: u64,
    /// Auth failures.
    pub auth_failures: u64,
    /// Connect churn rejects.
    pub connect_churn_rejected: u64,
    /// Connects admitted.
    pub connects_admitted: u64,
    /// Expensive rejects.
    pub expensive_rejected: u64,
    /// Expensive started.
    pub expensive_started: u64,
    /// Expensive finished.
    pub expensive_finished: u64,
    /// Replay fresh.
    pub replay_fresh: u64,
    /// Replay retries.
    pub replay_retries: u64,
    /// Replay rejects.
    pub replay_rejected: u64,
}

impl From<AdmissionStats> for AdmissionStatsWire {
    fn from(s: AdmissionStats) -> Self {
        Self {
            admitted_rpcs: s.admitted_rpcs,
            rate_rejected: s.rate_rejected,
            auth_lockouts: s.auth_lockouts,
            auth_failures: s.auth_failures,
            connect_churn_rejected: s.connect_churn_rejected,
            connects_admitted: s.connects_admitted,
            expensive_rejected: s.expensive_rejected,
            expensive_started: s.expensive_started,
            expensive_finished: s.expensive_finished,
            replay_fresh: s.replay_fresh,
            replay_retries: s.replay_retries,
            replay_rejected: s.replay_rejected,
        }
    }
}

/// Process-wide metrics registry (share via [`Arc`]).
#[derive(Debug)]
pub struct MetricsRegistry {
    started: Instant,
    ops: [OpCounters; KNOWN_OPS_LEN],
    committed_true: AtomicU64,
    committed_false: AtomicU64,
    guarantee_miss: AtomicU64,
    requested_durable: AtomicU64,
    achieved_durable: AtomicU64,
    achieved_buffered: AtomicU64,
    achieved_memory: AtomicU64,
    connections_accepted: AtomicU64,
    connections_rejected: AtomicU64,
    rate_rejected: AtomicU64,
    auth_lockouts: AtomicU64,
    auth_failures: AtomicU64,
    expensive_rejected: AtomicU64,
    resource_limit_responses: AtomicU64,
}

const KNOWN_OPS_LEN: usize = 32;

// Compile-time guard: KNOWN_OPS length must match the fixed array.
const _: () = assert!(KNOWN_OPS.len() == KNOWN_OPS_LEN);

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    /// Empty registry; clocks start at construction.
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            ops: std::array::from_fn(|_| OpCounters::default()),
            committed_true: AtomicU64::new(0),
            committed_false: AtomicU64::new(0),
            guarantee_miss: AtomicU64::new(0),
            requested_durable: AtomicU64::new(0),
            achieved_durable: AtomicU64::new(0),
            achieved_buffered: AtomicU64::new(0),
            achieved_memory: AtomicU64::new(0),
            connections_accepted: AtomicU64::new(0),
            connections_rejected: AtomicU64::new(0),
            rate_rejected: AtomicU64::new(0),
            auth_lockouts: AtomicU64::new(0),
            auth_failures: AtomicU64::new(0),
            expensive_rejected: AtomicU64::new(0),
            resource_limit_responses: AtomicU64::new(0),
        }
    }

    /// Shared handle for `ServeOptions`.
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Record one completed application RPC (no payloads).
    #[allow(clippy::too_many_arguments)] // RPC observation fields are explicit
    pub fn observe_rpc(
        &self,
        op: &str,
        ok: bool,
        latency: Duration,
        guarantee_requested: Option<&str>,
        guarantee_achieved: Option<&str>,
        committed: Option<bool>,
        error_code: Option<&str>,
    ) {
        let idx = op_index(op);
        self.ops[idx].observe(ok, latency);

        if let Some(true) = committed {
            self.committed_true.fetch_add(1, Ordering::Relaxed);
        } else if let Some(false) = committed {
            self.committed_false.fetch_add(1, Ordering::Relaxed);
        }

        if matches!(guarantee_requested, Some("durable")) {
            self.requested_durable.fetch_add(1, Ordering::Relaxed);
        }
        match guarantee_achieved {
            Some("durable") => {
                self.achieved_durable.fetch_add(1, Ordering::Relaxed);
            }
            Some("buffered") => {
                self.achieved_buffered.fetch_add(1, Ordering::Relaxed);
            }
            Some("memory") => {
                self.achieved_memory.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        let miss = match (ok, committed, guarantee_requested, guarantee_achieved) {
            (true, Some(false), _, _) => true,
            (true, _, Some(req), Some(ach)) if !req.is_empty() && req != ach => true,
            (false, _, Some(_), _) if error_code == Some("durability_unavailable") => true,
            _ => false,
        };
        if miss {
            self.guarantee_miss.fetch_add(1, Ordering::Relaxed);
        }

        if error_code == Some("resource_limit") {
            self.resource_limit_responses
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record a connection admission outcome at the accept edge.
    pub fn observe_connection(&self, accepted: bool) {
        if accepted {
            self.connections_accepted.fetch_add(1, Ordering::Relaxed);
        } else {
            self.connections_rejected.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Mirror admission counters that are not per-RPC (optional top-up).
    pub fn observe_admission_edge(
        &self,
        rate_rejected_delta: u64,
        auth_lockouts_delta: u64,
        auth_failures_delta: u64,
        expensive_rejected_delta: u64,
    ) {
        if rate_rejected_delta > 0 {
            self.rate_rejected
                .fetch_add(rate_rejected_delta, Ordering::Relaxed);
        }
        if auth_lockouts_delta > 0 {
            self.auth_lockouts
                .fetch_add(auth_lockouts_delta, Ordering::Relaxed);
        }
        if auth_failures_delta > 0 {
            self.auth_failures
                .fetch_add(auth_failures_delta, Ordering::Relaxed);
        }
        if expensive_rejected_delta > 0 {
            self.expensive_rejected
                .fetch_add(expensive_rejected_delta, Ordering::Relaxed);
        }
    }

    /// Build a scrape snapshot. Optional live server/admission stats are merged.
    pub fn snapshot(
        &self,
        server: Option<ServerStats>,
        admission: Option<AdmissionStats>,
    ) -> MetricsSnapshot {
        let mut ops = Vec::new();
        for (i, name) in KNOWN_OPS.iter().enumerate() {
            let snap = self.ops[i].snapshot(name);
            if snap.total > 0 {
                ops.push(snap);
            }
        }
        // Prefer live admission counters when provided (authoritative).
        let edge = if let Some(a) = admission {
            EdgeMetrics {
                connections_accepted: server
                    .map(|s| s.accepted_connections)
                    .unwrap_or_else(|| self.connections_accepted.load(Ordering::Relaxed)),
                connections_rejected: server
                    .map(|s| s.rejected_connections)
                    .unwrap_or_else(|| self.connections_rejected.load(Ordering::Relaxed))
                    .saturating_add(a.connect_churn_rejected),
                rate_rejected: a.rate_rejected,
                auth_lockouts: a.auth_lockouts,
                auth_failures: a.auth_failures,
                expensive_rejected: a.expensive_rejected,
                resource_limit_responses: self.resource_limit_responses.load(Ordering::Relaxed),
            }
        } else {
            EdgeMetrics {
                connections_accepted: self.connections_accepted.load(Ordering::Relaxed),
                connections_rejected: self.connections_rejected.load(Ordering::Relaxed),
                rate_rejected: self.rate_rejected.load(Ordering::Relaxed),
                auth_lockouts: self.auth_lockouts.load(Ordering::Relaxed),
                auth_failures: self.auth_failures.load(Ordering::Relaxed),
                expensive_rejected: self.expensive_rejected.load(Ordering::Relaxed),
                resource_limit_responses: self.resource_limit_responses.load(Ordering::Relaxed),
            }
        };

        MetricsSnapshot {
            profile: METRICS_PROFILE,
            ts_ms: now_ms(),
            uptime_ms: self.started.elapsed().as_millis() as u64,
            ops,
            guarantees: GuaranteeMetrics {
                committed_true: self.committed_true.load(Ordering::Relaxed),
                committed_false: self.committed_false.load(Ordering::Relaxed),
                guarantee_miss: self.guarantee_miss.load(Ordering::Relaxed),
                requested_durable: self.requested_durable.load(Ordering::Relaxed),
                achieved_durable: self.achieved_durable.load(Ordering::Relaxed),
                achieved_buffered: self.achieved_buffered.load(Ordering::Relaxed),
                achieved_memory: self.achieved_memory.load(Ordering::Relaxed),
            },
            edge,
            server: server.map(ServerStatsWire::from),
            admission: admission.map(AdmissionStatsWire::from),
        }
    }
}

/// Whether an op is a public probe that must work without a token.
///
/// Kubernetes-style liveness/readiness checks need unauthenticated access even
/// when the data plane requires tokens. Detailed `health` and `metrics` stay
/// authenticated.
pub fn is_public_probe_op(op: &str) -> bool {
    matches!(op, "health_live" | "health_ready")
}

/// Health status string for probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Process is alive.
    Live,
    /// Ready to serve advertised guarantees.
    Ready,
    /// Not ready (still alive).
    NotReady,
}

/// Detailed health report (`health` / probe `value`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HealthReport {
    /// Schema / profile tag.
    pub profile: &'static str,
    /// Unix epoch milliseconds.
    pub ts_ms: u64,
    /// Aggregate status.
    pub status: HealthStatus,
    /// Liveness: accept path / process up.
    pub live: bool,
    /// Readiness: can provide advertised guarantees.
    pub ready: bool,
    /// Human reasons when not ready (bounded, no secrets).
    pub reasons: Vec<String>,
    /// Serve mode label when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Store path (no secrets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<String>,
    /// Dense node index when multi-node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_index: Option<u32>,
    /// Live subject count when store is open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_count: Option<usize>,
    /// Whether the process is draining.
    pub draining: bool,
    /// Whether network Raft control plane is attached.
    pub raft_attached: bool,
    /// Whether this process claims cluster replication (experimental serve-cluster).
    pub claims_replication: bool,
    /// Active connections when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_connections: Option<usize>,
    /// Metrics / log profiles for operator matrices.
    pub metrics_profile: &'static str,
    /// Structured log profile.
    pub log_profile: &'static str,
}

/// Inputs required to evaluate readiness (no I/O).
#[derive(Debug, Clone)]
pub struct HealthEvalInput<'a> {
    /// Process is running the accept loop (always true when handling an RPC).
    pub process_up: bool,
    /// Server is draining / shutting down.
    pub draining: bool,
    /// Local store opened successfully for this process.
    pub store_open: bool,
    /// Optional store path for the report.
    pub store_path: Option<&'a str>,
    /// Live subject count.
    pub live_count: Option<usize>,
    /// Serve mode (`serve` / `serve-cluster`).
    pub mode: Option<&'a str>,
    /// Dense node index.
    pub node_index: Option<u32>,
    /// Raft state is attached.
    pub raft_attached: bool,
    /// Process advertises multi-node replication (serve-cluster experimental).
    pub claims_replication: bool,
    /// Active connection count.
    pub active_connections: Option<usize>,
    /// Log profile constant for the report.
    pub log_profile: &'static str,
}

/// Evaluate liveness + readiness from process signals.
///
/// Readiness **fails** when:
/// - the process is draining;
/// - the store is not open;
/// - replication is claimed but Raft is not attached (cannot meet quorum writes).
pub fn evaluate_health(input: HealthEvalInput<'_>) -> HealthReport {
    let mut reasons = Vec::new();
    let live = input.process_up;
    if !live {
        reasons.push("process not up".into());
    }
    if input.draining {
        reasons.push("server draining".into());
    }
    if !input.store_open {
        reasons.push("store not open".into());
    }
    if input.claims_replication && !input.raft_attached {
        reasons.push("replication claimed but raft not attached".into());
    }

    let ready = (input.raft_attached || !input.claims_replication)
        && input.store_open
        && !input.draining
        && live;
    let status = if !live {
        HealthStatus::NotReady
    } else if ready {
        HealthStatus::Ready
    } else {
        HealthStatus::NotReady
    };

    HealthReport {
        profile: HEALTH_PROFILE,
        ts_ms: now_ms(),
        status,
        live,
        ready,
        reasons,
        mode: input.mode.map(|m| m.to_string()),
        store: input.store_path.map(|s| s.to_string()),
        node_index: input.node_index,
        live_count: input.live_count,
        draining: input.draining,
        raft_attached: input.raft_attached,
        claims_replication: input.claims_replication,
        active_connections: input.active_connections,
        metrics_profile: METRICS_PROFILE,
        log_profile: input.log_profile,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_stable() {
        assert_eq!(METRICS_PROFILE, "dingo-metrics-v1");
        assert_eq!(HEALTH_PROFILE, "dingo-health-v1");
        assert_eq!(KNOWN_OPS.len(), KNOWN_OPS_LEN);
        assert_eq!(LATENCY_BUCKET_MS.len() + 1, LATENCY_BUCKET_COUNT);
    }

    #[test]
    fn unknown_op_maps_to_other_bucket() {
        let m = MetricsRegistry::new();
        m.observe_rpc(
            "totally_unknown_op",
            true,
            Duration::from_millis(3),
            None,
            None,
            None,
            None,
        );
        let snap = m.snapshot(None, None);
        let other = snap.ops.iter().find(|o| o.op == "other").unwrap();
        assert_eq!(other.total, 1);
        assert_eq!(other.ok, 1);
        // 3ms falls in le=5 bucket (index of 5 in LATENCY_BUCKET_MS is 2).
        assert!(other
            .latency_ms_buckets
            .iter()
            .any(|b| b.le == "5" && b.count >= 1));
    }

    #[test]
    fn guarantee_miss_counted() {
        let m = MetricsRegistry::new();
        m.observe_rpc(
            "put",
            true,
            Duration::from_millis(1),
            Some("durable"),
            Some("memory"),
            Some(false),
            None,
        );
        let snap = m.snapshot(None, None);
        assert_eq!(snap.guarantees.guarantee_miss, 1);
        assert_eq!(snap.guarantees.committed_false, 1);
        assert_eq!(snap.guarantees.requested_durable, 1);
        assert_eq!(snap.guarantees.achieved_memory, 1);
    }

    #[test]
    fn readiness_fails_when_draining() {
        let r = evaluate_health(HealthEvalInput {
            process_up: true,
            draining: true,
            store_open: true,
            store_path: Some("/data"),
            live_count: Some(0),
            mode: Some("serve"),
            node_index: None,
            raft_attached: false,
            claims_replication: false,
            active_connections: Some(0),
            log_profile: "dingo-log-v1",
        });
        assert!(r.live);
        assert!(!r.ready);
        assert_eq!(r.status, HealthStatus::NotReady);
        assert!(r.reasons.iter().any(|x| x.contains("draining")));
    }

    #[test]
    fn readiness_fails_when_replication_claimed_without_raft() {
        let r = evaluate_health(HealthEvalInput {
            process_up: true,
            draining: false,
            store_open: true,
            store_path: Some("/data"),
            live_count: Some(1),
            mode: Some("serve-cluster"),
            node_index: Some(0),
            raft_attached: false,
            claims_replication: true,
            active_connections: Some(1),
            log_profile: "dingo-log-v1",
        });
        assert!(!r.ready);
        assert!(r.reasons.iter().any(|x| x.contains("replication claimed")));
    }

    #[test]
    fn ready_when_store_open_single_node() {
        let r = evaluate_health(HealthEvalInput {
            process_up: true,
            draining: false,
            store_open: true,
            store_path: Some("/data"),
            live_count: Some(2),
            mode: Some("serve"),
            node_index: None,
            raft_attached: false,
            claims_replication: false,
            active_connections: Some(1),
            log_profile: "dingo-log-v1",
        });
        assert!(r.live && r.ready);
        assert_eq!(r.status, HealthStatus::Ready);
        assert!(r.reasons.is_empty());
        assert_eq!(r.profile, HEALTH_PROFILE);
    }

    #[test]
    fn public_probe_ops() {
        assert!(is_public_probe_op("health_live"));
        assert!(is_public_probe_op("health_ready"));
        assert!(!is_public_probe_op("health"));
        assert!(!is_public_probe_op("metrics"));
        assert!(!is_public_probe_op("ping"));
    }

    #[test]
    fn cardinality_bounded_to_known_ops() {
        let m = MetricsRegistry::new();
        for i in 0..200 {
            m.observe_rpc(
                &format!("custom_op_{i}"),
                true,
                Duration::from_millis(1),
                None,
                None,
                None,
                None,
            );
        }
        let snap = m.snapshot(None, None);
        assert!(snap.ops.len() <= MAX_OP_LABELS);
        assert_eq!(snap.ops.len(), 1);
        assert_eq!(snap.ops[0].op, "other");
        assert_eq!(snap.ops[0].total, 200);
    }
}
