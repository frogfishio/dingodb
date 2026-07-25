//! Client-side partition directory cache (CLUSTER_SPEC §13, Stage 8d).
//!
//! Clients MAY cache partition → leader/replica routes and refresh on stale
//! placement epochs. A stale route may cost a redirect; it MUST NOT authorize
//! an obsolete writer (CLUSTER_SPEC §13).

use dingo_cluster::{
    NodeId, PartitionAssignment, PartitionDirectory, PartitionId, PartitionMap, PlacementEpoch,
    Term,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Snapshot of one partition route held by a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedRoute {
    /// Virtual partition.
    pub partition: PartitionId,
    /// Cached leader / primary for linearizable writes.
    pub leader: NodeId,
    /// Leadership term observed when this entry was cached.
    pub term: Term,
    /// Placement epoch for this assignment.
    pub placement_epoch: PlacementEpoch,
    /// Voting replicas (may be used for available reads / convergent ingest).
    pub replicas: Vec<NodeId>,
}

impl From<&PartitionAssignment> for CachedRoute {
    fn from(a: &PartitionAssignment) -> Self {
        Self {
            partition: a.partition,
            leader: a.leader,
            term: a.term,
            placement_epoch: a.placement_epoch,
            replicas: a.replicas.clone(),
        }
    }
}

/// Wire-friendly directory snapshot for RPC / multi-seed connect (Stage 8d).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectorySnapshot {
    /// Virtual partition count.
    pub virtual_partitions: u32,
    /// Published hash profile name.
    pub hash_profile: String,
    /// Whole-directory placement epoch.
    pub placement_epoch: u64,
    /// Per-partition assignments.
    pub assignments: Vec<AssignmentWire>,
    /// Optional node id → `host:port` endpoints for direct routing.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub endpoints: HashMap<u32, String>,
}

/// One assignment on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentWire {
    /// Partition index.
    pub partition: u32,
    /// Replica node indexes.
    pub replicas: Vec<u32>,
    /// Leader node index.
    pub leader: u32,
    /// Leadership term.
    pub term: u64,
    /// Placement epoch for this assignment.
    pub placement_epoch: u64,
}

impl DirectorySnapshot {
    /// Build from a live placement directory and optional endpoint map.
    pub fn from_directory(dir: &PartitionDirectory, endpoints: HashMap<u32, String>) -> Self {
        Self {
            virtual_partitions: dir.map.virtual_partitions,
            hash_profile: dir.map.hash_profile.clone(),
            placement_epoch: dir.placement_epoch.0,
            assignments: dir
                .assignments
                .iter()
                .map(|a| AssignmentWire {
                    partition: a.partition.get(),
                    replicas: a.replicas.iter().map(|n| n.index()).collect(),
                    leader: a.leader.index(),
                    term: a.term.0,
                    placement_epoch: a.placement_epoch.0,
                })
                .collect(),
            endpoints,
        }
    }

    /// Convert into a [`PartitionDirectory`] (endpoints are separate).
    pub fn to_directory(&self) -> PartitionDirectory {
        let map = PartitionMap {
            virtual_partitions: self.virtual_partitions,
            hash_profile: self.hash_profile.clone(),
        };
        let mut assignments: Vec<PartitionAssignment> = self
            .assignments
            .iter()
            .map(|a| PartitionAssignment {
                partition: PartitionId::new(a.partition),
                replicas: a.replicas.iter().copied().map(NodeId::new).collect(),
                leader: NodeId::new(a.leader),
                term: Term(a.term),
                placement_epoch: PlacementEpoch(a.placement_epoch),
            })
            .collect();
        assignments.sort_by_key(|a| a.partition);
        PartitionDirectory {
            map,
            placement_epoch: PlacementEpoch(self.placement_epoch),
            assignments,
        }
    }
}

