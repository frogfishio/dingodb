//! DingoDB cluster federation (Stage 8 + product freezes).
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
//! - Durable Raft hard state / log / membership / snapshots
//!   ([`raft_persist`]; DEF-035 / `dingo-raft-persist-v1`)
//! - Convergent-append dual-accept + reconcile ([`convergent`]; CLUSTER_SPEC §9.2)
//! - Distributed find/scan with coverage honesty (Stage 8e; CLUSTER_SPEC §17)
//! - Interruptible partition rebalance (Stage 8f; CLUSTER_SPEC §14)
//! - Node-local salvage without cluster software
//! - Network advertise path: [`endpoints`] + `dingo serve-cluster`
//!
//! **Product freeze:** [`CLUSTER_PROFILE_VERSION`] labels the Stage 8
//! conformance profile (no cross-partition atomic writes; CLUSTER_SPEC).

#![deny(missing_docs)]

/// Cluster federation profile freeze label (DELIVERY_PLAN §7: Cluster profile v1).
///
/// Stage 8a–8f conformance is locked: partitions, coverage, Raft-equivalent
/// leadership, convergent-append, find coverage, rebalance. Network multi-hop
/// continues to harden on the same rules without changing this label.
pub const CLUSTER_PROFILE_VERSION: &str = "v1";

mod ack;
mod cluster;
mod config;
mod convergent;
mod coverage;
mod directory;
mod endpoints;
mod error;
mod id;
mod modes;
mod partition;
pub mod raft;
pub mod raft_persist;
mod rebalance;

pub use ack::ClusterWriteAck;
pub use cluster::Cluster;
pub use config::{node_store_path, ClusterConfig, ClusterMeta};
pub use convergent::{
    body_content_hash, ConvergentIdentity, ReconcileReport, SubjectConflict, SubjectVariant,
};
pub use coverage::{Coverage, FindResult, GetResult, PartitionFrontier, ScanOptions, ScanResult};
pub use directory::{PartitionAssignment, PartitionDirectory};
pub use endpoints::{
    load_endpoints, save_endpoints, upsert_endpoint, EndpointsFile, ENDPOINTS_FILE,
};
pub use error::ClusterError;
pub use id::{ClusterId, LogPosition, NodeId, PartitionId, PlacementEpoch, Term};
pub use modes::{CommitStatus, ConsistencyMode, DeploymentProfile, ReadMode};
pub use partition::{
    default_partition_key, PartitionMap, DEFAULT_VIRTUAL_PARTITIONS, HASH_PROFILE_BLAKE3_MOD,
};
pub use raft::{CommitEvidence, LogCommand, LogEntry, PartitionRaft, RaftPeer, RaftRole};
pub use raft_persist::{
    snapshot_meta_for, ConsensusEvidenceClass, HardState, MembershipState, RaftPeerStore,
    Snapshot, SnapshotMeta, RAFT_PERSIST_PROFILE,
};
pub use rebalance::{RebalanceJob, RebalancePhase, RebalanceReport};

/// Re-export durability modes used on cluster acks.
pub use dingo_store::DurabilityMode;
