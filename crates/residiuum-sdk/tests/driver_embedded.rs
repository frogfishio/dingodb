//! DRV-1/DRV-5 embedded smart-client contract and concurrency evidence.

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapCap, HeapId, HeapSecuritySnapshot, HeapSlot, Rights,
    SecurityRevision, TrustedInstant, VerifiedCertificate,
};
use residiuum_sdk::driver::{
    Client, Collection, CreateCollectionOptions, DeleteOptions, EmbeddedOptions, ErrorCode,
    OperationContext, PutOptions, ReplaceOptions,
};
use residiuum_sdk::ResidiuumDeployment;
use residiuum_store::{publish_staged_genesis, stage_heap_genesis, HeapMetaLayout};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use tempfile::tempdir;

struct ThreadWake(std::thread::Thread);

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::park(),
        }
    }
}

fn mint_cap_for(heap: HeapId, deployment: DeploymentId) -> HeapCap {
    let snapshot = HeapSecuritySnapshot {
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: [7; 32],
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [9; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let slot = Arc::new(HeapSlot::new(snapshot));
    let certificate = VerifiedCertificate {
        cose_bytes: vec![1],
        fingerprint: [3; 32],
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        certificate_id: CertificateId::new_random().unwrap(),
        holder_public_key: [4; 32],
        rights: Rights::from_bits_certificate(0x0d).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: [5; 32],
    };
    mint_capability(
        slot,
        &certificate,
        TrustedInstant {
            unix_s: 1_700_000_000,
        },
    )
    .unwrap()
}

fn prepared_deployment() -> (tempfile::TempDir, HeapCap) {
    let directory = tempdir().unwrap();
    let root = directory.path();
    let deployment = ResidiuumDeployment::create(root).unwrap();
    let layout = HeapMetaLayout::new(root);
    let deployment_id = DeploymentId::new_random().unwrap();
    let heap_id = HeapId::new_random().unwrap();
    let collection_seed = *residiuum_heap::CollectionId::new_random()
        .unwrap()
        .as_bytes();
    let staged = stage_heap_genesis(
        &layout,
        *deployment_id.as_bytes(),
        *heap_id.as_bytes(),
        collection_seed,
        "driver-test",
    )
    .unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    let capability = mint_cap_for(heap_id, deployment_id);
    drop(deployment);
    (directory, capability)
}

fn operation(byte: u8) -> residiuum_client::OperationId {
    residiuum_client::OperationId([byte; 16])
}

fn assert_handle_contract<T: Clone + Send + Sync>() {}

#[test]
fn compile_contract_handles_are_clone_send_sync() {
    assert_handle_contract::<Client>();
    assert_handle_contract::<Collection<Value>>();
}

#[test]
fn embedded_driver_is_bounded_concurrent_idempotent_and_shared_close() {
    let (directory, capability) = prepared_deployment();
    let client = block_on(Client::open_embedded(
        EmbeddedOptions::new(directory.path(), capability)
            .workers(2)
            .queue_capacity(8),
    ))
    .unwrap();
    assert!(client.capabilities().embedded);
    assert!(client.capabilities().mutation_identity);
    assert!(!client.capabilities().remote_pooling);
    assert!(client.open_report().total_ns > 0);

    let collection: Collection<Value> = block_on(client.create_collection(
        "orders",
        CreateCollectionOptions {
            context: OperationContext {
                operation_id: Some(operation(1)),
                ..OperationContext::default()
            },
        },
    ))
    .unwrap();
    assert_eq!(block_on(client.list_collections()).unwrap().len(), 1);

    let mut threads = Vec::new();
    for index in 0..8u8 {
        let collection = collection.clone();
        threads.push(std::thread::spawn(move || {
            block_on(collection.put(format!("k-{index}"), &json!({ "n": index }))).unwrap()
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(client.inspect().workers, 2);
    assert_eq!(client.inspect().queue_capacity, 8);

    let options = PutOptions {
        context: OperationContext {
            operation_id: Some(operation(2)),
            ..OperationContext::default()
        },
    };
    let first =
        block_on(collection.put_with("stable", &json!({ "v": 1 }), options.clone())).unwrap();
    let replay =
        block_on(collection.put_with("stable", &json!({ "v": 1 }), options.clone())).unwrap();
    assert!(!first.deduplicated);
    assert!(replay.deduplicated);
    assert_eq!(first.storage.event_id, replay.storage.event_id);

    let conflict =
        block_on(collection.put_with("stable", &json!({ "v": 2 }), options)).unwrap_err();
    assert_eq!(conflict.code, ErrorCode::OperationIdentityConflict);

    let replaced = block_on(collection.replace(
        "stable",
        &json!({ "v": 3 }),
        ReplaceOptions {
            if_version: first.storage.event_id,
            context: OperationContext {
                operation_id: Some(operation(3)),
                ..OperationContext::default()
            },
        },
    ))
    .unwrap();
    assert_eq!(
        block_on(collection.get("stable")).unwrap(),
        Some(json!({ "v": 3 }))
    );

    let delete_options = DeleteOptions {
        if_version: Some(replaced.storage.event_id),
        context: OperationContext {
            operation_id: Some(operation(4)),
            ..OperationContext::default()
        },
        ..DeleteOptions::default()
    };
    let deleted = block_on(collection.delete("stable", delete_options.clone())).unwrap();
    let delete_replay = block_on(collection.delete("stable", delete_options)).unwrap();
    assert!(deleted.storage.removed);
    assert!(delete_replay.storage.removed);
    assert!(delete_replay.deduplicated);
    assert_eq!(deleted.storage.event_id, delete_replay.storage.event_id);

    let clone = client.clone();
    block_on(client.close()).unwrap();
    assert!(clone.inspect().closed);
    let closed = block_on(collection.get("stable")).unwrap_err();
    assert_eq!(closed.code, ErrorCode::Closed);
}
