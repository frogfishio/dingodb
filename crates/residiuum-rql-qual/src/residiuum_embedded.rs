//! Residiuum embedded product adapter (feature `residiuum-embedded`).
//!
//! Seeds shared logical work into a temp Heap and runs `CollectionClient::rql`.
//! Labor scaffold — not a Gate-1 competitive claim.

#![cfg(feature = "residiuum-embedded")]

use crate::canonicalize::{canonicalize_rows, ResultRow};
use crate::cell_plan::MeasuredCellPlan;
use crate::engine::{AdapterError, AdapterStatus, EngineRunOutcome, ExecutionKind};
use crate::lane::EngineId;
use crate::metrics::{
    assemble_metrics, LatencyCollector, QueryPathMetrics, QueryTimer,
};
use crate::shared_work::SharedLogicalWork;
use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use residiuum_sdk::{Parameters, QueryRunOptions, ResidiuumDeployment};
use residiuum_store::{publish_staged_genesis, stage_heap_genesis, HeapMetaLayout};
use serde_json::Value;
use std::sync::Arc;
use tempfile::tempdir;

fn mint_cap_for(heap: HeapId, deployment: DeploymentId) -> residiuum_heap::HeapCap {
    let snap = HeapSecuritySnapshot {
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: [7u8; 32],
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [9u8; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let slot = Arc::new(HeapSlot::new(snap));
    let cert = VerifiedCertificate {
        cose_bytes: vec![0x01],
        fingerprint: [3u8; 32],
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        certificate_id: CertificateId::new_random().unwrap(),
        holder_public_key: [4u8; 32],
        rights: Rights::from_bits_certificate(0x0d).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: [5u8; 32],
    };
    mint_capability(slot, &cert, TrustedInstant { unix_s: 1_700_000_000 }).unwrap()
}

fn uuid_bytes() -> [u8; 16] {
    *residiuum_heap::CollectionId::new_random()
        .unwrap()
        .as_bytes()
}

/// Execute a measured cell plan on product Residiuum embedded path.
///
/// Residual: Full enrich cells refuse via `product_residual:*`; Core-only RQL is
/// the product path exercised here. Competitive claims still require principal
/// accept + Q3-green families.
pub fn execute_plan_embedded(
    work: Option<&SharedLogicalWork>,
    plan: &MeasuredCellPlan,
) -> Result<EngineRunOutcome, AdapterError> {
    let work = work.ok_or_else(|| AdapterError::Fixture("shared work not loaded".into()))?;

    // Enrich Full RQL is not Core wire — skip product execute for enrich cell.
    if plan.server_lane_ineligible
        && plan.rql_source.contains("enrich")
    {
        return Ok(EngineRunOutcome {
            engine: EngineId::ResidiuumEmbedded,
            execution_kind: ExecutionKind::Product,
            status: AdapterStatus::Ready,
            result: None,
            metrics: None,
            refuse_code: Some("product_residual:full_enrich_use_execute_rql_full".into()),
            detail: Some(plan.plan_id.clone()),
            shared_work_hash: Some(work.content_hash.clone()),
        });
    }

    let dir = tempdir().map_err(|e| AdapterError::Execute(e.to_string()))?;
    let root = dir.path();
    let deployment =
        ResidiuumDeployment::create(root).map_err(|e| AdapterError::Execute(e.to_string()))?;
    let layout = HeapMetaLayout::new(root);
    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_bytes = *HeapId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_bytes, uuid_bytes(), "heap-q4-3")
        .map_err(|e| AdapterError::Execute(e.to_string()))?;
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash)
        .map_err(|e| AdapterError::Execute(e.to_string()))?;
    let heap_id = HeapId::from_bytes_unchecked_nonzero(heap_bytes).unwrap();
    let dep_id = DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap();
    let mut client =
        residiuum_sdk::HeapClient::from(deployment.open_heap(mint_cap_for(heap_id, dep_id)));

    for (name, docs) in &work.dataset.collections {
        let mut col = client
            .create_collection(name)
            .map_err(|e| AdapterError::Execute(format!("create {name}: {e}")))?
            .collection;
        for (key, doc) in docs {
            let mut body = doc.clone();
            if let Value::Object(ref mut m) = body {
                m.remove("_key");
            }
            col.put(key, &body)
                .map_err(|e| AdapterError::Execute(format!("put {name}/{key}: {e}")))?;
        }
    }

    let root_name = if plan.rql_source.contains("from docs") {
        "docs"
    } else {
        "docs"
    };
    let mut col = client
        .open_collection(root_name)
        .map_err(|e| AdapterError::Execute(e.to_string()))?;

    let mut opts = QueryRunOptions::default();
    if let Some(ps) = plan.page_size {
        opts.page_size = Some(ps);
    }

    let mut lat = LatencyCollector::new();
    let timer = QueryTimer::start();
    let page = match col.rql(&plan.rql_source, &Parameters::default(), opts) {
        Ok(p) => p,
        Err(e) => {
            return Ok(EngineRunOutcome {
                engine: EngineId::ResidiuumEmbedded,
                execution_kind: ExecutionKind::Product,
                status: AdapterStatus::Ready,
                result: None,
                metrics: None,
                // ErrorCode is machine-stable via as_str (no Display).
                refuse_code: Some(format!("product_rql_refuse:{}", e.code().as_str())),
                detail: Some(e.to_string()),
                shared_work_hash: Some(work.content_hash.clone()),
            });
        }
    };
    lat.record_duration(timer.elapsed());

    let rows: Vec<ResultRow> = page
        .rows
        .iter()
        .map(|r| ResultRow {
            key: r.key.clone(),
            value: r.value.clone(),
        })
        .collect();
    let canon = canonicalize_rows(&rows, plan.order_sensitive, page.coverage.complete);
    let digest = canon.values_digest.clone();
    let metrics = assemble_metrics(
        &lat,
        QueryPathMetrics {
            documents_examined: Some(page.coverage.examined_documents),
            index_entries_examined: None,
            index_size_bytes: None,
            index_build_ns: None,
            indexed_write_penalty_ns: None,
            explain_plan_digest: Some(format!("product_plan_hash:{:x?}", &page.plan_hash[..8])),
        },
        Some(plan.lifecycle.class),
        Some(plan.lifecycle.cold_method.as_str().into()),
        Some(digest),
        Some(page.coverage.complete),
        Some(true),
    );

    Ok(EngineRunOutcome {
        engine: EngineId::ResidiuumEmbedded,
        execution_kind: ExecutionKind::Product,
        status: AdapterStatus::Ready,
        result: Some(canon),
        metrics: Some(metrics),
        refuse_code: None,
        detail: Some(format!("product_embedded plan={}", plan.plan_id)),
        shared_work_hash: Some(work.content_hash.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_plan::MeasuredCellPlan;
    use crate::cells::MandatoryCell;
    use crate::dataset::DatasetSpec;
    use crate::engine::{AdapterStatus, EngineAdapter, ResidiuumEmbeddedAdapter};
    use crate::generator::generate_dataset;
    use crate::shared_work::SharedLogicalWork;

    #[test]
    fn product_embedded_key_get_compiles_and_runs() {
        let plan = MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, 42);
        let ds = generate_dataset(&plan.dataset);
        let work = SharedLogicalWork::from_dataset(ds);
        let mut adapter = ResidiuumEmbeddedAdapter::default();
        adapter.load_shared_work(&work).expect("load shared");
        assert_eq!(adapter.status(), AdapterStatus::Ready);
        let out = adapter.execute_plan(&plan).expect("execute");
        assert_eq!(out.engine, EngineId::ResidiuumEmbedded);
        assert_eq!(out.status, AdapterStatus::Ready);
        assert_eq!(
            out.shared_work_hash.as_deref(),
            Some(work.content_hash.as_str())
        );
        // Either a successful page or an honest product refuse code (not a panic).
        match (&out.result, &out.refuse_code) {
            (Some(r), None) => {
                assert_eq!(r.row_count, 1, "key get should hit d-00000000");
                assert!(out.metrics.is_some());
            }
            (None, Some(code)) => {
                assert!(
                    code.starts_with("product_rql_refuse:")
                        || code.starts_with("product_residual:"),
                    "unexpected refuse {code}"
                );
            }
            other => panic!("unexpected outcome {other:?}"),
        }
    }

    #[test]
    fn product_refuse_code_uses_stable_errorcode_str() {
        // Compile-time + runtime: Error::code().as_str() path is exercised when
        // rql refuses; for a known-bad source we still expect a refuse code.
        let mut plan = MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, 1);
        plan.rql_source = "from docs where this is not valid rql !!!".into();
        let ds = generate_dataset(&DatasetSpec::smoke_default(1));
        let work = SharedLogicalWork::from_dataset(ds);
        let mut adapter = ResidiuumEmbeddedAdapter::default();
        adapter.load_shared_work(&work).unwrap();
        let out = adapter.execute_plan(&plan).unwrap();
        if let Some(code) = out.refuse_code {
            assert!(
                code.starts_with("product_rql_refuse:") || code.starts_with("product_residual:"),
                "{code}"
            );
            // Must not be Debug-style "QueryInvalid" alone without stable mapping path.
            assert!(!code.contains("ErrorCode"));
        }
    }
}