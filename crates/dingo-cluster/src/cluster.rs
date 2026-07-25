//! In-process multi-node cluster (Stage 8a–8f).
//!
//! Each node is an ordinary [`dingo_store::Store`]. Writes in
//! `partition-linearizable` mode go through a per-partition Raft-equivalent
//! group (elections, log matching, majority commit), then apply committed
//! commands to replica stores.
//!
//! Stage 8a used static primary leadership from the placement directory.
//! Stage 8b elects leaders dynamically when the prior leader is offline and
//! a majority of configured voters remains reachable.
//! Stage 8c adds `convergent-append`: any online replica may accept unique
//! events without quorum; dual-accept across a split is allowed; reconcile
//! merges by subject + content hash and reports conflicts explicitly.
//! Stage 8e adds distributed find with coverage honesty on partial queries.
//! Stage 8f adds interruptible partition rebalance (CLUSTER_SPEC §14).

use crate::ack::ClusterWriteAck;
use crate::config::{node_store_path, ClusterConfig, ClusterMeta};
use crate::convergent::{body_content_hash, ReconcileReport, SubjectConflict, SubjectVariant};
use crate::coverage::{Coverage, FindResult, GetResult, ScanOptions, ScanResult};
use crate::directory::PartitionDirectory;
use crate::error::ClusterError;
use crate::id::{ClusterId, LogPosition, NodeId, PartitionId, PlacementEpoch, Term};
use crate::modes::{CommitStatus, ConsistencyMode, DeploymentProfile, ReadMode};
use crate::partition::{default_partition_key, PartitionMap};
use crate::raft::{ElectError, LogCommand, PartitionRaft, ProposeError};
use crate::rebalance::{RebalanceJob, RebalancePhase, RebalanceReport};
use dingo_store::{DurabilityMode, SalvageReport, Store};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Open cluster handle: federation of independently salvageable stores.
pub struct Cluster {
    root: PathBuf,
    cluster_id: ClusterId,
    profile: DeploymentProfile,
    consistency_mode: ConsistencyMode,
    partition_map: PartitionMap,
    directory: PartitionDirectory,
    /// Live nodes keyed by dense index. Absent / offline nodes are missing.
    nodes: HashMap<u32, Store>,
    /// Offline node ids (explicitly marked down for tests / ops).
    offline: Vec<u32>,
    /// Per-partition Raft groups (Stage 8b; unused for convergent writes).
    raft: HashMap<u32, PartitionRaft>,
    /// Partition-local accept counters for convergent-append (not Raft index).
    convergent_pos: HashMap<u32, u64>,
    /// In-flight rebalance jobs keyed by partition id (Stage 8f).
    rebalance_jobs: HashMap<u32, RebalanceJob>,
}

