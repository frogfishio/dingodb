//! Online partition rebalancing (CLUSTER_SPEC §14, Stage 8f).
//!
//! Rebalancing one partition follows an interruptible step machine. Failure at
//! any step leaves either the old placement authoritative or an explicit joint
//! configuration — never an unrecorded ownership gap.

use crate::id::{NodeId, PartitionId, PlacementEpoch};
use serde::{Deserialize, Serialize};

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
            Self::PlanCommitted
                | Self::LearnersAdded
                | Self::SegmentsCopied
                | Self::LogCaughtUp
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
}
