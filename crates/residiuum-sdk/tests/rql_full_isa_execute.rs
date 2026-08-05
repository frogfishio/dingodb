//! RQL-X5b: full-language execute only after ISA encode→decode.
//! Non-empty enrich / within / project from ISA bytes (not CompiledRqlFull sidecar).

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use residiuum_sdk::{
    compile_rql_full, decode_isa, encode_full_program, execute_full_isa_with, execute_isa_bytes,
    CollectionBindings, HeapClient, Parameters, QueryRunOptions, ResidiuumDeployment,
    RqlFullExecuteOptions, RQL_FULL_PROFILE,
};
use residiuum_store::{publish_staged_genesis, stage_heap_genesis, HeapMetaLayout};
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

fn uuid() -> [u8; 16] {
    *residiuum_heap::CollectionId::new_random()
        .unwrap()
        .as_bytes()
}

fn seed_orders_customers_lines_products(client: &mut HeapClient) {
    let mut orders = client.create_collection("orders").unwrap().collection;
    let mut customers = client.create_collection("customers").unwrap().collection;
    let mut lines = client.create_collection("line_items").unwrap().collection;
    let mut products = client.create_collection("products").unwrap().collection;

    orders
        .put(
            "o1",
            &serde_json::json!({"order_id": "o1", "customer_id": "c1"}),
        )
        .unwrap();
    customers
        .put("c1", &serde_json::json!({"id": "c1", "name": "Ada"}))
        .unwrap();
    lines
        .put(
            "l1",
            &serde_json::json!({"order_id": "o1", "product_id": "p1", "sku": "A"}),
        )
        .unwrap();
    products
        .put("p1", &serde_json::json!({"id": "p1", "name": "Widget"}))
        .unwrap();
}

const FULL_SRC: &str = r#"from orders
           enrich customer using customers matching customer_id = id expect exactly_one
           enrich items using line_items matching order_id = order_id expect many
           within items as item {
             enrich product using products as candidate
               matching item.product_id = candidate.id
               expect exactly_one
           }
           project {
             order_id,
             customer { name },
             items { sku, product { name } }
           }
           page size 64"#;

#[test]
fn execute_full_isa_enrich_within_project_nonempty() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let deployment = ResidiuumDeployment::create(root).unwrap();
    let layout = HeapMetaLayout::new(root);
    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_bytes = *HeapId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_bytes, uuid(), "heap-x5b").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    let heap_id = HeapId::from_bytes_unchecked_nonzero(heap_bytes).unwrap();
    let dep_id = DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap();
    let mut client = HeapClient::from(deployment.open_heap(mint_cap_for(heap_id, dep_id)));
    seed_orders_customers_lines_products(&mut client);

    let infos = client.list_collections().unwrap();
    let mut bindings = CollectionBindings::default();
    for info in &infos {
        bindings.bind(&info.name, info.collection_id);
    }
    let compiled = compile_rql_full(FULL_SRC, &bindings).expect("compile");
    let isa = encode_full_program(
        &compiled.base.plan,
        compiled.base.budget,
        &compiled.pipeline,
        &compiled.project,
    )
    .expect("encode");
    let prog = decode_isa(&isa).expect("decode");
    assert!(prog.full.is_some());
    assert_eq!(prog.full.as_ref().unwrap().pipeline, compiled.pipeline);

    // Core entry refuses full-language ISA.
    let mut base = client.open_collection("orders").unwrap();
    let heap_id = base.heap_id();
    let collection_id = base.id();
    let err = execute_isa_bytes(
        &mut base,
        &isa,
        &Parameters::default().values,
        &QueryRunOptions::default(),
        heap_id,
        collection_id,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("full ISA entry") || err.to_string().contains("RQL-X5b"),
        "{err}"
    );
    drop(base);

    let page = execute_full_isa_with(
        &mut client,
        &isa,
        &Parameters::default(),
        RqlFullExecuteOptions {
            query: QueryRunOptions::default(),
            force_enrich_scan: false,
        },
    )
    .expect("execute_full_isa");

    assert_eq!(page.profile, RQL_FULL_PROFILE);
    assert_eq!(page.pipeline, prog.full.as_ref().unwrap().pipeline);
    assert_eq!(page.project, prog.full.as_ref().unwrap().project);
    assert_eq!(page.rows.len(), 1);
    let row = &page.rows[0].1;
    assert_eq!(row["order_id"], "o1");
    assert_eq!(row["customer"]["name"], "Ada");
    let item = &row["items"].as_array().unwrap()[0];
    assert_eq!(item["sku"], "A");
    assert_eq!(item["product"]["name"], "Widget");
}

#[test]
fn corrupted_full_isa_refuses_execute() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let deployment = ResidiuumDeployment::create(root).unwrap();
    let layout = HeapMetaLayout::new(root);
    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_bytes = *HeapId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_bytes, uuid(), "heap-x5b-bad").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    let heap_id = HeapId::from_bytes_unchecked_nonzero(heap_bytes).unwrap();
    let dep_id = DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap();
    let mut client = HeapClient::from(deployment.open_heap(mint_cap_for(heap_id, dep_id)));
    seed_orders_customers_lines_products(&mut client);

    let infos = client.list_collections().unwrap();
    let mut bindings = CollectionBindings::default();
    for info in &infos {
        bindings.bind(&info.name, info.collection_id);
    }
    let compiled = compile_rql_full(FULL_SRC, &bindings).expect("compile");
    let mut isa = encode_full_program(
        &compiled.base.plan,
        compiled.base.budget,
        &compiled.pipeline,
        &compiled.project,
    )
    .expect("encode");
    let last = isa.len() - 1;
    isa[last] ^= 0x5a;

    let err = execute_full_isa_with(
        &mut client,
        &isa,
        &Parameters::default(),
        RqlFullExecuteOptions::default(),
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("isa") || err.to_string().contains("QueryInvalid"),
        "{err}"
    );
}
