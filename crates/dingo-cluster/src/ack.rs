//! Cluster write acknowledgements (CLUSTER_SPEC §11.2).

use crate::id::{ClusterId, LogPosition, NodeId, PartitionId, PlacementEpoch, Term};
use crate::modes::{CommitStatus, ConsistencyMode};
use dingo_store::DurabilityMode;

/// Full acknowledgement for a replicated (or single-node) write.
///
/// “Replicated” without these details is not a complete durability claim
/// (CLUSTER_SPEC §11.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterWriteAck {
    /// Cluster that accepted the write.
    pub cluster_id: ClusterId,
    /// Consistency mode in effect.
    pub consistency_mode: ConsistencyMode,
    /// Local durability mode applied on each accepting replica.
    pub durability_mode: DurabilityMode,
    /// Partition that ordered the write.
    pub partition: PartitionId,
    /// Leadership term at acceptance.
    pub term: Term,
    /// Partition-local log position assigned by the leader.
    pub position: LogPosition,
    /// Placement epoch observed by the writer.
    pub placement_epoch: PlacementEpoch,
    /// Number of replica acknowledgements (including the leader).
    pub replica_acks: u32,
    /// Whether logical commitment is proven (quorum for strong mode).
    pub committed: bool,
    /// Finer commit classification.
    pub commit_status: CommitStatus,
    /// Leader node that ordered the write.
    pub leader: NodeId,
    /// Store-level event id from the leader (when available).
    pub event_id: [u8; 16],
    /// Store id of the leader node.
    pub store_id: [u8; 16],
    /// Segment that received the leader frame.
    pub segment_id: [u8; 16],
}
