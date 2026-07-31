//! Real residiuum-store L4/L5/L6 cell driver (feature `store-driver`).
//!
//! **Smoke** may use small op budgets for unit/CI.
//! **Qualification** runs until SPEC §6.4 duration **and** byte floors are met
//! (unless safety max would be crossed); never silently caps to 4–16 ops.

use super::emitter::{emit_plan_from_receipts, WriteReceiptFact, STORE_SEAM_EMITTER_FROM_RECEIPTS};
use super::kinds::{DriverKind, MeasurementSurface};
use super::{cell_store_path, DriverCellReport, DriverError, DriverRunConfig};
use crate::campaign::RunClass;
use crate::envelope::{WindowDetector, WindowSample};
use crate::matrix::{AckLedger, CellRunReport, DurabilityMode as MatrixDur, MatrixError};
use crate::runner::RunBudgets;
use crate::workload::SizeSampler;
use residiuum_store::{DurabilityMode as StoreDur, Store};
use std::fs;
use std::time::{Duration, Instant};

pub fn run_real_store(cfg: &DriverRunConfig) -> Result<DriverCellReport, DriverError> {
    let work_root = cfg
        .work_root
        .as_ref()
        .ok_or_else(|| DriverError::Msg("real_store requires work_root".into()))?;

    if cfg.durability_mutant {
        return Err(MatrixError::DurabilityMutant(
            "durable label with memory barrier is a different product profile".into(),
        )
        .into());
    }

    let run_class = cfg.run_class_parsed();
    let store_path = cell_store_path(work_root, &cfg.cell.cell_id, &format!("s{:x}", cfg.seed));
    if store_path.exists() {
        let _ = fs::remove_dir_all(&store_path);
    }
    fs::create_dir_all(store_path.parent().unwrap_or(work_root))?;

    let mut store = Store::create(&store_path).map_err(|e| DriverError::Store(e.to_string()))?;
    let store_dur = map_durability(cfg.cell.durability);

    let sampler = match cfg.cell.distribution {
        Some(d) => SizeSampler::distribution(d),
        None => SizeSampler::fixed(cfg.cell.payload_size),
    };

    let min_dur = Duration::from_secs(run_class.min_duration_secs());
    let min_bytes = run_class.min_logical_bytes();
    // Safety ceilings from runner budgets (never cross free-space policy here —
    // caller preflight is responsible; we still cap wall/bytes for runaway).
    let safety = RunBudgets::default();
    let max_dur = Duration::from_secs(
        safety
            .max_duration_secs
            .max(run_class.min_duration_secs()),
    );
    let max_bytes = safety.max_bytes.max(run_class.min_logical_bytes());

    let mut ledger = AckLedger::new();
    let mut facts = Vec::new();
    let mut logical_bytes = 0u64;
    let mut last_seg: Option<[u8; 16]> = None;
    let mut seg_gen = 0u32;
    let mut records = Vec::new();
    let mut window_samples = Vec::new();
    let mut messages = Vec::new();
    messages.push(format!(
        "driver=real_store run_class={} path={}",
        run_class.as_str(),
        store_path.display()
    ));
    messages.push(format!("layer={}", cfg.cell.layer.as_str()));
    messages.push(format!(
        "floors: min_duration_secs={} min_logical_bytes={}",
        run_class.min_duration_secs(),
        run_class.min_logical_bytes()
    ));

    let t0 = Instant::now();
    let mut seq: u64 = 0;
    // Smoke: fixed op budget. Qualification: time+byte floors (no 4–16 op cap).
    let smoke_ops_limit = if run_class.allows_smoke_op_cap() {
        Some(cfg.cell.op_count.min(RunClass::SMOKE_MAX_OPS).max(1))
    } else {
        None
    };

    loop {
        if let Some(lim) = smoke_ops_limit {
            if seq >= lim {
                break;
            }
        } else {
            let elapsed = t0.elapsed();
            let floors_met = elapsed >= min_dur && logical_bytes >= min_bytes;
            if floors_met {
                break;
            }
            if elapsed >= max_dur || logical_bytes >= max_bytes {
                messages.push(format!(
                    "stopped at safety ceiling elapsed_s={} logical_bytes={} (floors may be unmet)",
                    elapsed.as_secs(),
                    logical_bytes
                ));
                break;
            }
        }

        ledger.record_attempt();
        let plen = if cfg.cell.distribution.is_some() {
            sampler.size_at(seq)
        } else {
            cfg.cell.payload_size
        };
        // Smoke may shrink payload for CI; qualification uses planned payload.
        let plen = if run_class.allows_smoke_op_cap() {
            plen.min(64 * 1024)
        } else {
            plen
        };

        let mut body = vec![0u8; plen as usize];
        let mut x = cfg.seed.wrapping_add(seq);
        for b in &mut body {
            x = x
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(1);
            *b = (x >> 33) as u8;
        }
        let subject = format!("pqh11-{seq:08x}");

        match store.put(&subject, &body, store_dur) {
            Ok(receipt) => {
                ledger.record_admit();
                if cfg.digest_mutant && seq == 0 {
                    ledger.record_ack_mutant_wrong_digest(seq, seq, plen, 0);
                } else {
                    ledger.record_ack(seq, seq, plen, 0);
                }
                records.push((seq, seq, plen, 0u32));
                logical_bytes = logical_bytes.saturating_add(plen);

                let rotate = match last_seg {
                    Some(prev) => prev != receipt.segment_id,
                    None => false,
                };
                if rotate {
                    seg_gen = seg_gen.saturating_add(1);
                }
                last_seg = Some(receipt.segment_id);

                let chunked = receipt.chunk_count > 0;
                let physical_len = plen.saturating_add(96);
                // Cap facts for plan emission size (still redacted).
                if facts.len() < 4096 {
                    facts.push(WriteReceiptFact {
                        logical_len: receipt.logical_len.max(plen),
                        physical_len,
                        durability: receipt.durability.as_str().into(),
                        segment_gen: seg_gen,
                        segment_rotate: rotate,
                        chunked,
                        chunk_count: receipt.chunk_count,
                    });
                }
            }
            Err(e) => {
                ledger.record_fail();
                messages.push(format!("put failed seq={seq}: {e}"));
            }
        }

        // Window sample every 32 acks or every ~100ms of wall for steady-state.
        if seq > 0 && (seq % 32 == 0 || t0.elapsed().as_millis() % 100 < 5) {
            let e2e = t0.elapsed().as_nanos().max(1) as f64;
            let bps = (logical_bytes as f64) * 1e9 / e2e;
            window_samples.push(WindowSample {
                throughput_bytes_per_sec: bps,
            });
        }
        seq = seq.saturating_add(1);
    }
    let e2e_ns = t0.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;

    if let Err(e) = ledger.verify_correctness() {
        return Err(MatrixError::InvalidCorrectness(e.to_string()).into());
    }
    // Reopen digest only over bounded record sample for large qualification runs.
    let sample_n = records.len().min(256);
    let reopen_ok = ledger.verify_reopen(&records[..sample_n]).is_ok();
    if !reopen_ok {
        return Err(MatrixError::InvalidCorrectness("reopen digest mismatch".into()).into());
    }

    let mut get_ok = 0u64;
    for s in 0..seq.min(4) {
        let subject = format!("pqh11-{s:08x}");
        if let Ok(Some(_)) = store.get(&subject) {
            get_ok += 1;
        }
    }
    messages.push(format!("sample_get_ok={get_ok} ops_done={seq}"));

    drop(store);
    let reopen_store = Store::open(&store_path).map_err(|e| DriverError::Store(e.to_string()))?;
    let live = reopen_store.live_count();
    messages.push(format!("reopen_live_count={live}"));
    drop(reopen_store);

    let plan = emit_plan_from_receipts(
        &cfg.cell.cell_id,
        &facts,
        32 * 1024,
        1024 * 1024,
        cfg.cell.batch_size.max(1),
    );
    plan.assert_redacted_json()
        .map_err(|e| DriverError::Msg(e))?;

    let tput = if e2e_ns == 0 {
        0.0
    } else {
        (logical_bytes as f64) * 1e9 / (e2e_ns as f64)
    };

    let window = WindowDetector::default().classify(&window_samples);
    let window_s = format!("{window:?}").to_ascii_lowercase();

    // Qualification without steady-state → inconclusive (SPEC §6.4).
    let floors_met = t0.elapsed() >= min_dur && logical_bytes >= min_bytes;
    let mut validity = "valid".to_string();
    if run_class.may_emit_bottleneck_verdict() {
        if !floors_met {
            validity = "inconclusive".into();
            messages.push("qualification floors not met → inconclusive".into());
        } else if !matches!(window, crate::envelope::WindowClass::Sustained) {
            validity = "inconclusive".into();
            messages.push(format!(
                "steady-state not demonstrated (window={window_s}) → inconclusive"
            ));
        }
    } else {
        messages.push(
            "smoke/diagnostic: not a qualification claim; no product bottleneck".into(),
        );
    }

    let cell = CellRunReport {
        cell_id: cfg.cell.cell_id.clone(),
        layer: cfg.cell.layer.as_str().into(),
        durability: cfg.cell.durability.as_str().into(),
        validity,
        attempted: ledger.attempted,
        admitted: ledger.admitted,
        acknowledged: ledger.acknowledged,
        failed: ledger.failed,
        logical_bytes_ack: logical_bytes,
        e2e_ns_proxy: e2e_ns,
        throughput_bytes_per_sec_proxy: tput,
        window: window_s,
        shadow_planned_bytes: plan.planned_bytes,
        shadow_completed_bytes: plan.planned_bytes,
        features: cfg.cell.features.name.clone(),
        interference: cfg.cell.interference.kind.as_str().into(),
        messages: messages.clone(),
        reopen_ok: true,
    };

    let surface = MeasurementSurface::RealStoreUncontrolled;
    let mut notes = messages;
    notes.push(format!("plan_source={STORE_SEAM_EMITTER_FROM_RECEIPTS}"));
    notes.push(format!(
        "run_class={} floors_met={} (smoke caps ops; qualification does not)",
        run_class.as_str(),
        floors_met
    ));

    Ok(DriverCellReport {
        cell,
        driver_kind: DriverKind::RealStore,
        measurement_surface: surface,
        product_claim_eligible: false, // campaign layer decides with --controlled
        plan: Some(plan),
        plan_source: STORE_SEAM_EMITTER_FROM_RECEIPTS.into(),
        notes,
    })
}

