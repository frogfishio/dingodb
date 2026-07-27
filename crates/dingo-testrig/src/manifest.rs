//! Workload manifest written next to the store so monitor/chaos share context.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Relative filename under the workdir (sibling of store root when using `run`).
pub const MANIFEST_FILE: &str = "testrig-manifest.v1.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadManifest {
    pub format_version: u32,
    pub store_path: String,
    pub target_bytes: u64,
    pub payload_size: usize,
    pub durability: String,
    pub keys_written: u64,
    pub first_key: String,
    pub last_key: String,
    pub key_prefix: String,
    pub bytes_on_disk: u64,
    pub pump_elapsed_ms: u64,
    pub pump_ops_per_sec: f64,
    pub pump_mb_per_sec: f64,
    pub seed: u64,
    /// Writer shard count used for the pump (DEF-096 Axis B). Default 1 when absent in old manifests.
    #[serde(default = "default_one")]
    pub writer_shards: usize,
    /// Client / append concurrency disclosed for BENCHMARK_DISCLOSURE.
    #[serde(default = "default_one")]
    pub concurrency: usize,
    /// `single_active_segment` or `sharded_active_segments`.
    #[serde(default = "default_single_model")]
    pub writer_model: String,
    /// Peak process RSS observed during pump (bytes), if sampled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_rss_bytes: Option<u64>,
    /// Peak process CPU% from `ps` during pump (macOS: 100% ≈ one core).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_cpu_pct: Option<f64>,
}

fn default_one() -> usize {
    1
}

fn default_single_model() -> String {
    "single_active_segment".into()
}

impl WorkloadManifest {
    pub fn key_at(&self, index: u64) -> String {
        format!("{}{:020}", self.key_prefix, index)
    }

    pub fn sample_keys(&self, n: usize) -> Vec<String> {
        if self.keys_written == 0 || n == 0 {
            return Vec::new();
        }
        let n = n.min(self.keys_written as usize);
        let mut out = Vec::with_capacity(n);
        if n == 1 {
            out.push(self.key_at(0));
            return out;
        }
        for i in 0..n {
            let idx = if n == 1 {
                0
            } else {
                (i as u64 * (self.keys_written - 1)) / (n as u64 - 1)
            };
            out.push(self.key_at(idx));
        }
        out
    }
}

pub fn manifest_path_for_store(store: &Path) -> PathBuf {
    store
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(MANIFEST_FILE)
}

pub fn write_manifest(path: &Path, m: &WorkloadManifest) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = serde_json::to_vec_pretty(m).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, body)
}

pub fn read_manifest(path: &Path) -> io::Result<WorkloadManifest> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
