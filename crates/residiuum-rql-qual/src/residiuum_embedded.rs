//! Residiuum embedded product adapter (feature `residiuum-embedded`).
//!
//! Seeds shared logical work into a temp Heap and runs `CollectionClient::rql`.
//! Labor scaffold — not a Gate-1 competitive claim.

#![cfg(feature = "residiuum-embedded")]

use crate::canonicalize::{canonicalize_rows, ResultRow};
use crate::cell_plan::MeasuredCellPlan;
use crate::engine::{AdapterError, AdapterStatus, EngineRunOutcome};
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
                status: AdapterStatus::Ready,
                result: None,
                metrics: None,
                refuse_code: Some(format!("product_rql_refuse:{}", e.code())),
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
        status: AdapterStatus::Ready,
        result: Some(canon),
        metrics: Some(metrics),
        refuse_code: None,
        detail: Some(format!("product_embedded plan={}", plan.plan_id)),
        shared_work_hash: Some(work.content_hash.clone()),
    })
}
