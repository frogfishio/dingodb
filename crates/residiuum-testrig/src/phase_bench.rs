//! Diagnostic phase breakdown: pure compute vs raw write vs store puts.
//!
//! Purpose: falsify "we're Blake/CPU bound" when Activity Monitor shows
//! ~40% of one core. If pure Blake is huge ops/s and store put is slow with
//! low process CPU, wall time is spent **waiting** (typically blocking I/O),
//! not computing.

use residiuum_format::{
    body_hash, encode_frame_into, FrameHeader, FrameKind, WIRE_MAJOR, WIRE_MINOR,
};
use residiuum_store::{DurabilityMode, Store};
use serde_json::json;
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PhaseBenchConfig {
    pub work: PathBuf,
    pub ops: u64,
    pub payload_size: usize,
    pub batch: usize,
    pub json_out: bool,
}

#[derive(Debug, Clone)]
struct Phase {
    name: &'static str,
    wall_ms: f64,
    ops_per_sec: f64,
    mib_per_sec: f64,
    note: String,
}

fn fill_payload(buf: &mut [u8], seed: u64) {
    let mut state = seed ^ 0xD1_160_B17_u64;
    for (i, b) in buf.iter_mut().enumerate() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        *b = ((state >> 33) as u8).wrapping_add((i & 0xff) as u8);
    }
}

fn mib_s(bytes: u64, secs: f64) -> f64 {
    (bytes as f64 / (1024.0 * 1024.0)) / secs.max(1e-12)
}

fn phase(name: &'static str, ops: u64, bytes_per_op: u64, wall_s: f64, note: impl Into<String>) -> Phase {
    Phase {
        name,
        wall_ms: wall_s * 1000.0,
        ops_per_sec: ops as f64 / wall_s.max(1e-12),
        mib_per_sec: mib_s(ops.saturating_mul(bytes_per_op), wall_s),
        note: note.into(),
    }
}

