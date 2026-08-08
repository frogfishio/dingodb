//! Smoke runner + evidence publication (Q4.3).
//!
//! Runs mandatory cell plans against:
//! - logical harness (Ready digests + metrics)
//! - Mongo/CBL adapters (shared work loaded; execute NotConfigured)
//!
//! Publishes hashed evidence bundles. **Not competitive** / not Gate-1.

use crate::cell_plan::{section_7_2_expanded_portfolio, smoke_portfolio, MeasuredCellPlan};
use crate::engine::{
    AdapterStatus, CblEmbeddedAdapter, EngineAdapter, EngineRunOutcome, LogicalHarnessEngine,
    MongoLocalAdapter,
};
use crate::evidence::{
    compare_ready_outcomes, scaffold_cell, EnvFingerprint, EvidenceBundle, EvidenceError,
};
use crate::generator::generate_dataset;
use crate::lane::LanePairing;
use crate::shared_work::SharedLogicalWork;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("evidence: {0}")]
    Evidence(#[from] EvidenceError),
    #[error("adapter: {0}")]
    Adapter(String),
    #[error("io: {0}")]
    Io(String),
}

/// Result of one smoke cell on lane E pairing (logical vs CBL stub).
#[derive(Debug, Clone)]
pub struct SmokeCellResult {
    pub plan_id: String,
    pub side_a: EngineRunOutcome,
    pub side_b: EngineRunOutcome,
    pub shared_hash: String,
}

/// Run smoke portfolio: logical harness (A) + CBL adapter (B) with shared work.
pub fn run_smoke_lane_embedded(seed: u64) -> Result<Vec<SmokeCellResult>, RunError> {
    let plans = smoke_portfolio(seed);
    let mut out = Vec::with_capacity(plans.len());
    for plan in plans {
        out.push(run_one_embedded_pair(&plan)?);
    }
    Ok(out)
}

/// F2: expanded §7.2 portfolio (enrich/rw/concurrency variants) executed.
pub fn run_section_7_2_expanded(seed: u64) -> Result<Vec<SmokeCellResult>, RunError> {
    let plans = section_7_2_expanded_portfolio(seed);
    let mut out = Vec::with_capacity(plans.len());
    for plan in plans {
        out.push(run_one_embedded_pair(&plan)?);
    }
    Ok(out)
}

fn run_one_embedded_pair(plan: &MeasuredCellPlan) -> Result<SmokeCellResult, RunError> {
    let ds = generate_dataset(&plan.dataset);
    let work = SharedLogicalWork::from_dataset(ds);
    let hash = work.content_hash.clone();

    let mut logical = LogicalHarnessEngine::new();
    logical
        .load_shared_work(&work)
        .map_err(|e| RunError::Adapter(e.to_string()))?;
    let side_a = logical
        .execute_plan(plan)
        .map_err(|e| RunError::Adapter(e.to_string()))?;

    let mut cbl = CblEmbeddedAdapter::default();
    cbl.load_shared_work(&work)
        .map_err(|e| RunError::Adapter(e.to_string()))?;
    let side_b = cbl
        .execute_plan(plan)
        .map_err(|e| RunError::Adapter(e.to_string()))?;

    // Cross-engine fixture identity: both sides must report same shared hash.
    if side_a.shared_work_hash.as_deref() != Some(hash.as_str()) {
        return Err(RunError::Adapter(format!(
            "side_a shared hash mismatch plan={}",
            plan.plan_id
        )));
    }
    if side_b.shared_work_hash.as_deref() != Some(hash.as_str()) {
        return Err(RunError::Adapter(format!(
            "side_b shared hash mismatch plan={}",
            plan.plan_id
        )));
    }

    Ok(SmokeCellResult {
        plan_id: plan.plan_id.clone(),
        side_a,
        side_b,
        shared_hash: hash,
    })
}

