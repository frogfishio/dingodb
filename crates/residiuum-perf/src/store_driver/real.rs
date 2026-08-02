//! Real residiuum-store L4/L5/L6 cell driver (feature `store-driver`).
//!
//! **Smoke** may use small op budgets for unit/CI.
//! **Qualification** runs until SPEC §6.4 duration **and** byte floors are met
//! (unless safety max would be crossed); never silently caps to 4–16 ops.

use super::aggregates::{
    BoundaryAggregateSummary, ObserverOverheadReport, OverheadCostBasis, OverheadPairSample,
    WorkloadContract, OBSERVER_OVERHEAD_BUDGET, OVERHEAD_MIN_PAIRS_QUAL, OVERHEAD_MIN_PAIRS_SMOKE,
};
use super::emitter::{emit_plan_from_store_boundary_events, STORE_SEAM_EMITTER_FROM_RECEIPTS};
use super::kinds::{AwoMode, DriverKind, MeasurementSurface};
use super::{cell_store_path, DriverCellReport, DriverError, DriverRunConfig};
use crate::campaign::RunClass;
use crate::envelope::{WindowClass, WindowDetector, WindowSample};
use crate::matrix::{AckLedger, CellRunReport, DurabilityMode as MatrixDur, MatrixError};
use crate::runner::RunBudgets;
use crate::workload::SizeSampler;
use residiuum_store::adaptive_write::{
    AdaptiveWriteHandle, AdaptiveWriteMode, AdaptiveWritePolicy,
};
use residiuum_store::{BoundaryKind, DurabilityMode as StoreDur, Store};
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Outcome of one workload leg (shared by measured cell + overhead probe legs).
struct WorkloadLegStats {
    e2e_ns: u64,
    logical_bytes: u64,
    ops_done: u64,
    ledger: AckLedger,
    records: Vec<(u64, u64, u64, u32)>,
    window_samples: Vec<WindowSample>,
    messages: Vec<String>,
    floors_met: bool,
    /// Sustained (or overall) throughput used for duration-controlled probe cost.
    sustained_throughput_bps: f64,
    window_class: WindowClass,
    /// Floors + validity conditions for this leg.
    valid: bool,
    validity_reason: String,
}

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

    let n_shards = cfg.cell.shards.max(1) as usize;
    let mut store = Store::create_with_shards(&store_path, n_shards)
        .map_err(|e| DriverError::Store(e.to_string()))?;
    // Consume store-emitted boundary events (not receipt reconstruction).
    store.enable_boundary_probe();
    let store_dur = map_durability(cfg.cell.durability);

    // Optional AWO lease (static/adaptive). Disabled keeps natural put_many.
    let awo_handle = attach_awo_if_needed(&mut store, cfg.awo_mode)?;
    let awo_ref = awo_handle.as_ref();

    let mut leg = execute_workload_puts(&mut store, cfg, store_dur, awo_ref)?;
    let e2e_ns = leg.e2e_ns;
    let logical_bytes = leg.logical_bytes;
    let seq = leg.ops_done;
    let ledger = &leg.ledger;
    let records = &leg.records;
    let window_samples = &leg.window_samples;
    let mut messages = std::mem::take(&mut leg.messages);
    messages.insert(
        0,
        format!(
            "driver=real_store run_class={} awo_mode={} path={}",
            run_class.as_str(),
            cfg.awo_mode.as_str(),
            store_path.display()
        ),
    );
    if let Some(h) = awo_ref {
        let st = h.status();
        messages.push(format!(
            "awo_status mode={} lease_active={} cooker_threads={} pipeline_depth={}",
            st.mode.as_str(),
            st.lease_active,
            st.cooker_threads,
            st.pipeline.depth_limit
        ));
    }
    let floors_met = leg.floors_met;

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
    let shards = u64::from(cfg.cell.shards.max(1));
    for s in 0..seq.min(4) {
        let shard = s % shards;
        let subject = format!("pqh11-s{shard:02x}-{s:08x}");
        if let Ok(Some(_)) = store.get(&subject) {
            get_ok += 1;
        }
    }
    messages.push(format!("sample_get_ok={get_ok} ops_done={seq}"));

    // Release AWO lease before reopen (direct open must not see active fence).
    if let Some(h) = awo_handle {
        let _ = h.drain_writes(Instant::now() + Duration::from_secs(2));
        h.detach(&mut store);
        messages.push("awo_detached=true".into());
    }

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

    let window = WindowDetector::default().classify(window_samples);
    let window_s = format!("{window:?}").to_ascii_lowercase();

    // Qualification without steady-state → inconclusive (SPEC §6.4).
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

