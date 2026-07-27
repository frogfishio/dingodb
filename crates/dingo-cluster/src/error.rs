//! Cluster-layer errors (CLUSTER_SPEC §6, §9, §15).

use dingo_store::StoreError;
use thiserror::Error;

/// Errors from cluster open, routing, replication, and distributed reads.
#[derive(Debug, Error)]
pub enum ClusterError {
    /// Underlying single-node store failure.
    #[error(transparent)]
    Store(#[from] StoreError),

    /// Cluster metadata is missing or corrupt.
    #[error("corrupt cluster metadata: {0}")]
    CorruptMeta(&'static str),

    /// Path exists but is not a DingoDB cluster root.
    #[error("not a dingodb cluster: {0}")]
    NotACluster(String),

    /// Cluster already exists at path when exclusive create was requested.
    #[error("cluster already exists at {0}")]
    AlreadyExists(String),

    /// Required partition / replica is offline or unmarked.
    #[error("partition unavailable: partition {partition} ({reason})")]
    PartitionUnavailable {
        /// Virtual partition identifier.
        partition: u32,
        /// Human-readable reason.
        reason: &'static str,
    },

    /// Linearizable (or other) read cannot be proven with current coverage.
    #[error("coverage incomplete: {0}")]
    CoverageIncomplete(String),

    /// Requested durability / replica class cannot be met.
    #[error("durability unavailable: {0}")]
    DurabilityUnavailable(String),

    /// No leader (or authorized primary) for the partition in this term.
    #[error("no leader for partition {0}")]
    NoLeader(u32),

    /// Follower rejected a write (stale term, fencing, or body mismatch).
    #[error("replication rejected: {0}")]
    ReplicationRejected(String),

    /// Consistency mode does not allow this operation.
    #[error("consistency mode violation: {0}")]
    ConsistencyViolation(String),

    /// Rebalance job is missing, finished, or in a bad state.
    #[error("rebalance error: {0}")]
    Rebalance(String),

    /// Distributed query continuation token is invalid, stale, or tampered (DEF-040).
    #[error("continuation invalid: {0}")]
    ContinuationInvalid(String),

    /// Underlying IO failure outside the store layer.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl ClusterError {
    /// Stable machine code string (aligns with SDK `ErrorCode` names where shared).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Store(_) => "store",
            Self::CorruptMeta(_) => "corrupt_meta",
            Self::NotACluster(_) => "not_a_cluster",
            Self::AlreadyExists(_) => "already_exists",
            Self::PartitionUnavailable { .. } => "partition_unavailable",
            Self::CoverageIncomplete(_) => "coverage_incomplete",
            Self::DurabilityUnavailable(_) => "durability_unavailable",
            Self::NoLeader(_) => "no_leader",
            Self::ReplicationRejected(_) => "replication_rejected",
            Self::ConsistencyViolation(_) => "consistency_violation",
            Self::Rebalance(_) => "rebalance",
            Self::ContinuationInvalid(_) => "continuation_invalid",
            Self::Io(_) => "io",
        }
    }
}
