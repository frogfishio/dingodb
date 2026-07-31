//! Online partition rebalancing (CLUSTER_SPEC §14, Stage 8f).
//!
//! Rebalancing one partition follows an interruptible step machine. Failure at
//! any step leaves either the old placement authoritative or an explicit joint
//! configuration — never an unrecorded ownership gap.
//!
//! # Durability (DEF-038)
//!
//! In-flight jobs are persisted under the cluster root as
//! [`REBALANCE_JOBS_FILE`] (atomic replace + previous generation). Restart at
//! any phase reloads jobs so the coordinator can resume or leave old/joint
//! placement authoritative without inventing ownership.

use crate::error::ClusterError;
use crate::id::{NodeId, PartitionId, PlacementEpoch};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Profile tag for durable rebalance control-plane documents (DEF-038).
pub const REBALANCE_CONTROL_PROFILE: &str = "residiuum-rebalance-control-v1";

/// Filename under the cluster root for in-flight rebalance jobs.
pub const REBALANCE_JOBS_FILE: &str = "rebalance_jobs.json";

/// Phase of an in-process rebalance job (CLUSTER_SPEC §14 steps 1–9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RebalancePhase {
    /// Placement plan committed (old + proposed replica sets recorded).
    PlanCommitted,
    /// Destination nodes registered as non-voting learners.
    LearnersAdded,
    /// Immutable live subjects copied and verified onto destinations.
    SegmentsCopied,
    /// Active log tail streamed; destinations at declared safe position.
    LogCaughtUp,
    /// Consensus-safe membership change applied (joint or final voters).
    MembershipChanged,
    /// New placement epoch activated; directory points at new set.
    EpochActivated,
    /// Old replicas retained for the safety window.
    SafetyWindow,
    /// Old copies reclaimed after independent placement checks.
    Reclaimed,
}

impl RebalancePhase {
    /// Stable name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlanCommitted => "plan-committed",
            Self::LearnersAdded => "learners-added",
            Self::SegmentsCopied => "segments-copied",
            Self::LogCaughtUp => "log-caught-up",
            Self::MembershipChanged => "membership-changed",
            Self::EpochActivated => "epoch-activated",
            Self::SafetyWindow => "safety-window",
            Self::Reclaimed => "reclaimed",
        }
    }

    /// Next phase after a successful advance, if any.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::PlanCommitted => Some(Self::LearnersAdded),
            Self::LearnersAdded => Some(Self::SegmentsCopied),
            Self::LogCaughtUp => Some(Self::MembershipChanged),
            Self::SegmentsCopied => Some(Self::LogCaughtUp),
            Self::MembershipChanged => Some(Self::EpochActivated),
            Self::EpochActivated => Some(Self::SafetyWindow),
            Self::SafetyWindow => Some(Self::Reclaimed),
            Self::Reclaimed => None,
        }
    }

    /// Whether the old replica set is still the sole authoritative membership.
    pub fn old_placement_authoritative(self) -> bool {
        matches!(
            self,
            Self::PlanCommitted | Self::LearnersAdded | Self::SegmentsCopied | Self::LogCaughtUp
        )
    }

    /// Whether voters are the joint (old ∪ new) set.
    pub fn is_joint(self) -> bool {
        matches!(self, Self::MembershipChanged)
    }

    /// Whether the new replica set is authoritative.
    pub fn new_placement_authoritative(self) -> bool {
        matches!(
            self,
            Self::EpochActivated | Self::SafetyWindow | Self::Reclaimed
        )
    }
}

