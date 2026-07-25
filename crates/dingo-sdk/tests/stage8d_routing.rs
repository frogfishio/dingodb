//! Stage 8d: SDK cluster routing + client directory cache (CLUSTER_SPEC §13, §22.5).

use dingo_cluster::{ClusterConfig, NodeId};
use dingo_sdk::{
    json, parse_dingo_url, ClientDirectoryCache, ClusterConfig as SdkClusterConfig, Dingo,
    DirectorySnapshot, ErrorCode, Filter,
};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn create_cluster_same_collection_api() {
    let dir = tempdir().unwrap();
    let mut db = Dingo::create_cluster(
        SdkClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(8),
    )
    .unwrap();
    assert!(db.is_cluster());
    assert!(!db.is_remote());
    assert!(db.path().is_some());

    {
        let mut users = db.collection("users").unwrap();
        let r = users
            .put("alice", &json!({"name": "Alice", "status": "active"}))
            .unwrap();
        assert!(r.committed);
        assert_eq!(
            users.get("alice").unwrap().unwrap()["name"],
            json!("Alice")
        );
        users.put_bytes("bin", b"\x00\xff").unwrap();
        assert_eq!(users.get_bytes("bin").unwrap().unwrap(), b"\x00\xff");
        users.delete("bin").unwrap();
        assert!(users.get_bytes("bin").unwrap().is_none());
    }

    let keys = db.collection("users").unwrap().scan_keys().unwrap();
    assert!(keys.contains(&"alice".to_string()));

    let found = db
        .collection("users")
        .unwrap()
        .find(&Filter::field("status").eq("active"))
        .unwrap();
    assert_eq!(found.len(), 1);

    let hist = db.collection("users").unwrap().history("alice").unwrap();
    assert!(!hist.versions.is_empty());
    assert_eq!(hist.versions[0].kind, "put");
}

#[test]
fn open_cluster_roundtrip() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("c");
    {
        let mut db = Dingo::create_cluster(
            ClusterConfig::development(&root).with_virtual_partitions(4),
        )
        .unwrap();
        db.collection("docs")
            .unwrap()
            .put("k", &json!({"v": 1}))
            .unwrap();
    }
    let mut db = Dingo::open_cluster(&root).unwrap();
    assert_eq!(
        db.collection("docs").unwrap().get("k").unwrap().unwrap()["v"],
        1
    );
}

#[test]
fn client_cache_routes_and_refreshes_on_poisoned_leader() {
    let dir = tempdir().unwrap();
    let mut db = Dingo::create_cluster(
        ClusterConfig::dependable_local(dir.path().join("c")).with_virtual_partitions(8),
    )
    .unwrap();

    // Warm leaders.
    db.collection("t")
        .unwrap()
        .put("warm", &json!(0))
        .unwrap();

    let backend = db.cluster_backend_mut().unwrap();
    let subject = {
        // Encode the same way the SDK does for partition hashing.
        dingo_sdk::encode_subject("t", "k2").unwrap()
    };
    let p = backend.cache().partition_of(&subject);
    let refreshes_before = backend.cache().entry_refresh_count();
    backend.cache_mut().poison_leader(p, NodeId::new(99));

    db.collection("t")
        .unwrap()
        .put("k2", &json!({"ok": true}))
        .unwrap();

    let backend = db.cluster_backend_mut().unwrap();
    assert!(
        backend.cache().entry_refresh_count() > refreshes_before,
        "stale placement must refresh the cached entry"
    );
    // Live route should no longer claim fake leader 99.
    let route = backend.cache().get(p).unwrap();
    assert_ne!(route.leader, NodeId::new(99));

    assert_eq!(
        db.collection("t").unwrap().get("k2").unwrap().unwrap()["ok"],
        true
    );
}

#[test]
fn multi_seed_url_parse() {
    let p = parse_dingo_url("dingo://127.0.0.1:7400,127.0.0.1:7401/app").unwrap();
    assert_eq!(p.seeds.len(), 2);
    assert_eq!(p.label.as_deref(), Some("app"));
}

fn free_bind() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("127.0.0.1:{port}")
}

fn wait_for(bind: &str) {
    for _ in 0..80 {
        if std::net::TcpStream::connect(bind).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("server did not accept on {bind}");
}

#[test]
fn remote_directory_op_and_cache() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("app.dingo");
    {
        let mut db = Dingo::open(&path).unwrap();
        db.collection("c")
            .unwrap()
            .put("k", &json!({"x": 1}))
            .unwrap();
    }

    let bind = free_bind();
    let path_c = path.clone();
    let bind_c = bind.clone();
    thread::spawn(move || {
        let _ = dingo_sdk::serve_store(path_c, &bind_c);
    });
    wait_for(&bind);

    let url = format!("dingo://{bind}/app");
    let mut client = dingo_sdk::RemoteClient::connect(&bind, url.clone()).unwrap();
    let snap: DirectorySnapshot = client.fetch_directory().unwrap();
    assert!(snap.virtual_partitions >= 1);
    assert_eq!(snap.placement_epoch, 1);
    assert!(!snap.assignments.is_empty());
    assert!(snap.assignments.iter().all(|a| a.leader == 0));

    let cache = ClientDirectoryCache::from_snapshot(&snap);
    assert!(cache.route(b"any-subject").is_some());
    assert_eq!(cache.refresh_count(), 1);

    // Collection API still works on a separate connection (serve is sequential).
    drop(client);
    let mut db = Dingo::connect(&url).unwrap();
    assert_eq!(db.collection("c").unwrap().get("k").unwrap().unwrap()["x"], 1);
}

#[test]
fn partition_unavailable_code_is_stable() {
    // Ensure ErrorCode mapping exists for cluster-style failures.
    let err = dingo_sdk::Error::PartitionUnavailable {
        partition: 3,
        reason: "test",
    };
    assert_eq!(err.code(), ErrorCode::PartitionUnavailable);
    assert_eq!(err.code().as_str(), "partition_unavailable");
    let err = dingo_sdk::Error::StaleRoute {
        partition: 1,
        message: "x".into(),
    };
    assert_eq!(err.code(), ErrorCode::StaleRoute);
}