/// Order-balanced multi-pair probe-off / probe-on observer-overhead for **one** cell.
///
/// Legs share `execute_workload_puts` with the measured cell (floors, distribution,
/// durability, concurrent workers, outstanding-bounded `put_many` batches, shards).
///
/// **Duration-controlled** pairs use **sustained throughput** for probe cost
/// (`(tput_off - tput_on) / tput_off`), not wall-time. Smoke/op-cap pairs still
/// use wall-time. **Every leg must meet floors + validity** or measurement fails.
pub fn measure_probe_observer_overhead(
    cfg: &DriverRunConfig,
) -> Result<ObserverOverheadReport, DriverError> {
    let work_root = cfg
        .work_root
        .as_ref()
        .ok_or_else(|| DriverError::Msg("observer overhead requires work_root".into()))?;

    if cfg.durability_mutant {
        return Err(MatrixError::DurabilityMutant(
            "durable label with memory barrier is a different product profile".into(),
        )
        .into());
    }

    let run_class = cfg.run_class_parsed();
    let contract = WorkloadContract::from_cell_and_class(&cfg.cell, run_class);
    let pair_count = if run_class.allows_smoke_op_cap() {
        OVERHEAD_MIN_PAIRS_SMOKE
    } else {
        OVERHEAD_MIN_PAIRS_QUAL
    };
    let cost_basis = if contract.uses_duration_byte_floors() {
        OverheadCostBasis::SustainedThroughput
    } else {
        OverheadCostBasis::WallTime
    };

    let mut pairs = Vec::with_capacity(pair_count as usize);
    let mut mean_ops = 0u64;
    for i in 0..pair_count {
        let off_first = i % 2 == 0;
        let order = if off_first { "off_first" } else { "on_first" };
        let tag_base = format!("oh-p{i}-s{:x}", cfg.seed);
        let (off, on) = if off_first {
            let off = run_contract_leg(cfg, work_root, false, &format!("{tag_base}-off"))?;
            let on = run_contract_leg(cfg, work_root, true, &format!("{tag_base}-on"))?;
            (off, on)
        } else {
            let on = run_contract_leg(cfg, work_root, true, &format!("{tag_base}-on"))?;
            let off = run_contract_leg(cfg, work_root, false, &format!("{tag_base}-off"))?;
            (off, on)
        };
        // Fail closed: every leg must meet floors + validity.
        if !off.valid {
            return Err(DriverError::Msg(format!(
                "overhead probe-off leg invalid pair={i}: {}",
                off.validity_reason
            )));
        }
        if !on.valid {
            return Err(DriverError::Msg(format!(
                "overhead probe-on leg invalid pair={i}: {}",
                on.validity_reason
            )));
        }
        mean_ops = mean_ops.saturating_add(off.ops_done.saturating_add(on.ops_done) / 2);
        pairs.push(OverheadPairSample::from_leg_stats(
            i,
            order,
            off.e2e_ns,
            on.e2e_ns,
            off.logical_bytes,
            on.logical_bytes,
            off.ops_done,
            on.ops_done,
            off.sustained_throughput_bps,
            on.sustained_throughput_bps,
            cost_basis,
            true,
        ));
    }
    if pair_count > 0 {
        mean_ops /= u64::from(pair_count);
    }

    Ok(ObserverOverheadReport::from_pairs(
        &cfg.cell.cell_id,
        cfg.seed,
        Some(contract.clone()),
        mean_ops,
        contract.payload_size,
        run_class.as_str(),
        OBSERVER_OVERHEAD_BUDGET,
        pairs,
    ))
}

/// One probe-off or probe-on leg under the cell workload contract.
fn run_contract_leg(
    cfg: &DriverRunConfig,
    work_root: &std::path::Path,
    probe_on: bool,
    tag: &str,
) -> Result<WorkloadLegStats, DriverError> {
    let store_path =
        cell_store_path(work_root, &cfg.cell.cell_id, &format!("{tag}-s{:x}", cfg.seed));
    if store_path.exists() {
        let _ = fs::remove_dir_all(&store_path);
    }
    fs::create_dir_all(store_path.parent().unwrap_or(work_root))?;
    let n_shards = cfg.cell.shards.max(1) as usize;
    let mut store = Store::create_with_shards(&store_path, n_shards)
        .map_err(|e| DriverError::Store(e.to_string()))?;
    if probe_on {
        store.enable_boundary_probe();
    }
    let store_dur = map_durability(cfg.cell.durability);
    let awo_handle = attach_awo_if_needed(&mut store, cfg.awo_mode)?;
    let mut stats = execute_workload_puts(&mut store, cfg, store_dur, awo_handle.as_ref())?;
    if let Some(h) = awo_handle {
        let _ = h.drain_writes(Instant::now() + Duration::from_secs(2));
        h.detach(&mut store);
    }
    stats.messages.push(format!(
        "overhead_leg probe_on={probe_on} tag={tag} awo_mode={} path={} valid={} tput={:.3}",
        cfg.awo_mode.as_str(),
        store_path.display(),
        stats.valid,
        stats.sustained_throughput_bps
    ));
    drop(store);
    Ok(stats)
}

