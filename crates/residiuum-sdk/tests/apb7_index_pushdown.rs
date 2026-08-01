//! APB-7 T4: equality index pushdown vs full-scan oracle.

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use residiuum_sdk::{
    field, param, HeapClient, Parameters, QueryRunOptions, ResidiuumDeployment,
};
use residiuum_store::{publish_staged_genesis, stage_heap_genesis, HeapMetaLayout, IndexState};
use std::sync::Arc;
use tempfile::{tempdir, TempDir};

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
        // Read|Write|IndexAdmin = 13
        rights: Rights::from_bits_certificate(0x0d).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: [5u8; 32],
    };
    mint_capability(slot, &cert, TrustedInstant { unix_s: 1_700_000_000 }).unwrap()
}

fn uuid() -> [u8; 16] {
    *residiuum_heap::CollectionId::new_random()
        .unwrap()
        .as_bytes()
}

fn open_bound_client() -> (TempDir, HeapClient) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let deployment = ResidiuumDeployment::create(root).unwrap();
    let layout = HeapMetaLayout::new(root);
    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_bytes = *HeapId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_bytes, uuid(), "heap-apb7-t4").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    let heap_id = HeapId::from_bytes_unchecked_nonzero(heap_bytes).unwrap();
    let dep_id = DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap();
    let cap = mint_cap_for(heap_id, dep_id);
    (dir, HeapClient::from(deployment.open_heap(cap)))
}

#[test]
fn index_pushdown_matches_full_scan_oracle() {
    let (_dir, mut client) = open_bound_client();
    let mut col = client.create_collection("orders").unwrap().collection;

    col.put("a", &serde_json::json!({"status": "open", "n": 1}))
        .unwrap();
    col.put("b", &serde_json::json!({"status": "closed", "n": 2}))
        .unwrap();
    col.put("c", &serde_json::json!({"status": "open", "n": 3}))
        .unwrap();
    col.put("d", &serde_json::json!({"status": "open", "n": 4}))
        .unwrap();
    col.put("e", &serde_json::json!({"status": "closed", "n": 5}))
        .unwrap();

    // Oracle without index: list_keys + get.
    let mut oracle = Vec::new();
    for k in col.list_keys(Some(100), None).unwrap() {
        if let Some(v) = col.get(&k).unwrap() {
            if v["status"] == "open" {
                oracle.push(k);
            }
        }
    }
    assert_eq!(
        oracle,
        vec!["a".to_string(), "c".to_string(), "d".to_string()]
    );

    // Build ready index on status.
    let info = col
        .indexes()
        .create("by_status", &["status"])
        .expect("create_index");
    assert_eq!(info.state, IndexState::Ready);
    assert!(info.complete_coverage);

    // Direct lookup should return open keys (order not guaranteed → sort).
    let mut hit = col
        .lookup_index_keys(&[("status".into(), serde_json::json!("open"))])
        .expect("lookup")
        .expect("index hit");
    hit.sort();
    assert_eq!(hit, oracle);

    // Application Core query must match oracle (index path under the hood).
    let mut params = Parameters::default();
    params
        .values
        .insert("status".into(), serde_json::json!("open"));
    let page = col
        .query()
        .where_(field("status").unwrap().eq(param("status")))
        .run(&params, QueryRunOptions::default())
        .expect("query with index");
    let keys: Vec<_> = page.rows.iter().map(|r| r.key.clone()).collect();
    assert_eq!(keys, oracle);
    assert!(page.exhausted);

    // RQL path same.
    let rql = col
        .rql(
            r#"from orders where status = $status"#,
            &params,
            QueryRunOptions::default(),
        )
        .expect("rql");
    let rql_keys: Vec<_> = rql.rows.iter().map(|r| r.key.clone()).collect();
    assert_eq!(rql_keys, oracle);
}

#[test]
fn index_miss_empty_when_ready_complete() {
    let (_dir, mut client) = open_bound_client();
    let mut col = client.create_collection("orders").unwrap().collection;
    col.put("a", &serde_json::json!({"status": "open"}))
        .unwrap();
    let info = col.indexes().create("by_status", &["status"]).unwrap();
    assert_eq!(info.state, IndexState::Ready);
    assert!(info.complete_coverage);

    let hit = col
        .lookup_index_keys(&[("status".into(), serde_json::json!("missing"))])
        .unwrap()
        .expect("usable index path");
    assert!(hit.is_empty(), "ready+complete may prove absence");

    let page = col
        .rql(
            r#"from orders where status = "missing""#,
            &Parameters::default(),
            QueryRunOptions::default(),
        )
        .unwrap();
    assert!(page.rows.is_empty());
    assert!(page.exhausted);
}

#[test]
fn without_index_still_scans() {
    let (_dir, mut client) = open_bound_client();
    let mut col = client.create_collection("orders").unwrap().collection;
    col.put("a", &serde_json::json!({"status": "open"})).unwrap();
    col.put("b", &serde_json::json!({"status": "closed"}))
        .unwrap();
    // No index created — must still answer correctly via scan.
    assert!(col
        .lookup_index_keys(&[("status".into(), serde_json::json!("open"))])
        .unwrap()
        .is_none());
    let page = col
        .rql(
            r#"from orders where status = "open""#,
            &Parameters::default(),
            QueryRunOptions::default(),
        )
        .unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].key, "a");
}