/// Bounded client cache of partition routes (CLUSTER_SPEC §13).
///
/// Hot path: hash subject → partition → cached leader, then open a connection
/// to that node. On `stale_epoch` / misrouting, refresh the affected entry
/// (or the whole directory) and retry with the same event identity.
#[derive(Debug, Clone)]
pub struct ClientDirectoryCache {
    map: PartitionMap,
    placement_epoch: PlacementEpoch,
    /// partition index → route
    routes: HashMap<u32, CachedRoute>,
    /// node index → `host:port` (empty for in-process cluster handles)
    endpoints: HashMap<u32, String>,
    /// Partitions marked stale until the next successful refresh of that entry.
    stale: HashMap<u32, bool>,
    /// How many full-directory refreshes have been applied (diagnostics/tests).
    refresh_count: u64,
    /// How many single-entry refreshes have been applied.
    entry_refresh_count: u64,
}

impl ClientDirectoryCache {
    /// Empty cache (must [`Self::replace`] before routing).
    pub fn empty() -> Self {
        Self {
            map: PartitionMap::new(1),
            placement_epoch: PlacementEpoch(0),
            routes: HashMap::new(),
            endpoints: HashMap::new(),
            stale: HashMap::new(),
            refresh_count: 0,
            entry_refresh_count: 0,
        }
    }

    /// Build a cache from a placement directory (no network endpoints).
    pub fn from_directory(dir: &PartitionDirectory) -> Self {
        let mut cache = Self::empty();
        cache.replace(dir, HashMap::new());
        cache
    }

    /// Build from a wire snapshot (includes optional endpoints).
    pub fn from_snapshot(snap: &DirectorySnapshot) -> Self {
        let dir = snap.to_directory();
        let mut cache = Self::empty();
        cache.replace(&dir, snap.endpoints.clone());
        cache
    }

    /// Replace the entire cache from a fresh directory + endpoints.
    pub fn replace(&mut self, dir: &PartitionDirectory, endpoints: HashMap<u32, String>) {
        self.map = dir.map.clone();
        self.placement_epoch = dir.placement_epoch;
        self.routes.clear();
        for a in &dir.assignments {
            self.routes.insert(a.partition.get(), CachedRoute::from(a));
        }
        self.endpoints = endpoints;
        self.stale.clear();
        self.refresh_count = self.refresh_count.saturating_add(1);
    }

    /// Refresh a single partition assignment (CLUSTER_SPEC §13 step 4).
    pub fn refresh_entry(&mut self, assignment: &PartitionAssignment) {
        self.routes
            .insert(assignment.partition.get(), CachedRoute::from(assignment));
        self.stale.remove(&assignment.partition.get());
        // Keep whole-directory epoch at least as high as the entry.
        if assignment.placement_epoch > self.placement_epoch {
            self.placement_epoch = assignment.placement_epoch;
        }
        self.entry_refresh_count = self.entry_refresh_count.saturating_add(1);
    }

    /// Mark a partition route as stale (forces refresh on next route lookup
    /// when using [`Self::route_checked`]).
    pub fn mark_stale(&mut self, partition: PartitionId) {
        self.stale.insert(partition.get(), true);
    }

    /// Poison the cached leader for tests (simulates stale placement).
    #[doc(hidden)]
    pub fn poison_leader(&mut self, partition: PartitionId, fake_leader: NodeId) {
        if let Some(r) = self.routes.get_mut(&partition.get()) {
            r.leader = fake_leader;
            // Keep epoch so the caller can observe a leader mismatch without
            // an epoch bump; production servers use epoch and/or not_leader.
        }
    }

    /// Partition map used for hashing.
    pub fn partition_map(&self) -> &PartitionMap {
        &self.map
    }

    /// Whole-directory placement epoch.
    pub fn placement_epoch(&self) -> PlacementEpoch {
        self.placement_epoch
    }

    /// Number of full-directory refreshes applied.
    pub fn refresh_count(&self) -> u64 {
        self.refresh_count
    }

    /// Number of single-entry refreshes applied.
    pub fn entry_refresh_count(&self) -> u64 {
        self.entry_refresh_count
    }

    /// Endpoint for a node, if known.
    pub fn endpoint(&self, node: NodeId) -> Option<&str> {
        self.endpoints.get(&node.index()).map(|s| s.as_str())
    }

    /// All known endpoints.
    pub fn endpoints(&self) -> &HashMap<u32, String> {
        &self.endpoints
    }

