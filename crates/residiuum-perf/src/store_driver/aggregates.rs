//! Exact boundary aggregate summaries — separate from lossless plan/replay.
//!
//! Counters, latency histograms, chain digests, and coverage are always valid
//! when the store probe ran. PhysicalWritePlan + L2 replay claims require a
//! **lossless** sample set (zero dropped samples).

use serde::{Deserialize, Serialize};

/// Exact probe aggregates for evidence (never claim lossless plan from these alone).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryAggregateSummary {
    /// Blake3 hex of the full event chain (all observations, including dropped samples).
    pub event_chain_digest: String,
    /// Total observations while probe enabled.
    pub total_observed: u64,
    /// Samples retained in the vector.
    pub samples_retained: u64,
    /// Observations not retained (capacity full).
    pub samples_dropped: u64,
    /// Sample vector capacity.
    pub sample_capacity: u64,
    /// True when any sample was dropped from the vector.
    pub sample_vector_capped: bool,
    /// Explicit drop reason when capped.
    pub drop_reason: Option<String>,
    /// Per-kind counts (append, file_write, file_sync, …) as name→count map.
    pub counts_by_kind: Vec<(String, u64)>,
    /// Outcome totals.
    pub outcome_ok: u64,
    pub outcome_short_write: u64,
    pub outcome_io_error: u64,
    pub total_requested_bytes: u64,
    pub total_completed_bytes: u64,
    /// Latency sample counts (histograms remain in store snapshot; counts only here).
    pub write_latency_samples: u64,
    pub sync_latency_samples: u64,
    pub append_latency_samples: u64,
    pub write_latency_mean_ns: f64,
    pub sync_latency_mean_ns: f64,
    /// True only when sample set is complete — plan/replay may be claimed.
    pub lossless_plan_eligible: bool,
    /// Why plan/replay is invalid when not eligible.
    pub plan_replay_invalidate_reason: Option<String>,
}

impl BoundaryAggregateSummary {
    /// Build from store snapshot fields (feature `store-driver`).
    #[cfg(feature = "store-driver")]
    pub fn from_store_snapshot(snap: &residiuum_store::BoundarySnapshot) -> Self {
        use residiuum_store::BoundaryKind;
        let cov = &snap.coverage;
        let lossless = !cov.sample_vector_capped && cov.samples_dropped == 0;
        let counts_by_kind = BoundaryKind::ALL
            .iter()
            .map(|k| (k.as_str().to_string(), snap.counters.count(*k)))
            .collect();
        Self {
            event_chain_digest: snap.event_chain_digest.clone(),
            total_observed: cov.total_observed,
            samples_retained: cov.samples_retained,
            samples_dropped: cov.samples_dropped,
            sample_capacity: cov.sample_capacity,
            sample_vector_capped: cov.sample_vector_capped,
            drop_reason: cov.drop_reason.clone(),
            counts_by_kind,
            outcome_ok: snap.counters.outcome_ok,
            outcome_short_write: snap.counters.outcome_short_write,
            outcome_io_error: snap.counters.outcome_io_error,
            total_requested_bytes: snap.counters.total_requested_bytes,
            total_completed_bytes: snap.counters.total_completed_bytes,
            write_latency_samples: snap.write_latency.samples,
            sync_latency_samples: snap.sync_latency.samples,
            append_latency_samples: snap.append_latency.samples,
            write_latency_mean_ns: snap.write_latency.mean_ns(),
            sync_latency_mean_ns: snap.sync_latency.mean_ns(),
            lossless_plan_eligible: lossless,
            plan_replay_invalidate_reason: if lossless {
                None
            } else {
                Some(format!(
                    "sample drops invalidate lossless plan/replay: dropped={} capped={} capacity={}",
                    cov.samples_dropped, cov.sample_vector_capped, cov.sample_capacity
                ))
            },
        }
    }
}

/// Matched probe-off vs probe-on observer overhead (same cell work).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObserverOverheadReport {
    pub cell_id: String,
    pub seed: u64,
    pub probe_off_e2e_ns: u64,
    pub probe_on_e2e_ns: u64,
    pub probe_off_logical_bytes: u64,
    pub probe_on_logical_bytes: u64,
    /// (on - off) / off when off > 0; else 0.
    pub overhead_fraction: f64,
    pub notes: Vec<String>,
}

impl ObserverOverheadReport {
    /// Compute overhead fraction from paired wall times.
    pub fn from_pair(
        cell_id: &str,
        seed: u64,
        off_e2e_ns: u64,
        on_e2e_ns: u64,
        off_bytes: u64,
        on_bytes: u64,
    ) -> Self {
        let overhead_fraction = if off_e2e_ns > 0 {
            (on_e2e_ns as f64 - off_e2e_ns as f64) / off_e2e_ns as f64
        } else {
            0.0
        };
        let mut notes = vec![
            "matched probe-off / probe-on observer overhead (same seed/cell work)".into(),
        ];
        if off_bytes != on_bytes {
            notes.push(format!(
                "logical_bytes differ off={off_bytes} on={on_bytes}; overhead wall-time only"
            ));
        }
        if overhead_fraction < 0.0 {
            notes.push(
                "probe-on faster than probe-off (noise); report negative overhead".into(),
            );
        }
        Self {
            cell_id: cell_id.into(),
            seed,
            probe_off_e2e_ns: off_e2e_ns,
            probe_on_e2e_ns: on_e2e_ns,
            probe_off_logical_bytes: off_bytes,
            probe_on_logical_bytes: on_bytes,
            overhead_fraction,
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overhead_fraction_basic() {
        let r = ObserverOverheadReport::from_pair("c1", 1, 1000, 1100, 100, 100);
        assert!((r.overhead_fraction - 0.1).abs() < 1e-9);
    }
}
