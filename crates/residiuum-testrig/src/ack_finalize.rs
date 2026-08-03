//! Acknowledgement / finalisation split (NEXT_MEASUREMENT.md).
//!
//! Separates hot-path acknowledged-write throughput from drain, seal, close,
//! reopen, and verification. Measurement only — no storage or AWO changes.

use crate::size::{dir_size_bytes, ensure_free_space, format_bytes};
use crate::write_mimic::{PEER_DATA_BYTES_PER_OP, PEER_OPS};
use residiuum_format::body_hash;
use residiuum_store::{DiagnosticIoSink, DurabilityMode, Store};
use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DISCLOSURE: &str = "Diagnostic only — ack/finalisation split \
    (doc/todo/performance-qualification/NEXT_MEASUREMENT.md; \
    doc/reference/operations/BENCHMARK_DISCLOSURE.md). Not a published SLO.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureCell {
    RealFull,
    RealSkipIndex,
    Discard,
    RawMimic,
}

impl MeasureCell {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "real" | "real-full" | "full" => Ok(Self::RealFull),
            "skip-index" | "real-skip-index" | "indexing-disabled" => Ok(Self::RealSkipIndex),
            "discard" => Ok(Self::Discard),
            "raw-mimic" | "mimic" | "write-mimic" => Ok(Self::RawMimic),
            other => Err(format!(
                "unknown cell `{other}` (real-full|skip-index|discard|raw-mimic)"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RealFull => "Real full",
            Self::RealSkipIndex => "Real, indexing disabled",
            Self::Discard => "Discard",
            Self::RawMimic => "Raw mimic",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::RealFull => "real-full",
            Self::RealSkipIndex => "skip-index",
            Self::Discard => "discard",
            Self::RawMimic => "raw-mimic",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AckFinalizeConfig {
    pub work: PathBuf,
    pub cell: MeasureCell,
    pub target_bytes: u64,
    pub payload_size: usize,
    pub concurrency: usize,
    pub seed: u64,
    pub seal_threshold: u64,
    pub min_free_bytes: u64,
    pub json_out: bool,
}

#[derive(Debug, Clone)]
pub struct AckFinalizeResult {
    pub cell: MeasureCell,
    pub keys: u64,
    pub payload_size: usize,
    pub concurrency: usize,
    pub ack_elapsed: Duration,
    pub drain_elapsed: Duration,
    pub seal_elapsed: Duration,
    pub close_elapsed: Duration,
    pub reopen_elapsed: Duration,
    pub verify_elapsed: Duration,
    pub lifecycle_elapsed: Duration,
    pub acknowledged_write_ops_per_sec: f64,
    pub lifecycle_ops_per_sec: f64,
    pub reopen_exact: bool,
    pub logical_bytes: u64,
    pub on_disk_bytes: u64,
    pub timestamps: Timestamps,
}

#[derive(Debug, Clone)]
pub struct Timestamps {
    pub workload_start_unix_ns: u128,
    pub last_successful_ack_unix_ns: u128,
    pub drain_complete_unix_ns: u128,
    pub seal_complete_unix_ns: u128,
    pub close_complete_unix_ns: u128,
    pub reopen_complete_unix_ns: u128,
    pub verification_complete_unix_ns: u128,
}

fn unix_ns_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn fill_payload(size: usize, seed: u64) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    for b in &mut out {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        *b = (state >> 33) as u8;
    }
    out
}

/// Run one matrix cell. Fail-closed: seal errors and reopen mismatches abort.
pub fn run_ack_finalize(cfg: &AckFinalizeConfig) -> Result<AckFinalizeResult, String> {
    if cfg.payload_size == 0 {
        return Err("--payload-size must be > 0".into());
    }
    if cfg.target_bytes == 0 {
        return Err("--target-bytes must be > 0".into());
    }
    fs::create_dir_all(&cfg.work).map_err(|e| format!("create work: {e}"))?;
    let _ = ensure_free_space(&cfg.work, cfg.min_free_bytes)?;

    let result = match cfg.cell {
        MeasureCell::RawMimic => run_raw_mimic(cfg)?,
        _ => run_store_cell(cfg)?,
    };

    if cfg.json_out {
        println!(
            "{}",
            serde_json::to_string_pretty(&result_json(&result))
                .map_err(|e| format!("serialize cell json: {e}"))?
        );
    } else {
        eprintln!(
            "ack-finalize {}: keys={} ack_tps={:.0} seal_ms={:.1} lifecycle_tps={:.0} reopen_exact={}",
            result.cell.slug(),
            result.keys,
            result.acknowledged_write_ops_per_sec,
            result.seal_elapsed.as_secs_f64() * 1000.0,
            result.lifecycle_ops_per_sec,
            result.reopen_exact
        );
    }
    Ok(result)
}

/// Run the four-cell matrix; write evidence JSON + markdown table under `evidence_dir`.
pub fn run_ack_finalize_matrix(
    work_root: &Path,
    evidence_dir: &Path,
    target_bytes: u64,
    payload_size: usize,
    concurrency: usize,
    seed: u64,
    seal_threshold: u64,
    min_free_bytes: u64,
) -> Result<(), String> {
    fs::create_dir_all(evidence_dir).map_err(|e| format!("create evidence dir: {e}"))?;
    fs::create_dir_all(work_root).map_err(|e| format!("create work root: {e}"))?;

    let cells = [
        MeasureCell::RealFull,
        MeasureCell::RealSkipIndex,
        MeasureCell::Discard,
        MeasureCell::RawMimic,
    ];
    let mut rows: Vec<AckFinalizeResult> = Vec::with_capacity(4);
    let mut errors: Vec<String> = Vec::new();

    for cell in cells {
        let work = work_root.join(cell.slug());
        if work.exists() {
            fs::remove_dir_all(&work).map_err(|e| format!("cleanup {}: {e}", work.display()))?;
        }
        let cfg = AckFinalizeConfig {
            work: work.clone(),
            cell,
            target_bytes,
            payload_size,
            concurrency,
            seed,
            seal_threshold,
            min_free_bytes,
            json_out: false,
        };
        match run_ack_finalize(&cfg) {
            Ok(r) => {
                let cell_path = evidence_dir.join(format!("{}.json", cell.slug()));
                fs::write(
                    &cell_path,
                    serde_json::to_string_pretty(&result_json(&r))
                        .map_err(|e| format!("serialize {}: {e}", cell.slug()))?,
                )
                .map_err(|e| format!("write {}: {e}", cell_path.display()))?;
                rows.push(r);
            }
            Err(e) => {
                errors.push(format!("{}: {e}", cell.label()));
                eprintln!("ack-finalize cell {} FAILED: {e}", cell.slug());
            }
        }
        // Disk hygiene — always remove work dir after evidence is captured.
        if work.exists() {
            let _ = fs::remove_dir_all(&work);
        }
    }

    let table = render_markdown_table(&rows);
    let summary = json!({
        "kind": "ack_finalize_matrix",
        "disclosure": DISCLOSURE,
        "recipe": {
            "fs": "APFS",
            "payload_size": payload_size,
            "logical_data_bytes": target_bytes,
            "concurrency": concurrency,
            "durability": "Buffered",
            "awo": "Disabled",
            "seed": seed,
            "seal_threshold": seal_threshold,
        },
        "cells": rows.iter().map(result_json).collect::<Vec<_>>(),
        "errors": errors,
        "markdown_table": table,
    });
    let summary_path = evidence_dir.join("matrix.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).map_err(|e| format!("serialize matrix: {e}"))?,
    )
    .map_err(|e| format!("write {}: {e}", summary_path.display()))?;
    let table_path = evidence_dir.join("EVIDENCE_TABLE.md");
    fs::write(&table_path, format!("{table}\n"))
        .map_err(|e| format!("write {}: {e}", table_path.display()))?;

    println!("{table}");
    eprintln!(
        "ack-finalize matrix: wrote {} and {}",
        summary_path.display(),
        table_path.display()
    );

    if !errors.is_empty() {
        return Err(format!(
            "matrix incomplete ({} cell failure(s)): {}",
            errors.len(),
            errors.join("; ")
        ));
    }
    if rows.len() != 4 {
        return Err(format!("expected 4 cells, got {}", rows.len()));
    }
    Ok(())
}

fn run_store_cell(cfg: &AckFinalizeConfig) -> Result<AckFinalizeResult, String> {
    let store_path = cfg.work.join("store");
    if store_path.exists() {
        fs::remove_dir_all(&store_path)
            .map_err(|e| format!("remove prior store: {e}"))?;
    }
    let mut store = Store::create_with_shards(&store_path, 1)
        .map_err(|e| format!("create store: {e}"))?;
    let seal_thr = if cfg.seal_threshold == 0 {
        64 * 1024 * 1024
    } else {
        cfg.seal_threshold
    };
    store.set_seal_threshold(seal_thr);

    let sink = match cfg.cell {
        MeasureCell::Discard => DiagnosticIoSink::Discard,
        MeasureCell::RealFull | MeasureCell::RealSkipIndex => DiagnosticIoSink::Real,
        MeasureCell::RawMimic => unreachable!("raw mimic handled separately"),
    };
    store
        .set_diagnostic_io_sink(sink)
        .map_err(|e| format!("set_diagnostic_io_sink: {e}"))?;
    if cfg.cell == MeasureCell::RealSkipIndex {
        store.set_diagnostic_skip_index(true);
    }

    let target_keys = cfg
        .target_bytes
        .div_ceil(cfg.payload_size as u64)
        .max(1);
    let payload = Arc::new(fill_payload(cfg.payload_size, cfg.seed));
    let expected_hash = body_hash(payload.as_slice());
    let store_arc = Arc::new(Mutex::new(store));
    let next = Arc::new(AtomicU64::new(0));
    let err: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let workers = cfg.concurrency.max(1);

    let workload_start_unix_ns = unix_ns_now();
    let t_workload = Instant::now();

    thread::scope(|scope| {
        for _ in 0..workers {
            let store_arc = Arc::clone(&store_arc);
            let next = Arc::clone(&next);
            let err = Arc::clone(&err);
            let payload = Arc::clone(&payload);
            scope.spawn(move || {
                loop {
                    if err.lock().ok().and_then(|g| g.clone()).is_some() {
                        break;
                    }
                    let seq = next.fetch_add(1, Ordering::Relaxed);
                    if seq >= target_keys {
                        break;
                    }
                    let key = format!("peer/{:020}", seq);
                    let put_err = {
                        let mut g = match store_arc.lock() {
                            Ok(g) => g,
                            Err(_) => {
                                let _ = err.lock().map(|mut e| {
                                    *e = Some("store lock poisoned during put".into());
                                });
                                break;
                            }
                        };
                        g.put_many(&[(&key[..], payload.as_slice())], DurabilityMode::Buffered)
                            .map(|_| ())
                            .map_err(|e| format!("put_many: {e}"))
                    };
                    if let Err(e) = put_err {
                        let _ = err.lock().map(|mut slot| {
                            if slot.is_none() {
                                *slot = Some(e);
                            }
                        });
                        break;
                    }
                }
            });
        }
    });

    if let Some(e) = err.lock().ok().and_then(|g| g.clone()) {
        return Err(e);
    }
    let keys_written = target_keys.min(next.load(Ordering::Relaxed));
    if keys_written != target_keys {
        return Err(format!(
            "ack incomplete: wrote {keys_written} of {target_keys} keys"
        ));
    }
    let last_successful_ack_unix_ns = unix_ns_now();
    let ack_elapsed = t_workload.elapsed();

    // AWO disabled — drain is a no-op boundary for stage accounting.
    let t_drain = Instant::now();
    let drain_complete_unix_ns = unix_ns_now();
    let drain_elapsed = t_drain.elapsed();

    let mut store = Arc::try_unwrap(store_arc)
        .map_err(|_| "store Arc still shared after ack phase".to_string())?
        .into_inner()
        .map_err(|_| "store mutex poisoned extract".to_string())?;

    let t_seal = Instant::now();
    store
        .seal_active()
        .map_err(|e| format!("seal_active failed (fail-closed): {e}"))?;
    let seal_complete_unix_ns = unix_ns_now();
    let seal_elapsed = t_seal.elapsed();

    let t_close = Instant::now();
    drop(store);
    let close_complete_unix_ns = unix_ns_now();
    let close_elapsed = t_close.elapsed();

    let t_reopen = Instant::now();
    let store = Store::open(&store_path).map_err(|e| format!("reopen failed (fail-closed): {e}"))?;
    let reopen_complete_unix_ns = unix_ns_now();
    let reopen_elapsed = t_reopen.elapsed();

    let t_verify = Instant::now();
    let mut mismatches = 0u64;
    let mut missing = 0u64;
    for seq in 0..keys_written {
        let key = format!("peer/{:020}", seq);
        match store.get(&key) {
            Ok(Some(body)) => {
                let h = body_hash(&body);
                if h != expected_hash {
                    mismatches = mismatches.saturating_add(1);
                }
            }
            Ok(None) => missing = missing.saturating_add(1),
            Err(e) => {
                return Err(format!("verify get({key}) failed (fail-closed): {e}"));
            }
        }
    }
    let verification_complete_unix_ns = unix_ns_now();
    let verify_elapsed = t_verify.elapsed();
    drop(store);

    let reopen_exact = mismatches == 0 && missing == 0;
    // Discard never writes put bytes to media; seal may succeed from RAM but
    // ordinary reopen cannot reconstruct the ledger. Report reopen_exact=false
    // honestly — do not invent durability. Real cells remain fail-closed.
    if !reopen_exact && cfg.cell != MeasureCell::Discard {
        return Err(format!(
            "reopen ledger mismatch (fail-closed): missing={missing} hash_mismatch={mismatches} of {keys_written}"
        ));
    }

    let lifecycle_elapsed = drain_elapsed
        + seal_elapsed
        + close_elapsed
        + reopen_elapsed
        + verify_elapsed;
    let total_elapsed = ack_elapsed + lifecycle_elapsed;
    let ack_secs = ack_elapsed.as_secs_f64().max(1e-12);
    let life_secs = total_elapsed.as_secs_f64().max(1e-12);
    let on_disk = dir_size_bytes(&store_path).unwrap_or(0);

    Ok(AckFinalizeResult {
        cell: cfg.cell,
        keys: keys_written,
        payload_size: cfg.payload_size,
        concurrency: workers,
        ack_elapsed,
        drain_elapsed,
        seal_elapsed,
        close_elapsed,
        reopen_elapsed,
        verify_elapsed,
        lifecycle_elapsed: total_elapsed,
        acknowledged_write_ops_per_sec: keys_written as f64 / ack_secs,
        lifecycle_ops_per_sec: keys_written as f64 / life_secs,
        reopen_exact,
        logical_bytes: keys_written.saturating_mul(cfg.payload_size as u64),
        on_disk_bytes: on_disk,
        timestamps: Timestamps {
            workload_start_unix_ns,
            last_successful_ack_unix_ns,
            drain_complete_unix_ns,
            seal_complete_unix_ns,
            close_complete_unix_ns,
            reopen_complete_unix_ns,
            verification_complete_unix_ns,
        },
    })
}

fn run_raw_mimic(cfg: &AckFinalizeConfig) -> Result<AckFinalizeResult, String> {
    let ops = cfg
        .target_bytes
        .div_ceil(cfg.payload_size as u64)
        .max(1)
        .min(PEER_OPS.saturating_mul(4));
    // Mimic uses peer-calibrated encoded size, not logical payload size.
    let data_bytes_per_op = PEER_DATA_BYTES_PER_OP;
    let data_path = cfg.work.join("data.seg");
    let _ = fs::remove_file(&data_path);
    fs::create_dir_all(&cfg.work).map_err(|e| format!("create mimic work: {e}"))?;

    let data_buf = fill_payload(data_bytes_per_op, cfg.seed);
    let expected_hash = body_hash(&data_buf);

    let mut data = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&data_path)
        .map_err(|e| format!("open data: {e}"))?;

    let workload_start_unix_ns = unix_ns_now();
    let t_workload = Instant::now();
    let mut off = 0u64;
    for _ in 0..ops {
        data.seek(SeekFrom::Start(off))
            .map_err(|e| format!("seek: {e}"))?;
        data.write_all(&data_buf)
            .map_err(|e| format!("write_all: {e}"))?;
        off = off.saturating_add(data_bytes_per_op as u64);
    }
    let last_successful_ack_unix_ns = unix_ns_now();
    let ack_elapsed = t_workload.elapsed();

    let t_drain = Instant::now();
    data.flush().map_err(|e| format!("flush (drain): {e}"))?;
    let drain_complete_unix_ns = unix_ns_now();
    let drain_elapsed = t_drain.elapsed();

    // No store seal — fsync stands in as the durability boundary for mimic.
    let t_seal = Instant::now();
    data.sync_all()
        .map_err(|e| format!("sync_all (seal substitute) failed (fail-closed): {e}"))?;
    let seal_complete_unix_ns = unix_ns_now();
    let seal_elapsed = t_seal.elapsed();

    let t_close = Instant::now();
    drop(data);
    let close_complete_unix_ns = unix_ns_now();
    let close_elapsed = t_close.elapsed();

    let t_reopen = Instant::now();
    let mut f = File::open(&data_path).map_err(|e| format!("reopen mimic failed: {e}"))?;
    let reopen_complete_unix_ns = unix_ns_now();
    let reopen_elapsed = t_reopen.elapsed();

    let t_verify = Instant::now();
    let meta_len = f
        .metadata()
        .map_err(|e| format!("mimic metadata: {e}"))?
        .len();
    let expect_len = ops.saturating_mul(data_bytes_per_op as u64);
    if meta_len != expect_len {
        return Err(format!(
            "mimic reopen length mismatch (fail-closed): got {meta_len} want {expect_len}"
        ));
    }
    // Spot-check first and last op bodies (full scan of 256 MiB encoded is
    // unnecessary for length-exact grow mimic; hash endpoints for ledger shape).
    let mut buf = vec![0u8; data_bytes_per_op];
    f.seek(SeekFrom::Start(0))
        .map_err(|e| format!("seek verify start: {e}"))?;
    use std::io::Read;
    f.read_exact(&mut buf)
        .map_err(|e| format!("read first op: {e}"))?;
    if body_hash(&buf) != expected_hash {
        return Err("mimic first-op hash mismatch (fail-closed)".into());
    }
    if ops > 1 {
        f.seek(SeekFrom::Start(
            (ops - 1).saturating_mul(data_bytes_per_op as u64),
        ))
        .map_err(|e| format!("seek verify last: {e}"))?;
        f.read_exact(&mut buf)
            .map_err(|e| format!("read last op: {e}"))?;
        if body_hash(&buf) != expected_hash {
            return Err("mimic last-op hash mismatch (fail-closed)".into());
        }
    }
    let verification_complete_unix_ns = unix_ns_now();
    let verify_elapsed = t_verify.elapsed();
    drop(f);

    let lifecycle_elapsed = drain_elapsed
        + seal_elapsed
        + close_elapsed
        + reopen_elapsed
        + verify_elapsed;
    let total_elapsed = ack_elapsed + lifecycle_elapsed;
    let ack_secs = ack_elapsed.as_secs_f64().max(1e-12);
    let life_secs = total_elapsed.as_secs_f64().max(1e-12);

    Ok(AckFinalizeResult {
        cell: cfg.cell,
        keys: ops,
        payload_size: cfg.payload_size,
        concurrency: 1,
        ack_elapsed,
        drain_elapsed,
        seal_elapsed,
        close_elapsed,
        reopen_elapsed,
        verify_elapsed,
        lifecycle_elapsed: total_elapsed,
        acknowledged_write_ops_per_sec: ops as f64 / ack_secs,
        lifecycle_ops_per_sec: ops as f64 / life_secs,
        reopen_exact: true,
        logical_bytes: ops.saturating_mul(cfg.payload_size as u64),
        on_disk_bytes: expect_len,
        timestamps: Timestamps {
            workload_start_unix_ns,
            last_successful_ack_unix_ns,
            drain_complete_unix_ns,
            seal_complete_unix_ns,
            close_complete_unix_ns,
            reopen_complete_unix_ns,
            verification_complete_unix_ns,
        },
    })
}

fn result_json(r: &AckFinalizeResult) -> Value {
    json!({
        "kind": "ack_finalize_cell",
        "disclosure": DISCLOSURE,
        "cell": r.cell.slug(),
        "cell_label": r.cell.label(),
        "keys": r.keys,
        "payload_size": r.payload_size,
        "concurrency": r.concurrency,
        "logical_bytes": r.logical_bytes,
        "on_disk_bytes": r.on_disk_bytes,
        "on_disk_human": format_bytes(r.on_disk_bytes),
        "acknowledged_write_ops_per_sec": r.acknowledged_write_ops_per_sec,
        "ack_elapsed_ns": r.ack_elapsed.as_nanos() as u64,
        "ack_elapsed_ms": r.ack_elapsed.as_secs_f64() * 1000.0,
        "drain_elapsed_ns": r.drain_elapsed.as_nanos() as u64,
        "seal_elapsed_ns": r.seal_elapsed.as_nanos() as u64,
        "seal_elapsed_ms": r.seal_elapsed.as_secs_f64() * 1000.0,
        "close_elapsed_ns": r.close_elapsed.as_nanos() as u64,
        "reopen_elapsed_ns": r.reopen_elapsed.as_nanos() as u64,
        "verify_elapsed_ns": r.verify_elapsed.as_nanos() as u64,
        "lifecycle_elapsed_ns": r.lifecycle_elapsed.as_nanos() as u64,
        "lifecycle_ops_per_sec": r.lifecycle_ops_per_sec,
        "reopen_exact": r.reopen_exact,
        "timestamps_unix_ns": {
            "workload_start": r.timestamps.workload_start_unix_ns as u64,
            "last_successful_ack": r.timestamps.last_successful_ack_unix_ns as u64,
            "drain_complete": r.timestamps.drain_complete_unix_ns as u64,
            "seal_complete": r.timestamps.seal_complete_unix_ns as u64,
            "close_complete": r.timestamps.close_complete_unix_ns as u64,
            "reopen_complete": r.timestamps.reopen_complete_unix_ns as u64,
            "verification_complete": r.timestamps.verification_complete_unix_ns as u64,
        },
        // Ambiguous peer-era name intentionally not emitted as ops_per_sec.
        "note": "Use acknowledged_write_ops_per_sec vs lifecycle_ops_per_sec; no ambiguous ops_per_sec.",
    })
}

fn render_markdown_table(rows: &[AckFinalizeResult]) -> String {
    let mut out = String::from(
        "| Cell | Ack TPS | Ack time | Seal time | Lifecycle TPS | Reopen exact |\n\
         |---|---:|---:|---:|---:|---|\n",
    );
    let order = [
        MeasureCell::RealFull,
        MeasureCell::RealSkipIndex,
        MeasureCell::Discard,
        MeasureCell::RawMimic,
    ];
    for cell in order {
        if let Some(r) = rows.iter().find(|r| r.cell == cell) {
            out.push_str(&format!(
                "| {} | {:.0} | {:.2} s | {:.2} s | {:.0} | {} |\n",
                r.cell.label(),
                r.acknowledged_write_ops_per_sec,
                r.ack_elapsed.as_secs_f64(),
                r.seal_elapsed.as_secs_f64(),
                r.lifecycle_ops_per_sec,
                if r.reopen_exact { "yes" } else { "no" },
            ));
        } else {
            out.push_str(&format!(
                "| {} | — | — | — | — | — |\n",
                cell.label()
            ));
        }
    }
    out
}
