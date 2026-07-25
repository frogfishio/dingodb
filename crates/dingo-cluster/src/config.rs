//! Cluster configuration and on-disk cluster descriptor.

use crate::error::ClusterError;
use crate::id::ClusterId;
use crate::modes::{ConsistencyMode, DeploymentProfile};
use crate::partition::{PartitionMap, DEFAULT_VIRTUAL_PARTITIONS, HASH_PROFILE_BLAKE3_MOD};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// In-memory configuration used to create a cluster.
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    /// Root directory for cluster metadata and node stores.
    pub root: PathBuf,
    /// Deployment profile.
    pub profile: DeploymentProfile,
    /// Virtual partition map.
    pub partition_map: PartitionMap,
    /// Write consistency mode.
    pub consistency_mode: ConsistencyMode,
    /// Optional fixed cluster id (otherwise generated).
    pub cluster_id: Option<ClusterId>,
}

impl ClusterConfig {
    /// Development profile: one node under `root`.
    pub fn development(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            profile: DeploymentProfile::Development,
            partition_map: PartitionMap::new(DEFAULT_VIRTUAL_PARTITIONS),
            consistency_mode: ConsistencyMode::PartitionLinearizable,
            cluster_id: None,
        }
    }

    /// Dependable local: three voting nodes under `root`.
    pub fn dependable_local(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            profile: DeploymentProfile::DependableLocal,
            partition_map: PartitionMap::new(DEFAULT_VIRTUAL_PARTITIONS),
            consistency_mode: ConsistencyMode::PartitionLinearizable,
            cluster_id: None,
        }
    }

    /// Override virtual partition count.
    pub fn with_virtual_partitions(mut self, n: u32) -> Self {
        self.partition_map = PartitionMap::new(n);
        self
    }

    /// Override consistency mode.
    pub fn with_consistency_mode(mut self, mode: ConsistencyMode) -> Self {
        self.consistency_mode = mode;
        self
    }
}

/// Persistent cluster descriptor (`cluster.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMeta {
    /// Format tag for this draft descriptor.
    pub format: String,
    /// Cluster id.
    pub cluster_id: String,
    /// Profile name.
    pub profile: String,
    /// Consistency mode name.
    pub consistency_mode: String,
    /// Virtual partition count.
    pub virtual_partitions: u32,
    /// Hash profile name.
    pub hash_profile: String,
    /// Node count.
    pub node_count: u32,
    /// Placement epoch at create time.
    pub placement_epoch: u64,
}

impl ClusterMeta {
    pub(crate) const FORMAT: &'static str = "dingo-cluster-8f";
    /// Formats accepted on open (8a–8f).
    pub(crate) const FORMAT_COMPAT: &'static [&'static str] = &[
        "dingo-cluster-8f",
        "dingo-cluster-8e",
        "dingo-cluster-8c",
        "dingo-cluster-8b",
        "dingo-cluster-8a",
    ];

    pub(crate) fn from_config(cfg: &ClusterConfig, cluster_id: ClusterId) -> Self {
        Self {
            format: Self::FORMAT.to_string(),
            cluster_id: cluster_id.to_hex(),
            profile: cfg.profile.as_str().to_string(),
            consistency_mode: cfg.consistency_mode.as_str().to_string(),
            virtual_partitions: cfg.partition_map.virtual_partitions,
            hash_profile: cfg.partition_map.hash_profile.clone(),
            node_count: cfg.profile.default_node_count(),
            placement_epoch: 1,
        }
    }

    pub(crate) fn write(&self, root: &Path) -> Result<(), ClusterError> {
        let path = root.join("cluster.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| ClusterError::CorruptMeta("serialize cluster.json"))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub(crate) fn load(root: &Path) -> Result<Self, ClusterError> {
        let path = root.join("cluster.json");
        if !path.is_file() {
            return Err(ClusterError::NotACluster(format!(
                "missing cluster.json at {}",
                root.display()
            )));
        }
        let bytes = std::fs::read(&path)?;
        let meta: Self = serde_json::from_slice(&bytes)
            .map_err(|_| ClusterError::CorruptMeta("parse cluster.json"))?;
        if !Self::FORMAT_COMPAT.contains(&meta.format.as_str()) {
            return Err(ClusterError::CorruptMeta("unsupported cluster format"));
        }
        if meta.hash_profile != HASH_PROFILE_BLAKE3_MOD {
            return Err(ClusterError::CorruptMeta("unsupported hash profile"));
        }
        Ok(meta)
    }
}

/// Path helpers for a cluster root.
pub fn node_store_path(root: &Path, node_index: u32) -> PathBuf {
    root.join("nodes").join(format!("node-{node_index}"))
}
