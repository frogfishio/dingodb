//! APB-1 G1: `HeapClient` / `CollectionClient` façade over embedded [`Heap`].
//!
//! Normative: MUST_ADD §5; inventory `APB1_CLIENT_GAP_INVENTORY.md` G1–G2.
//! Remote bind covered by server `apb1_heap_client_from_remote_*` (G1b).
//! No package accept.

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use residiuum_sdk::{ErrorCode, HeapClient, ResidiuumDeployment};
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
        rights: Rights::from_bits_certificate(0x5).unwrap(),
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

#[test]
fn facade_from_heap_create_list_open_put_get() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let deployment = ResidiuumDeployment::create(root).unwrap();
    let layout = HeapMetaLayout::new(root);

    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_bytes = *HeapId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_bytes, uuid(), "heap-apb1").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();

    let heap_id = HeapId::from_bytes_unchecked_nonzero(heap_bytes).unwrap();
    let dep_id = DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap();
    let cap = mint_cap_for(heap_id, dep_id);
    let heap = deployment.open_heap(cap);

    let mut client = HeapClient::from(heap);
    assert!(client.is_bound());
    assert_eq!(client.id(), heap_id);

    let created = client.create_collection("orders").expect("create");
    assert_eq!(created.collection.name(), "orders");
    assert_eq!(created.collection.heap_id(), heap_id);
    assert!(created.collection.is_bound());
    assert_eq!(
        created.receipt.operation,
        residiuum_sdk::AdminOperation::CreateCollection
    );
    assert_eq!(created.receipt.heap_id, heap_id);
    assert_eq!(created.receipt.collection_id, created.collection.id());

    let listed = client.list_collections().expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "orders");
    assert_eq!(listed[0].collection_id, created.collection.id());

    let mut opened = client.open_collection("orders").expect("open");
    assert_eq!(opened.id(), created.collection.id());
    assert!(opened.is_bound());

    opened
        .put("k1", &serde_json::json!({"n": 1}))
        .expect("put via façade");
    let got = opened.get("k1").expect("get");
    assert_eq!(got, Some(serde_json::json!({"n": 1})));

    match client.create_collection("orders") {
        Ok(_) => panic!("duplicate name must fail"),
        Err(err) => assert_eq!(err.code(), ErrorCode::AlreadyExists),
    }
}

#[test]
fn unbound_facade_still_fail_closed() {
    let mut b = [1u8; 16];
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let hid = HeapId::from_bytes(b).expect("heap id");
    let mut client = HeapClient::from_id_for_contract(hid);
    assert!(!client.is_bound());
    let err = client.create_collection("x").unwrap_err();
    assert_eq!(err.code(), ErrorCode::Internal);
}