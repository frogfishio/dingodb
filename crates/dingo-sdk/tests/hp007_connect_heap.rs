//! HP-007 residual: `Dingo::connect_heap` qualified remote session + process ops.

use dingo_heap::{
    verify_certificate, HeapAdministrativeState, HeapSecuritySnapshot, HeapSlot, SecurityRevision,
    HEAP_PROFILE,
};
use dingo_sdk::{
    Dingo, HeapCredential, InMemoryHolderKey, RemoteHeapOptions, TlsClientOptions, TlsServerOptions,
};
use dingo_server::{
    serve_store_with, HeapAuthAuditLog, ResidentHeap, ResidentHeapRegistry, ServeOptions,
};
use dingo_store::Store;
use rcgen::{
    BasicConstraints, CertificateParams, DnType, IsCa, KeyPair, KeyUsagePurpose, SanType,
};
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn hex(s: &str) -> Vec<u8> {
    hex::decode(s).unwrap()
}

fn vectors() -> serde_json::Value {
    let root = workspace_root().join("spec/heap/vectors-v1.json");
    serde_json::from_str(&fs::read_to_string(root).unwrap()).unwrap()
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_server(bind: &str) {
    for _ in 0..100 {
        if TcpStream::connect(bind).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("server did not accept on {bind}");
}

fn issue_localhost_tls(dir: &std::path::Path) -> (PathBuf, PathBuf, PathBuf) {
    let ca_key = KeyPair::generate().unwrap();
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params
        .distinguished_name
        .push(DnType::CommonName, "dingo-heap-test-ca");
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();
    let ca_path = dir.join("ca.pem");
    fs::write(&ca_path, ca_cert.pem()).unwrap();

    let srv_key = KeyPair::generate().unwrap();
    let mut params = CertificateParams::new(vec!["localhost".into()]).unwrap();
    params
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    params.subject_alt_names = vec![SanType::DnsName("localhost".try_into().unwrap())];
    let srv_cert = params.signed_by(&srv_key, &ca_cert, &ca_key).unwrap();
    let cert_path = dir.join("server.crt");
    let key_path = dir.join("server.key");
    fs::write(&cert_path, srv_cert.pem()).unwrap();
    fs::write(&key_path, srv_key.serialize_pem()).unwrap();
    (ca_path, cert_path, key_path)
}

#[test]
fn connect_heap_welcome_and_process_ops() {
    let doc = vectors();
    let inputs = &doc["inputs"];
    let master_pk: [u8; 32] = hex(inputs["master_public_key"].as_str().unwrap())
        .try_into()
        .unwrap();
    let holder_seed: [u8; 32] = hex(inputs["holder_seed"].as_str().unwrap())
        .try_into()
        .unwrap();
    let cose = hex(doc["accepted"][0]["cose_sign1"].as_str().unwrap());
    let verified = verify_certificate(&cose, &master_pk).unwrap();
    let now_unix = verified.not_before + 10;

    let snap = HeapSecuritySnapshot {
        deployment_id: verified.deployment_id,
        heap_id: verified.heap_id,
        authority_epoch: verified.authority_epoch,
        authority_generation: verified.authority_generation,
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: master_pk,
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [1u8; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let registry = Arc::new(ResidentHeapRegistry::new());
    registry.insert(
        verified.heap_id,
        ResidentHeap {
            slot: Arc::new(HeapSlot::new(snap)),
            display_name: Some("accounts".into()),
        },
    );
    let audit = Arc::new(HeapAuthAuditLog::default());

    let tmp = TempDir::new().unwrap();
    let store_path = tmp.path().join("app.dingo");
    Store::open(&store_path).unwrap();
    let (ca_path, cert_path, key_path) = issue_localhost_tls(tmp.path());

    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    let opts = ServeOptions::new()
        .tls(TlsServerOptions::new(&cert_path, &key_path))
        .qualified_heap_key(true)
        .heap_registry(Arc::clone(&registry))
        .deployment_id(verified.deployment_id.to_string())
        .heap_auth_audit(Arc::clone(&audit))
        .security_now_unix_s(now_unix)
        .shutdown_flag(Arc::clone(&shutdown));
    let store_c = store_path.clone();
    let bind_c = bind.clone();
    thread::spawn(move || {
        let _ = serve_store_with(store_c, &bind_c, opts);
    });
    wait_for_server(&bind);

    let holder = Arc::new(InMemoryHolderKey::from_seed(holder_seed));
    let credential = HeapCredential::new(&cose, holder).expect("credential");
    assert_eq!(credential.heap_id(), verified.heap_id);

    let options = RemoteHeapOptions::new(
        TlsClientOptions::new("localhost").ca_path(ca_path),
        credential,
    )
    .expected_heap_name("accounts")
    .now_unix_s(now_unix);

    let url = format!("dingo://127.0.0.1:{port}/accounts");
    let mut heap = Dingo::connect_heap(&url, options).expect("connect_heap");
    assert_eq!(heap.id(), verified.heap_id);
    assert_eq!(heap.welcome().msg, "welcome");
    assert_eq!(heap.heap_profile(), HEAP_PROFILE);

    heap.ping().expect("ping");
    assert!(heap.live().expect("live"));
    assert!(heap.ready().expect("ready"));

    drop(heap);
    flag.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(50));

    assert!(
        audit.snapshot().iter().any(|e| matches!(
            e,
            dingo_server::HeapAuthAuditEvent::Welcome { .. }
        )),
        "audit must record welcome"
    );
}

#[test]
fn connect_heap_wrong_name_rejects() {
    let doc = vectors();
    let inputs = &doc["inputs"];
    let master_pk: [u8; 32] = hex(inputs["master_public_key"].as_str().unwrap())
        .try_into()
        .unwrap();
    let holder_seed: [u8; 32] = hex(inputs["holder_seed"].as_str().unwrap())
        .try_into()
        .unwrap();
    let cose = hex(doc["accepted"][0]["cose_sign1"].as_str().unwrap());
    let verified = verify_certificate(&cose, &master_pk).unwrap();
    let now_unix = verified.not_before + 10;

    let snap = HeapSecuritySnapshot {
        deployment_id: verified.deployment_id,
        heap_id: verified.heap_id,
        authority_epoch: verified.authority_epoch,
        authority_generation: verified.authority_generation,
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: master_pk,
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [1u8; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let registry = Arc::new(ResidentHeapRegistry::new());
    registry.insert(
        verified.heap_id,
        ResidentHeap {
            slot: Arc::new(HeapSlot::new(snap)),
            display_name: Some("accounts".into()),
        },
    );

    let tmp = TempDir::new().unwrap();
    let store_path = tmp.path().join("app.dingo");
    Store::open(&store_path).unwrap();
    let (ca_path, cert_path, key_path) = issue_localhost_tls(tmp.path());

    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    let opts = ServeOptions::new()
        .tls(TlsServerOptions::new(&cert_path, &key_path))
        .qualified_heap_key(true)
        .heap_registry(Arc::clone(&registry))
        .deployment_id(verified.deployment_id.to_string())
        .security_now_unix_s(now_unix)
        .shutdown_flag(Arc::clone(&shutdown));
    let store_c = store_path.clone();
    let bind_c = bind.clone();
    thread::spawn(move || {
        let _ = serve_store_with(store_c, &bind_c, opts);
    });
    wait_for_server(&bind);

    let holder = Arc::new(InMemoryHolderKey::from_seed(holder_seed));
    let credential = HeapCredential::new(&cose, holder).unwrap();
    let options = RemoteHeapOptions::new(
        TlsClientOptions::new("localhost").ca_path(ca_path),
        credential,
    )
    .expected_heap_name("wrong-name")
    .now_unix_s(now_unix)
    .max_connect_attempts(std::num::NonZeroU32::new(1).unwrap());

    let url = format!("dingo://127.0.0.1:{port}/wrong-name");
    let err = match Dingo::connect_heap(&url, options) {
        Ok(_) => panic!("name mismatch should reject"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("heap_unavailable") || msg.contains("unavailable"),
        "got {msg}"
    );

    flag.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(50));
}

#[test]
fn connect_heap_put_get_delete_subject_v2() {
    use dingo_store::{
        create_object, publish_staged_genesis, stage_heap_genesis, HeapMetaLayout, ObjectKind,
    };

    let doc = vectors();
    let inputs = &doc["inputs"];
    let master_pk: [u8; 32] = hex(inputs["master_public_key"].as_str().unwrap())
        .try_into()
        .unwrap();
    let holder_seed: [u8; 32] = hex(inputs["holder_seed"].as_str().unwrap())
        .try_into()
        .unwrap();
    let cose = hex(doc["accepted"][0]["cose_sign1"].as_str().unwrap());
    let verified = verify_certificate(&cose, &master_pk).unwrap();
    let now_unix = verified.not_before + 10;

    let snap = HeapSecuritySnapshot {
        deployment_id: verified.deployment_id,
        heap_id: verified.heap_id,
        authority_epoch: verified.authority_epoch,
        authority_generation: verified.authority_generation,
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: master_pk,
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [1u8; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let registry = Arc::new(ResidentHeapRegistry::new());
    registry.insert(
        verified.heap_id,
        ResidentHeap {
            slot: Arc::new(HeapSlot::new(snap)),
            display_name: Some("accounts".into()),
        },
    );

    let tmp = TempDir::new().unwrap();
    let store_path = tmp.path().join("app.dingo");
    Store::open(&store_path).unwrap();
    // Heap meta + collection on the same store root the server opens.
    let layout = HeapMetaLayout::new(&store_path);
    let dep = *verified.deployment_id.as_bytes();
    let heap = *verified.heap_id.as_bytes();
    let coll = *dingo_heap::CollectionId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap, [9u8; 16], "accounts").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    create_object(
        &layout,
        &heap,
        ObjectKind::Collection,
        coll,
        [8u8; 16],
        "users",
    )
    .unwrap();

    let (ca_path, cert_path, key_path) = issue_localhost_tls(tmp.path());
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    let opts = ServeOptions::new()
        .tls(TlsServerOptions::new(&cert_path, &key_path))
        .qualified_heap_key(true)
        .heap_registry(Arc::clone(&registry))
        .deployment_id(verified.deployment_id.to_string())
        .security_now_unix_s(now_unix)
        .shutdown_flag(Arc::clone(&shutdown));
    let store_c = store_path.clone();
    let bind_c = bind.clone();
    thread::spawn(move || {
        let _ = serve_store_with(store_c, &bind_c, opts);
    });
    wait_for_server(&bind);

    let holder = Arc::new(InMemoryHolderKey::from_seed(holder_seed));
    let credential = HeapCredential::new(&cose, holder).unwrap();
    let options = RemoteHeapOptions::new(
        TlsClientOptions::new("localhost").ca_path(ca_path),
        credential,
    )
    .expected_heap_name("accounts")
    .now_unix_s(now_unix);

    let mut heap = Dingo::connect_heap(format!("dingo://127.0.0.1:{port}/accounts"), options)
        .expect("connect_heap");
    let cid = heap.collection_open("users").expect("collection_open");
    heap.put_json(&cid, "user-1", &serde_json::json!({"name": "Alice"}))
        .expect("put");
    let got = heap.get_json(&cid, "user-1").expect("get").expect("found");
    assert_eq!(got["name"], "Alice");
    heap.put_bytes(&cid, "blob-1", b"\x00\xff").expect("put_bytes");
    assert_eq!(
        heap.get_bytes(&cid, "blob-1").expect("get_bytes").unwrap(),
        b"\x00\xff"
    );
    assert!(heap.delete(&cid, "user-1").expect("delete"));
    assert!(heap.get_json(&cid, "user-1").expect("get after delete").is_none());

    drop(heap);
    flag.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(50));
}

#[test]
fn connect_heap_list_and_scan_json() {
    use dingo_store::{
        create_object, publish_staged_genesis, stage_heap_genesis, HeapMetaLayout, ObjectKind,
    };

    let doc = vectors();
    let inputs = &doc["inputs"];
    let master_pk: [u8; 32] = hex(inputs["master_public_key"].as_str().unwrap())
        .try_into()
        .unwrap();
    let holder_seed: [u8; 32] = hex(inputs["holder_seed"].as_str().unwrap())
        .try_into()
        .unwrap();
    let cose = hex(doc["accepted"][0]["cose_sign1"].as_str().unwrap());
    let verified = verify_certificate(&cose, &master_pk).unwrap();
    let now_unix = verified.not_before + 10;

    let snap = HeapSecuritySnapshot {
        deployment_id: verified.deployment_id,
        heap_id: verified.heap_id,
        authority_epoch: verified.authority_epoch,
        authority_generation: verified.authority_generation,
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: master_pk,
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [1u8; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let registry = Arc::new(ResidentHeapRegistry::new());
    registry.insert(
        verified.heap_id,
        ResidentHeap {
            slot: Arc::new(HeapSlot::new(snap)),
            display_name: Some("accounts".into()),
        },
    );

    let tmp = TempDir::new().unwrap();
    let store_path = tmp.path().join("app.dingo");
    Store::open(&store_path).unwrap();
    let layout = HeapMetaLayout::new(&store_path);
    let dep = *verified.deployment_id.as_bytes();
    let heap_b = *verified.heap_id.as_bytes();
    let coll = *dingo_heap::CollectionId::new_random().unwrap().as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_b, [9u8; 16], "accounts").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    create_object(
        &layout,
        &heap_b,
        ObjectKind::Collection,
        coll,
        [8u8; 16],
        "users",
    )
    .unwrap();

    let (ca_path, cert_path, key_path) = issue_localhost_tls(tmp.path());
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    let opts = ServeOptions::new()
        .tls(TlsServerOptions::new(&cert_path, &key_path))
        .qualified_heap_key(true)
        .heap_registry(Arc::clone(&registry))
        .deployment_id(verified.deployment_id.to_string())
        .security_now_unix_s(now_unix)
        .shutdown_flag(Arc::clone(&shutdown));
    let store_c = store_path.clone();
    let bind_c = bind.clone();
    thread::spawn(move || {
        let _ = serve_store_with(store_c, &bind_c, opts);
    });
    wait_for_server(&bind);

    let holder = Arc::new(InMemoryHolderKey::from_seed(holder_seed));
    let credential = HeapCredential::new(&cose, holder).unwrap();
    let options = RemoteHeapOptions::new(
        TlsClientOptions::new("localhost").ca_path(ca_path),
        credential,
    )
    .expected_heap_name("accounts")
    .now_unix_s(now_unix);

    let mut remote = Dingo::connect_heap(format!("dingo://127.0.0.1:{port}/accounts"), options)
        .expect("connect_heap");
    let listed = remote.list_collections().expect("list_collections");
    assert!(
        listed.iter().any(|(_, n)| n == "users"),
        "users missing from {listed:?}"
    );
    let cid = remote.collection_open("users").expect("open");
    remote
        .put_json(&cid, "a", &serde_json::json!({"n": 1}))
        .unwrap();
    remote
        .put_json(&cid, "b", &serde_json::json!({"n": 2}))
        .unwrap();
    remote
        .put_json(&cid, "c", &serde_json::json!({"n": 3}))
        .unwrap();

    let keys = remote.list_keys(&cid, Some(10), None).expect("list_keys");
    assert_eq!(keys, vec!["a", "b", "c"]);
    let page = remote.list_keys(&cid, Some(1), Some("a")).expect("page");
    assert_eq!(page, vec!["b"]);

    let rows = remote.scan_json(&cid, Some(10), None).expect("scan");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, "a");
    assert_eq!(rows[0].1["n"], 1);
    assert_eq!(rows[2].1["n"], 3);

    drop(remote);
    flag.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(50));
}

#[test]
fn credential_rejects_holder_mismatch() {
    let doc = vectors();
    let cose = hex(doc["accepted"][0]["cose_sign1"].as_str().unwrap());
    let wrong = Arc::new(InMemoryHolderKey::from_seed([9u8; 32]));
    let err = match HeapCredential::new(&cose, wrong) {
        Ok(_) => panic!("holder mismatch should reject"),
        Err(e) => e,
    };
    assert_eq!(err, dingo_sdk::CredentialError::HolderKeyMismatch);
}