/// Lane S smoke: logical (as Residiuum stand-in is not server) + Mongo stub.
/// Uses Mongo + ResidiuumServer adapters both NotConfigured after shared load —
/// proves fixture identity, not comparative digests.
pub fn run_smoke_lane_server_fixture_identity(seed: u64) -> Result<(String, bool), RunError> {
    let plan = MeasuredCellPlan::smoke_for(crate::cells::MandatoryCell::KeyGet, seed);
    let ds = generate_dataset(&plan.dataset);
    let work = SharedLogicalWork::from_dataset(ds);
    let mut mongo = MongoLocalAdapter::default();
    mongo
        .load_shared_work(&work)
        .map_err(|e| RunError::Adapter(e.to_string()))?;
    let mut server = crate::engine::ResidiuumServerAdapter::default();
    server
        .load_shared_work(&work)
        .map_err(|e| RunError::Adapter(e.to_string()))?;
    let a = server
        .execute_plan(&plan)
        .map_err(|e| RunError::Adapter(e.to_string()))?;
    let b = mongo
        .execute_plan(&plan)
        .map_err(|e| RunError::Adapter(e.to_string()))?;
    let ok = a.shared_work_hash == b.shared_work_hash
        && a.shared_work_hash.as_deref() == Some(work.content_hash.as_str());
    Ok((work.content_hash, ok))
}

/// Publish evidence bundle for smoke portfolio under workspace paths.
pub fn publish_smoke_evidence(
    workspace_root: &Path,
    seed: u64,
    campaign_id: &str,
) -> Result<PathBuf, RunError> {
    let env = EnvFingerprint::capture(workspace_root)?;
    // Scaffold campaigns may run on dirty trees; competitive Q5 uses assert_clean.
    let mut bundle = EvidenceBundle::new(env, campaign_id);
    bundle.notes.push("Q4.3 smoke — not competitive / not Gate-1".into());
    bundle
        .notes
        .push("side_a=logical_harness digests; side_b=CBL NotConfigured with shared_work".into());

    let results = run_smoke_lane_embedded(seed)?;
    let mut ready_ok = 0u64;
    let mut configured_b = 0u64;
    for r in results {
        if r.side_a.status == AdapterStatus::Ready && r.side_a.result.is_some() {
            ready_ok += 1;
        }
        if r.side_b.shared_work_hash.is_some() {
            configured_b += 1;
        }
        let mut cell = scaffold_cell(
            &r.plan_id,
            LanePairing::EMBEDDED,
            r.side_a.clone(),
            r.side_b.clone(),
        );
        cell.case_id = Some(r.plan_id.clone());
        cell.metrics_a = r.side_a.metrics.clone();
        cell.metrics_b = r.side_b.metrics.clone();
        // Equivalence: only when both Ready with results (CBL not ready → None).
        let (eq, detail) = compare_ready_outcomes(&r.side_a, &r.side_b);
        cell.equivalent = eq;
        cell.equivalence_detail = detail.or_else(|| {
            Some(format!(
                "shared_work_hash={} side_b={}",
                r.shared_hash,
                r.side_b
                    .refuse_code
                    .clone()
                    .unwrap_or_else(|| "ok".into())
            ))
        });
        bundle.push_cell(cell);
    }

    let (lane_s_hash, lane_s_ok) = run_smoke_lane_server_fixture_identity(seed)?;
    bundle.notes.push(format!(
        "lane_s_fixture_identity ok={lane_s_ok} hash={lane_s_hash}"
    ));

    let out = workspace_root.join("spec/rql/qualification/harness-v1/q4_3_smoke_evidence_bundle.json");
    bundle.write_json(&out)?;

    // Architecture/labor report
    let report = json!({
        "format": "residiuum-rql-q4-3-metrics-adapters-report-v1",
        "harness_profile": crate::HARNESS_PROFILE,
        "summary": {
            "smoke_cells": bundle.cells.len(),
            "logical_ready_with_result": ready_ok,
            "cbl_shared_work_loaded": configured_b,
            "lane_s_fixture_identity": lane_s_ok,
            "content_hash": bundle.content_hash,
            "metric_envelope": "§7.4 collectors + path digests",
            "mongo_cbl_execute": "not_configured_honest",
        },
        "bundle_path": out.display().to_string(),
        "non_claims": [
            "not_gate1",
            "not_competitive",
            "not_q4_package_accept",
            "mongo_cbl_driver_residual",
            "residiuum_server_residual"
        ],
        "authority": "doc/todo/rql/RQL_Q4_3_METRICS_ADAPTERS.md",
    });
    let report_path =
        workspace_root.join("spec/rql/qualification/harness-v1/q4_3_metrics_adapters_report.json");
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).map_err(|e| RunError::Io(e.to_string()))?,
    )
    .map_err(|e| RunError::Io(e.to_string()))?;

    let _ = ready_ok;
    Ok(out)
}

