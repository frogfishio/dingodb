//! PEER-SQL — same-bed Residiuum vs SQLite peer pump (diagnostic only).
//!
//! Contract: `doc/wip/status/surveys/README-PEER-SQL.md`
//!
//! Target is **logical payload** bytes (`keys * payload_size`), not Residiuum
//! on-disk footprint, so engines are comparable for ops/s and logical MiB/s.

use crate::size::{dir_size_bytes, ensure_free_space, format_bytes};
use residiuum_store::{DurabilityMode, Store};
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const DISCLOSURE: &str = "Diagnostic only — not a published SLO. PEER-SQL same-bed peer \
    (doc/wip/status/surveys/README-PEER-SQL.md; \
    doc/reference/operations/BENCHMARK_DISCLOSURE.md). Compare A vs A and B vs B only.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerEngine {
    Residiuum,
    Sqlite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerMode {
    /// Autocommit / Residiuum put_batch_size=1.
    A,
    /// SQLite txn-128 / Residiuum put_batch_size=128.
    B,
}

impl PeerMode {
    pub fn batch_size(self) -> usize {
        match self {
            PeerMode::A => 1,
            PeerMode::B => 128,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PeerMode::A => "A_autocommit",
            PeerMode::B => "B_txn_128",
        }
    }
}