    /// Map subject bytes to a virtual partition.
    pub fn partition_of(&self, partition_key: &[u8]) -> PartitionId {
        self.map.partition_of(partition_key)
    }

    /// Lookup a cached route for a partition (no staleness check).
    pub fn get(&self, partition: PartitionId) -> Option<&CachedRoute> {
        self.routes.get(&partition.get())
    }

    /// Whether the partition entry is marked stale.
    pub fn is_stale(&self, partition: PartitionId) -> bool {
        self.stale.get(&partition.get()).copied().unwrap_or(false)
    }

    /// Route a partition key (subject) to a cached leader route.
    ///
    /// Returns `None` if the partition is unknown or marked stale.
    pub fn route_checked(&self, partition_key: &[u8]) -> Option<&CachedRoute> {
        let p = self.partition_of(partition_key);
        if self.is_stale(p) {
            return None;
        }
        self.get(p)
    }

    /// Route without staleness gate (may return a known-stale entry).
    pub fn route(&self, partition_key: &[u8]) -> Option<&CachedRoute> {
        let p = self.partition_of(partition_key);
        self.get(p)
    }

    /// True when `observed` is strictly newer than the cached epoch for that
    /// partition (or the directory epoch when the partition is unknown).
    pub fn is_observed_epoch_newer(
        &self,
        partition: PartitionId,
        observed: PlacementEpoch,
    ) -> bool {
        let cached = self
            .get(partition)
            .map(|r| r.placement_epoch)
            .unwrap_or(self.placement_epoch);
        observed > cached
    }

    /// Snapshot for wire / debugging.
    pub fn snapshot(&self) -> DirectorySnapshot {
        let mut assignments: Vec<AssignmentWire> = self
            .routes
            .values()
            .map(|r| AssignmentWire {
                partition: r.partition.get(),
                replicas: r.replicas.iter().map(|n| n.index()).collect(),
                leader: r.leader.index(),
                term: r.term.0,
                placement_epoch: r.placement_epoch.0,
            })
            .collect();
        assignments.sort_by_key(|a| a.partition);
        DirectorySnapshot {
            virtual_partitions: self.map.virtual_partitions,
            hash_profile: self.map.hash_profile.clone(),
            placement_epoch: self.placement_epoch.0,
            assignments,
            endpoints: self.endpoints.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dingo_cluster::PartitionMap;

    #[test]
    fn route_is_stable_for_subject() {
        let dir = PartitionDirectory::balanced(PartitionMap::new(8), 3, PlacementEpoch(1));
        let cache = ClientDirectoryCache::from_directory(&dir);
        let a = cache.route(b"users/alice").unwrap().clone();
        let b = cache.route(b"users/alice").unwrap().clone();
        assert_eq!(a, b);
        assert_eq!(a.partition, cache.partition_of(b"users/alice"));
    }

    #[test]
    fn stale_gate_hides_route_until_refresh() {
        let dir = PartitionDirectory::balanced(PartitionMap::new(4), 2, PlacementEpoch(1));
        let mut cache = ClientDirectoryCache::from_directory(&dir);
        let p = cache.partition_of(b"k");
        cache.mark_stale(p);
        assert!(cache.route_checked(b"k").is_none());
        let assignment = dir.get(p).unwrap().clone();
        cache.refresh_entry(&assignment);
        assert!(cache.route_checked(b"k").is_some());
        assert_eq!(cache.entry_refresh_count(), 1);
    }

    #[test]
    fn snapshot_roundtrip() {
        let dir = PartitionDirectory::balanced(PartitionMap::new(4), 2, PlacementEpoch(3));
        let mut endpoints = HashMap::new();
        endpoints.insert(0, "127.0.0.1:7400".into());
        endpoints.insert(1, "127.0.0.1:7401".into());
        let snap = DirectorySnapshot::from_directory(&dir, endpoints.clone());
        let cache = ClientDirectoryCache::from_snapshot(&snap);
        assert_eq!(cache.placement_epoch().0, 3);
        assert_eq!(cache.endpoint(NodeId::new(0)), Some("127.0.0.1:7400"));
        assert_eq!(cache.routes.len(), 4);
    }
}