/// Machine JSON for verify (also used as lightweight call without full publish).
pub fn q4_3_report_value(workspace_root: &Path) -> Result<Value, RunError> {
    let _ = publish_smoke_evidence(workspace_root, 0x04_43, "q4-3-smoke")?;
    let p = workspace_root.join("spec/rql/qualification/harness-v1/q4_3_metrics_adapters_report.json");
    let raw = fs::read_to_string(&p).map_err(|e| RunError::Io(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| RunError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }

    #[test]
    fn smoke_portfolio_logical_ready() {
        let results = run_smoke_lane_embedded(3).expect("smoke");
        assert_eq!(results.len(), 12);
        for r in &results {
            assert_eq!(r.side_a.status, AdapterStatus::Ready);
            assert!(r.side_a.result.is_some(), "plan {}", r.plan_id);
            assert!(r.side_a.metrics.is_some());
            assert_eq!(r.side_b.status, AdapterStatus::NotConfigured);
            assert_eq!(
                r.side_b.shared_work_hash.as_deref(),
                Some(r.shared_hash.as_str())
            );
        }
    }

    #[test]
    fn section_7_2_expanded_executes_variants() {
        let results = run_section_7_2_expanded(7).expect("expanded");
        // 12 smoke + 3 enrich + 2 rw + 5 concurrency = 22
        assert!(
            results.len() >= 22,
            "expected ≥22 plans, got {}",
            results.len()
        );
        let mut saw_deep = false;
        let mut saw_writes = false;
        for r in &results {
            assert!(r.side_a.result.is_some(), "{}", r.plan_id);
            let d = r.side_a.detail.as_deref().unwrap_or("");
            if d.contains("cursor pages=") && d.contains("deep_start=") {
                saw_deep = true;
            }
            if d.contains("writes=") {
                // detail format: writes=N with N>0
                if d.contains("writes=0") {
                    panic!("mixed R/W produced zero writes: {d}");
                }
                saw_writes = true;
            }
        }
        assert!(saw_deep, "expected deep cursor detail");
        assert!(saw_writes, "expected mixed R/W writes");
        assert!(
            results.iter().any(|r| r.plan_id.contains("agg_stats")),
            "expected agg plan"
        );
        assert!(
            results.iter().any(|r| r.plan_id.contains("many")),
            "expected enrich many variant"
        );
        assert!(
            results.iter().any(|r| r.plan_id.contains("_c8")),
            "expected concurrency 8 plan executed"
        );
        assert!(
            results.iter().any(|r| r.plan_id.contains("high_band")
                || r.side_a
                    .detail
                    .as_deref()
                    .unwrap_or("")
                    .contains("conditional high_band")),
            "expected conditional computed"
        );
    }

    #[test]
    fn publish_evidence_bundle() {
        let root = workspace_root();
        let path = publish_smoke_evidence(&root, 0x04_43, "q4-3-unit").expect("publish");
        assert!(path.is_file());
        let report = root.join("spec/rql/qualification/harness-v1/q4_3_metrics_adapters_report.json");
        assert!(report.is_file());
        let v: Value = serde_json::from_str(&fs::read_to_string(report).unwrap()).unwrap();
        assert_eq!(v["format"], "residiuum-rql-q4-3-metrics-adapters-report-v1");
        assert_eq!(v["summary"]["smoke_cells"], 12);
        assert_eq!(v["summary"]["logical_ready_with_result"], 12);
        assert_eq!(v["summary"]["lane_s_fixture_identity"], true);
    }

    #[test]
    fn lane_s_fixture_identity() {
        let (hash, ok) = run_smoke_lane_server_fixture_identity(1).unwrap();
        assert!(ok);
        assert_eq!(hash.len(), 64);
    }
}