impl PeerEngine {
    pub fn label(self) -> &'static str {
        match self {
            PeerEngine::Residiuum => "residiuum",
            PeerEngine::Sqlite => "sqlite",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerConfig {
    /// Work directory (created if missing). Residiuum store: `work/store`;
    /// SQLite file: `work/peer.sqlite`.
    pub work: PathBuf,
    pub engine: PeerEngine,
    pub mode: PeerMode,
    /// Logical payload budget: stop when `keys * payload_size >= target_bytes`.
    pub target_bytes: u64,
    pub payload_size: usize,
    pub seed: u64,
    pub min_free_bytes: u64,
    pub json_out: bool,
    /// Soft seal threshold (bytes). Default 64 MiB matches surveys; raise to
    /// measure Mode A without mid-run seal cost (seal is separate from put prep).
    pub seal_threshold: u64,
}

#[derive(Debug, Clone, Default)]
struct ProcessSamples {
    peak_rss_bytes: Option<u64>,
    peak_cpu_pct: Option<f64>,
    last_cpu_pct: Option<f64>,
    sample_count: u64,
}

pub fn parse_engine(s: &str) -> Result<PeerEngine, String> {
    match s.to_ascii_lowercase().as_str() {
        "residiuum" | "res" | "rr" => Ok(PeerEngine::Residiuum),
        "sqlite" | "sql" => Ok(PeerEngine::Sqlite),
        other => Err(format!(
            "unknown engine `{other}` (residiuum|sqlite)"
        )),
    }
}

pub fn parse_mode(s: &str) -> Result<PeerMode, String> {
    match s.to_ascii_lowercase().as_str() {
        "a" | "a_autocommit" | "autocommit" => Ok(PeerMode::A),
        "b" | "b_txn_128" | "txn" | "txn128" => Ok(PeerMode::B),
        other => Err(format!("unknown mode `{other}` (A|B)")),
    }
}

pub fn run_peer_pump(cfg: &PeerConfig) -> Result<(), String> {
    if cfg.target_bytes == 0 {
        return Err("target-bytes must be > 0".into());
    }
    if cfg.payload_size == 0 {
        return Err("payload-size must be > 0".into());
    }
    fs::create_dir_all(&cfg.work).map_err(|e| format!("create work dir: {e}"))?;
    let free = ensure_free_space(&cfg.work, cfg.min_free_bytes)?;
    if !cfg.json_out && cfg.min_free_bytes > 0 {
        eprintln!(
            "peer-pump free-space ok: free={} min-free={} path={}",
            format_bytes(free),
            format_bytes(cfg.min_free_bytes),
            cfg.work.display()
        );
    }

    let mut payload = vec![0u8; cfg.payload_size];
    fill_payload(&mut payload, cfg.seed);

    let result = match cfg.engine {
        PeerEngine::Residiuum => pump_residiuum(cfg, &payload)?,
        PeerEngine::Sqlite => pump_sqlite(cfg, &payload)?,
    };

    let report = json!({
        "prong": "peer-pump",
        "ok": result.ok,
        "engine": cfg.engine.label(),
        "mode": cfg.mode.label(),
        "payload_size": cfg.payload_size,
        "target_bytes": cfg.target_bytes,
        "target_kind": "logical_payload",
        "keys_written": result.keys_written,
        "logical_bytes": result.logical_bytes,
        "bytes_on_disk": result.bytes_on_disk,
        "elapsed_ms": result.elapsed_ms,
        "ops_per_sec": result.ops_per_sec,
        "mb_per_sec": result.mb_per_sec_logical,
        "mb_per_sec_disk": result.mb_per_sec_disk,
        "put_batch_size": cfg.mode.batch_size(),
        "peak_rss_bytes": result.peak_rss_bytes,
        "peak_cpu_pct": result.peak_cpu_pct,
        "process_sample_count": result.sample_count,
        "work": cfg.work.display().to_string(),
        "store_or_db": result.path,
        "sqlite_journal": if cfg.engine == PeerEngine::Sqlite { "WAL" } else { "" },
        "sqlite_synchronous": if cfg.engine == PeerEngine::Sqlite { "NORMAL" } else { "" },
        "residiuum_durability": if cfg.engine == PeerEngine::Residiuum { "buffered" } else { "" },
        "disclosure": DISCLOSURE,
        "contract": "doc/wip/status/surveys/README-PEER-SQL.md",
    });

    if cfg.json_out {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "peer-pump done: engine={} mode={} keys={} logical={} disk={} in {:.2}s — {:.1} ops/s, {:.2} logical MiB/s",
            cfg.engine.label(),
            cfg.mode.label(),
            result.keys_written,
            format_bytes(result.logical_bytes),
            format_bytes(result.bytes_on_disk),
            result.elapsed_ms as f64 / 1000.0,
            result.ops_per_sec,
            result.mb_per_sec_logical
        );
        if let Some(rss) = result.peak_rss_bytes {
            println!(
                "  peak_rss={}  peak_cpu%={}",
                format_bytes(rss),
                result
                    .peak_cpu_pct
                    .map(|c| format!("{c:.0}"))
                    .unwrap_or_else(|| "n/a".into())
            );
        }
        println!("  path={}", result.path);
        println!("  {DISCLOSURE}");
    }

    if !result.ok {
        return Err(format!(
            "peer-pump under logical target: {} < {}",
            format_bytes(result.logical_bytes),
            format_bytes(cfg.target_bytes)
        ));
    }
    Ok(())
}

struct PeerResult {
    ok: bool,
    keys_written: u64,
    logical_bytes: u64,
    bytes_on_disk: u64,
    elapsed_ms: u64,
    ops_per_sec: f64,
    mb_per_sec_logical: f64,
    mb_per_sec_disk: f64,
    peak_rss_bytes: Option<u64>,
    peak_cpu_pct: Option<f64>,
    sample_count: u64,
    path: String,
}

fn pump_residiuum(cfg: &PeerConfig, payload: &[u8]) -> Result<PeerResult, String> {
    let store_path = cfg.work.join("store");
    if store_path.exists() {
        fs::remove_dir_all(&store_path)
            .map_err(|e| format!("remove prior residiuum store: {e}"))?;
    }
    let mut store = Store::create_with_shards(&store_path, 1)
        .map_err(|e| format!("create store: {e}"))?;
    let seal = if cfg.seal_threshold == 0 {
        64 * 1024 * 1024
    } else {
        cfg.seal_threshold
    };
    store.set_seal_threshold(seal);

    let batch = cfg.mode.batch_size();
    let mut samples = ProcessSamples::default();
    let t0 = Instant::now();
    let mut keys_written = 0u64;
    let mut pending: Vec<String> = Vec::with_capacity(batch);
    let mut last_report = Instant::now();

    while keys_written.saturating_mul(cfg.payload_size as u64) < cfg.target_bytes {
        let key = format!("peer/{:020}", keys_written);
        pending.push(key);
        keys_written += 1;
        if pending.len() >= batch {
            flush_residiuum(&mut store, &pending, payload)?;
            pending.clear();
        }
        if last_report.elapsed().as_secs() >= 2 || keys_written == 1 {
            sample_process(&mut samples);
            if !cfg.json_out {
                let logical = keys_written.saturating_mul(cfg.payload_size as u64);
                let elapsed = t0.elapsed().as_secs_f64().max(1e-9);
                eprintln!(
                    "peer-pump residiuum: keys={keys_written} logical≈{} / {} {:.1} ops/s",
                    format_bytes(logical),
                    format_bytes(cfg.target_bytes),
                    keys_written as f64 / elapsed
                );
            }
            last_report = Instant::now();
        }
        if keys_written >= 50_000_000 {
            return Err("peer-pump abort: 50M keys without reaching logical target".into());
        }
    }
    if !pending.is_empty() {
        flush_residiuum(&mut store, &pending, payload)?;
        pending.clear();
    }
    sample_process(&mut samples);
    let _ = store.seal_active();
    drop(store);

    let elapsed = t0.elapsed();
    finish_result(cfg, keys_written, &store_path, elapsed, &samples)
}

fn flush_residiuum(
    store: &mut Store,
    keys: &[String],
    payload: &[u8],
) -> Result<(), String> {
    if keys.is_empty() {
        return Ok(());
    }
    let items: Vec<(&str, &[u8])> = keys.iter().map(|k| (k.as_str(), payload)).collect();
    store
        .put_many(&items, DurabilityMode::Buffered)
        .map_err(|e| format!("put_many ({} keys): {e}", keys.len()))?;
    Ok(())
}

fn pump_sqlite(cfg: &PeerConfig, payload: &[u8]) -> Result<PeerResult, String> {
    let db_path = cfg.work.join("peer.sqlite");
    if db_path.exists() {
        fs::remove_file(&db_path).map_err(|e| format!("remove prior sqlite: {e}"))?;
    }
    // Remove WAL/SHM leftovers if any.
    let _ = fs::remove_file(cfg.work.join("peer.sqlite-wal"));
    let _ = fs::remove_file(cfg.work.join("peer.sqlite-shm"));

    let conn = Connection::open(&db_path).map_err(|e| format!("sqlite open: {e}"))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE kv (
           k TEXT PRIMARY KEY NOT NULL,
           v BLOB NOT NULL
         );",
    )
    .map_err(|e| format!("sqlite setup: {e}"))?;

