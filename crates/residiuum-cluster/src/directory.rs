//! Partition directory / placement (CLUSTER_SPEC §5, §12, §13).

use crate::id::{NodeId, PartitionId, PlacementEpoch, Term};
use crate::partition::PartitionMap;
use serde::{Deserialize, Serialize};

/// Replica-set assignment for one virtual partition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionAssignment {
    /// Virtual partition.
    pub partition: PartitionId,
    /// Voting replicas (order is not rank; leader is separate).
    pub replicas: Vec<NodeId>,
    /// Current leader / primary for partition-linearizable writes.
    pub leader: NodeId,
    /// Leadership term.
    pub term: Term,
    /// Placement generation under which this assignment is valid.
    pub placement_epoch: PlacementEpoch,
}

/// Derived routing map from partitions to replica sets (CLUSTER_SPEC §5).
///
/// Placement determines where data *should* be, not what data *is*. Loss of
/// this directory must not invalidate verified frames (§6.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionDirectory {
    /// Partition map generation parameters.
    pub map: PartitionMap,
    /// Placement epoch for the whole directory.
    pub placement_epoch: PlacementEpoch,
    /// Per-partition assignments (sorted by partition id).
    pub assignments: Vec<PartitionAssignment>,
}

impl PartitionDirectory {
    /// Build a default placement: every node is a replica of every partition;
    /// leader is `partition_id % node_count` for even spread.
    ///
    /// For the development profile (`node_count == 1`) every partition lives
    /// on node 0.
    pub fn balanced(map: PartitionMap, node_count: u32, placement_epoch: PlacementEpoch) -> Self {
        assert!(node_count >= 1);
        let replicas: Vec<NodeId> = (0..node_count).map(NodeId::new).collect();
        let mut assignments = Vec::with_capacity(map.virtual_partitions as usize);
        for p in map.all_partitions() {
            let leader = NodeId::new(p.get() % node_count);
            assignments.push(PartitionAssignment {
                partition: p,
                replicas: replicas.clone(),
                leader,
                term: Term(1),
                placement_epoch,
            });
        }
        Self {
            map,
            placement_epoch,
            assignments,
        }
    }

    /// Lookup assignment for a partition.
    pub fn get(&self, partition: PartitionId) -> Option<&PartitionAssignment> {
        self.assignments
            .binary_search_by_key(&partition, |a| a.partition)
            .ok()
            .map(|i| &self.assignments[i])
    }

    /// Mutable lookup.
    pub fn get_mut(&mut self, partition: PartitionId) -> Option<&mut PartitionAssignment> {
        self.assignments
            .binary_search_by_key(&partition, |a| a.partition)
            .ok()
            .map(|i| &mut self.assignments[i])
    }

    /// Leader for a partition, if assigned.
    pub fn leader_of(&self, partition: PartitionId) -> Option<NodeId> {
        self.get(partition).map(|a| a.leader)
    }

    /// Bump term and reaffirm leader (Stage 8a static election helper).
    pub fn set_leader(&mut self, partition: PartitionId, leader: NodeId, new_term: Term) {
        if let Some(a) = self.get_mut(partition) {
            a.leader = leader;
            a.term = new_term;
        }
    }

    /// Partitions currently led by `node`.
    pub fn partitions_led_by(&self, node: NodeId) -> Vec<PartitionId> {
        self.assignments
            .iter()
            .filter(|a| a.leader == node)
            .map(|a| a.partition)
            .collect()
    }

    /// Replace the voting replica set for a partition (rebalance).
    ///
    /// If the current leader is not in `replicas`, the first replica becomes
    /// leader (term unchanged until Raft re-elects).
    pub fn set_replicas(
        &mut self,
        partition: PartitionId,
        replicas: Vec<NodeId>,
        placement_epoch: PlacementEpoch,
    ) {
        assert!(!replicas.is_empty(), "replica set must be non-empty");
        if let Some(a) = self.get_mut(partition) {
            if !replicas.contains(&a.leader) {
                a.leader = replicas[0];
            }
            a.replicas = replicas;
            a.placement_epoch = placement_epoch;
        }
        self.placement_epoch = placement_epoch;
    }

    /// Persist the directory under a cluster root (`placement.json`).
    ///
    /// Atomic durable replace with previous generation retained (DEF-021).
    pub fn save(&self, root: &std::path::Path) -> Result<(), crate::error::ClusterError> {
        let path = root.join("placement.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| crate::error::ClusterError::CorruptMeta("serialize placement.json"))?;
        residiuum_store::write_atomic_keep_previous(&path, json.as_bytes())?;
        Ok(())
    }

    /// Load a directory from `placement.json`, if present.
    ///
    /// Falls back to `placement.json.prev` when the primary is corrupt (DEF-021).
    pub fn load(root: &std::path::Path) -> Result<Option<Self>, crate::error::ClusterError> {
        let path = root.join("placement.json");
        if let Some(dir) = try_parse_placement(&path)? {
            return Ok(Some(dir));
        }
        let prev = residiuum_store::previous_path(&path);
        if let Some(dir) = try_parse_placement(&prev)? {
            return Ok(Some(dir));
        }
        if path.is_file() || prev.is_file() {
            return Err(crate::error::ClusterError::CorruptMeta(
                "placement.json unreadable; restore .prev or recreate cluster placement",
            ));
        }
        Ok(None)
    }
}

fn try_parse_placement(
    path: &std::path::Path,
) -> Result<Option<PartitionDirectory>, crate::error::ClusterError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    match serde_json::from_slice::<PartitionDirectory>(&bytes) {
        Ok(dir) => Ok(Some(dir)),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_three_nodes() {
        let dir = PartitionDirectory::balanced(PartitionMap::new(8), 3, PlacementEpoch(1));
        assert_eq!(dir.assignments.len(), 8);
        for a in &dir.assignments {
            assert_eq!(a.replicas.len(), 3);
            assert_eq!(a.leader, NodeId::new(a.partition.get() % 3));
        }
    }
}
