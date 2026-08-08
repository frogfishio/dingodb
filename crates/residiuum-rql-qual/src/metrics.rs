//! §7.4 metric envelopes — types for Q4.1; collectors land in Q4.3.

use serde::{Deserialize, Serialize};

/// Latency quantiles in nanoseconds (empty until measured).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LatencyQuantilesNs {
    pub p50: Option<u64>,
    pub p95: Option<u64>,
    pub p99: Option<u64>,
    pub max: Option<u64>,
    pub samples: u64,
}

/// Resource snapshot for one cell / repetition.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub cpu_time_ns: Option<u64>,
    pub rss_bytes: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
    pub physical_bytes_read: Option<u64>,
    pub physical_bytes_written: Option<u64>,
    pub read_amplification: Option<f64>,
}

/// Query-path accounting.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QueryPathMetrics {
    pub documents_examined: Option<u64>,
    pub index_entries_examined: Option<u64>,
    pub index_size_bytes: Option<u64>,
    pub index_build_ns: Option<u64>,
    pub indexed_write_penalty_ns: Option<u64>,
    pub explain_plan_digest: Option<String>,
}

/// Cache / lifecycle class label (programme §7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleClass {
    WarmSteady,
    FreshReopen,
    LargerThanMemory,
    ReadOnly,
    ConcurrentWrites,
    RotationCompaction,
    DeclaredDamage,
}

impl LifecycleClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WarmSteady => "warm_steady",
            Self::FreshReopen => "fresh_reopen",
            Self::LargerThanMemory => "larger_than_memory",
            Self::ReadOnly => "read_only",
            Self::ConcurrentWrites => "concurrent_writes",
            Self::RotationCompaction => "rotation_compaction",
            Self::DeclaredDamage => "declared_damage",
        }
    }
}

/// Full per-cell metrics envelope (programme §7.4 list).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CellMetrics {
    pub queries_per_s: Option<f64>,
    pub latency: LatencyQuantilesNs,
    pub resource: ResourceSnapshot,
    pub path: QueryPathMetrics,
    pub lifecycle: Option<LifecycleClass>,
    /// How “cold” was obtained when lifecycle claims cold (required honesty).
    pub cold_method: Option<String>,
    pub deferred_work_units: Option<u64>,
    pub deferred_drained: Option<bool>,
    /// Result validity + digest carried on CanonicalResult; optional echo.
    pub result_digest_echo: Option<String>,
    pub coverage_complete: Option<bool>,
}

/// Required metric field names for evidence completeness checks (Q4.3 fills).
pub const REQUIRED_METRIC_KEYS: &[&str] = &[
    "result_digest",
    "coverage",
    "validity",
    "queries_per_s",
    "latency_p50_p95_p99_max",
    "cpu_rss",
    "physical_bytes_rw_amplification",
    "docs_index_examined",
    "index_size_build_write_penalty",
    "explain_plan",
    "cache_lifecycle_state",
    "deferred_work_drain",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_keys_cover_programme_list() {
        assert!(REQUIRED_METRIC_KEYS.len() >= 10);
        assert!(REQUIRED_METRIC_KEYS.contains(&"result_digest"));
        assert!(REQUIRED_METRIC_KEYS.contains(&"latency_p50_p95_p99_max"));
    }
}