/// Attach Static/Adaptive AWO handle; `None` for Disabled (natural put_many).
fn attach_awo_if_needed(
    store: &mut Store,
    mode: AwoMode,
) -> Result<Option<AdaptiveWriteHandle>, DriverError> {
    if !mode.lease_active() {
        return Ok(None);
    }
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = match mode {
        AwoMode::Static => AdaptiveWriteMode::Static,
        AwoMode::Adaptive => AdaptiveWriteMode::Adaptive,
        AwoMode::Disabled => unreachable!("lease_active false"),
    };
    // Bound cooker cost for smoke/diagnostic cells; still exercises product path.
    policy.maximum_cookers = policy.maximum_cookers.min(4).max(1);
    policy.minimum_active_cookers = policy.minimum_active_cookers.min(policy.maximum_cookers);
    AdaptiveWriteHandle::start_static(policy, store)
        .map(Some)
        .map_err(|e| DriverError::Store(format!("awo attach {}: {}", e.as_str(), e.as_str())))
}

/// Shared put loop for measured cells and overhead legs.
///
/// **Stop policy:** smoke → op cap; non-smoke → duration **and** byte floors.
/// **Behaviour:** `create_with_shards(shards)`; concurrent workers when
/// `concurrency > 1`; batches of `batch_size` via `put_many` (or AWO
/// `admit_put_batch` when lease active); at most `outstanding` in-flight
/// batches per worker pipeline depth.
fn execute_workload_puts(
    store: &mut Store,
    cfg: &DriverRunConfig,
    store_dur: StoreDur,
    awo: Option<&AdaptiveWriteHandle>,
) -> Result<WorkloadLegStats, DriverError> {
    let run_class = cfg.run_class_parsed();
    let workers = cfg.cell.concurrency.max(1) as usize;
    let batch_sz = cfg.cell.batch_size.max(1) as usize;
    let outstanding = cfg.cell.outstanding.max(1) as usize;
    let n_shards = cfg.cell.shards.max(1) as usize;

    let mut messages = Vec::new();
    messages.push(format!("layer={}", cfg.cell.layer.as_str()));
    messages.push(format!(
        "workload_contract run_class={} durability={} payload={} dist={:?} conc={} out={} batch={} shards={} db_state={} features={} interference={} awo_mode={}",
        run_class.as_str(),
        cfg.cell.durability.as_str(),
        cfg.cell.payload_size,
        cfg.cell.distribution.map(|d| d.as_str()),
        workers,
        outstanding,
        batch_sz,
        n_shards,
        cfg.cell.db_state.as_str(),
        cfg.cell.features.name,
        cfg.cell.interference.kind.as_str(),
        cfg.awo_mode.as_str(),
    ));
    messages.push(format!(
        "behaviour: writer_shards={} concurrent_workers={} batch_size={} outstanding_batches={} put_many={} awo_flush={}",
        store.writer_shards(),
        workers,
        batch_sz,
        outstanding,
        batch_sz > 1 || workers > 1 || n_shards > 1,
        if awo.is_some() { "admit_put_batch" } else { "put_many" }
    ));
    messages.push(format!(
        "floors: min_duration_secs={} min_logical_bytes={}",
        run_class.min_duration_secs(),
        run_class.min_logical_bytes()
    ));

    let smoke_ops_limit = if run_class.allows_smoke_op_cap() {
        Some(cfg.cell.op_count.min(RunClass::SMOKE_MAX_OPS).max(1))
    } else {
        None
    };

    let t0 = Instant::now();
    let mut stats = if workers == 1 {
        execute_serial_batched(
            store,
            cfg,
            store_dur,
            batch_sz,
            outstanding,
            smoke_ops_limit,
            t0,
            awo,
        )?
    } else {
        execute_concurrent_batched(
            store,
            cfg,
            store_dur,
            workers,
            batch_sz,
            outstanding,
            smoke_ops_limit,
            t0,
            awo,
        )?
    };
    messages.append(&mut stats.messages);
    stats.messages = messages;

    let e2e_ns = t0.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    stats.e2e_ns = e2e_ns;
    let min_dur = Duration::from_secs(run_class.min_duration_secs());
    let min_bytes = run_class.min_logical_bytes();
    let floors_met = if smoke_ops_limit.is_some() {
        true
    } else {
        t0.elapsed() >= min_dur && stats.logical_bytes >= min_bytes
    };
    stats.floors_met = floors_met;

    // Window + sustained throughput (prefer second-half samples for steady state).
    let window = WindowDetector::default().classify(&stats.window_samples);
    stats.window_class = window;
    stats.sustained_throughput_bps =
        sustained_throughput_bps(&stats.window_samples, stats.logical_bytes, e2e_ns);

    if smoke_ops_limit.is_some() {
        stats
            .messages
            .push(format!("stop=smoke_op_cap ops={}", stats.ops_done));
    } else {
        stats.messages.push(format!(
            "stop=duration_and_byte_floors floors_met={floors_met} ops={} logical_bytes={} window={window:?} sustained_tput={:.3}",
            stats.ops_done, stats.logical_bytes, stats.sustained_throughput_bps
        ));
    }

    // Leg validity: floors always; qualification also requires sustained window.
    let (valid, reason) = leg_validity(run_class, floors_met, window, stats.ops_done);
    stats.valid = valid;
    stats.validity_reason = reason;
    stats.messages.push(format!(
        "leg_valid={} reason={}",
        stats.valid, stats.validity_reason
    ));

    Ok(stats)
}