impl Cluster {
    /// Create a new cluster at `cfg.root` (must not already exist).
    pub fn create(cfg: ClusterConfig) -> Result<Self, ClusterError> {
        let root = cfg.root.clone();
        if root.exists() {
            if root.join("cluster.json").is_file() {
                return Err(ClusterError::AlreadyExists(root.display().to_string()));
            }
        } else {
            std::fs::create_dir_all(&root)?;
        }

        let seed = format!(
            "{}-{}",
            root.display(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let cluster_id = cfg
            .cluster_id
            .unwrap_or_else(|| ClusterId::generate(seed.as_bytes()));

        let meta = ClusterMeta::from_config(&cfg, cluster_id);
        meta.write(&root)?;

        let node_count = cfg.profile.default_node_count();
        std::fs::create_dir_all(root.join("nodes"))?;

        let mut nodes = HashMap::new();
        for i in 0..node_count {
            let path = node_store_path(&root, i);
            let store = Store::create(&path)?;
            nodes.insert(i, store);
        }

        let directory = PartitionDirectory::balanced(
            cfg.partition_map.clone(),
            node_count,
            PlacementEpoch(meta.placement_epoch),
        );
        directory.save(&root)?;

        let raft = Self::build_raft_groups(&directory);

        Ok(Self {
            root,
            cluster_id,
            profile: cfg.profile,
            consistency_mode: cfg.consistency_mode,
            partition_map: cfg.partition_map,
            directory,
            nodes,
            offline: Vec::new(),
            raft,
            convergent_pos: HashMap::new(),
            rebalance_jobs: HashMap::new(),
        })
    }

    /// Open an existing cluster root, bringing all on-disk nodes online.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ClusterError> {
        let root = root.as_ref().to_path_buf();
        let meta = ClusterMeta::load(&root)?;
        let cluster_id = ClusterId::from_hex(&meta.cluster_id)
            .ok_or(ClusterError::CorruptMeta("bad cluster_id hex"))?;
        let profile = match meta.profile.as_str() {
            "development" => DeploymentProfile::Development,
            "dependable-local" => DeploymentProfile::DependableLocal,
            _ => return Err(ClusterError::CorruptMeta("unknown profile")),
        };
        let consistency_mode = ConsistencyMode::parse(&meta.consistency_mode)
            .ok_or(ClusterError::CorruptMeta("unknown consistency mode"))?;
        let partition_map = PartitionMap {
            virtual_partitions: meta.virtual_partitions,
            hash_profile: meta.hash_profile,
        };
        let directory = match PartitionDirectory::load(&root)? {
            Some(d) => d,
            None => {
                let d = PartitionDirectory::balanced(
                    partition_map.clone(),
                    meta.node_count,
                    PlacementEpoch(meta.placement_epoch),
                );
                d.save(&root)?;
                d
            }
        };

        let mut nodes = HashMap::new();
        for i in 0..meta.node_count {
            let path = node_store_path(&root, i);
            if path.exists() {
                let store = Store::open(&path)?;
                nodes.insert(i, store);
            }
        }

        let raft = Self::build_raft_groups(&directory);

        Ok(Self {
            root,
            cluster_id,
            profile,
            consistency_mode,
            partition_map,
            directory,
            nodes,
            offline: Vec::new(),
            raft,
            convergent_pos: HashMap::new(),
            rebalance_jobs: HashMap::new(),
        })
    }

    fn build_raft_groups(directory: &PartitionDirectory) -> HashMap<u32, PartitionRaft> {
        let mut raft = HashMap::new();
        for a in &directory.assignments {
            raft.insert(
                a.partition.get(),
                PartitionRaft::new(a.partition, a.replicas.clone(), a.placement_epoch),
            );
        }
        raft
    }

    /// Cluster root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Cluster identifier.
    pub fn cluster_id(&self) -> ClusterId {
        self.cluster_id
    }

    /// Deployment profile.
    pub fn profile(&self) -> DeploymentProfile {
        self.profile
    }

    /// Consistency mode.
    pub fn consistency_mode(&self) -> ConsistencyMode {
        self.consistency_mode
    }

    /// Partition map.
    pub fn partition_map(&self) -> &PartitionMap {
        &self.partition_map
    }

    /// Placement directory (derived routing; leader fields refreshed by Raft).
    pub fn directory(&self) -> &PartitionDirectory {
        &self.directory
    }

    /// Raft group for a partition (Stage 8b diagnostics / tests).
    pub fn raft_group(&self, partition: PartitionId) -> Option<&PartitionRaft> {
        self.raft.get(&partition.get())
    }

    /// Whether this profile can claim replicated durability.
    pub fn replicated_durability_available(&self) -> bool {
        self.profile != DeploymentProfile::Development && self.online_node_count() >= 2
    }

    /// Write quorum for the configured profile (floor(N/2)+1).
    pub fn write_quorum(&self) -> u32 {
        self.profile.write_quorum()
    }

    /// Number of nodes currently online.
    pub fn online_node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    /// Online node ids (sorted).
    pub fn online_nodes(&self) -> Vec<NodeId> {
        let mut v: Vec<NodeId> = self.nodes.keys().copied().map(NodeId::new).collect();
        v.sort();
        v
    }

    /// Path of a node store directory (exists even if offline).
    pub fn node_path(&self, node: NodeId) -> PathBuf {
        node_store_path(&self.root, node.index())
    }

    /// Mark a node offline (simulates failure; CLUSTER_SPEC §15 tests).
    pub fn mark_offline(&mut self, node: NodeId) -> Result<(), ClusterError> {
        let idx = node.index();
        // Drop the open store handle so the directory is idle for salvage.
        self.nodes.remove(&idx);
        if !self.offline.contains(&idx) {
            self.offline.push(idx);
            self.offline.sort();
        }
        // Step down any Raft leadership held by this node.
        for group in self.raft.values_mut() {
            if let Some((leader, _)) = group.current_leader() {
                if leader == node {
                    if let Some(p) = group.peer_mut(node) {
                        p.role = crate::raft::RaftRole::Follower;
                    }
                }
            }
        }
        Ok(())
    }

    /// Bring a previously offline node back online by reopening its store.
    pub fn mark_online(&mut self, node: NodeId) -> Result<(), ClusterError> {
        let idx = node.index();
        self.offline.retain(|&x| x != idx);
        if !self.nodes.contains_key(&idx) {
            let path = node_store_path(&self.root, idx);
            let store = Store::open(&path)?;
            self.nodes.insert(idx, store);
        }
        Ok(())
    }

    /// Whether a node is currently online.
    pub fn is_online(&self, node: NodeId) -> bool {
        self.nodes.contains_key(&node.index())
    }

    /// Resolve the virtual partition for a subject (default partition key).
    pub fn partition_for_subject(&self, subject: &str) -> PartitionId {
        self.partition_map
            .partition_of(default_partition_key(subject))
    }

    /// Ensure a live Raft leader for `partition`, updating the placement directory.
    pub fn ensure_partition_leader(
        &mut self,
        partition: PartitionId,
    ) -> Result<(NodeId, Term), ClusterError> {
        let online = self.online_nodes();
        let group = self
            .raft
            .get_mut(&partition.get())
            .ok_or(ClusterError::NoLeader(partition.get()))?;

        let (leader, term) = group.ensure_leader(&online).map_err(|e| match e {
            ElectError::NoQuorum { .. }
            | ElectError::NoOnlineVoters
            | ElectError::CandidateOffline => ClusterError::PartitionUnavailable {
                partition: partition.get(),
                reason: "no leader elected (quorum unavailable)",
            },
            ElectError::NotAVoter => ClusterError::NoLeader(partition.get()),
            ElectError::HigherTerm(_) => ClusterError::PartitionUnavailable {
                partition: partition.get(),
                reason: "election observed higher term",
            },
        })?;

        self.directory.set_leader(partition, leader, term);
        Ok((leader, term))
    }

    /// Put a subject on its partition.
    ///
    /// - `partition-linearizable`: Raft quorum commit + store apply.
    /// - `convergent-append`: any online replica set may accept (no quorum);
    ///   dual-accept across a network split is allowed (CLUSTER_SPEC §9.2).
    pub fn put(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
    ) -> Result<ClusterWriteAck, ClusterError> {
        self.write_event(subject, value, mode, false)
    }

    /// Delete a subject (tombstone) with the same routing/replication path.
    pub fn delete(
        &mut self,
        subject: &str,
        mode: DurabilityMode,
    ) -> Result<ClusterWriteAck, ClusterError> {
        self.write_event(subject, &[], mode, true)
    }

    /// Accept a convergent append on one specific online node (ingest endpoint).
    ///
    /// Used for dual-accept tests and direct routing to a known ingest node.
    /// Requires [`ConsistencyMode::ConvergentAppend`]. Does not require quorum
    /// or Raft leadership.
    pub fn append_local(
        &mut self,
        node: NodeId,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
    ) -> Result<ClusterWriteAck, ClusterError> {
        if self.consistency_mode != ConsistencyMode::ConvergentAppend {
            return Err(ClusterError::ConsistencyViolation(
                "append_local requires convergent-append mode".into(),
            ));
        }
        if !self.is_online(node) {
            return Err(ClusterError::PartitionUnavailable {
                partition: self.partition_for_subject(subject).get(),
                reason: "target ingest node offline",
            });
        }
        self.write_convergent_on(&[node], subject, value, mode, false)
    }

    /// Reconcile convergent state across all currently online replicas.
    ///
    /// Copies missing `(subject, body)` pairs by content identity. When the same
    /// subject has different live bodies on different nodes, both variants are
    /// retained on each participant (history keeps both; live value is not
    /// silently chosen) and the conflict is reported (CLUSTER_SPEC §9.2, §15.2).
    pub fn reconcile(&mut self) -> Result<ReconcileReport, ClusterError> {
        if self.consistency_mode != ConsistencyMode::ConvergentAppend {
            return Err(ClusterError::ConsistencyViolation(
                "reconcile is for convergent-append mode".into(),
            ));
        }

        let participants = self.online_nodes();
        let mut report = ReconcileReport {
            events_replicated: 0,
            conflicts: Vec::new(),
            participants: participants.clone(),
        };

        // Collect live (subject -> (node, body)) from every online node.
        // Balanced placement: every node may hold every subject.
        let mut by_subject: HashMap<String, Vec<(NodeId, Vec<u8>)>> = HashMap::new();
        for node in &participants {
            let store = match self.nodes.get(&node.index()) {
                Some(s) => s,
                None => continue,
            };
            for (subj_bytes, _body) in store.live_entries() {
                let Ok(subject) = std::str::from_utf8(subj_bytes) else {
                    continue;
                };
                let live = match store.get(subject) {
                    Ok(Some(v)) => v,
                    Ok(None) => continue,
                    Err(dingo_store::StoreError::PayloadPartial) => continue,
                    Err(e) => return Err(e.into()),
                };
                by_subject
                    .entry(subject.to_string())
                    .or_default()
                    .push((*node, live));
            }
        }

        // Detect conflicts: same subject, differing bodies.
        let mut conflict_subjects: HashSet<String> = HashSet::new();
        for (subject, variants) in &by_subject {
            let mut hashes: HashSet<[u8; 32]> = HashSet::new();
            for (_, body) in variants {
                hashes.insert(body_content_hash(body));
            }
            if hashes.len() > 1 {
                conflict_subjects.insert(subject.clone());
                let partition = self.partition_for_subject(subject);
                // Dedup variants by content hash for the report.
                let mut seen = HashSet::new();
                let mut unique = Vec::new();
                for (node, body) in variants {
                    let h = body_content_hash(body);
                    if seen.insert(h) {
                        unique.push(SubjectVariant {
                            node: *node,
                            body: body.clone(),
                            content_hash: h,
                        });
                    }
                }
                report.conflicts.push(SubjectConflict {
                    subject: subject.clone(),
                    partition,
                    variants: unique,
                });
            }
        }

        // Fan-out: for each distinct (subject, body) seen, ensure every online
        // node that is a replica of the partition holds that body in history.
        // For non-conflicts, missing keys get a put. For conflicts, put each
        // missing variant onto nodes that lack that content so both sides
        // survive (history) without inventing a single live winner beyond the
        // last put — we put only onto nodes that do not already have that hash
        // as their live value *and* lack it in a simple live check.
        //
        // Practical rule: if node N's live body hash != H for variant H of
        // subject S, append variant H onto N (store put). This preserves both
        // event_ids in history; the final live value is the last put applied
        // (deterministic: sort by content hash then apply).
        for (subject, variants) in &by_subject {
            let partition = self.partition_for_subject(subject);
            let Some(assignment) = self.directory.get(partition).cloned() else {
                continue;
            };

            // Unique bodies for this subject.
            let mut unique_bodies: Vec<Vec<u8>> = Vec::new();
            let mut seen_h = HashSet::new();
            for (_, body) in variants {
                let h = body_content_hash(body);
                if seen_h.insert(h) {
                    unique_bodies.push(body.clone());
                }
            }
            // Deterministic order for conflict multi-put.
            unique_bodies.sort_by(|a, b| body_content_hash(a).cmp(&body_content_hash(b)));

            let is_conflict = conflict_subjects.contains(subject);

            for node in &participants {
                if !assignment.replicas.contains(node) {
                    continue;
                }
                let store = match self.nodes.get_mut(&node.index()) {
                    Some(s) => s,
                    None => continue,
                };
                let current = store.get(subject)?;
                let current_hash = current.as_ref().map(|b| body_content_hash(b));

                if !is_conflict {
                    // Single variant: copy if missing or different (should not differ).
                    let body = &unique_bodies[0];
                    let h = body_content_hash(body);
                    if current_hash != Some(h) {
                        store.put(subject, body, DurabilityMode::Durable)?;
                        report.events_replicated += 1;
                    }
                } else {
                    // Conflict: ensure every variant is applied so history holds
                    // both; final live is last in hash-sorted order (explicit,
                    // not silent "newest wall clock wins").
                    for body in &unique_bodies {
                        let h = body_content_hash(body);
                        // Skip if already live with this body.
                        if current_hash == Some(h) {
                            // Still may need other variants in history — put others only.
                            continue;
                        }
                        // Always append other variants so history has both.
                        store.put(subject, body, DurabilityMode::Durable)?;
                        report.events_replicated += 1;
                    }
                }
            }
        }

        report.conflicts.sort_by(|a, b| a.subject.cmp(&b.subject));
        Ok(report)
    }

    fn write_event(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
        is_delete: bool,
    ) -> Result<ClusterWriteAck, ClusterError> {
        if self.consistency_mode == ConsistencyMode::ConvergentAppend {
            return self.write_convergent(subject, value, mode, is_delete);
        }
        self.write_linearizable(subject, value, mode, is_delete)
    }

    /// Convergent-append: accept on all currently online replicas of the
    /// partition. No Raft, no quorum — minority and split sides may accept.
    fn write_convergent(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
        is_delete: bool,
    ) -> Result<ClusterWriteAck, ClusterError> {
        let partition = self.partition_for_subject(subject);
        let assignment = self
            .directory
            .get(partition)
            .cloned()
            .ok_or(ClusterError::NoLeader(partition.get()))?;

        let targets: Vec<NodeId> = assignment
            .replicas
            .iter()
            .copied()
            .filter(|n| self.is_online(*n))
            .collect();
        if targets.is_empty() {
            return Err(ClusterError::PartitionUnavailable {
                partition: partition.get(),
                reason: "no online replica for convergent append",
            });
        }
        self.write_convergent_on(&targets, subject, value, mode, is_delete)
    }

    fn write_convergent_on(
        &mut self,
        targets: &[NodeId],
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
        is_delete: bool,
    ) -> Result<ClusterWriteAck, ClusterError> {
        let partition = self.partition_for_subject(subject);
        let assignment = self
            .directory
            .get(partition)
            .cloned()
            .ok_or(ClusterError::NoLeader(partition.get()))?;
        let epoch = assignment.placement_epoch;

        if targets.is_empty() {
            return Err(ClusterError::PartitionUnavailable {
                partition: partition.get(),
                reason: "no ingest targets",
            });
        }
        for n in targets {
            if !self.is_online(*n) {
                return Err(ClusterError::PartitionUnavailable {
                    partition: partition.get(),
                    reason: "ingest target offline",
                });
            }
            if !assignment.replicas.contains(n) {
                return Err(ClusterError::ReplicationRejected(format!(
                    "node {n} is not a replica of partition {}",
                    partition.get()
                )));
            }
        }

        let pos = {
            let e = self.convergent_pos.entry(partition.get()).or_insert(0);
            *e += 1;
            LogPosition(*e)
        };

        let ingest = targets[0];
        let mut first_receipt = None;
        let mut acks = 0u32;

        for node in targets {
            let store =
                self.nodes
                    .get_mut(&node.index())
                    .ok_or(ClusterError::PartitionUnavailable {
                        partition: partition.get(),
                        reason: "ingest store missing",
                    })?;
            let receipt = if is_delete {
                store.delete(subject, mode)?
            } else {
                store.put(subject, value, mode)?
            };
            acks += 1;
            if first_receipt.is_none() {
                first_receipt = Some(receipt);
            }
        }

        let receipt = first_receipt.expect("at least one target");

        // Convergent accepts are local/durable on ingest nodes but do not claim
        // linearizable commitment (CLUSTER_SPEC §9.2).
        Ok(ClusterWriteAck {
            cluster_id: self.cluster_id,
            consistency_mode: ConsistencyMode::ConvergentAppend,
            durability_mode: mode,
            partition,
            term: Term(0),
            position: pos,
            placement_epoch: epoch,
            replica_acks: acks,
            committed: false,
            commit_status: CommitStatus::Prepared,
            leader: ingest,
            event_id: receipt.event_id,
            store_id: receipt.store_id,
            segment_id: receipt.segment_id,
        })
    }

    fn write_linearizable(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
        is_delete: bool,
    ) -> Result<ClusterWriteAck, ClusterError> {
        let partition = self.partition_for_subject(subject);
        let assignment = self
            .directory
            .get(partition)
            .cloned()
            .ok_or(ClusterError::NoLeader(partition.get()))?;
        let epoch = assignment.placement_epoch;

        let (leader, _term) = self.ensure_partition_leader(partition)?;

        let command = if is_delete {
            LogCommand::Delete {
                subject: subject.to_string(),
            }
        } else {
            LogCommand::Put {
                subject: subject.to_string(),
                value: value.to_vec(),
            }
        };

        let online = self.online_nodes();
        let propose = {
            let group = self
                .raft
                .get_mut(&partition.get())
                .ok_or(ClusterError::NoLeader(partition.get()))?;
            group
                .propose(leader, command, &online)
                .map_err(|e| match e {
                    ProposeError::NotLeader => ClusterError::NoLeader(partition.get()),
                    ProposeError::SteppedDown(_) => ClusterError::PartitionUnavailable {
                        partition: partition.get(),
                        reason: "leader stepped down during propose",
                    },
                })?
        };

        if !propose.committed {
            return Err(ClusterError::DurabilityUnavailable(format!(
                "quorum not reached: got {} of {} for partition {}",
                propose.replica_acks,
                self.write_quorum(),
                partition.get()
            )));
        }

        // Apply committed entries to every online replica that has them.
        let mut leader_receipt = None;
        for node in online {
            let batch = self
                .raft
                .get_mut(&partition.get())
                .map(|g| {
                    g.sync_follower_commit(leader, node);
                    g.take_apply_batch(node)
                })
                .unwrap_or_default();
            if batch.is_empty() {
                continue;
            }
            let store = match self.nodes.get_mut(&node.index()) {
                Some(s) => s,
                None => continue,
            };
            for entry in batch {
                let receipt = match &entry.command {
                    LogCommand::Put { subject, value } => store.put(subject, value, mode)?,
                    LogCommand::Delete { subject } => store.delete(subject, mode)?,
                };
                if node == leader {
                    leader_receipt = Some(receipt);
                }
            }
        }

        let receipt = leader_receipt.ok_or(ClusterError::PartitionUnavailable {
            partition: partition.get(),
            reason: "leader apply missing receipt",
        })?;

        Ok(ClusterWriteAck {
            cluster_id: self.cluster_id,
            consistency_mode: self.consistency_mode,
            durability_mode: mode,
            partition,
            term: propose.term,
            position: propose.position,
            placement_epoch: epoch,
            replica_acks: propose.replica_acks,
            committed: propose.committed,
            commit_status: CommitStatus::Committed,
            leader,
            event_id: receipt.event_id,
            store_id: receipt.store_id,
            segment_id: receipt.segment_id,
        })
    }

    /// Get a subject under the given read mode, with coverage.
    pub fn get(&mut self, subject: &str, mode: ReadMode) -> Result<GetResult, ClusterError> {
        let partition = self.partition_for_subject(subject);
        let mut coverage = Coverage::single(partition);
        if !self.replicated_durability_available() {
            coverage.note("development profile: replicated durability unavailable");
        }
        if self.consistency_mode == ConsistencyMode::ConvergentAppend {
            coverage.note("convergent-append: not linearizable");
        }

        let assignment = match self.directory.get(partition) {
            Some(a) => a.clone(),
            None => {
                coverage.mark_unavailable(partition);
                return Ok(GetResult {
                    value: None,
                    coverage,
                    absence_proven: false,
                });
            }
        };

        match mode {
            ReadMode::Linearizable => {
                if self.consistency_mode == ConsistencyMode::ConvergentAppend {
                    return Err(ClusterError::ConsistencyViolation(
                        "linearizable reads are not available in convergent-append mode".into(),
                    ));
                }
                // Need a live leader (may re-elect after failure).
                let (leader, term) = match self.ensure_partition_leader(partition) {
                    Ok(lt) => lt,
                    Err(e) => {
                        coverage.mark_unavailable(partition);
                        return Err(e);
                    }
                };
                let store =
                    self.nodes
                        .get(&leader.index())
                        .ok_or(ClusterError::PartitionUnavailable {
                            partition: partition.get(),
                            reason: "leader handle missing",
                        })?;
                let value = store.get(subject)?;
                let pos = self
                    .raft
                    .get(&partition.get())
                    .map(|g| LogPosition(g.max_commit_index()))
                    .unwrap_or(LogPosition(0));
                coverage.mark_completed(partition, term, pos, Some(leader.index()));
                Ok(GetResult {
                    value,
                    coverage,
                    absence_proven: true,
                })
            }
            ReadMode::Available | ReadMode::Salvage => {
                // Prefer current Raft leader, then any online replica.
                let mut order = Vec::new();
                if let Some((leader, _)) = self
                    .raft
                    .get(&partition.get())
                    .and_then(|g| g.current_leader())
                {
                    order.push(leader);
                }
                order.push(assignment.leader);
                for r in &assignment.replicas {
                    if !order.contains(r) {
                        order.push(*r);
                    }
                }
                for node in order {
                    if !self.is_online(node) {
                        continue;
                    }
                    let store = match self.nodes.get(&node.index()) {
                        Some(s) => s,
                        None => continue,
                    };
                    match store.get(subject) {
                        Ok(value) => {
                            let (term, pos) = self
                                .raft
                                .get(&partition.get())
                                .map(|g| {
                                    let term =
                                        g.current_leader().map(|(_, t)| t).unwrap_or(Term(0));
                                    (term, LogPosition(g.max_commit_index()))
                                })
                                .unwrap_or((Term(0), LogPosition(0)));
                            coverage.mark_completed(partition, term, pos, Some(node.index()));
                            return Ok(GetResult {
                                value,
                                coverage,
                                absence_proven: false,
                            });
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                coverage.mark_unavailable(partition);
                Ok(GetResult {
                    value: None,
                    coverage,
                    absence_proven: false,
                })
            }
        }
    }

    /// Scan all live subjects across partitions that are currently reachable.
    ///
    /// Incomplete coverage is always reported; never claims an offline partition
    /// as an empty success (CLUSTER_SPEC §6.7).
    pub fn scan_all(&mut self) -> Result<ScanResult, ClusterError> {
        let find = self.scan_with(ScanOptions::default())?;
        Ok(ScanResult {
            entries: find.entries,
            coverage: find.coverage,
        })
    }

    /// Distributed scan/find with coverage (CLUSTER_SPEC §17, Stage 8e).
    ///
    /// Returns matching subjects from completed partitions only. Always attach
    /// honest coverage: unavailable partitions are listed, never treated as
    /// empty success. Resource budgets set `coverage.resource_limit_reached`.
    pub fn scan_with(&mut self, options: ScanOptions) -> Result<FindResult, ClusterError> {
        let requested: Vec<PartitionId> = match &options.partitions {
            Some(p) => {
                let mut v = p.clone();
                v.sort();
                v.dedup();
                v
            }
            None => self.partition_map.all_partitions().collect(),
        };
        let query_id =
            FindResult::make_query_id(&requested, options.subject_prefix.as_deref(), options.limit);

        let mut coverage = Coverage::for_partitions(requested.iter().copied());
        coverage.with_read_mode(options.read_mode);
        if !self.replicated_durability_available() {
            coverage.note("development profile: replicated durability unavailable");
        }

        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
        let mut seen_subjects = HashSet::new();
        let mut docs_scanned = 0usize;
        let mut budget_stopped = false;

        for partition in requested {
            if budget_stopped {
                // Remaining requested partitions are incomplete due to budget.
                if !coverage.completed.contains(&partition)
                    && !coverage.unavailable.contains(&partition)
                {
                    coverage.note(format!(
                        "partition {} not examined (resource budget)",
                        partition.get()
                    ));
                }
                continue;
            }

            let Some(assignment) = self.directory.get(partition).cloned() else {
                coverage.mark_unavailable(partition);
                continue;
            };

            let mut batch = Vec::new();
            let served = match self.contact_partition(
                partition,
                &assignment,
                options.read_mode,
                &mut batch,
                &mut seen_subjects,
                options.subject_prefix.as_deref(),
                &mut docs_scanned,
                options.max_docs_scanned,
            )? {
                ContactOutcome::Served { term, pos, node } => {
                    coverage.mark_completed(partition, term, pos, Some(node.index()));
                    true
                }
                ContactOutcome::BudgetExhausted { term, pos, node } => {
                    coverage.mark_completed(partition, term, pos, Some(node.index()));
                    coverage.mark_resource_limit(format!(
                        "max_docs_scanned budget reached after {docs_scanned} subjects"
                    ));
                    budget_stopped = true;
                    true
                }
                ContactOutcome::Unavailable => false,
            };

            if !served {
                coverage.mark_unavailable(partition);
            } else {
                entries.extend(batch);
            }
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut truncated = false;
        if let Some(limit) = options.limit {
            if entries.len() > limit {
                entries.truncate(limit);
                truncated = true;
                coverage.note(format!("result truncated to limit {limit}"));
            }
        }

        Ok(FindResult {
            entries,
            coverage,
            query_id,
            truncated,
        })
    }

    /// Convenience find: subjects matching optional prefix under scan options.
    pub fn find(&mut self, options: ScanOptions) -> Result<FindResult, ClusterError> {
        self.scan_with(options)
    }

    /// Begin an interruptible rebalance of one partition to `new_replicas`
    /// (CLUSTER_SPEC §14, Stage 8f). Advances only to `PlanCommitted`.
    pub fn begin_rebalance(
        &mut self,
        partition: PartitionId,
        new_replicas: Vec<NodeId>,
    ) -> Result<RebalanceJob, ClusterError> {
        if new_replicas.is_empty() {
            return Err(ClusterError::Rebalance(
                "new replica set must be non-empty".into(),
            ));
        }
        // Destinations must exist as node stores (online or offline on disk).
        for n in &new_replicas {
            let path = self.node_path(*n);
            if !path.exists() && !self.is_online(*n) {
                return Err(ClusterError::Rebalance(format!(
                    "destination node {} has no store path",
                    n.index()
                )));
            }
        }
        if self.rebalance_jobs.contains_key(&partition.get()) {
            return Err(ClusterError::Rebalance(format!(
                "rebalance already in progress for partition {}",
                partition.get()
            )));
        }
        let old = self
            .directory
            .get(partition)
            .ok_or_else(|| {
                ClusterError::Rebalance(format!("no assignment for partition {}", partition.get()))
            })?
            .replicas
            .clone();
        let plan_epoch = self.directory.placement_epoch;
        let job_id = format!(
            "rb-{}-{}",
            partition.get(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );
        let job = RebalanceJob::plan(job_id, partition, old, new_replicas, plan_epoch);
        self.rebalance_jobs.insert(partition.get(), job.clone());
        Ok(job)
    }

    /// Advance the in-flight rebalance for `partition` by one phase.
    pub fn advance_rebalance(
        &mut self,
        partition: PartitionId,
    ) -> Result<RebalanceJob, ClusterError> {
        let job = self
            .rebalance_jobs
            .get(&partition.get())
            .cloned()
            .ok_or_else(|| {
                ClusterError::Rebalance(format!(
                    "no in-flight rebalance for partition {}",
                    partition.get()
                ))
            })?;
        if job.phase == RebalancePhase::Reclaimed {
            return Err(ClusterError::Rebalance("rebalance already complete".into()));
        }
        let next = job
            .phase
            .next()
            .ok_or_else(|| ClusterError::Rebalance("no next phase".into()))?;

        let updated = match next {
            RebalancePhase::LearnersAdded => self.rb_learners_added(job)?,
            RebalancePhase::SegmentsCopied => self.rb_segments_copied(job)?,
            RebalancePhase::LogCaughtUp => self.rb_log_caught_up(job)?,
            RebalancePhase::MembershipChanged => self.rb_membership_changed(job)?,
            RebalancePhase::EpochActivated => self.rb_epoch_activated(job)?,
            RebalancePhase::SafetyWindow => self.rb_safety_window(job)?,
            RebalancePhase::Reclaimed => self.rb_reclaimed(job)?,
            RebalancePhase::PlanCommitted => {
                return Err(ClusterError::Rebalance("cannot re-enter plan".into()));
            }
        };
        self.rebalance_jobs.insert(partition.get(), updated.clone());
        if updated.phase == RebalancePhase::Reclaimed {
            self.rebalance_jobs.remove(&partition.get());
        }
        Ok(updated)
    }

    /// Run all remaining rebalance steps to completion.
    pub fn rebalance_partition(
        &mut self,
        partition: PartitionId,
        new_replicas: Vec<NodeId>,
    ) -> Result<RebalanceReport, ClusterError> {
        let mut phases = vec![RebalancePhase::PlanCommitted];
        let mut job = self.begin_rebalance(partition, new_replicas)?;
        while job.phase != RebalancePhase::Reclaimed {
            job = self.advance_rebalance(partition)?;
            phases.push(job.phase);
        }
        Ok(RebalanceReport {
            job,
            phases_completed: phases,
        })
    }

    /// Current in-flight rebalance job for a partition, if any.
    pub fn rebalance_job(&self, partition: PartitionId) -> Option<&RebalanceJob> {
        self.rebalance_jobs.get(&partition.get())
    }

    /// Reconstruct a placement directory from live node stores after control-
    /// plane loss (CLUSTER_SPEC §12.3 / §22 item 10, simplified).
    ///
    /// Scans every online node for subjects, maps them to partitions, and
    /// builds a balanced-style directory with those nodes as replicas. Does
    /// not invent commitment evidence.
    pub fn reconstruct_directory_from_stores(
        &mut self,
    ) -> Result<PartitionDirectory, ClusterError> {
        let online = self.online_nodes();
        if online.is_empty() {
            return Err(ClusterError::Rebalance(
                "cannot reconstruct directory with no online nodes".into(),
            ));
        }
        let epoch = PlacementEpoch(self.directory.placement_epoch.0 + 1);
        let mut directory = PartitionDirectory::balanced(
            self.partition_map.clone(),
            online.iter().map(|n| n.index()).max().unwrap_or(0) + 1,
            epoch,
        );
        // Restrict each partition's replicas to online nodes that currently
        // hold at least one subject for that partition (or all online if none).
        let mut holders: HashMap<u32, HashSet<u32>> = HashMap::new();
        for node in &online {
            let store = match self.nodes.get(&node.index()) {
                Some(s) => s,
                None => continue,
            };
            for (subj_bytes, _) in store.live_entries() {
                let Ok(subject) = std::str::from_utf8(subj_bytes) else {
                    continue;
                };
                let p = self.partition_for_subject(subject).get();
                holders.entry(p).or_default().insert(node.index());
            }
        }
        for a in &mut directory.assignments {
            let p = a.partition.get();
            if let Some(set) = holders.get(&p) {
                let mut reps: Vec<NodeId> = set.iter().copied().map(NodeId::new).collect();
                reps.sort();
                if !reps.is_empty() {
                    a.replicas = reps.clone();
                    a.leader = reps[0];
                }
            } else {
                // Empty partition: any online node may host it.
                a.replicas = online.clone();
                a.leader = online[0];
            }
            a.placement_epoch = epoch;
        }
        directory.placement_epoch = epoch;
        self.directory = directory.clone();
        self.raft = Self::build_raft_groups(&self.directory);
        self.directory.save(&self.root)?;
        Ok(directory)
    }

    fn rb_learners_added(&mut self, mut job: RebalanceJob) -> Result<RebalanceJob, ClusterError> {
        // Register peer slots for destinations (non-voting until membership).
        if let Some(group) = self.raft.get_mut(&job.partition.get()) {
            for d in &job.destinations {
                group.ensure_peer(*d);
            }
        }
        // Ensure destination stores are open when on disk.
        for d in &job.destinations {
            if !self.is_online(*d) {
                let path = self.node_path(*d);
                if path.exists() {
                    let store = Store::open(&path)?;
                    self.nodes.insert(d.index(), store);
                    self.offline.retain(|&x| x != d.index());
                } else {
                    return Err(ClusterError::Rebalance(format!(
                        "learner destination {} offline with no store",
                        d.index()
                    )));
                }
            }
        }
        job.phase = RebalancePhase::LearnersAdded;
        Ok(job)
    }

    fn rb_segments_copied(&mut self, mut job: RebalanceJob) -> Result<RebalanceJob, ClusterError> {
        // Pick a source: first online old replica with data.
        let source = job
            .old_replicas
            .iter()
            .copied()
            .find(|n| self.is_online(*n))
            .ok_or_else(|| {
                ClusterError::Rebalance(
                    "no online source replica during segment copy (CLUSTER_SPEC §22.13)".into(),
                )
            })?;

        let subjects: Vec<(String, Vec<u8>)> = {
            let store = self
                .nodes
                .get(&source.index())
                .ok_or_else(|| ClusterError::Rebalance("source store missing".into()))?;
            let mut out = Vec::new();
            for (subj_bytes, _) in store.live_entries() {
                let Ok(subject) = std::str::from_utf8(subj_bytes) else {
                    continue;
                };
                if self.partition_for_subject(subject) != job.partition {
                    continue;
                }
                if let Ok(Some(body)) = store.get(subject) {
                    out.push((subject.to_string(), body));
                }
            }
            out
        };

        for dest in &job.destinations {
            if !self.is_online(*dest) {
                return Err(ClusterError::Rebalance(format!(
                    "destination {} offline during segment copy",
                    dest.index()
                )));
            }
            let store = self
                .nodes
                .get_mut(&dest.index())
                .ok_or_else(|| ClusterError::Rebalance("dest store missing".into()))?;
            for (subject, body) in &subjects {
                store.put(subject, body, DurabilityMode::Durable)?;
                job.subjects_copied += 1;
            }
        }
        job.phase = RebalancePhase::SegmentsCopied;
        Ok(job)
    }

    fn rb_log_caught_up(&mut self, mut job: RebalanceJob) -> Result<RebalanceJob, ClusterError> {
        let group = self
            .raft
            .get_mut(&job.partition.get())
            .ok_or_else(|| ClusterError::Rebalance("raft group missing".into()))?;
        let leader = group
            .current_leader()
            .map(|(n, _)| n)
            .or_else(|| {
                job.old_replicas
                    .iter()
                    .copied()
                    .find(|n| group.peer(*n).is_some())
            })
            .ok_or_else(|| ClusterError::Rebalance("no leader/source for log catch-up".into()))?;
        for dest in &job.destinations {
            let n = group.stream_log_to(leader, *dest);
            job.log_entries_streamed += n;
        }
        job.phase = RebalancePhase::LogCaughtUp;
        Ok(job)
    }

    fn rb_membership_changed(
        &mut self,
        mut job: RebalanceJob,
    ) -> Result<RebalanceJob, ClusterError> {
        // Joint configuration: old ∪ new.
        let mut joint = job.old_replicas.clone();
        for n in &job.new_replicas {
            if !joint.contains(n) {
                joint.push(*n);
            }
        }
        joint.sort();
        let epoch = job.plan_epoch;
        if let Some(group) = self.raft.get_mut(&job.partition.get()) {
            group.set_voters(joint, epoch);
        }
        job.joint = true;
        job.phase = RebalancePhase::MembershipChanged;
        Ok(job)
    }

    fn rb_epoch_activated(&mut self, mut job: RebalanceJob) -> Result<RebalanceJob, ClusterError> {
        let new_epoch = PlacementEpoch(job.plan_epoch.0 + 1);
        if let Some(group) = self.raft.get_mut(&job.partition.get()) {
            group.set_voters(job.new_replicas.clone(), new_epoch);
        }
        self.directory
            .set_replicas(job.partition, job.new_replicas.clone(), new_epoch);
        // Prefer a live leader from the new set.
        let _ = self.ensure_partition_leader(job.partition);
        self.directory.save(&self.root)?;
        // Bump meta epoch for open() compatibility.
        if let Ok(mut meta) = ClusterMeta::load(&self.root) {
            meta.placement_epoch = new_epoch.0;
            meta.format = ClusterMeta::FORMAT.to_string();
            let _ = meta.write(&self.root);
        }
        job.activated_epoch = Some(new_epoch);
        job.joint = false;
        job.phase = RebalancePhase::EpochActivated;
        Ok(job)
    }

    fn rb_safety_window(&mut self, mut job: RebalanceJob) -> Result<RebalanceJob, ClusterError> {
        // Old replicas retain data; nothing reclaimed yet.
        job.phase = RebalancePhase::SafetyWindow;
        Ok(job)
    }

    fn rb_reclaimed(&mut self, mut job: RebalanceJob) -> Result<RebalanceJob, ClusterError> {
        // Soft reclaim: note only. Physical purge of partition subjects on
        // removed nodes is optional and left for ops; we do not invent deletes
        // that would destroy salvage evidence.
        if !job.removals.is_empty() {
            // no-op data plane; placement already excludes them
        }
        job.phase = RebalancePhase::Reclaimed;
        Ok(job)
    }

    fn contact_partition(
        &mut self,
        partition: PartitionId,
        assignment: &crate::directory::PartitionAssignment,
        read_mode: ReadMode,
        entries: &mut Vec<(String, Vec<u8>)>,
        seen: &mut HashSet<String>,
        prefix: Option<&str>,
        docs_scanned: &mut usize,
        max_docs: Option<usize>,
    ) -> Result<ContactOutcome, ClusterError> {
        // Linearizable prefers electable leader; available/salvage any replica.
        if read_mode == ReadMode::Linearizable
            && self.consistency_mode != ConsistencyMode::ConvergentAppend
        {
            if let Ok((leader, term)) = self.ensure_partition_leader(partition) {
                if let Some(store) = self.nodes.get(&leader.index()) {
                    let hit_budget = self.collect_partition_entries(
                        store,
                        partition,
                        entries,
                        seen,
                        prefix,
                        docs_scanned,
                        max_docs,
                    )?;
                    let pos = self
                        .raft
                        .get(&partition.get())
                        .map(|g| LogPosition(g.max_commit_index()))
                        .unwrap_or(LogPosition(0));
                    return Ok(if hit_budget {
                        ContactOutcome::BudgetExhausted {
                            term,
                            pos,
                            node: leader,
                        }
                    } else {
                        ContactOutcome::Served {
                            term,
                            pos,
                            node: leader,
                        }
                    });
                }
            }
        }

        for r in &assignment.replicas {
            if !self.is_online(*r) {
                continue;
            }
            if let Some(store) = self.nodes.get(&r.index()) {
                let hit_budget = self.collect_partition_entries(
                    store,
                    partition,
                    entries,
                    seen,
                    prefix,
                    docs_scanned,
                    max_docs,
                )?;
                let (term, pos) = self
                    .raft
                    .get(&partition.get())
                    .map(|g| {
                        (
                            g.current_leader()
                                .map(|(_, t)| t)
                                .unwrap_or(assignment.term),
                            LogPosition(g.max_commit_index()),
                        )
                    })
                    .unwrap_or((assignment.term, LogPosition(0)));
                return Ok(if hit_budget {
                    ContactOutcome::BudgetExhausted {
                        term,
                        pos,
                        node: *r,
                    }
                } else {
                    ContactOutcome::Served {
                        term,
                        pos,
                        node: *r,
                    }
                });
            }
        }
        Ok(ContactOutcome::Unavailable)
    }

    /// Collect live entries for one partition. Returns true if a docs budget stopped early.
    fn collect_partition_entries(
        &self,
        store: &Store,
        partition: PartitionId,
        entries: &mut Vec<(String, Vec<u8>)>,
        seen: &mut HashSet<String>,
        prefix: Option<&str>,
        docs_scanned: &mut usize,
        max_docs: Option<usize>,
    ) -> Result<bool, ClusterError> {
        for (subj_bytes, body) in store.live_entries() {
            let Ok(subject) = std::str::from_utf8(subj_bytes) else {
                continue;
            };
            if self.partition_for_subject(subject) != partition {
                continue;
            }
            if let Some(p) = prefix {
                if !subject.starts_with(p) {
                    continue;
                }
            }
            if !seen.insert(subject.to_string()) {
                continue;
            }
            *docs_scanned += 1;
            if let Some(max) = max_docs {
                if *docs_scanned > max {
                    return Ok(true);
                }
            }
            match store.get(subject) {
                Ok(Some(v)) => entries.push((subject.to_string(), v)),
                Ok(None) => {}
                Err(dingo_store::StoreError::PayloadPartial) => {
                    let _ = body;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(false)
    }

    /// Per-subject history from the current partition leader's store.
    ///
    /// Used by the SDK cluster backend (Stage 8d). Elects a leader when needed.
    pub fn subject_history(
        &mut self,
        subject: &str,
    ) -> Result<dingo_store::SubjectHistory, ClusterError> {
        let partition = self.partition_for_subject(subject);
        let (leader, _) = self.ensure_partition_leader(partition)?;
        let store = self
            .nodes
            .get(&leader.index())
            .ok_or(ClusterError::PartitionUnavailable {
                partition: partition.get(),
                reason: "leader handle missing",
            })?;
        Ok(store.history(subject)?)
    }

    /// Run ordinary store salvage on one node without using cluster metadata.
    ///
    /// Proves CLUSTER_SPEC §6.1 / §22 item 19: node salvage without cluster
    /// software yields ordinary segments.
    pub fn salvage_node(&self, node: NodeId) -> Result<SalvageReport, ClusterError> {
        let path = self.node_path(node);
        if let Some(store) = self.nodes.get(&node.index()) {
            return Ok(store.salvage()?);
        }
        let store = Store::open_inspect(&path)?;
        Ok(store.salvage()?)
    }

    /// Salvage a node store path with **no** cluster handle (standalone API).
    pub fn salvage_node_path(path: impl AsRef<Path>) -> Result<SalvageReport, ClusterError> {
        let store = Store::open_inspect(path)?;
        Ok(store.salvage()?)
    }
}

enum ContactOutcome {
    Served {
        term: Term,
        pos: LogPosition,
        node: NodeId,
    },
    BudgetExhausted {
        term: Term,
        pos: LogPosition,
        node: NodeId,
    },
    Unavailable,
}
