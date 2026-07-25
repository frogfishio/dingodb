//! Node endpoint map for network multi-node serve (post–Stage 8 follow-on).
//!
//! In-process clusters do not need host:port maps. Process-per-node TCP serve
//! records `node_index → host:port` in `endpoints.json` so clients and
//! `directory` RPC responses can advertise real routes (CLUSTER_SPEC §13).

use crate::error::ClusterError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Filename under the cluster root.
pub const ENDPOINTS_FILE: &str = "endpoints.json";

/// On-disk endpoints document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointsFile {
    /// Format tag.
    #[serde(default = "default_format")]
    pub format: String,
    /// Node dense index → `host:port` (or other connect string).
    #[serde(default)]
    pub endpoints: HashMap<String, String>,
}

fn default_format() -> String {
    "dingo-cluster-endpoints-1".into()
}

impl EndpointsFile {
    /// Empty map.
    pub fn new() -> Self {
        Self {
            format: default_format(),
            endpoints: HashMap::new(),
        }
    }

    /// Convert to `u32 → host:port` map (invalid keys skipped).
    pub fn as_u32_map(&self) -> HashMap<u32, String> {
        let mut out = HashMap::new();
        for (k, v) in &self.endpoints {
            if let Ok(idx) = k.parse::<u32>() {
                out.insert(idx, v.clone());
            }
        }
        out
    }

    /// Build from a `u32 → host:port` map.
    pub fn from_u32_map(map: &HashMap<u32, String>) -> Self {
        let mut endpoints = HashMap::new();
        for (k, v) in map {
            endpoints.insert(k.to_string(), v.clone());
        }
        Self {
            format: default_format(),
            endpoints,
        }
    }

    /// Set one node endpoint.
    pub fn set(&mut self, node_index: u32, hostport: impl Into<String>) {
        self.endpoints
            .insert(node_index.to_string(), hostport.into());
    }

    /// Load from `root/endpoints.json`, or empty if missing.
    pub fn load(root: &Path) -> Result<Self, ClusterError> {
        let path = root.join(ENDPOINTS_FILE);
        if !path.is_file() {
            return Ok(Self::new());
        }
        let bytes = std::fs::read(&path)?;
        let file: Self = serde_json::from_slice(&bytes)
            .map_err(|_| ClusterError::CorruptMeta("parse endpoints.json"))?;
        Ok(file)
    }

    /// Persist under the cluster root.
    pub fn save(&self, root: &Path) -> Result<(), ClusterError> {
        let path = root.join(ENDPOINTS_FILE);
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| ClusterError::CorruptMeta("serialize endpoints.json"))?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Load endpoints as a dense-index map (missing file → empty).
pub fn load_endpoints(root: &Path) -> Result<HashMap<u32, String>, ClusterError> {
    Ok(EndpointsFile::load(root)?.as_u32_map())
}

/// Save endpoints from a dense-index map.
pub fn save_endpoints(root: &Path, map: &HashMap<u32, String>) -> Result<(), ClusterError> {
    EndpointsFile::from_u32_map(map).save(root)
}

/// Upsert one node address and rewrite `endpoints.json`.
pub fn upsert_endpoint(
    root: &Path,
    node_index: u32,
    hostport: impl Into<String>,
) -> Result<HashMap<u32, String>, ClusterError> {
    let mut file = EndpointsFile::load(root)?;
    file.set(node_index, hostport);
    file.save(root)?;
    Ok(file.as_u32_map())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_endpoints_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut map = HashMap::new();
        map.insert(0, "127.0.0.1:7434".into());
        map.insert(1, "127.0.0.1:7435".into());
        save_endpoints(dir.path(), &map).unwrap();
        let loaded = load_endpoints(dir.path()).unwrap();
        assert_eq!(loaded.get(&0).map(String::as_str), Some("127.0.0.1:7434"));
        assert_eq!(loaded.get(&1).map(String::as_str), Some("127.0.0.1:7435"));
        let after = upsert_endpoint(dir.path(), 2, "127.0.0.1:7436").unwrap();
        assert_eq!(after.len(), 3);
    }
}
