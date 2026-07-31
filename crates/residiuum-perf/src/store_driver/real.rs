//! Real residiuum-store L4/L5/L6 cell driver (feature `store-driver`).
//!
//! **Smoke** may use small op budgets for unit/CI.
//! **Qualification** runs until SPEC §6.4 duration **and** byte floors are met
//! (unless safety max would be crossed); never silently caps to 4–16 ops.

use super::aggregates::{BoundaryAggregateSummary, ObserverOverheadReport};
use super::emitter::{emit_plan_from_store_boundary_events, STORE_SEAM_EMITTER_FROM_RECEIPTS};
use super::kinds::{DriverKind, MeasurementSurface};
use super::{cell_store_path, DriverCellReport, DriverError, DriverRunConfig};
use crate::campaign::RunClass;
use crate::envelope::{WindowDetector, WindowSample};
use crate::matrix::{AckLedger, CellRunReport, DurabilityMode as MatrixDur, MatrixError};
use crate::runner::RunBudgets;
use crate::workload::SizeSampler;
use residiuum_store::{BoundaryKind, DurabilityMode as StoreDur, Store};
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
    // Consume store-emitted boundary events (not receipt reconstruction).
    store.enable_boundary_probe();
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
    let mut logical_bytes = 0u64;
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

                // Cross-check receipt vs store probe policy only — do not rebuild events.
                let encoded_len = receipt.encoded_frame_len;
                if encoded_len == 0 && store_dur != StoreDur::Memory {
                    messages.push(format!(
                        "warn: missing encoded_frame_len at offset={} logical={}",
                        receipt.offset, plen
                    ));
                }
                if encoded_len > 0 && encoded_len < plen {
                    messages.push(format!(
                        "warn: encoded_frame_len={encoded_len} < logical={plen}"
                    ));
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

    // Drain store-native boundary instrumentation (exact counters + samples).
    let snap = store.take_boundary_snapshot();
    let cov = &snap.coverage;
    messages.push(format!(
        "boundary_coverage total_observed={} samples_retained={} samples_dropped={} capped={} capacity={}",
        cov.total_observed,
        cov.samples_retained,
        cov.samples_dropped,
        cov.sample_vector_capped,
        cov.sample_capacity
    ));
    if let Some(reason) = &cov.drop_reason {
        messages.push(format!("boundary_drop_reason={reason}"));
    }
    messages.push(format!(
        "boundary_counters append={} file_write={} file_sync={} publish={} rotate={} lifecycle={} req_bytes={} done_bytes={} write_lat_n={} sync_lat_n={}",
        snap.counters.count(BoundaryKind::AppendEncodedFrame),
        snap.counters.count(BoundaryKind::FileWrite),
        snap.counters.count(BoundaryKind::FileSync),
        snap.counters.count(BoundaryKind::PublishVisibility),
        snap.counters.count(BoundaryKind::SegmentRotate),
        snap.counters.count(BoundaryKind::LifecycleSeal),
        snap.counters.total_requested_bytes,
        snap.counters.total_completed_bytes,
        snap.write_latency.samples,
        snap.sync_latency.samples,
    ));
    messages.push(format!(
        "boundary_event_chain_digest={}",
        snap.event_chain_digest
    ));
    if cov.total_observed == 0 && seq > 0 {
        return Err(DriverError::Msg(
            "store boundary probe produced no observations; refuse reconstructed receipt stream"
                .into(),
        ));
    }

    drop(store);
    let reopen_store = Store::open(&store_path).map_err(|e| DriverError::Store(e.to_string()))?;
    let live = reopen_store.live_count();
    messages.push(format!("reopen_live_count={live}"));
    drop(reopen_store);

    // Exact aggregates always; lossless plan only when zero sample drops.
    let aggregates = BoundaryAggregateSummary::from_store_snapshot(&snap);
    let (plan, lossless_plan_eligible, plan_source) = if aggregates.lossless_plan_eligible {
        let plan = emit_plan_from_store_boundary_events(
            &cfg.cell.cell_id,
            &snap.samples,
            32 * 1024,
            1024 * 1024,
            cfg.cell.batch_size.max(1),
        );
        plan.assert_redacted_json()
            .map_err(|e| DriverError::Msg(e))?;
        messages.push(
            "lossless_plan=yes plan_source=store_boundary_probe (complete samples; not receipt reconstruction)"
                .into(),
        );
        (Some(plan), true, STORE_SEAM_EMITTER_FROM_RECEIPTS.to_string())
    } else {
        messages.push(format!(
            "lossless_plan=no: {}",
            aggregates
                .plan_replay_invalidate_reason
                .as_deref()
                .unwrap_or("sample drops")
        ));
        messages.push(
            "exact aggregates (counters/histograms/digest) remain valid; plan/replay claims withheld"
                .into(),
        );
        (
            None,
            false,
            "store_boundary_aggregates_only_v1".into(),
        )
    };
    messages.push(format!(
        "boundary_aggregates_digest={}",
        aggregates.event_chain_digest
    ));

    let planned_bytes = plan.as_ref().map(|p| p.planned_bytes).unwrap_or(0);

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
        shadow_planned_bytes: planned_bytes,
        shadow_completed_bytes: planned_bytes,
        features: cfg.cell.features.name.clone(),
        interference: cfg.cell.interference.kind.as_str().into(),
        messages: messages.clone(),
        reopen_ok: true,
    };

    let surface = MeasurementSurface::RealStoreUncontrolled;
    let mut notes = messages;
    notes.push(format!("plan_source={plan_source}"));
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
        plan,
        plan_source,
        lossless_plan_eligible,
        boundary_aggregates: Some(aggregates),
        notes,
    })
}

