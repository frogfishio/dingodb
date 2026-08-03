//! Experimental disk write-pattern mimic (not a product path).
//!
//! Replays Residiuum Mode A peer-shaped I/O without cook/Blake/index logic:
//! per-put data `write_all` sizes + optional index-shaped writes. Measures raw
//! OS write rates so we can compare against peer TPS (~10k) / ~85 MiB/s logical.
//!
//! Calibration defaults come from peer artifacts:
//! - skip-index store ≈ 276 629 312 B / 32 768 keys → **8440 B data/op**
//! - dual-index *publish* on the hot path is in-memory (~11 ms / 32k) — not a
//!   per-put disk write; derived index checkpoints are every 65 536 ops
//!   (`DERIVED_CHECKPOINT_EVERY_OPS`) plus end-of-run persist/seal.

use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;

/// Peer Mode A recipe key count (256 MiB / 8 KiB).
pub const PEER_OPS: u64 = 32_768;
/// Measured mean on-disk growth per put under `--diag-skip-index` (data path).
pub const PEER_DATA_BYTES_PER_OP: usize = 8_440;
/// Store derived checkpoint cadence (matches `residiuum-store`).
pub const PEER_INDEX_CHECKPOINT_EVERY: u64 = 65_536;
/// Approximate locator/index record size if index were an append log (hypothesis).
pub const PEER_INDEX_APPEND_BYTES: usize = 64;
/// Extra full-path disk vs skip-index for peer recipe (seal/catalog/index amp).
pub const PEER_INDEX_END_BYTES: usize = 270_528_296;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimicMode {
    /// Per-op append `write_all` to one data file (grow-on-append).
    DataOnly,
    /// Data appends + end (and optional periodic) atomic index rewrite with fsync.
    DataPlusIndexAtomic,
    /// Data appends + per-op small append to a separate index log (no fsync).
    DataPlusIndexAppend,
}

impl MimicMode {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "data" | "data-only" | "dataonly" => Ok(Self::DataOnly),
            "atomic" | "data+atomic" | "index-atomic" => Ok(Self::DataPlusIndexAtomic),
            "append" | "data+append" | "index-append" => Ok(Self::DataPlusIndexAppend),
            other => Err(format!(
                "unknown write-mimic mode `{other}` (data|atomic|append)"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DataOnly => "data-only",
            Self::DataPlusIndexAtomic => "data+index-atomic",
            Self::DataPlusIndexAppend => "data+index-append",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteMimicConfig {
    pub work: PathBuf,
    pub mode: MimicMode,
    pub ops: u64,
    pub data_bytes_per_op: usize,
    pub index_append_bytes: usize,
    pub index_checkpoint_every: u64,
    pub index_end_bytes: usize,
    pub json_out: bool,
}

#[derive(Debug, Clone)]
pub struct WriteMimicResult {
    pub mode: &'static str,
    pub ops: u64,
    pub data_writes: u64,
    pub data_bytes: u64,
    pub index_writes: u64,
    pub index_bytes: u64,
    pub elapsed_ms: u64,
    pub ops_per_sec: f64,
    pub data_mib_per_sec: f64,
    pub index_mib_per_sec: f64,
    pub total_mib_per_sec: f64,
}

pub fn run_write_mimic(cfg: &WriteMimicConfig) -> Result<(), String> {
    if cfg.ops == 0 {
        return Err("--ops must be > 0".into());
    }
    if cfg.data_bytes_per_op == 0 {
        return Err("--data-bytes-per-op must be > 0".into());
    }
    fs::create_dir_all(&cfg.work).map_err(|e| format!("create work: {e}"))?;
    let data_path = cfg.work.join("data.seg");
    let index_path = cfg.work.join("primary.idx");
    let _ = fs::remove_file(&data_path);
    let _ = fs::remove_file(&index_path);
    let _ = fs::remove_file(cfg.work.join("primary.idx.prev"));

    let data_buf = vec![0xA5u8; cfg.data_bytes_per_op];
    let index_append_buf = vec![0x11u8; cfg.index_append_bytes.max(1)];

    let mut data = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&data_path)
        .map_err(|e| format!("open data: {e}"))?;
    let mut index_log: Option<File> = None;
    if cfg.mode == MimicMode::DataPlusIndexAppend {
        index_log = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&index_path)
                .map_err(|e| format!("open index log: {e}"))?,
        );
    }

    let mut data_writes = 0u64;
    let mut data_bytes = 0u64;
    let mut index_writes = 0u64;
    let mut index_bytes = 0u64;
    let mut off = 0u64;

