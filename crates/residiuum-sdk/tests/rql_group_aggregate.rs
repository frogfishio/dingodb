//! Q2 `pkg_group_aggregate` — group by + count/sum/min/max/avg on product path.

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use residiuum_sdk::{
    compile_rql_full, execute_rql_full, CollectionBindings, HeapClient, Parameters,
    QueryRunOptions, ResidiuumDeployment,
};
use residiuum_store::{publish_staged_genesis, stage_heap_genesis, HeapMetaLayout};
use std::collections::BTreeMap;
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
    mint_capability(
        slot,
        &cert,
        TrustedInstant {
            unix_s: 1_700_000_000,
        },
    )
    .unwrap()
}

fn uuid() -> [u8; 16] {
    *residiuum_heap::CollectionId::new_random()
        .unwrap()
        .as_bytes()
}

fn open_client() -> HeapClient {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    // Keep tempdir alive for the process duration of the test via leak (tests are short).
    std::mem::forget(dir);
    let deployment = ResidiuumDeployment::create(&root).unwrap();
    let layout = HeapMetaLayout::new(&root);
    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_bytes = *HeapId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_bytes, uuid(), "heap-agg").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    let heap_id = HeapId::from_bytes_unchecked_nonzero(heap_bytes).unwrap();
    let dep_id = DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap();
    HeapClient::from(deployment.open_heap(mint_cap_for(heap_id, dep_id)))
}

#[test]
fn group_by_region_count_and_sum() {
    let mut client = open_client();
    let mut orders = client.create_collection("orders").unwrap().collection;
    for (k, region, amount, status) in [
        ("o1", "us", 10, "paid"),
        ("o2", "us", 5, "paid"),
        ("o3", "eu", 7, "open"),
        ("o4", "eu", 3, "paid"),
        ("o5", "us", 2, "open"),
    ] {
        orders
            .put(
                k,
                &serde_json::json!({"region": region, "amount": amount, "status": status}),
            )
            .unwrap();
    }

    let page = execute_rql_full(
        &mut client,
        "from orders group by region project region, count() as order_count, sum(amount) as total_amount",
        &Parameters::default(),
        QueryRunOptions::default(),
    )
    .expect("execute group by");

    assert_eq!(page.rows.len(), 2);
    let mut by = BTreeMap::new();
    for (_k, v) in &page.rows {
        let r = v
            .get("region")
            .and_then(|x| x.as_str())
            .unwrap()
            .to_string();
        let c = v.get("order_count").and_then(|x| x.as_u64()).unwrap();
        let s = v.get("total_amount").and_then(|x| x.as_i64()).unwrap();
        by.insert(r, (c, s));
    }
    assert_eq!(by.get("us"), Some(&(3, 17)));
    assert_eq!(by.get("eu"), Some(&(2, 10)));
}

#[test]
fn global_min_max_avg_with_where() {
    let mut client = open_client();
    let mut orders = client.create_collection("orders").unwrap().collection;
    for (k, amount, status) in [
        ("o1", 10, "paid"),
        ("o2", 30, "paid"),
        ("o3", 5, "open"),
        ("o4", 20, "paid"),
    ] {
        orders
            .put(k, &serde_json::json!({"amount": amount, "status": status}))
            .unwrap();
    }

    let page = execute_rql_full(
        &mut client,
        r#"from orders where status = "paid" project min(amount) as min_amount, max(amount) as max_amount, avg(amount) as avg_amount"#,
        &Parameters::default(),
        QueryRunOptions::default(),
    )
    .expect("execute global agg");

    assert_eq!(page.rows.len(), 1);
    let v = &page.rows[0].1;
    assert_eq!(v.get("min_amount").and_then(|x| x.as_i64()), Some(10));
    assert_eq!(v.get("max_amount").and_then(|x| x.as_i64()), Some(30));
    // avg of 10,30,20 = 20
    assert_eq!(v.get("avg_amount").and_then(|x| x.as_i64()), Some(20));
}

#[test]
fn compile_group_plan_has_group_agg() {
    let mut client = open_client();
    let _ = client.create_collection("orders").unwrap();
    let infos = client.list_collections().unwrap();
    let mut bindings = CollectionBindings::default();
    for info in &infos {
        bindings.bind(&info.name, info.collection_id);
    }
    let compiled = compile_rql_full(
        "from orders group by status project status, count() as n",
        &bindings,
    )
    .expect("compile");
    assert!(compiled.base.plan.group_agg.is_active());
    assert_eq!(compiled.base.plan.group_agg.group_by.len(), 1);
    assert_eq!(compiled.base.plan.group_agg.aggregates.len(), 1);
}