fn map_durability(d: MatrixDur) -> StoreDur {
    match d {
        MatrixDur::Memory => StoreDur::Memory,
        MatrixDur::Buffered => StoreDur::Buffered,
        MatrixDur::Durable => StoreDur::Durable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::{
        DatabaseState, FeatureProfile, InterferenceProfile, LayerProfile, MatrixCell,
    };

    #[test]
    fn real_store_smoke_cell() {
        let dir = tempfile::tempdir().unwrap();
        let cell = MatrixCell {
            cell_id: "L4-durable-s1024-c1-o1-0".into(),
            layer: LayerProfile::L4,
            durability: MatrixDur::Buffered,
            payload_size: 256,
            distribution: None,
            concurrency: 1,
            outstanding: 1,
            batch_size: 1,
            shards: 1,
            db_state: DatabaseState::Empty,
            features: FeatureProfile::l4_minimal(),
            interference: InterferenceProfile::absent(),
            op_count: 8,
            order_rank: 0,
        };
        let report = run_real_store(&DriverRunConfig {
            cell,
            seed: 42,
            kind: DriverKind::RealStore,
            work_root: Some(dir.path().to_path_buf()),
            durability_mutant: false,
            digest_mutant: false,
            run_class: "smoke".into(),
        })
        .expect("real store smoke cell");
        assert_eq!(report.driver_kind, DriverKind::RealStore);
        assert!(!report.product_claim_eligible);
        assert_eq!(report.cell.validity, "valid");
        assert!(report.cell.acknowledged > 0);
        assert!(report.cell.acknowledged <= RunClass::SMOKE_MAX_OPS);
        assert!(report.plan.is_some());
        report.plan.as_ref().unwrap().assert_redacted_json().unwrap();
        assert!(report.notes.iter().any(|n| n.contains("run_class=smoke")));
    }

    #[test]
    fn qualification_does_not_use_smoke_op_cap_path() {
        // Structural: qualification run_class reports floors and does not claim
        // smoke op budget. Full 120s/512MiB is not executed in unit tests.
        assert!(!RunClass::Qualification.allows_smoke_op_cap());
        assert!(RunClass::Qualification.min_duration_secs() >= 120);
        assert!(RunClass::Qualification.min_logical_bytes() >= 512 * 1024 * 1024);
    }
}