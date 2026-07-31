//! Real residiuum-store L4/L5/L6 cell driver (feature `store-driver`).

use super::emitter::{emit_plan_from_receipts, WriteReceiptFact, STORE_SEAM_EMITTER_FROM_RECEIPTS};
use super::kinds::{DriverKind, MeasurementSurface};
use super::{cell_store_path, DriverCellReport, DriverError, DriverRunConfig};
use crate::matrix::{AckLedger, CellRunReport, DurabilityMode as MatrixDur, MatrixError};
use crate::workload::SizeSampler;
use residiuum_store::{DurabilityMode as StoreDur, Store};
use std::fs;
use std::time::Instant;

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

    let ops = cfg.cell.op_count.min(32).max(1);
    let mut ledger = AckLedger::new();
    let mut facts = Vec::new();
    let mut logical_bytes = 0u64;
    let mut last_seg: Option<[u8; 16]> = None;
    let mut seg_gen = 0u32;
    let mut records = Vec::new();
    let mut messages = Vec::new();
    messages.push(format!(
        "driver=real_store path={}",
        store_path.display()
    ));
    messages.push(format!("layer={}", cfg.cell.layer.as_str()));

    let t0 = Instant::now();
    for seq in 0..ops {
        ledger.record_attempt();
        let plen = if cfg.cell.distribution.is_some() {
            sampler.size_at(seq)
        } else {
            cfg.cell.payload_size
        };
        // Bound body for unit/smoke speed (still real store path).
        let plen = plen.min(64 * 1024);

        // Deterministic payload (not stored in plan).
        let mut body = vec![0u8; plen as usize];
        let mut x = cfg.seed.wrapping_add(seq);
        for b in &mut body {
            x = x
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(1);
            *b = (x >> 33) as u8;
        }
        // Subject is a harness key — never enters the plan.
        let subject = format!("pqh10-{seq:08x}");

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
                // Physical length: use logical + fixed envelope estimate; store does
                // not export frame length on WriteReceipt.
                let physical_len = plen.saturating_add(96);
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
            Err(e) => {
                ledger.record_fail();
                messages.push(format!("put failed seq={seq}: {e}"));
            }
        }
    }
    let e2e_ns = t0.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;

    if let Err(e) = ledger.verify_correctness() {
        return Err(MatrixError::InvalidCorrectness(e.to_string()).into());
    }
    let reopen_ok = ledger.verify_reopen(&records).is_ok();
    if !reopen_ok {
        return Err(MatrixError::InvalidCorrectness("reopen digest mismatch".into()).into());
    }

    // Independent get check for a sample of acknowledged keys.
    let mut get_ok = 0u64;
    for seq in 0..ops.min(4) {
        let subject = format!("pqh10-{seq:08x}");
        if let Ok(Some(_)) = store.get(&subject) {
            get_ok += 1;
        }
    }
    messages.push(format!("sample_get_ok={get_ok}"));

    // Drop store before reopen probe.
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

    let cell = CellRunReport {
        cell_id: cfg.cell.cell_id.clone(),
        layer: cfg.cell.layer.as_str().into(),
        durability: cfg.cell.durability.as_str().into(),
        validity: "valid".into(),
        attempted: ledger.attempted,
        admitted: ledger.admitted,
        acknowledged: ledger.acknowledged,
        failed: ledger.failed,
        logical_bytes_ack: logical_bytes,
        e2e_ns_proxy: e2e_ns,
        throughput_bytes_per_sec_proxy: tput,
        window: "sustained".into(),
        shadow_planned_bytes: plan.planned_bytes,
        shadow_completed_bytes: plan.planned_bytes,
        features: cfg.cell.features.name.clone(),
        interference: cfg.cell.interference.kind.as_str().into(),
        messages: messages.clone(),
        reopen_ok: true,
    };

    // Uncontrolled by default — product claim only when campaign marks controlled.
    let surface = MeasurementSurface::RealStoreUncontrolled;
    let mut notes = messages;
    notes.push(format!("plan_source={STORE_SEAM_EMITTER_FROM_RECEIPTS}"));
    notes.push(
        "product_claim_eligible=false unless campaign platform is controlled runner".into(),
    );

    Ok(DriverCellReport {
        cell,
        driver_kind: DriverKind::RealStore,
        measurement_surface: surface,
        product_claim_eligible: surface.allows_product_claim(),
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
        })
        .expect("real store cell");
        assert_eq!(report.driver_kind, DriverKind::RealStore);
        assert!(!report.product_claim_eligible);
        assert_eq!(report.cell.validity, "valid");
        assert!(report.cell.acknowledged > 0);
        assert!(report.plan.is_some());
        report.plan.as_ref().unwrap().assert_redacted_json().unwrap();
    }
}