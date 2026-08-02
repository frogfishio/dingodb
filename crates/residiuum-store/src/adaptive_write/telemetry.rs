//! Bounded AWO inspection / operational telemetry (AWO-7).
//!
//! Snapshots only — no bodies, subjects, or secrets. Mode default remains
//! disabled until principal G12 accept.

use super::controller::ScaleDecision;
use super::policy::AdaptiveWriteMode;
use super::runtime::AdaptiveWriteStatus;
use super::types::AWO_PROFILE;

/// Operator-facing inspection report (support matrix + live status).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveWriteInspect {
    /// Profile identity.
    pub profile: &'static str,
    /// Configured mode.
    pub mode: AdaptiveWriteMode,
    /// Whether product default is still disabled (always true until G12).
    pub default_mode_is_disabled: bool,
    /// Lease fences direct store mutation.
    pub lease_active: bool,
    /// Draining.
    pub draining: bool,
    /// Pipeline depth limit.
    pub pipeline_depth_limit: usize,
    /// Pipeline in-flight count.
    pub pipeline_in_flight: usize,
    /// Cooker threads created.
    pub cooker_threads: usize,
    /// Active cooker permits.
    pub active_cookers: usize,
    /// Queue entries reserved.
    pub entries_used: usize,
    /// Queue bytes reserved.
    pub bytes_used: usize,
    /// Support matrix row labels (static).
    pub support_matrix: &'static [&'static str],
    /// Upgrade / rollback note (static).
    pub upgrade_rollback_note: &'static str,
    /// Benchmark disclosure note (static).
    pub benchmark_disclosure: &'static str,
}

/// Closed support matrix rows for V1 productisation docs.
pub const SUPPORT_MATRIX: &[&str] = &[
    "mode_default=disabled (principal G12 for default-on)",
    "eligible=unconditional_inline_put|unconditional_delete",
    "natural=conditional|chunked|memory|atomics|maintenance",
    "async_runtime=none (std threads only)",
    "durability=Buffered|Durable (Memory is natural-only)",
    "pipeline_depth_default=2",
    "crash_matrix=process-local failpoints + limited multi-process abort cells",
    "pqh_g8=not claimed by smoke",
];

/// Upgrade/rollback honesty string.
pub const UPGRADE_ROLLBACK_NOTE: &str =
    "AWO attach is opt-in via create/open_with_adaptive_write; ordinary create/open unchanged. \
     Rolling back: reopen without adaptive attach (or mode=disabled). No on-disk format change \
     from AWO enablement alone.";

/// Benchmark disclosure honesty string.
pub const BENCHMARK_DISCLOSURE: &str =
    "AWO batching may change write latency distribution and tail; do not compare mixed-mode \
     runs without disclosing AdaptiveWriteMode, policy limits, and cooker count. Smoke never \
     marks AWO-G8.";

impl AdaptiveWriteInspect {
    /// Build from a live status snapshot (or defaults when unattached).
    pub fn from_status(status: Option<&AdaptiveWriteStatus>) -> Self {
        match status {
            Some(s) => Self {
                profile: AWO_PROFILE,
                mode: s.mode,
                default_mode_is_disabled: true,
                lease_active: s.lease_active,
                draining: s.draining,
                pipeline_depth_limit: s.pipeline.depth_limit,
                pipeline_in_flight: s.pipeline.in_flight,
                cooker_threads: s.cooker_threads,
                active_cookers: s.active_cookers,
                entries_used: s.entries_used,
                bytes_used: s.bytes_used,
                support_matrix: SUPPORT_MATRIX,
                upgrade_rollback_note: UPGRADE_ROLLBACK_NOTE,
                benchmark_disclosure: BENCHMARK_DISCLOSURE,
            },
            None => Self {
                profile: AWO_PROFILE,
                mode: AdaptiveWriteMode::Disabled,
                default_mode_is_disabled: true,
                lease_active: false,
                draining: false,
                pipeline_depth_limit: 2,
                pipeline_in_flight: 0,
                cooker_threads: 0,
                active_cookers: 0,
                entries_used: 0,
                bytes_used: 0,
                support_matrix: SUPPORT_MATRIX,
                upgrade_rollback_note: UPGRADE_ROLLBACK_NOTE,
                benchmark_disclosure: BENCHMARK_DISCLOSURE,
            },
        }
    }
}

/// Last scale decision placeholder for future telemetry hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScaleTelemetry {
    /// Most recent scale decision observed.
    pub last_decision: Option<ScaleDecision>,
}