/// Matched probe-off / probe-on observer-overhead qualification (same seed/cell).
///
/// Runs the cell twice under separate store roots: once without the boundary
/// probe, once with it. Wall-time delta is the measured observer overhead.
pub fn measure_probe_observer_overhead(
    cfg: &DriverRunConfig,
) -> Result<ObserverOverheadReport, DriverError> {
    let work_root = cfg
        .work_root
        .as_ref()
        .ok_or_else(|| DriverError::Msg("observer overhead requires work_root".into()))?;

    let off = run_real_store_with_probe(cfg, work_root, false, "probe-off")?;
    let on = run_real_store_with_probe(cfg, work_root, true, "probe-on")?;
    Ok(ObserverOverheadReport::from_pair(
        &cfg.cell.cell_id,
        cfg.seed,
        off.0,
        on.0,
        off.1,
        on.1,
    ))
}

/// Minimal timed put loop with probe on/off; returns (e2e_ns, logical_bytes).
fn run_real_store_with_probe(
    cfg: &DriverRunConfig,
    work_root: &std::path::Path,
    probe_on: bool,
    tag: &str,
) -> Result<(u64, u64), DriverError> {
    let store_path = cell_store_path(work_root, &cfg.cell.cell_id, &format!("{tag}-s{:x}", cfg.seed));
    if store_path.exists() {
        let _ = fs::remove_dir_all(&store_path);
    }
    fs::create_dir_all(store_path.parent().unwrap_or(work_root))?;
    let mut store = Store::create(&store_path).map_err(|e| DriverError::Store(e.to_string()))?;
    if probe_on {
        store.enable_boundary_probe();
    }
    let store_dur = map_durability(cfg.cell.durability);
    let run_class = cfg.run_class_parsed();
    let op_n = if run_class.allows_smoke_op_cap() {
        cfg.cell.op_count.min(RunClass::SMOKE_MAX_OPS).max(1)
    } else {
        cfg.cell.op_count.max(1).min(64) // bound overhead helper
    };
    let plen = cfg.cell.payload_size.min(64 * 1024).max(1);
    let body = vec![0xABu8; plen as usize];
    let t0 = Instant::now();
    let mut logical = 0u64;
    for i in 0..op_n {
        let subject = format!("oh-{i:08x}");
        store
            .put(&subject, &body, store_dur)
            .map_err(|e| DriverError::Store(e.to_string()))?;
        logical = logical.saturating_add(plen);
    }
    let e2e = t0.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    drop(store);
    Ok((e2e, logical))
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
    use residiuum_store::{DurabilityMode as StoreDur, Store};

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
        assert!(report.boundary_aggregates.is_some());
        let agg = report.boundary_aggregates.as_ref().unwrap();
        assert!(agg.total_observed > 0);
        assert!(!agg.event_chain_digest.is_empty());
        // Small smoke run should not drop samples → lossless plan eligible.
        assert!(report.lossless_plan_eligible);
        assert!(report.plan.is_some());
        let plan = report.plan.as_ref().unwrap();
        plan.assert_redacted_json().unwrap();
        assert!(plan.planned_bytes > 0);
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.contains("lossless_plan=yes")),
            "expected lossless plan note"
        );
        assert!(report.notes.iter().any(|n| n.contains("run_class=smoke")));
    }

    #[test]
    fn dropped_samples_invalidate_lossless_plan_not_aggregates() {
        let dir = tempfile::tempdir().unwrap();
        let cell = MatrixCell {
            cell_id: "L4-drop-s256-c1-o1-0".into(),
            layer: LayerProfile::L4,
            durability: MatrixDur::Buffered,
            payload_size: 128,
            distribution: None,
            concurrency: 1,
            outstanding: 1,
            batch_size: 1,
            shards: 1,
            db_state: DatabaseState::Empty,
            features: FeatureProfile::l4_minimal(),
            interference: InterferenceProfile::absent(),
            op_count: 16,
            order_rank: 0,
        };
        // Tiny sample capacity forces drops while counters stay exact.
        let store_path = cell_store_path(dir.path(), &cell.cell_id, "drop");
        fs::create_dir_all(store_path.parent().unwrap()).unwrap();
        let mut store = Store::create(&store_path).unwrap();
        store.enable_boundary_probe_with_capacity(2);
        for i in 0..8 {
            store
                .put(
                    format!("k{i}").as_bytes(),
                    &[0u8; 32],
                    StoreDur::Buffered,
                )
                .unwrap();
        }
        let snap = store.take_boundary_snapshot();
        assert!(snap.coverage.samples_dropped > 0 || snap.coverage.sample_vector_capped);
        let agg = BoundaryAggregateSummary::from_store_snapshot(&snap);
        assert!(!agg.lossless_plan_eligible);
        assert!(agg.plan_replay_invalidate_reason.is_some());
        assert!(agg.total_observed > agg.samples_retained);
        assert!(!agg.event_chain_digest.is_empty());
    }

    #[test]
    fn matched_probe_observer_overhead_runs() {
        let dir = tempfile::tempdir().unwrap();
        let cell = MatrixCell {
            cell_id: "L4-oh-s256-c1-o1-0".into(),
            layer: LayerProfile::L4,
            durability: MatrixDur::Buffered,
            payload_size: 64,
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
        let report = measure_probe_observer_overhead(&DriverRunConfig {
            cell,
            seed: 9,
            kind: DriverKind::RealStore,
            work_root: Some(dir.path().to_path_buf()),
            durability_mutant: false,
            digest_mutant: false,
            run_class: "smoke".into(),
        })
        .expect("observer overhead");
        assert!(report.probe_off_e2e_ns > 0);
        assert!(report.probe_on_e2e_ns > 0);
        assert_eq!(report.probe_off_logical_bytes, report.probe_on_logical_bytes);
        assert!(report.notes.iter().any(|n| n.contains("probe-off")));
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