/// In-flight rebalance job for one partition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebalanceJob {
    /// Opaque job identity.
    pub job_id: String,
    /// Partition being moved.
    pub partition: PartitionId,
    /// Replica set before the job started.
    pub old_replicas: Vec<NodeId>,
    /// Target voting replica set after success.
    pub new_replicas: Vec<NodeId>,
    /// Nodes being added (subset of new not in old).
    pub destinations: Vec<NodeId>,
    /// Nodes being removed (subset of old not in new); retained until reclaim.
    pub removals: Vec<NodeId>,
    /// Current phase.
    pub phase: RebalancePhase,
    /// Placement epoch at plan commit (pre-activation).
    pub plan_epoch: PlacementEpoch,
    /// Placement epoch after activation (set at EpochActivated).
    pub activated_epoch: Option<PlacementEpoch>,
    /// Subjects copied during SegmentsCopied.
    pub subjects_copied: u64,
    /// Log entries streamed during LogCaughtUp.
    pub log_entries_streamed: u64,
    /// Explicit joint configuration flag (MembershipChanged until EpochActivated).
    pub joint: bool,
}

impl RebalanceJob {
    /// Build a new plan at `PlanCommitted`.
    pub fn plan(
        job_id: impl Into<String>,
        partition: PartitionId,
        old_replicas: Vec<NodeId>,
        new_replicas: Vec<NodeId>,
        plan_epoch: PlacementEpoch,
    ) -> Self {
        let mut destinations: Vec<NodeId> = new_replicas
            .iter()
            .copied()
            .filter(|n| !old_replicas.contains(n))
            .collect();
        destinations.sort();
        let mut removals: Vec<NodeId> = old_replicas
            .iter()
            .copied()
            .filter(|n| !new_replicas.contains(n))
            .collect();
        removals.sort();
        Self {
            job_id: job_id.into(),
            partition,
            old_replicas,
            new_replicas,
            destinations,
            removals,
            phase: RebalancePhase::PlanCommitted,
            plan_epoch,
            activated_epoch: None,
            subjects_copied: 0,
            log_entries_streamed: 0,
            joint: false,
        }
    }

    /// Authoritative voting set for the current phase.
    pub fn authoritative_voters(&self) -> Vec<NodeId> {
        if self.phase.old_placement_authoritative() {
            self.old_replicas.clone()
        } else if self.phase.is_joint() || self.joint {
            let mut v = self.old_replicas.clone();
            for n in &self.new_replicas {
                if !v.contains(n) {
                    v.push(*n);
                }
            }
            v.sort();
            v
        } else {
            self.new_replicas.clone()
        }
    }
}

/// Outcome of running rebalance steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebalanceReport {
    /// Final job state.
    pub job: RebalanceJob,
    /// Phases successfully entered (including plan commit).
    pub phases_completed: Vec<RebalancePhase>,
}

/// On-disk document for all in-flight rebalance jobs (DEF-038).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebalanceJobsFile {
    /// Format / profile tag.
    #[serde(default = "default_rebalance_format")]
    pub format: String,
    /// Monotonic generation; increments on every successful save.
    #[serde(default)]
    pub generation: u64,
    /// BLAKE3-256 hex of the canonical job list (sorted by partition id).
    #[serde(default)]
    pub content_blake3: String,
    /// In-flight jobs (empty when none).
    #[serde(default)]
    pub jobs: Vec<RebalanceJob>,
}

fn default_rebalance_format() -> String {
    REBALANCE_CONTROL_PROFILE.into()
}

impl RebalanceJobsFile {
    /// Empty document.
    pub fn new() -> Self {
        Self {
            format: default_rebalance_format(),
            generation: 0,
            content_blake3: String::new(),
            jobs: Vec::new(),
        }
    }

    /// Build from a set of jobs.
    pub fn from_jobs(jobs: impl IntoIterator<Item = RebalanceJob>) -> Self {
        let mut file = Self {
            format: default_rebalance_format(),
            generation: 0,
            content_blake3: String::new(),
            jobs: jobs.into_iter().collect(),
        };
        file.jobs.sort_by_key(|j| j.partition.get());
        file.refresh_checksum();
        file
    }

    /// Recompute [`Self::content_blake3`].
    pub fn refresh_checksum(&mut self) {
        self.jobs.sort_by_key(|j| j.partition.get());
        self.content_blake3 = jobs_content_hash(&self.jobs);
    }