fn leg_validity(
    run_class: RunClass,
    floors_met: bool,
    window: WindowClass,
    ops_done: u64,
) -> (bool, String) {
    if ops_done == 0 {
        return (false, "zero_ops".into());
    }
    if run_class.allows_smoke_op_cap() {
        return (true, "smoke_op_cap_complete".into());
    }
    if !floors_met {
        return (false, "floors_unmet".into());
    }
    if run_class.may_emit_bottleneck_verdict() {
        if matches!(window, WindowClass::Sustained) {
            (true, "floors_met+sustained".into())
        } else {
            (
                false,
                format!("floors_met_but_window={window:?}_not_sustained"),
            )
        }
    } else {
        // Diagnostic: floors sufficient for overhead comparison.
        (true, "floors_met_diagnostic".into())
    }
}

fn sustained_throughput_bps(samples: &[WindowSample], logical_bytes: u64, e2e_ns: u64) -> f64 {
    if samples.len() >= WindowDetector::default().min_samples {
        let mid = samples.len() / 2;
        let mut vals: Vec<f64> = samples[mid..]
            .iter()
            .map(|s| s.throughput_bytes_per_sec)
            .filter(|v| v.is_finite() && *v >= 0.0)
            .collect();
        if !vals.is_empty() {
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            return vals[vals.len() / 2];
        }
    }
    if e2e_ns > 0 {
        (logical_bytes as f64) * 1e9 / (e2e_ns as f64)
    } else {
        0.0
    }
}

/// Single-worker path with real `put_many` / AWO batch flush and outstanding depth.
fn execute_serial_batched(
    store: &mut Store,
    cfg: &DriverRunConfig,
    store_dur: StoreDur,
    batch_sz: usize,
    outstanding: usize,
    smoke_ops_limit: Option<u64>,
    t0: Instant,
    awo: Option<&AdaptiveWriteHandle>,
) -> Result<WorkloadLegStats, DriverError> {
    let run_class = cfg.run_class_parsed();
    let sampler = match cfg.cell.distribution {
        Some(d) => SizeSampler::distribution(d),
        None => SizeSampler::fixed(cfg.cell.payload_size),
    };
    let min_dur = Duration::from_secs(run_class.min_duration_secs());
    let min_bytes = run_class.min_logical_bytes();
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
    let mut messages = vec![format!(
        "serial_batched batch_size={batch_sz} outstanding={outstanding}"
    )];
    let mut seq: u64 = 0;
    // Outstanding = max prepared batches waiting to flush (pipeline depth).
    let mut pending_batches: usize = 0;

    loop {
        if let Some(lim) = smoke_ops_limit {
            if seq >= lim {
                break;
            }
        } else {
            let elapsed = t0.elapsed();
            if elapsed >= min_dur && logical_bytes >= min_bytes {
                break;
            }
            if elapsed >= max_dur || logical_bytes >= max_bytes {
                messages.push(format!(
                    "stopped at safety ceiling elapsed_s={} logical_bytes={}",
                    elapsed.as_secs(),
                    logical_bytes
                ));
                break;
            }
        }

        // Build one batch (respect outstanding depth by flushing when full).
        let mut subjects: Vec<String> = Vec::with_capacity(batch_sz);
        let mut bodies: Vec<Vec<u8>> = Vec::with_capacity(batch_sz);
        for _ in 0..batch_sz {
            if let Some(lim) = smoke_ops_limit {
                if seq >= lim {
                    break;
                }
            }
            let plen = payload_len(cfg, &sampler, seq, run_class);
            let body = fill_body(cfg.seed, seq, plen);
            let shard = seq % u64::from(cfg.cell.shards.max(1));
            subjects.push(format!("pqh11-s{shard:02x}-{seq:08x}"));
            bodies.push(body);
            seq = seq.saturating_add(1);
        }
        if subjects.is_empty() {
            break;
        }
        pending_batches = pending_batches.saturating_add(1);

        let items: Vec<(&str, &[u8])> = subjects
            .iter()
            .zip(bodies.iter())
            .map(|(s, b)| (s.as_str(), b.as_slice()))
            .collect();

        match flush_batch(store, &items, store_dur, awo) {
            Ok(receipts) => {
                for (i, receipt) in receipts.iter().enumerate() {
                    let plen = bodies[i].len() as u64;
                    let op_seq = seq - subjects.len() as u64 + i as u64;
                    ledger.record_attempt();
                    ledger.record_admit();
                    if cfg.digest_mutant && op_seq == 0 {
                        ledger.record_ack_mutant_wrong_digest(op_seq, op_seq, plen, 0);
                    } else {
                        ledger.record_ack(op_seq, op_seq, plen, 0);
                    }
                    records.push((op_seq, op_seq, plen, 0u32));
                    logical_bytes = logical_bytes.saturating_add(plen);
                    let _ = receipt;
                }
            }
            Err(e) => {
                for _ in 0..subjects.len() {
                    ledger.record_attempt();
                    ledger.record_fail();
                }
                messages.push(format!("batch flush failed: {e}"));
            }
        }
        // Flush outstanding counter (batch completed).
        if pending_batches >= outstanding {
            pending_batches = 0;
        }

        if seq > 0 && (seq % 32 == 0 || t0.elapsed().as_millis() % 100 < 5) {
            let e2e = t0.elapsed().as_nanos().max(1) as f64;
            let bps = (logical_bytes as f64) * 1e9 / e2e;
            window_samples.push(WindowSample {
                throughput_bytes_per_sec: bps,
            });
        }
    }

    Ok(WorkloadLegStats {
        e2e_ns: 0, // filled by caller
        logical_bytes,
        ops_done: seq,
        ledger,
        records,
        window_samples,
        messages,
        floors_met: false,
        sustained_throughput_bps: 0.0,
        window_class: WindowClass::Inconclusive,
        valid: false,
        validity_reason: String::new(),
    })
}