    let batch = cfg.mode.batch_size();
    let mut samples = ProcessSamples::default();
    let t0 = Instant::now();
    let mut keys_written = 0u64;
    let mut last_report = Instant::now();

    let insert_sql = "INSERT INTO kv(k, v) VALUES (?1, ?2)";

    while keys_written.saturating_mul(cfg.payload_size as u64) < cfg.target_bytes {
        match cfg.mode {
            PeerMode::A => {
                let key = format!("peer/{:020}", keys_written);
                conn.execute(insert_sql, rusqlite::params![key, payload])
                    .map_err(|e| format!("sqlite insert: {e}"))?;
                keys_written += 1;
            }
            PeerMode::B => {
                let tx = conn
                    .unchecked_transaction()
                    .map_err(|e| format!("sqlite begin: {e}"))?;
                let mut stmt = tx
                    .prepare_cached(insert_sql)
                    .map_err(|e| format!("sqlite prepare: {e}"))?;
                let mut n = 0usize;
                while n < batch
                    && keys_written.saturating_mul(cfg.payload_size as u64) < cfg.target_bytes
                {
                    let key = format!("peer/{:020}", keys_written);
                    stmt.execute(rusqlite::params![key, payload])
                        .map_err(|e| format!("sqlite insert: {e}"))?;
                    keys_written += 1;
                    n += 1;
                }
                drop(stmt);
                tx.commit().map_err(|e| format!("sqlite commit: {e}"))?;
            }
        }

        if last_report.elapsed().as_secs() >= 2 || keys_written == 1 {
            sample_process(&mut samples);
            if !cfg.json_out {
                let logical = keys_written.saturating_mul(cfg.payload_size as u64);
                let elapsed = t0.elapsed().as_secs_f64().max(1e-9);
                eprintln!(
                    "peer-pump sqlite: keys={keys_written} logical≈{} / {} {:.1} ops/s",
                    format_bytes(logical),
                    format_bytes(cfg.target_bytes),
                    keys_written as f64 / elapsed
                );
            }
            last_report = Instant::now();
        }
        if keys_written >= 50_000_000 {
            return Err("peer-pump abort: 50M keys without reaching logical target".into());
        }
    }