    /// Validate format + checksum when a hash is present.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.format != REBALANCE_CONTROL_PROFILE && !self.format.is_empty() {
            return Err(ClusterError::CorruptMeta(
                "unsupported rebalance_jobs format",
            ));
        }
        if !self.content_blake3.is_empty() {
            let expect = jobs_content_hash(&self.jobs);
            if self.content_blake3 != expect {
                return Err(ClusterError::CorruptMeta(
                    "rebalance_jobs.json content_blake3 mismatch",
                ));
            }
        }
        Ok(())
    }

    /// Load from `root/rebalance_jobs.json`, or empty if missing.
    ///
    /// Falls back to `.prev` when the primary is corrupt (DEF-021).
    pub fn load(root: &Path) -> Result<Self, ClusterError> {
        let path = root.join(REBALANCE_JOBS_FILE);
        if let Some(file) = try_parse_jobs(&path)? {
            return Ok(file);
        }
        let prev = residiuum_store::previous_path(&path);
        if let Some(file) = try_parse_jobs(&prev)? {
            return Ok(file);
        }
        if path.is_file() || prev.is_file() {
            return Err(ClusterError::CorruptMeta(
                "rebalance_jobs.json unreadable; restore .prev or clear rebalance state",
            ));
        }
        Ok(Self::new())
    }

    /// Persist under the cluster root (atomic durable; keeps previous generation).
    pub fn save(&self, root: &Path) -> Result<(), ClusterError> {
        let path = root.join(REBALANCE_JOBS_FILE);
        let mut out = self.clone();
        out.generation = self.generation.saturating_add(1).max(1);
        out.refresh_checksum();
        let json = serde_json::to_string_pretty(&out)
            .map_err(|_| ClusterError::CorruptMeta("serialize rebalance_jobs.json"))?;
        residiuum_store::write_atomic_keep_previous(&path, json.as_bytes())?;
        Ok(())
    }
}

impl Default for RebalanceJobsFile {
    fn default() -> Self {
        Self::new()
    }
}

fn jobs_content_hash(jobs: &[RebalanceJob]) -> String {
    // Canonical JSON per job in partition order (already sorted by callers).
    let mut h = Hasher::new();
    for job in jobs {
        let bytes = serde_json::to_vec(job).unwrap_or_default();
        h.update(&(bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    h.finalize().to_hex().to_string()
}

fn try_parse_jobs(path: &Path) -> Result<Option<RebalanceJobsFile>, ClusterError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let file: RebalanceJobsFile = match serde_json::from_slice(&bytes) {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };
    if file.validate().is_err() {
        return Ok(None);
    }
    Ok(Some(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_machine_reaches_reclaimed() {
        let mut p = RebalancePhase::PlanCommitted;
        let mut n = 1;
        while let Some(next) = p.next() {
            p = next;
            n += 1;
        }
        assert_eq!(p, RebalancePhase::Reclaimed);
        assert_eq!(n, 8);
    }

    #[test]
    fn plan_splits_destinations_and_removals() {
        let job = RebalanceJob::plan(
            "j1",
            PartitionId::new(0),
            vec![NodeId::new(0), NodeId::new(1)],
            vec![NodeId::new(1), NodeId::new(2)],
            PlacementEpoch(1),
        );
        assert_eq!(job.destinations, vec![NodeId::new(2)]);
        assert_eq!(job.removals, vec![NodeId::new(0)]);
        assert!(job.phase.old_placement_authoritative());
    }

    #[test]
    fn jobs_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let job = RebalanceJob::plan(
            "j1",
            PartitionId::new(2),
            vec![NodeId::new(0)],
            vec![NodeId::new(0), NodeId::new(1)],
            PlacementEpoch(1),
        );
        let file = RebalanceJobsFile::from_jobs([job.clone()]);
        file.save(dir.path()).unwrap();
        let loaded = RebalanceJobsFile::load(dir.path()).unwrap();
        assert_eq!(loaded.jobs.len(), 1);
        assert_eq!(loaded.jobs[0].job_id, "j1");
        assert_eq!(loaded.jobs[0].phase, RebalancePhase::PlanCommitted);
        assert!(loaded.generation >= 1);
        assert!(!loaded.content_blake3.is_empty());
    }
}
