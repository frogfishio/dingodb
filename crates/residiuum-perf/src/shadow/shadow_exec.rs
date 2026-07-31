//! Opaque-byte L2 shadow executor — executes a PhysicalWritePlan via IoAdapter.

use super::plan::{DestinationClass, PhysicalWritePlan, SyncBoundary};
use super::replay::validate_plan_replay;
use super::ShadowError;
use crate::envelope::{IoAdapter, IoMode, SyncMode};
use crate::metrics::LatencyHistogram;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShadowStats {
    pub ops_attempted: u64,
    pub ops_completed: u64,
    pub ops_failed: u64,
    pub bytes_requested: u64,
    pub bytes_completed: u64,
    pub syncs_completed: u64,
    pub rotations_seen: u64,
    pub partial_or_short: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowReport {
    pub schema: String,
    pub layer: String,
    pub plan_id: String,
    pub plan_digest: String,
    pub stats: ShadowStats,
    pub latency: LatencyHistogram,
    pub planned_bytes: u64,
    pub requested_vs_planned_match: bool,
    pub errors: Vec<String>,
    pub store_seam_status: String,
}

/// Execute plan with opaque bytes only (no encoding/checksums).
pub fn execute_shadow<A: IoAdapter>(
    adapter: &mut A,
    plan: &PhysicalWritePlan,
) -> Result<ShadowReport, ShadowError> {
    validate_plan_replay(plan)?;

    let mut stats = ShadowStats::default();
    let mut latency = LatencyHistogram::new();
    let mut errors = Vec::new();
    let mut current_file = segment_file_id(0);
    adapter.create_file(&current_file)?;

    for op in &plan.ops {
        if op.segment_rotate {
            stats.rotations_seen = stats.rotations_seen.saturating_add(1);
            current_file = segment_file_id(op.segment_gen);
            adapter.create_file(&current_file)?;
        }

        stats.ops_attempted = stats.ops_attempted.saturating_add(1);
        stats.bytes_requested = stats.bytes_requested.saturating_add(op.size);

        if op.size > 0 {
            let file = match op.dest {
                DestinationClass::SegmentData | DestinationClass::SegmentMeta => {
                    current_file.clone()
                }
                DestinationClass::ChunkData => format!("chunk-{}", op.segment_gen),
                DestinationClass::Directory => {
                    // No data write.
                    current_file.clone()
                }
            };
            if matches!(op.dest, DestinationClass::ChunkData) {
                let _ = adapter.create_file(&file);
            }
            if !matches!(op.dest, DestinationClass::Directory) {
                match adapter.write_block(&file, None, op.size as usize, IoMode::Buffered) {
                    Ok(r) => {
                        stats.ops_completed = stats.ops_completed.saturating_add(1);
                        stats.bytes_completed =
                            stats.bytes_completed.saturating_add(r.bytes_completed);
                        latency.record(r.latency_ns.max(1));
                        if r.bytes_completed < op.size {
                            stats.partial_or_short = stats.partial_or_short.saturating_add(1);
                        }
                    }
                    Err(e) => {
                        stats.ops_failed = stats.ops_failed.saturating_add(1);
                        let msg = format!("seq {}: {e}", op.seq);
                        // Classify short/partial.
                        if matches!(
                            e,
                            crate::envelope::IoError::ShortWrite { .. }
                                | crate::envelope::IoError::Partial { .. }
                        ) {
                            stats.partial_or_short = stats.partial_or_short.saturating_add(1);
                        }
                        errors.push(msg);
                    }
                }
            } else {
                stats.ops_completed = stats.ops_completed.saturating_add(1);
            }
        } else {
            // pure sync / zero-size marker
            stats.ops_completed = stats.ops_completed.saturating_add(1);
        }

        let sync_mode = match op.sync_after {
            SyncBoundary::None => None,
            SyncBoundary::DataOnly => Some(SyncMode::DataOnly),
            SyncBoundary::FullFile | SyncBoundary::Directory => Some(SyncMode::FullFile),
        };
        if let Some(sm) = sync_mode {
            match adapter.sync(&current_file, sm) {
                Ok(r) => {
                    stats.syncs_completed = stats.syncs_completed.saturating_add(1);
                    latency.record(r.latency_ns.max(1));
                }
                Err(e) => {
                    stats.ops_failed = stats.ops_failed.saturating_add(1);
                    errors.push(format!("sync seq {}: {e}", op.seq));
                }
            }
        }
    }

    let match_bytes = stats.bytes_requested == plan.planned_bytes;
    Ok(ShadowReport {
        schema: "residiuum-pqh5-l2-shadow-report-v1".into(),
        layer: "L2".into(),
        plan_id: plan.plan_id.clone(),
        plan_digest: plan.shape_hash.clone(),
        stats,
        latency,
        planned_bytes: plan.planned_bytes,
        requested_vs_planned_match: match_bytes,
        errors,
        store_seam_status: super::STORE_SEAM_STATUS.into(),
    })
}

fn segment_file_id(gen: u32) -> String {
    format!("seg-{gen:05}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{FakeIoAdapter, FakeIoConfig};
    use crate::shadow::plan::{PlanBuilder, ShapeConfig};

    #[test]
    fn shadow_executes_plan_opaque() {
        let plan = PlanBuilder::build(&ShapeConfig {
            plan_id: "s1".into(),
            write_sizes: vec![512, 1024, 512],
            segment_threshold: 2000,
            batch_size: 2,
            sync_every_ops: 2,
            final_sync: true,
            ..ShapeConfig::default()
        });
        let mut adapter = FakeIoAdapter::new(FakeIoConfig::default());
        let report = execute_shadow(&mut adapter, &plan).unwrap();
        assert!(report.requested_vs_planned_match);
        assert_eq!(report.stats.bytes_requested, plan.planned_bytes);
        assert!(report.stats.ops_completed > 0);
        assert!(report.stats.syncs_completed > 0);
        assert_eq!(report.store_seam_status, super::super::STORE_SEAM_STATUS);
    }

    #[test]
    fn partial_io_recorded() {
        let plan = PlanBuilder::build(&ShapeConfig {
            write_sizes: vec![100, 100],
            batch_size: 100,
            sync_every_ops: 0,
            final_sync: false,
            ..ShapeConfig::default()
        });
        let mut adapter = FakeIoAdapter::new(FakeIoConfig {
            short_write_after: Some(0),
            ..FakeIoConfig::default()
        });
        let report = execute_shadow(&mut adapter, &plan).unwrap();
        assert!(report.stats.ops_failed > 0 || report.stats.partial_or_short > 0);
    }
}