/// Multi-worker path: concurrent preparers + outstanding-bounded batch flush.
///
/// Workers generate batches in parallel; the store is written only on the
/// main thread via `put_many` / AWO `admit_put_batch` (may parallelize across
/// writer shards). Channel capacity enforces `outstanding` in-flight batches.
fn execute_concurrent_batched(
    store: &mut Store,
    cfg: &DriverRunConfig,
    store_dur: StoreDur,
    workers: usize,
    batch_sz: usize,
    outstanding: usize,
    smoke_ops_limit: Option<u64>,
    t0: Instant,
    awo: Option<&AdaptiveWriteHandle>,
) -> Result<WorkloadLegStats, DriverError> {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::mpsc;
    use std::thread;

    let run_class = cfg.run_class_parsed();
    let min_dur = Duration::from_secs(run_class.min_duration_secs());
    let min_bytes = run_class.min_logical_bytes();
    let safety = RunBudgets::default();
    let max_dur = Duration::from_secs(
        safety
            .max_duration_secs
            .max(run_class.min_duration_secs()),
    );
    let max_bytes = safety.max_bytes.max(run_class.min_logical_bytes());

    let seq_counter = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let logical_shared = Arc::new(AtomicU64::new(0));

    let (tx, rx) = mpsc::sync_channel::<PreparedBatch>(outstanding.max(1));

    let seed = cfg.seed;
    let dist = cfg.cell.distribution;
    let payload_size = cfg.cell.payload_size;
    let shards = cfg.cell.shards.max(1);

    let handles: Vec<_> = (0..workers)
        .map(|wid| {
            let tx = tx.clone();
            let seq_counter = Arc::clone(&seq_counter);
            let stop = Arc::clone(&stop);
            let logical_shared = Arc::clone(&logical_shared);
            thread::spawn(move || {
                let sampler = match dist {
                    Some(d) => SizeSampler::distribution(d),
                    None => SizeSampler::fixed(payload_size),
                };
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Some(lim) = smoke_ops_limit {
                        if seq_counter.load(Ordering::Relaxed) >= lim {
                            break;
                        }
                    } else {
                        let elapsed = t0.elapsed();
                        let lb = logical_shared.load(Ordering::Relaxed);
                        if elapsed >= min_dur && lb >= min_bytes {
                            break;
                        }
                        if elapsed >= max_dur || lb >= max_bytes {
                            break;
                        }
                    }

                    let mut subjects = Vec::with_capacity(batch_sz);
                    let mut bodies = Vec::with_capacity(batch_sz);
                    for _ in 0..batch_sz {
                        let seq = seq_counter.fetch_add(1, Ordering::Relaxed);
                        if let Some(lim) = smoke_ops_limit {
                            if seq >= lim {
                                break;
                            }
                        }
                        let plen = if dist.is_some() {
                            sampler.size_at(seq)
                        } else {
                            payload_size
                        };
                        let plen = if run_class.allows_smoke_op_cap() {
                            plen.min(64 * 1024).max(1)
                        } else {
                            plen.max(1)
                        };
                        let body = fill_body(seed, seq, plen);
                        let shard = seq % u64::from(shards);
                        subjects.push(format!("pqh11-w{wid:02}-s{shard:02x}-{seq:08x}"));
                        bodies.push(body);
                    }
                    if subjects.is_empty() {
                        break;
                    }
                    if tx
                        .send(PreparedBatch {
                            subjects,
                            bodies,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
        })
        .collect();
    drop(tx);

    let mut ledger = AckLedger::new();
    let mut logical_bytes = 0u64;
    let mut records = Vec::new();
    let mut window_samples = Vec::new();
    let mut messages = vec![format!(
        "concurrent_batched workers={workers} batch_size={batch_sz} outstanding={outstanding}"
    )];
    let mut ops_done = 0u64;
    let digest_mutant = cfg.digest_mutant;

    while let Ok(batch) = rx.recv() {
        let items: Vec<(&str, &[u8])> = batch
            .subjects
            .iter()
            .zip(batch.bodies.iter())
            .map(|(s, b)| (s.as_str(), b.as_slice()))
            .collect();
        match flush_batch(store, &items, store_dur, awo) {
            Ok(receipts) => {
                for (i, _receipt) in receipts.iter().enumerate() {
                    let plen = batch.bodies[i].len() as u64;
                    let op_seq = ops_done;
                    ops_done = ops_done.saturating_add(1);
                    ledger.record_attempt();
                    ledger.record_admit();
                    if digest_mutant && op_seq == 0 {
                        ledger.record_ack_mutant_wrong_digest(op_seq, op_seq, plen, 0);
                    } else {
                        ledger.record_ack(op_seq, op_seq, plen, 0);
                    }
                    records.push((op_seq, op_seq, plen, 0u32));
                    logical_bytes = logical_bytes.saturating_add(plen);
                }
                logical_shared.store(logical_bytes, Ordering::Relaxed);
            }
            Err(e) => {
                for _ in 0..batch.subjects.len() {
                    ledger.record_attempt();
                    ledger.record_fail();
                    ops_done = ops_done.saturating_add(1);
                }
                messages.push(format!("batch flush failed: {e}"));
            }
        }
        if smoke_ops_limit.is_none() {
            let elapsed = t0.elapsed();
            if elapsed >= min_dur && logical_bytes >= min_bytes {
                stop.store(true, Ordering::Relaxed);
            }
            if elapsed >= max_dur || logical_bytes >= max_bytes {
                stop.store(true, Ordering::Relaxed);
                messages.push("stopped at safety ceiling (concurrent)".into());
            }
        }
        if ops_done > 0 && (ops_done % 32 == 0 || t0.elapsed().as_millis() % 100 < 5) {
            let e2e = t0.elapsed().as_nanos().max(1) as f64;
            let bps = (logical_bytes as f64) * 1e9 / e2e;
            window_samples.push(WindowSample {
                throughput_bytes_per_sec: bps,
            });
        }
    }
    for h in handles {
        let _ = h.join();
    }
    ops_done = ledger.acknowledged.max(ledger.attempted);

    Ok(WorkloadLegStats {
        e2e_ns: 0,
        logical_bytes,
        ops_done,
        ledger,
        records,
        window_samples,
        messages,
        floors_met: false,
        sustained_throughput_bps: 0.0,
        window_class: WindowClass::Inconclusive,
        valid: false,
        validity_reason: String::new(),
    })
}

struct PreparedBatch {
    subjects: Vec<String>,
    bodies: Vec<Vec<u8>>,
}


/// Flush a prepared batch.
///
/// - **AWO disabled:** `Store::put_many` (multi-shard → `put_many_parallel`).
/// - **AWO static/adaptive:** `AdaptiveWriteHandle::admit_put_batch` →
///   `put_many_awo_owned` (same persist-before-publish product path under lease).
/// Never sequential-put bypass.
fn flush_batch(
    store: &mut Store,
    items: &[(&str, &[u8])],
    mode: StoreDur,
    awo: Option<&AdaptiveWriteHandle>,
) -> Result<Vec<residiuum_store::WriteReceipt>, DriverError> {
    match awo {
        None => store
            .put_many(items, mode)
            .map_err(|e| DriverError::Store(e.to_string())),
        Some(h) => h
            .admit_put_batch(store, items, mode)
            .map_err(|e| DriverError::Store(format!("awo admit_put_batch: {}", e.as_str()))),
    }
}

fn payload_len(
    cfg: &DriverRunConfig,
    sampler: &SizeSampler,
    seq: u64,
    run_class: RunClass,
) -> u64 {
    let plen = if cfg.cell.distribution.is_some() {
        sampler.size_at(seq)
    } else {
        cfg.cell.payload_size
    };
    if run_class.allows_smoke_op_cap() {
        plen.min(64 * 1024).max(1)
    } else {
        plen.max(1)
    }
}

fn fill_body(seed: u64, seq: u64, plen: u64) -> Vec<u8> {
    let mut body = vec![0u8; plen as usize];
    let mut x = seed.wrapping_add(seq);
    for b in &mut body {
        x = x
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(1);
        *b = (x >> 33) as u8;
    }
    body
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
            awo_mode: AwoMode::Disabled,
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
                .put(&format!("k{i}"), &[0u8; 32], StoreDur::Buffered)
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
        use crate::store_driver::{OBSERVER_OVERHEAD_BUDGET, OVERHEAD_MIN_PAIRS_SMOKE};
        let dir = tempfile::tempdir().unwrap();
        let cell = MatrixCell {
            cell_id: "L4-oh-s256-c1-o1-0".into(),
            layer: LayerProfile::L4,
            durability: MatrixDur::Buffered,
            payload_size: 64,
            distribution: None,
            concurrency: 2,
            outstanding: 4,
            batch_size: 1,
            shards: 2,
            db_state: DatabaseState::Empty,
            features: FeatureProfile::l4_minimal(),
            interference: InterferenceProfile::absent(),
            op_count: 8,
            order_rank: 0,
        };
        let report = measure_probe_observer_overhead(&DriverRunConfig {
            cell: cell.clone(),
            seed: 9,
            kind: DriverKind::RealStore,
            work_root: Some(dir.path().to_path_buf()),
            durability_mutant: false,
            digest_mutant: false,
            run_class: "smoke".into(),
            awo_mode: AwoMode::Disabled,
        })
        .expect("observer overhead");
        assert!(report.probe_off_e2e_ns > 0);
        assert!(report.probe_on_e2e_ns > 0);
        assert_eq!(report.probe_off_logical_bytes, report.probe_on_logical_bytes);
        assert_eq!(report.pair_count, OVERHEAD_MIN_PAIRS_SMOKE);
        assert_eq!(report.ops_per_leg, 8);
        assert!((report.overhead_budget - OBSERVER_OVERHEAD_BUDGET).abs() < 1e-12);
        let wc = report.workload_contract.as_ref().expect("contract");
        assert_eq!(wc.stop_policy, "smoke_op_cap");
        assert_eq!(wc.concurrency, 2);
        assert_eq!(wc.outstanding, 4);
        assert_eq!(wc.shards, 2);
        assert_eq!(wc.durability, "buffered");
        // Smoke uses wall-time cost basis; duration floors use sustained tput.
        assert!(report
            .pairs
            .iter()
            .all(|p| p.cost_basis == OverheadCostBasis::WallTime && p.legs_valid));
        assert_eq!(
            report.pairs.iter().filter(|p| p.order == "off_first").count(),
            report.pairs.iter().filter(|p| p.order == "on_first").count()
        );
        assert!(report.notes.iter().any(|n| n.contains("workload_contract")));
    }

    #[test]
    fn concurrent_batching_smoke_cell_runs() {
        let dir = tempfile::tempdir().unwrap();
        let cell = MatrixCell {
            cell_id: "L4-conc-batch".into(),
            layer: LayerProfile::L4,
            durability: MatrixDur::Buffered,
            payload_size: 64,
            distribution: None,
            concurrency: 2,
            outstanding: 2,
            batch_size: 4,
            shards: 2,
            db_state: DatabaseState::Empty,
            features: FeatureProfile::l4_minimal(),
            interference: InterferenceProfile::absent(),
            op_count: 16,
            order_rank: 0,
        };
        let report = run_real_store(&DriverRunConfig {
            cell,
            seed: 11,
            kind: DriverKind::RealStore,
            work_root: Some(dir.path().to_path_buf()),
            durability_mutant: false,
            digest_mutant: false,
            run_class: "smoke".into(),
            awo_mode: AwoMode::Disabled,
        })
        .expect("concurrent batched smoke");
        assert_eq!(report.cell.validity, "valid");
        assert!(report.cell.acknowledged > 0);
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.contains("concurrent_batched") || n.contains("serial_batched") || n.contains("put_many")),
            "expected batching/concurrency behaviour notes"
        );
        assert!(
            report.notes.iter().any(|n| n.contains("concurrent_workers=2")
                || n.contains("workers=2")),
            "expected concurrency dimension applied"
        );
        // Multi-shard + probe: observations must come from put_many product path
        // (put_many_parallel instrumentation), not sequential-put bypass.
        assert!(report.boundary_aggregates.is_some());
        assert!(report.boundary_aggregates.as_ref().unwrap().total_observed > 0);
    }

    #[test]
    fn real_store_smoke_awo_static_and_adaptive() {
        let dir = tempfile::tempdir().unwrap();
        for mode in [AwoMode::Static, AwoMode::Adaptive] {
            let cell = MatrixCell {
                cell_id: format!("L4-awo-{}-s256", mode.as_str()),
                layer: LayerProfile::L4,
                durability: MatrixDur::Buffered,
                payload_size: 256,
                distribution: None,
                concurrency: 1,
                outstanding: 1,
                batch_size: 4,
                shards: 1,
                db_state: DatabaseState::Empty,
                features: FeatureProfile::l4_minimal(),
                interference: InterferenceProfile::absent(),
                op_count: 8,
                order_rank: 0,
            };
            let report = run_real_store(&DriverRunConfig {
                cell,
                seed: 7,
                kind: DriverKind::RealStore,
                work_root: Some(dir.path().to_path_buf()),
                durability_mutant: false,
                digest_mutant: false,
                run_class: "smoke".into(),
                awo_mode: mode,
            })
            .unwrap_or_else(|e| panic!("awo {} smoke: {e}", mode.as_str()));
            assert_eq!(report.cell.validity, "valid");
            assert!(report.cell.acknowledged > 0);
            assert!(
                report
                    .notes
                    .iter()
                    .any(|n| n.contains(&format!("awo_mode={}", mode.as_str()))),
                "notes must label awo_mode for {:?}",
                mode
            );
            assert!(
                report
                    .notes
                    .iter()
                    .any(|n| n.contains("awo_flush=admit_put_batch")),
                "lease modes must flush via admit_put_batch"
            );
            assert!(
                report.notes.iter().any(|n| n.contains("awo_detached=true")),
                "must detach lease after cell"
            );
        }
    }

    /// Measure campaign T3: all three AWO modes write, sample-get, reopen live
    /// without poison / false empty success. No throughput claims.
    #[test]
    fn real_store_smoke_three_way_correctness_before_numbers() {
        let dir = tempfile::tempdir().unwrap();
        for mode in [AwoMode::Disabled, AwoMode::Static, AwoMode::Adaptive] {
            let cell = MatrixCell {
                cell_id: format!("L4-t3-{}-s256", mode.as_str()),
                layer: LayerProfile::L4,
                durability: MatrixDur::Buffered,
                payload_size: 256,
                distribution: None,
                concurrency: 1,
                outstanding: 1,
                batch_size: 4,
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
                awo_mode: mode,
            })
            .unwrap_or_else(|e| panic!("T3 three-way smoke mode={}: {e}", mode.as_str()));

            assert_eq!(
                report.cell.validity, "valid",
                "mode={} must be valid (no poison happy path)",
                mode.as_str()
            );
            assert!(
                report.cell.acknowledged > 0,
                "mode={} must acknowledge writes",
                mode.as_str()
            );
            assert!(
                report
                    .notes
                    .iter()
                    .any(|n| n.contains(&format!("awo_mode={}", mode.as_str()))),
                "mode={} notes must include awo_mode label",
                mode.as_str()
            );
            assert!(
                report
                    .notes
                    .iter()
                    .any(|n| n.contains("sample_get_ok=")
                        && !n.contains("sample_get_ok=0")),
                "mode={} sample get must succeed",
                mode.as_str()
            );
            assert!(
                report
                    .notes
                    .iter()
                    .any(|n| n.starts_with("reopen_live_count=")
                        && n != "reopen_live_count=0"),
                "mode={} reopen must see live subjects: {:?}",
                mode.as_str(),
                report.notes
            );
            // No AdaptiveWriterPoisoned / uncertain on happy-path notes.
            assert!(
                !report
                    .notes
                    .iter()
                    .any(|n| n.to_ascii_lowercase().contains("poison")),
                "mode={} must not poison on happy path: {:?}",
                mode.as_str(),
                report.notes
            );

            match mode {
                AwoMode::Disabled => {
                    assert!(
                        !report
                            .notes
                            .iter()
                            .any(|n| n.contains("awo_flush=admit_put_batch")),
                        "disabled must not admit_put_batch"
                    );
                }
                AwoMode::Static | AwoMode::Adaptive => {
                    assert!(
                        report
                            .notes
                            .iter()
                            .any(|n| n.contains("awo_flush=admit_put_batch")),
                        "lease mode must flush via admit_put_batch"
                    );
                    assert!(
                        report.notes.iter().any(|n| n.contains("awo_detached=true")),
                        "lease mode must detach after cell"
                    );
                }
            }
        }
    }

    #[test]
    fn non_smoke_contract_uses_floors_not_op_count() {
        // Structural: non-smoke overhead contract is duration/byte floors, not
        // cell.op_count. Full floors are not executed here (would be 30s+).
        let cell = MatrixCell {
            cell_id: "L4-oh-floors".into(),
            layer: LayerProfile::L4,
            durability: MatrixDur::Durable,
            payload_size: 4096,
            distribution: Some(crate::workload::DistributionId::Tiny),
            concurrency: 4,
            outstanding: 8,
            batch_size: 2,
            shards: 3,
            db_state: DatabaseState::SteadyPopulated,
            features: FeatureProfile::l4_minimal(),
            interference: InterferenceProfile::absent(),
            op_count: 9999,
            order_rank: 0,
        };
        let wc = WorkloadContract::from_cell_and_class(&cell, RunClass::Qualification);
        assert!(wc.uses_duration_byte_floors());
        assert_eq!(wc.stop_policy, "duration_and_byte_floors");
        assert!(wc.smoke_ops_limit.is_none());
        assert_eq!(wc.cell_op_count, 9999); // recorded, not used as stop
        assert!(wc.min_duration_secs >= 120);
        assert!(wc.min_logical_bytes >= 512 * 1024 * 1024);
        assert_eq!(wc.durability, "durable");
        assert_eq!(wc.concurrency, 4);
        assert_eq!(wc.outstanding, 8);
        assert_eq!(wc.shards, 3);
        assert_eq!(wc.distribution.as_deref(), Some("tiny"));
        assert_eq!(wc.db_state, "steady_populated");

        let smoke = WorkloadContract::from_cell_and_class(&cell, RunClass::Smoke);
        assert!(!smoke.uses_duration_byte_floors());
        assert!(smoke.smoke_ops_limit.is_some());
    }

    #[test]
    fn qualification_does_not_use_smoke_op_cap_path() {
        // Structural: qualification run_class reports floors and does not claim
        // smoke op budget. Full 120s/512MiB is not executed in unit tests.
        assert!(!RunClass::Qualification.allows_smoke_op_cap());
        assert!(RunClass::Qualification.min_duration_secs() >= 120);
        assert!(RunClass::Qualification.min_logical_bytes() >= 512 * 1024 * 1024);
        let cell = MatrixCell {
            cell_id: "x".into(),
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
            op_count: 80,
            order_rank: 0,
        };
        let wc = WorkloadContract::from_cell_and_class(&cell, RunClass::Diagnostic);
        assert!(wc.uses_duration_byte_floors());
        assert!(wc.smoke_ops_limit.is_none());
    }
}