    // Checkpoint WAL so on-disk size is meaningful for diagnostics.
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("sqlite checkpoint: {e}"))?;
    // Confirm row count roughly matches.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM kv", [], |r| r.get(0))
        .map_err(|e| format!("sqlite count: {e}"))?;
    if count as u64 != keys_written {
        return Err(format!(
            "sqlite row count mismatch: table={count} keys_written={keys_written}"
        ));
    }
    drop(conn);

    let elapsed = t0.elapsed();
    sample_process(&mut samples);
    finish_result(cfg, keys_written, &db_path, elapsed, &samples)
}

fn finish_result(
    cfg: &PeerConfig,
    keys_written: u64,
    path: &Path,
    elapsed: std::time::Duration,
    samples: &ProcessSamples,
) -> Result<PeerResult, String> {
    let logical_bytes = keys_written.saturating_mul(cfg.payload_size as u64);
    let bytes_on_disk = if path.is_dir() {
        dir_size_bytes(path).map_err(|e| format!("dir size: {e}"))?
    } else {
        // Include WAL/SHM if present next to the db.
        let mut total = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        for suffix in ["-wal", "-shm"] {
            let p = PathBuf::from(format!("{}{suffix}", path.display()));
            if let Ok(m) = fs::metadata(&p) {
                total = total.saturating_add(m.len());
            }
        }
        total
    };
    let secs = elapsed.as_secs_f64().max(1e-9);
    let ops_per_sec = keys_written as f64 / secs;
    let mb_per_sec_logical = (logical_bytes as f64 / (1024.0 * 1024.0)) / secs;
    let mb_per_sec_disk = (bytes_on_disk as f64 / (1024.0 * 1024.0)) / secs;
    Ok(PeerResult {
        ok: logical_bytes >= cfg.target_bytes,
        keys_written,
        logical_bytes,
        bytes_on_disk,
        elapsed_ms: elapsed.as_millis() as u64,
        ops_per_sec,
        mb_per_sec_logical,
        mb_per_sec_disk,
        peak_rss_bytes: samples.peak_rss_bytes,
        peak_cpu_pct: samples.peak_cpu_pct,
        sample_count: samples.sample_count,
        path: path.display().to_string(),
    })
}

fn fill_payload(buf: &mut [u8], seed: u64) {
    let mut state = seed ^ 0xD1_160_B17_u64;
    for (i, b) in buf.iter_mut().enumerate() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        *b = ((state >> 33) as u8).wrapping_add((i & 0xff) as u8);
    }
    let magic = b"RESIDIUUM-PEER-SQL-PAYLOAD-v1\n";
    let n = magic.len().min(buf.len());
    buf[..n].copy_from_slice(&magic[..n]);
}

fn sample_process(samples: &mut ProcessSamples) {
    let Some((cpu, rss_kib)) = read_self_ps() else {
        return;
    };
    samples.sample_count += 1;
    samples.last_cpu_pct = Some(cpu);
    samples.peak_cpu_pct = Some(
        samples
            .peak_cpu_pct
            .map(|p| p.max(cpu))
            .unwrap_or(cpu),
    );
    let rss_bytes = rss_kib.saturating_mul(1024);
    samples.peak_rss_bytes = Some(
        samples
            .peak_rss_bytes
            .map(|p| p.max(rss_bytes))
            .unwrap_or(rss_bytes),
    );
}

fn read_self_ps() -> Option<(f64, u64)> {
    let pid = std::process::id().to_string();
    let output = Command::new("ps")
        .args(["-o", "%cpu=,rss=", "-p", &pid])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let cpu: f64 = parts[0].parse().ok()?;
    let rss_kib: u64 = parts[1].parse().ok()?;
    Some((cpu, rss_kib))
}