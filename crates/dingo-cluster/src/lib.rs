//! DingoDB cluster federation (Stage 8).
//!
//! A cluster is a **federation of independently recoverable partitions and
//! segments** ([CLUSTER_SPEC](../../CLUSTER_SPEC.md)). Consensus decides who may
//! coordinate writes; it does not give bytes their meaning.
//!
//! ## Stage 8 surface
//!
//! - Deterministic virtual partition map ([`PartitionMap`])
//! - Consistency / read / commit modes
//! - [`Coverage`] on every multi-partition result
//! - Placement [`PartitionDirectory`]
//! - In-process multi-node [`Cluster`] (development + dependable-local)
//! - Per-partition Raft-equivalent elections, log matching, and commit evidence
//!   ([`raft`] module; CLUSTER_SPEC §10)
//! - Convergent-append dual-accept + reconcile ([`convergent`]; CLUSTER_SPEC §9.2)
//! - Distributed find/scan with coverage honesty (Stage 8e; CLUSTER_SPEC §17)
//! - Interruptible partition rebalance (Stage 8f; CLUSTER_SPEC §14)
//! - Node-local salvage without cluster software

#![deny(missing_docs)]

mod ack;
mod cluster;
mod config;
mod convergent;
mod coverage;
mod directory;
mod error;
mod id;
mod modes;
mod partition;
pub mod raft;
mod rebalance;

pub use ack::ClusterWriteAck;
pub use cluster::Cluster;
pub use config::ClusterConfig;
pub use convergent::{
    body_content_hash, ConvergentIdentity, ReconcileReport, SubjectConflict, SubjectVariant,
};
pub use coverage::{
    Coverage, FindResult, GetResult, PartitionFrontier, ScanOptions, ScanResult,
};
pub use directory::{PartitionAssignment, PartitionDirectory};
pub use error::ClusterError;
pub use id::{ClusterId, LogPosition, NodeId, PartitionId, PlacementEpoch, Term};
pub use modes::{CommitStatus, ConsistencyMode, DeploymentProfile, ReadMode};
pub use partition::{
    default_partition_key, PartitionMap, DEFAULT_VIRTUAL_PARTITIONS, HASH_PROFILE_BLAKE3_MOD,
};
pub use raft::{
    CommitEvidence, LogCommand, LogEntry, PartitionRaft, RaftPeer, RaftRole,
};
pub use rebalance::{RebalanceJob, RebalancePhase, RebalanceReport};

/// Re-export durability modes used on cluster acks.
pub use dingo_store::DurabilityMode;
