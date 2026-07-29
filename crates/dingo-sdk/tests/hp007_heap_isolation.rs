//! HP-007 Accept: two heaps with identical collection names cannot exchange
//! typed handles, cursors, batch members, or pooled connections.

use dingo_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use dingo_sdk::DingoDeployment;
use dingo_store::{
    create_object, publish_staged_genesis, stage_heap_genesis, HeapMetaLayout, ObjectKind,
};
use std::sync::Arc;
use tempfile::tempdir;

fn mint_cap_for(heap: HeapId, deployment: DeploymentId) -> dingo_heap::HeapCap {
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
    *dingo_heap::CollectionId::new_random().unwrap().as_bytes()
}

#[test]
fn identical_collection_names_cannot_cross_heaps() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let deployment = DingoDeployment::create(root).unwrap();
    let layout = HeapMetaLayout::new(root);

    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_a = *HeapId::new_random().unwrap().as_bytes();
    let heap_b = *HeapId::new_random().unwrap().as_bytes();
    let coll_a = uuid();
    let coll_b = uuid();

    let sa = stage_heap_genesis(&layout, dep, heap_a, uuid(), "heap-a").unwrap();
    let sb = stage_heap_genesis(&layout, dep, heap_b, uuid(), "heap-b").unwrap();
    publish_staged_genesis(&layout, &sa.staging_id, &sa.descriptor_hash).unwrap();
    publish_staged_genesis(&layout, &sb.staging_id, &sb.descriptor_hash).unwrap();

    // Same human name, distinct immutable ids, distinct owner heaps.
    create_object(
        &layout,
        &heap_a,
        ObjectKind::Collection,
        coll_a,
        uuid(),
        "users",
    )
    .unwrap();
    create_object(
        &layout,
        &heap_b,
        ObjectKind::Collection,
        coll_b,
        uuid(),
        "users",
    )
    .unwrap();

    let cap_a = mint_cap_for(
        HeapId::from_bytes_unchecked_nonzero(heap_a).unwrap(),
        DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap(),
    );
    let cap_b = mint_cap_for(
        HeapId::from_bytes_unchecked_nonzero(heap_b).unwrap(),
        DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap(),
    );

    let ha = deployment.open_heap(cap_a);
    let hb = deployment.open_heap(cap_b);
    assert!(!ha.same_capability(&hb));
    assert!(ha.require_same_instance(&hb).is_err());

    let ca = ha.collection("users").unwrap();
    let cb = hb.collection("users").unwrap();
    assert_eq!(ca.name(), "users");
    assert_eq!(cb.name(), "users");
    assert_ne!(ca.id().as_bytes(), cb.id().as_bytes());
    assert_ne!(ca.heap_id().as_bytes(), cb.heap_id().as_bytes());
    assert!(!ca.same_heap_instance(&cb));

    // Cursors are not transferable across heaps.
    let cursor_a = ca.sign_cursor(b"pos-1");
    assert!(ca.verify_cursor(&cursor_a).is_ok());
    assert!(cb.verify_cursor(&cursor_a).is_err());

    // Batch members must share capability instance.
    let mut batch = ha.begin_batch();
    batch.add_collection(&ca).unwrap();
    assert!(batch.add_collection(&cb).is_err());

    // Pooled connections are heap-instance tagged.
    let conn_a = ha.checkout_connection().unwrap();
    assert!(conn_a.require_heap(&ha).is_ok());
    assert!(conn_a.require_heap(&hb).is_err());
}