/// Best-effort process CPU samples via `ps` (macOS: 100% ≈ one core).
fn sample_cpu_pct() -> Option<f64> {
    let pid = std::process::id();
    let out = std::process::Command::new("ps")
        .args(["-o", "%cpu=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse().ok()
}

pub fn run_phase_bench(cfg: &PhaseBenchConfig) -> Result<(), String> {
    if cfg.ops == 0 {
        return Err("ops must be > 0".into());
    }
    if cfg.payload_size == 0 {
        return Err("payload-size must be > 0".into());
    }
    let batch = cfg.batch.max(1);
    fs::create_dir_all(&cfg.work).map_err(|e| format!("create work: {e}"))?;

    let mut payload = vec![0u8; cfg.payload_size];
    fill_payload(&mut payload, 42);
    let payload_bytes = cfg.payload_size as u64;
    let ops = cfg.ops;

    let mut phases: Vec<Phase> = Vec::new();

    // --- 1. Pure BLAKE3 (format body_hash) — should saturate ~1 core if "Blake bound" ---
    {
        let t0 = Instant::now();
        let mut sink = [0u8; 32];
        for _ in 0..ops {
            sink = body_hash(&payload);
        }
        // prevent optimize-away
        if sink[0] == 0xFF && sink[1] == 0x00 {
            eprintln!("unlikely");
        }
        let s = t0.elapsed().as_secs_f64();
        phases.push(phase(
            "blake3_body_hash_only",
            ops,
            payload_bytes,
            s,
            "pure user CPU; if store is Blake-bound, put rates should be near this order",
        ));
    }

    // --- 1b. Full frame encode into growing Vec (Blake + prefix/suffix + memcpy) ---
    {
        let env = b"\xa0"; // empty CBOR map
        let mut event_id = [0u8; 16];
        let mut buf = Vec::with_capacity(ops as usize * (cfg.payload_size + 256));
        let t0 = Instant::now();
        for i in 0..ops {
            event_id[0] = (i & 0xff) as u8;
            event_id[1] = ((i >> 8) & 0xff) as u8;
            let header = FrameHeader {
                wire_major: WIRE_MAJOR,
                wire_minor: WIRE_MINOR,
                frame_kind: FrameKind::ItemEvent.as_u8(),
                flags: Default::default(),
                envelope_len: env.len() as u32,
                body_len: payload.len() as u64,
                logical_len: payload.len() as u64,
                writer_sequence: i,
                event_id,
            };
            encode_frame_into(&mut buf, &header, env, &payload)
                .map_err(|e| format!("encode_frame_into: {e}"))?;
        }
        let s = t0.elapsed().as_secs_f64();
        let _ = buf.len();
        phases.push(phase(
            "encode_frame_into_growing_vec",
            ops,
            payload_bytes,
            s,
            "format encode only (includes Blake); no store index, no OS write",
        ));
    }

    // --- 1c. Seek+write_all per op (store tail pattern) vs append-only write_all ---
    {
        let frame_pad = 256usize;
        let mut frame = vec![0u8; cfg.payload_size + frame_pad];
        frame[..cfg.payload_size].copy_from_slice(&payload);
        let path = cfg.work.join("raw-seek-write.bin");
        let _ = fs::remove_file(&path);
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("open seek-write: {e}"))?;
        let mut off = 0u64;
        let t0 = Instant::now();
        for _ in 0..ops {
            f.seek(SeekFrom::Start(off))
                .map_err(|e| format!("seek: {e}"))?;
            f.write_all(&frame).map_err(|e| format!("seek write: {e}"))?;
            off += frame.len() as u64;
        }
        f.flush().map_err(|e| format!("seek flush: {e}"))?;
        let s = t0.elapsed().as_secs_f64();
        phases.push(phase(
            "raw_seek_plus_write_all_per_op",
            ops,
            frame.len() as u64,
            s,
            "mimics write_segment_tail: seek(durable_len)+write_all each put",
        ));
        drop(f);
        let _ = fs::remove_file(&path);
    }

    // --- 2. Raw sequential write of payload-sized chunks (no Residiuum) ---
    {
        let path = cfg.work.join("raw-seq.bin");
        let _ = fs::remove_file(&path);
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("open raw-seq: {e}"))?;
        let t0 = Instant::now();
        for _ in 0..ops {
            f.write_all(&payload).map_err(|e| format!("raw write: {e}"))?;
        }
        f.flush().map_err(|e| format!("raw flush: {e}"))?;
        let s = t0.elapsed().as_secs_f64();
        phases.push(phase(
            "raw_write_all_payload_only",
            ops,
            payload_bytes,
            s,
            "OS write path only (no fsync); device + page cache ceiling for small writes",
        ));
        drop(f);
        let _ = fs::remove_file(&path);
    }

    // --- 3. Raw write of ~frame-sized chunks (payload + ~256 B overhead stand-in) ---
    {
        let frame_pad = 256usize;
        let mut frame = vec![0u8; cfg.payload_size + frame_pad];
        frame[..cfg.payload_size].copy_from_slice(&payload);
        let path = cfg.work.join("raw-frame.bin");
        let _ = fs::remove_file(&path);
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("open raw-frame: {e}"))?;
        let t0 = Instant::now();
        for _ in 0..ops {
            f.write_all(&frame).map_err(|e| format!("raw frame write: {e}"))?;
        }
        f.flush().map_err(|e| format!("raw frame flush: {e}"))?;
        let s = t0.elapsed().as_secs_f64();
        phases.push(phase(
            "raw_write_all_frame_sized",
            ops,
            (cfg.payload_size + frame_pad) as u64,
            s,
            "closer to segment tail write size (~payload+envelope)",
        ));
        drop(f);
        let _ = fs::remove_file(&path);
    }

    // --- 4. Store Memory puts (visibility only — no segment file append) ---
    {
        let store_path = cfg.work.join("mem-store");
        let _ = fs::remove_dir_all(&store_path);
        let mut store =
            Store::create(&store_path).map_err(|e| format!("create mem store: {e}"))?;
        let t0 = Instant::now();
        for i in 0..ops {
            let key = format!("m/{i:020}");
            store
                .put(&key, &payload, DurabilityMode::Memory)
                .map_err(|e| format!("memory put: {e}"))?;
        }
        let s = t0.elapsed().as_secs_f64();
        phases.push(phase(
            "store_put_memory",
            ops,
            payload_bytes,
            s,
            "index/visibility path only — NO durable segment write_all",
        ));
        drop(store);
        let _ = fs::remove_dir_all(&store_path);
    }

    // --- 5. Store Buffered single puts (general load) ---
    {
        let store_path = cfg.work.join("buf1-store");
        let _ = fs::remove_dir_all(&store_path);
        let mut store =
            Store::create(&store_path).map_err(|e| format!("create buf1 store: {e}"))?;
        store.set_seal_threshold(64 * 1024 * 1024);
        store.enable_boundary_probe();
        let cpu0 = sample_cpu_pct();
        let t0 = Instant::now();
        for i in 0..ops {
            let key = format!("b/{i:020}");
            store
                .put(&key, &payload, DurabilityMode::Buffered)
                .map_err(|e| format!("buffered put: {e}"))?;
        }
        let s = t0.elapsed().as_secs_f64();
        let cpu1 = sample_cpu_pct();
        let snap = store.boundary_snapshot();
        let wall_ms = s * 1000.0;
        let prep_ms = snap.prep_latency.sum_ns as f64 / 1e6;
        let enc_ms = snap.encode_latency.sum_ns as f64 / 1e6;
        let app_ms = snap.append_latency.sum_ns as f64 / 1e6;
        let pub_ms = snap.publish_latency.sum_ns as f64 / 1e6;
        let post_ms = snap.post_latency.sum_ns as f64 / 1e6;
        let wr_ms = snap.write_latency.sum_ns as f64 / 1e6;
        let sync_ms = snap.sync_latency.sum_ns as f64 / 1e6;
        let accounted_ms = prep_ms + enc_ms + app_ms + pub_ms + post_ms + wr_ms + sync_ms;
        let other_ms = (wall_ms - accounted_ms).max(0.0);
        let pct = |part: f64| 100.0 * part / wall_ms.max(1e-9);
        let probe_note = format!(
            " MODE_A breakdown: prep sum_ms={prep_ms:.1} mean_us={:.1} ({:.0}%) | encode_env sum_ms={enc_ms:.1} mean_us={:.1} ({:.0}%) | append_frame sum_ms={app_ms:.1} mean_us={:.1} ({:.0}%) | publish_index sum_ms={pub_ms:.1} mean_us={:.1} ({:.0}%) | post_derived sum_ms={post_ms:.1} mean_us={:.1} ({:.0}%) | file_write sum_ms={wr_ms:.1} mean_us={:.1} ({:.0}%) | file_sync n={} | other_ms={other_ms:.1} ({:.0}%) | accounted={accounted_ms:.1}/{wall_ms:.1}ms",
            snap.prep_latency.mean_ns() / 1e3,
            pct(prep_ms),
            snap.encode_latency.mean_ns() / 1e3,
            pct(enc_ms),
            snap.append_latency.mean_ns() / 1e3,
            pct(app_ms),
            snap.publish_latency.mean_ns() / 1e3,
            pct(pub_ms),
            snap.post_latency.mean_ns() / 1e3,
            pct(post_ms),
            snap.write_latency.mean_ns() / 1e3,
            pct(wr_ms),
            snap.sync_latency.samples,
            pct(other_ms),
        );
        phases.push(phase(
            "store_put_buffered_batch1",
            ops,
            payload_bytes,
            s,
            format!(
                "wall_ms={wall_ms:.1} probe_encode+append+publish+write_ms={accounted_ms:.1} ({:.0}% of wall); ps%cpu≈{:?}→{:?};{}",
                100.0 * accounted_ms / wall_ms.max(1.0),
                cpu0,
                cpu1,
                probe_note
            ),
        ));
        drop(store);
        let _ = fs::remove_dir_all(&store_path);
    }

    // --- 6. Store Buffered put_many in batches ---
    {
        let store_path = cfg.work.join("bufn-store");
        let _ = fs::remove_dir_all(&store_path);
        let mut store =
            Store::create(&store_path).map_err(|e| format!("create bufn store: {e}"))?;
        store.set_seal_threshold(64 * 1024 * 1024);
        let t0 = Instant::now();
        let mut i = 0u64;
        while i < ops {
            let end = (i + batch as u64).min(ops);
            let keys: Vec<String> = (i..end).map(|k| format!("n/{k:020}")).collect();
            let items: Vec<(&str, &[u8])> = keys.iter().map(|k| (k.as_str(), payload.as_slice())).collect();
            store
                .put_many(&items, DurabilityMode::Buffered)
                .map_err(|e| format!("put_many: {e}"))?;
            i = end;
        }
        let s = t0.elapsed().as_secs_f64();
        phases.push(phase(
            "store_put_many_buffered",
            ops,
            payload_bytes,
            s,
            format!("batch={batch}; one tail write per batch"),
        ));
        drop(store);
        let _ = fs::remove_dir_all(&store_path);
    }

    // Interpretation helper rates
    let blake = phases.iter().find(|p| p.name == "blake3_body_hash_only");
    let raw = phases.iter().find(|p| p.name == "raw_write_all_payload_only");
    let mem = phases.iter().find(|p| p.name == "store_put_memory");
    let buf1 = phases.iter().find(|p| p.name == "store_put_buffered_batch1");

    let mut interpretation = Vec::new();
    if let (Some(b), Some(p)) = (blake, buf1) {
        let ratio = p.ops_per_sec / b.ops_per_sec.max(1.0);
        if ratio < 0.15 {
            interpretation.push(format!(
                "Buffered put is only {:.1}% of pure-Blake ops/s — NOT Blake-bound (Blake is far faster).",
                100.0 * ratio
            ));
        } else {
            interpretation.push(format!(
                "Buffered put is {:.0}% of pure-Blake ops/s — Blake-scale cost may matter.",
                100.0 * ratio
            ));
        }
    }
    if let (Some(m), Some(p)) = (mem, buf1) {
        if p.ops_per_sec < m.ops_per_sec * 0.5 {
            interpretation.push(format!(
                "Memory put {:.0} ops/s vs Buffered {:.0} — large gap ⇒ time spent in segment OS write path (stall/wait), not only index CPU.",
                m.ops_per_sec, p.ops_per_sec
            ));
        } else {
            interpretation.push(format!(
                "Memory {:.0} ≈ Buffered {:.0} ops/s — dominant cost is pre-disk (index/encode), not write_all.",
                m.ops_per_sec, p.ops_per_sec
            ));
        }
    }
    if let (Some(r), Some(p)) = (raw, buf1) {
        interpretation.push(format!(
            "Raw write_all {:.0} ops/s vs Buffered put {:.0} — store adds {:.1}× wall vs raw same-size writes.",
            r.ops_per_sec,
            p.ops_per_sec,
            r.ops_per_sec / p.ops_per_sec.max(1.0)
        ));
    }
    interpretation.push(
        "Low process %CPU + low disk gauges with large Memory→Buffered gap = classic blocking I/O wait (thread not runnable), not pure compute.".into(),
    );

    let report = json!({
        "prong": "phase_bench",
        "ok": true,
        "work": cfg.work.display().to_string(),
        "ops": ops,
        "payload_size": cfg.payload_size,
        "batch": batch,
        "phases": phases.iter().map(|p| json!({
            "name": p.name,
            "wall_ms": p.wall_ms,
            "ops_per_sec": p.ops_per_sec,
            "logical_mib_per_sec": p.mib_per_sec,
            "note": p.note,
        })).collect::<Vec<_>>(),
        "interpretation": interpretation,
        "disclosure": "Diagnostic only — not a published SLO.",
    });

    if cfg.json_out {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("=== phase-bench (falsify Blake-bound narrative) ===");
        println!(
            "work={} ops={} payload={} batch={}",
            cfg.work.display(),
            ops,
            cfg.payload_size,
            batch
        );
        println!();
        println!(
            "{:<32} {:>10} {:>12} {:>12}  {}",
            "phase", "wall_ms", "ops/s", "MiB/s", "note"
        );
        for p in &phases {
            println!(
                "{:<32} {:>10.1} {:>12.0} {:>12.1}  {}",
                p.name, p.wall_ms, p.ops_per_sec, p.mib_per_sec, p.note
            );
        }
        println!();
        println!("interpretation:");
        for line in &interpretation {
            println!("  • {line}");
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn _touch(path: &Path) -> std::io::Result<File> {
    File::create(path)
}