    let t0 = Instant::now();
    for i in 0..cfg.ops {
        data.seek(SeekFrom::Start(off))
            .map_err(|e| format!("seek data: {e}"))?;
        data.write_all(&data_buf)
            .map_err(|e| format!("write data: {e}"))?;
        off = off.saturating_add(cfg.data_bytes_per_op as u64);
        data_writes += 1;
        data_bytes = data_bytes.saturating_add(cfg.data_bytes_per_op as u64);

        match cfg.mode {
            MimicMode::DataOnly => {}
            MimicMode::DataPlusIndexAppend => {
                let f = index_log.as_mut().expect("index log");
                f.write_all(&index_append_buf)
                    .map_err(|e| format!("write index append: {e}"))?;
                index_writes += 1;
                index_bytes = index_bytes.saturating_add(cfg.index_append_bytes as u64);
            }
            MimicMode::DataPlusIndexAtomic => {
                let n = i.saturating_add(1);
                if cfg.index_checkpoint_every > 0 && n % cfg.index_checkpoint_every == 0 {
                    // Growing checkpoint ≈ entry × ops so far (locator-sized).
                    let sz = (n as usize).saturating_mul(cfg.index_append_bytes.max(1));
                    atomic_rewrite(&index_path, sz)?;
                    index_writes += 1;
                    index_bytes = index_bytes.saturating_add(sz as u64);
                }
            }
        }
    }

    // End-of-run: Mode A peer seal/persist path (Buffered puts; final durable control).
    if cfg.mode == MimicMode::DataPlusIndexAtomic && cfg.index_end_bytes > 0 {
        atomic_rewrite(&index_path, cfg.index_end_bytes)?;
        index_writes += 1;
        index_bytes = index_bytes.saturating_add(cfg.index_end_bytes as u64);
    }
    if let Some(f) = index_log.as_mut() {
        // Peer Mode A is Buffered (no per-put fsync); flush OS buffers at end only.
        let _ = f.flush();
    }
    let _ = data.flush();

    let elapsed = t0.elapsed();
    let secs = elapsed.as_secs_f64().max(1e-9);
    let elapsed_ms = (elapsed.as_secs_f64() * 1000.0).round() as u64;
    let total_bytes = data_bytes.saturating_add(index_bytes);
    let result = WriteMimicResult {
        mode: cfg.mode.label(),
        ops: cfg.ops,
        data_writes,
        data_bytes,
        index_writes,
        index_bytes,
        elapsed_ms,
        ops_per_sec: cfg.ops as f64 / secs,
        data_mib_per_sec: (data_bytes as f64 / (1024.0 * 1024.0)) / secs,
        index_mib_per_sec: (index_bytes as f64 / (1024.0 * 1024.0)) / secs,
        total_mib_per_sec: (total_bytes as f64 / (1024.0 * 1024.0)) / secs,
    };

    if cfg.json_out {
        println!(
            "{}",
            json!({
                "prong": "write-mimic",
                "disclosure": "Experimental disk I/O mimic — not product TPS. Calibrated to peer Mode A write sizes.",
                "mode": result.mode,
                "ops": result.ops,
                "data_writes": result.data_writes,
                "data_bytes": result.data_bytes,
                "data_bytes_per_op": cfg.data_bytes_per_op,
                "index_writes": result.index_writes,
                "index_bytes": result.index_bytes,
                "index_append_bytes": cfg.index_append_bytes,
                "index_checkpoint_every": cfg.index_checkpoint_every,
                "index_end_bytes": cfg.index_end_bytes,
                "elapsed_ms": result.elapsed_ms,
                "ops_per_sec": result.ops_per_sec,
                "data_mib_per_sec": result.data_mib_per_sec,
                "index_mib_per_sec": result.index_mib_per_sec,
                "total_mib_per_sec": result.total_mib_per_sec,
                "work": cfg.work,
            })
        );
    } else {
        eprintln!(
            "write-mimic {}: ops={} data_writes={} ({:.1} MiB) index_writes={} ({:.1} MiB) in {:.2}s — {:.1} ops/s, data {:.1} MiB/s, total {:.1} MiB/s",
            result.mode,
            result.ops,
            result.data_writes,
            result.data_bytes as f64 / (1024.0 * 1024.0),
            result.index_writes,
            result.index_bytes as f64 / (1024.0 * 1024.0),
            secs,
            result.ops_per_sec,
            result.data_mib_per_sec,
            result.total_mib_per_sec,
        );
    }
    Ok(())
}

/// Same shape as `residiuum_store::atomic_file::write_atomic` (tmp + sync + rename + dir sync).
fn atomic_rewrite(path: &Path, len: usize) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    let tmp = path.with_extension("idx.tmp");
    let _ = fs::remove_file(&tmp);
    {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)
            .map_err(|e| format!("open tmp index: {e}"))?;
        let chunk = vec![0u8; 1024 * 1024];
        let mut left = len;
        while left > 0 {
            let n = left.min(chunk.len());
            f.write_all(&chunk[..n])
                .map_err(|e| format!("write tmp index: {e}"))?;
            left -= n;
        }
        f.sync_all().map_err(|e| format!("sync tmp index: {e}"))?;
    }
    if path.is_file() {
        let prev = path.with_extension("idx.prev");
        let _ = fs::remove_file(&prev);
        let _ = fs::rename(path, &prev);
    }
    fs::rename(&tmp, path).map_err(|e| format!("rename index: {e}"))?;
    // Parent dir sync (best-effort; macOS/Linux).
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}
